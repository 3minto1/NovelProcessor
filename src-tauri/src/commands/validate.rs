use crate::domain::Job;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn start_validation(
    state: State<'_, AppState>,
    novel_id: String,
    profile_id: String,
) -> Result<Job, String> {
    let job = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        crate::services::validate::start_validation(&conn, &novel_id, &profile_id)?
    };
    
    // Get profile and chapters for actual processing
    let (profile, api_key, chapters) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let profile = get_profile(&conn, &profile_id)?;
        let api_key = get_api_key(&conn, &profile_id)?;
        let chapters = crate::repositories::chapters::list_chapters(&conn, &novel_id)?;
        (profile, api_key, chapters)
    };
    
    // Process chapters in background
    let job_id = job.id.clone();
    let db_path = "novel_processor.db".to_string();
    
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut processed = 0;
        
        for chapter in &chapters {
            // Update progress
            {
                let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
                crate::services::progress::update_job_progress(
                    &conn,
                    &job_id,
                    processed,
                    &format!("正在验证: {}", chapter.title),
                )?;
            }
            
            // Call AI for validation
            match crate::services::validate::validate_chapter(
                &client,
                &profile,
                &api_key,
                chapter,
            ).await {
                Ok((is_valid, reason)) => {
                    let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
                    crate::services::validate::update_chapter_validation(
                        &conn,
                        &chapter.id,
                        is_valid,
                        reason.as_deref(),
                    )?;
                }
                Err(e) => {
                    eprintln!("Validation failed for chapter {}: {}", chapter.index, e);
                }
            }
            
            processed += 1;
        }
        
        // Complete the job
        let conn = rusqlite::Connection::open(&db_path).map_err(|e| e.to_string())?;
        crate::services::progress::complete_job(
            &conn,
            &job_id,
            &format!("验证完成，共 {} 章", chapters.len()),
        )?;
        
        Ok::<(), String>(())
    });
    
    Ok(job)
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
