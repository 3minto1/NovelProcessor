use crate::domain::{Chapter, Job};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn start_review(
    state: State<'_, AppState>,
    novel_id: String,
    profile_id: String,
) -> Result<Job, String> {
    if !state.validation_task.start() {
        return Err("任务已在运行中，请等待完成或终止后再试。".to_string());
    }

    let (job, profile, api_key, valid_chapters, batch_size, parallelism) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let job = crate::services::review::start_review(&conn, &novel_id, &profile_id)?;
        let profile = get_profile(&conn, &profile_id)?;
        let api_key = get_api_key(&conn, &profile_id)?;
        let chapters = crate::repositories::chapters::list_chapters(&conn, &novel_id)?;
        let valid_chapters: Vec<_> = chapters.into_iter().filter(|c| c.is_valid).collect();
        let batch_size = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'chapter_batch_size'",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .unwrap_or(None)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(30);
        let parallelism = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'review_parallelism'",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .unwrap_or(None)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10);
        (job, profile, api_key, valid_chapters, batch_size, parallelism)
    };

    let db_path = state.db_path.clone();
    let job_id = job.id.clone();
    let novel_id_clone = novel_id.clone();
    let profile_id_clone = profile_id.clone();
    let task_control = state.validation_task.clone();

    tokio::spawn(async move {
        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Review] FATAL: Cannot open database: {}", e);
                task_control.finish();
                return;
            }
        };

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut slices: Vec<Vec<Chapter>> = Vec::new();
        let slice_size = if parallelism > 0 { batch_size / parallelism } else { batch_size };
        let slice_size = slice_size.max(1);
        for batch in valid_chapters.chunks(batch_size) {
            for pair in batch.chunks(slice_size) {
                slices.push(pair.to_vec());
            }
        }

        let total_slices = slices.len();
        let total_chapters = valid_chapters.len();
        let mut processed_chapters = 0i64;
        let mut error_message: Option<String> = None;

        for (chunk_idx, chunk) in slices.chunks(parallelism).enumerate() {
            let global_slice_start = chunk_idx * parallelism;

            if task_control.is_cancelled() {
                let _ = crate::services::progress::complete_job(
                    &conn,
                    &job_id,
                    &format!("审查已终止，已完成 {} / {} 章", processed_chapters, total_chapters),
                );
                task_control.finish();
                eprintln!("[Review] Cancelled");
                return;
            }

            while task_control.is_paused() {
                let _ = crate::services::progress::update_job_progress(
                    &conn,
                    &job_id,
                    processed_chapters,
                    &format!("已暂停 · 已审查 {} / {} 章 · 第 {}~{} 片段", processed_chapters, total_chapters, global_slice_start + 1, (global_slice_start + chunk.len()).min(total_slices)),
                );
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if task_control.is_cancelled() {
                    let _ = crate::services::progress::complete_job(
                        &conn,
                        &job_id,
                        &format!("审查已终止，已完成 {} / {} 章", processed_chapters, total_chapters),
                    );
                    task_control.finish();
                    eprintln!("[Review] Cancelled during pause");
                    return;
                }
            }

            let mut handles = Vec::new();

            for (slice_idx, slice) in chunk.iter().enumerate() {
                let client = client.clone();
                let profile = profile.clone();
                let api_key = api_key.clone();
                let slice = slice.clone();
                let first_title = slice.first().map(|c| c.title.as_str()).unwrap_or("").to_string();

                let _ = crate::services::progress::update_job_progress(
                    &conn,
                    &job_id,
                    processed_chapters,
                    &format!("正在审查第 {}~{} 片段 (共 {} 片段) · {}/{}", global_slice_start + 1, (global_slice_start + chunk.len()).min(total_slices), total_slices, processed_chapters, total_chapters),
                );

                handles.push(tokio::spawn(async move {
                    let mut result = crate::services::review::review_batch(
                        &client,
                        &profile,
                        &api_key,
                        &slice,
                    ).await;
                    if result.is_err() {
                        eprintln!("[Review] Slice {} failed, retrying...", slice_idx + 1);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        result = crate::services::review::review_batch(
                            &client,
                            &profile,
                            &api_key,
                            &slice,
                        ).await;
                    }
                    (slice_idx, first_title, result)
                }));
            }

            for handle in handles {
                let (slice_idx, first_title, result) = match handle.await {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("[Review] Task join error: {}", e);
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
                            "review",
                            Some(&first_title),
                            "success",
                            &output.text,
                            output.reasoning.as_deref(),
                            &output.raw_response,
                        );
                        for (chapter_id, corrected_text) in &results {
                            if let Err(e) = crate::services::review::update_chapter_review(
                                &conn,
                                chapter_id,
                                corrected_text,
                            ) {
                                eprintln!("[Review] Failed to update chapter {}: {}", chapter_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[Review] ERROR slice {}: {}", slice_idx + 1, e);
                        error_message = Some(e.clone());
                        crate::services::validate::log_ai_call(
                            &conn,
                            &novel_id_clone,
                            &profile_id_clone,
                            "review",
                            Some(&first_title),
                            "error",
                            &e,
                            None,
                            "",
                        );
                    }
                }
            }

            processed_chapters += chunk.iter().map(|s| s.len() as i64).sum::<i64>();
            let _ = crate::services::progress::update_job_progress(
                &conn,
                &job_id,
                processed_chapters,
                &format!("已审查 {} / {} 章 · 第 {}~{} 片段", processed_chapters, total_chapters, global_slice_start + 1, (global_slice_start + chunk.len()).min(total_slices)),
            );
        }

        let message = match error_message {
            Some(e) => format!("审查完成（部分失败），共 {} 章。错误：{}", total_chapters, e),
            None => format!("审查完成，共 {} 章", total_chapters),
        };
        let _ = crate::services::progress::complete_job(&conn, &job_id, &message);
        eprintln!("[Review] Job completed successfully");
        task_control.finish();
    });

    Ok(job)
}

#[tauri::command]
pub(crate) async fn cancel_review(
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.validation_task.is_active() {
        state.validation_task.cancel();
        Ok(())
    } else {
        Err("当前没有正在运行的审查任务。".to_string())
    }
}

#[tauri::command]
pub(crate) async fn pause_review(
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.validation_task.is_active() && !state.validation_task.is_paused() {
        state.validation_task.pause();
        Ok(())
    } else {
        Err("当前没有正在运行的审查任务。".to_string())
    }
}

#[tauri::command]
pub(crate) async fn resume_review(
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.validation_task.is_active() && state.validation_task.is_paused() {
        state.validation_task.resume();
        Ok(())
    } else {
        Err("当前没有暂停的审查任务。".to_string())
    }
}

#[tauri::command]
pub(crate) async fn review_single_chapter(
    state: State<'_, AppState>,
    chapter_id: String,
    profile_id: String,
) -> Result<String, String> {
    let (chapter, profile, api_key) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let chapter = crate::repositories::chapters::get_chapter(&conn, &chapter_id)
            .map_err(|e| format!("章节不存在：{}", e))?;
        let profile = get_profile(&conn, &profile_id)?;
        let api_key = get_api_key(&conn, &profile_id)?;
        (chapter, profile, api_key)
    };

    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let result = crate::services::review::review_batch(
        &client,
        &profile,
        &api_key,
        &[chapter.clone()],
    ).await;

    let result = match result {
        Ok(r) => Ok(r),
        Err(e) => {
            eprintln!("[Review] Single chapter failed, retrying: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            crate::services::review::review_batch(
                &client,
                &profile,
                &api_key,
                &[chapter.clone()],
            ).await
        }
    };

    match result {
        Ok((results, output)) => {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            crate::services::validate::log_ai_call(
                &conn,
                &chapter.novel_id,
                &profile_id,
                "review",
                Some(&chapter.title),
                "success",
                &output.text,
                output.reasoning.as_deref(),
                &output.raw_response,
            );
            for (chapter_id, corrected_text) in &results {
                crate::services::review::update_chapter_review(
                    &conn,
                    chapter_id,
                    corrected_text,
                ).map_err(|e| e.to_string())?;
            }
            Ok("审查完成".to_string())
        }
        Err(e) => {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            crate::services::validate::log_ai_call(
                &conn,
                &chapter.novel_id,
                &profile_id,
                "review",
                Some(&chapter.title),
                "error",
                &e,
                None,
                "",
            );
            Err(e)
        }
    }
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
