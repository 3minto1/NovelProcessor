use crate::domain::Job;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn start_validation(
    state: State<'_, AppState>,
    novel_id: String,
    profile_id: String,
) -> Result<Job, String> {
    if !state.validation_task.start() {
        return Err("验证任务已在运行中，请等待完成或终止后再试。".to_string());
    }

    let job = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        crate::services::validate::start_validation(&conn, &novel_id, &profile_id)?
    };

    let (profile, api_key, chapters, batch_size, parallelism) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let profile = get_profile(&conn, &profile_id)?;
        let api_key = get_api_key(&conn, &profile_id)?;
        let chapters = crate::repositories::chapters::list_chapters(&conn, &novel_id)?;
        let batch_size = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'chapter_batch_size'",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .unwrap_or(None)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50);
        let parallelism = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'review_parallelism'",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .unwrap_or(None)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10);
        (profile, api_key, chapters, batch_size, parallelism)
    };

    let db_path = state.db_path.clone();
    let validation_task = state.validation_task.clone();
    let job_id = job.id.clone();
    let novel_id_clone = novel_id.clone();
    let profile_id_clone = profile_id.clone();
    let total_chapters = chapters.len();
    let total_batches = (total_chapters + batch_size - 1) / batch_size;
    eprintln!("[Validation] Starting validation for {} chapters, batch_size: {}, parallelism: {}, job_id: {}", total_chapters, batch_size, parallelism, job_id);

    tokio::spawn(async move {
        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Validation] FATAL: Cannot open database: {}", e);
                validation_task.finish();
                return;
            }
        };

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let batches: Vec<_> = chapters.chunks(batch_size).enumerate().collect();
        let mut processed = 0i64;
        let mut error_message: Option<String> = None;

        for (chunk_idx, chunk) in batches.chunks(parallelism).enumerate() {
            let global_batch_start = chunk_idx * parallelism;

            if validation_task.is_cancelled() {
                let _ = crate::services::progress::complete_job(
                    &conn,
                    &job_id,
                    &format!("验证已终止，已完成 {} / {} 章", processed, total_chapters),
                );
                validation_task.finish();
                eprintln!("[Validation] Cancelled");
                return;
            }

            let mut handles = Vec::new();

            for (batch_idx, batch) in chunk {
                let client = client.clone();
                let profile = profile.clone();
                let api_key = api_key.clone();
                let batch = batch.to_vec();
                let batch_idx = *batch_idx;
                let first_title = batch.first().map(|c| c.title.as_str()).unwrap_or("").to_string();

                let _ = crate::services::progress::update_job_progress(
                    &conn,
                    &job_id,
                    processed,
                    &format!("正在验证第 {}~{} 批 (共 {} 批) · {}/{}", global_batch_start + 1, (global_batch_start + chunk.len()).min(total_batches), total_batches, processed, total_chapters),
                );

                handles.push(tokio::spawn(async move {
                    let result = crate::services::validate::validate_batch(
                        &client,
                        &profile,
                        &api_key,
                        &batch,
                    ).await;
                    (batch_idx, first_title, result)
                }));
            }

            for handle in handles {
                let (batch_idx, first_title, result) = match handle.await {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("[Validation] Task join error: {}", e);
                        error_message = Some(format!("任务异常：{}", e));
                        continue;
                    }
                };

                match result {
                    Ok((results, output)) => {
                        crate::services::validate::log_ai_call(
                            &conn,
                            &novel_id_clone,
                            &profile_id_clone,
                            "validate",
                            Some(&first_title),
                            "success",
                            &output.text,
                            output.reasoning.as_deref(),
                            &output.raw_response,
                        );
                        for (chapter_id, is_valid, reason) in &results {
                            if let Err(e) = crate::services::validate::update_chapter_validation(
                                &conn,
                                chapter_id,
                                *is_valid,
                                reason.as_deref(),
                            ) {
                                eprintln!("[Validation] Failed to update chapter {}: {}", chapter_id, e);
                            }
                        }
                        eprintln!("[Validation] Batch {} updated {} chapters", batch_idx + 1, results.len());
                    }
                    Err(e) => {
                        eprintln!("[Validation] ERROR batch {}: {}", batch_idx + 1, e);
                        error_message = Some(e.clone());
                        crate::services::validate::log_ai_call(
                            &conn,
                            &novel_id_clone,
                            &profile_id_clone,
                            "validate",
                            Some(&first_title),
                            "error",
                            &e,
                            None,
                            "",
                        );
                    }
                }
            }

            processed += chunk.iter().map(|(_, b)| b.len() as i64).sum::<i64>();
            let _ = crate::services::progress::update_job_progress(
                &conn,
                &job_id,
                processed,
                &format!("已验证 {} / {} 章 · 第 {}~{} 批", processed, total_chapters, global_batch_start + 1, (global_batch_start + chunk.len()).min(total_batches)),
            );
        }

        let message = match error_message {
            Some(e) => format!("验证完成（部分失败），共 {} 章。错误：{}", total_chapters, e),
            None => format!("验证完成，共 {} 章", total_chapters),
        };
        let _ = crate::services::progress::complete_job(&conn, &job_id, &message);
        eprintln!("[Validation] Job completed successfully");

        validation_task.finish();
    });

    Ok(job)
}

#[tauri::command]
pub(crate) async fn cancel_validation(
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.validation_task.is_active() {
        state.validation_task.cancel();
        Ok(())
    } else {
        Err("当前没有正在运行的验证任务。".to_string())
    }
}

#[tauri::command]
pub(crate) async fn is_validation_active(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.validation_task.is_active())
}

#[tauri::command]
pub(crate) async fn is_task_paused(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.validation_task.is_paused())
}

fn get_profile(conn: &rusqlite::Connection, profile_id: &str) -> Result<crate::domain::ModelProfile, String> {
    conn.query_row(
        "SELECT id, name, provider, base_url, model, temperature, top_p, thinking_mode,
                CASE WHEN api_key IS NOT NULL AND api_key != '' THEN 1 ELSE 0 END as has_api_key,
                CASE
                    WHEN api_key IS NULL OR api_key = '' THEN 'none'
                    WHEN api_key = '__KEYRING__' THEN 'system'
                    ELSE 'database_fallback'
                END as api_key_storage,
                updated_at
         FROM model_profiles WHERE id = ?1",
        rusqlite::params![profile_id],
        |row| {
            Ok(crate::domain::ModelProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                base_url: row.get(3)?,
                model: row.get(4)?,
                temperature: row.get(5)?,
                top_p: row.get(6)?,
                thinking_mode: row.get(7)?,
                has_api_key: row.get::<_, i64>(8)? != 0,
                api_key_storage: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

fn get_api_key(conn: &rusqlite::Connection, profile_id: &str) -> Result<String, String> {
    let db_key: String = conn.query_row(
        "SELECT api_key FROM model_profiles WHERE id = ?1",
        rusqlite::params![profile_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .map_err(|e| e.to_string())?
    .unwrap_or_default();

    match db_key.as_str() {
        "__KEYRING__" => {
            let key = crate::credentials::load_api_key(profile_id)
                .map_err(|e| e.to_string())?
                .unwrap_or_default();
            Ok(key)
        }
        key if !key.is_empty() => Ok(key.to_string()),
        _ => Ok(String::new()),
    }
}
