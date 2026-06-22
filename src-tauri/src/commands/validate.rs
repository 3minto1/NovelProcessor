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

    let (profile, api_key, chapters) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let profile = get_profile(&conn, &profile_id)?;
        let api_key = get_api_key(&conn, &profile_id)?;
        let chapters = crate::repositories::chapters::list_chapters(&conn, &novel_id)?;
        (profile, api_key, chapters)
    };

    let db_path = state.db_path.clone();
    let validation_task = state.validation_task.clone();
    let job_id = job.id.clone();
    eprintln!("[Validation] Starting validation for {} chapters, job_id: {}", chapters.len(), job_id);

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let batch_size = 1000;
        let mut processed = 0;

        for batch in chapters.chunks(batch_size) {
            eprintln!("[Validation] Processing batch of {} chapters, total: {}/{}", batch.len(), processed, chapters.len());
            
            if validation_task.is_cancelled() {
                let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
                crate::services::progress::complete_job(
                    &conn,
                    &job_id,
                    &format!("验证已终止，已完成 {} / {} 章", processed, chapters.len()),
                )?;
                validation_task.finish();
                eprintln!("[Validation] Cancelled");
                return Ok::<(), String>(());
            }

            // Update progress at start of batch
            {
                let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
                let first = batch.first().map(|c| c.title.as_str()).unwrap_or("");
                let last = batch.last().map(|c| c.title.as_str()).unwrap_or("");
                crate::services::progress::update_job_progress(
                    &conn,
                    &job_id,
                    processed,
                    &format!("正在验证: {} ~ {}", first, last),
                )?;
            }

            eprintln!("[Validation] Calling AI for batch validation...");
            match crate::services::validate::validate_batch(
                &client,
                &profile,
                &api_key,
                batch,
            ).await {
                Ok(results) => {
                    eprintln!("[Validation] Got {} results from AI", results.len());
                    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
                    for (chapter_id, is_valid, reason) in &results {
                        crate::services::validate::update_chapter_validation(
                            &conn,
                            chapter_id,
                            *is_valid,
                            reason.as_deref(),
                        )?;
                    }
                    eprintln!("[Validation] Updated database for {} chapters", results.len());
                }
                Err(e) => {
                    eprintln!("[Validation] ERROR: {}", e);
                }
            }

            processed += batch.len() as i64;

            // Update progress after batch completes
            {
                let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
                crate::services::progress::update_job_progress(
                    &conn,
                    &job_id,
                    processed,
                    &format!("已验证 {} / {} 章", processed, chapters.len()),
                )?;
            }
        }

        let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
        crate::services::progress::complete_job(
            &conn,
            &job_id,
            &format!("验证完成，共 {} 章", chapters.len()),
        )?;
        eprintln!("[Validation] Job completed successfully");

        validation_task.finish();
        Ok::<(), String>(())
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

fn get_profile(conn: &rusqlite::Connection, profile_id: &str) -> Result<crate::domain::ModelProfile, String> {
    conn.query_row(
        "SELECT id, name, provider, base_url, model, temperature, top_p, thinking_mode,
                CASE WHEN api_key IS NOT NULL AND api_key != '' THEN 1 ELSE 0 END as has_api_key,
                'database' as api_key_storage, updated_at
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
    conn.query_row(
        "SELECT api_key FROM model_profiles WHERE id = ?1",
        rusqlite::params![profile_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .map_err(|e| e.to_string())
    .map(|opt| opt.unwrap_or_default())
}
