use crate::domain::{Chapter, ModelProfile, Job};
use crate::ai::prompts::build_chapter_validation_prompt;
use rusqlite::Connection;
use uuid::Uuid;
use reqwest::Client;

pub(crate) fn start_validation(
    conn: &Connection,
    novel_id: &str,
    _profile_id: &str,
) -> Result<Job, String> {
    let chapters = crate::repositories::chapters::list_chapters(conn, novel_id)?;
    let total = chapters.len() as i64;
    
    let job_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let job = Job {
        id: job_id.clone(),
        novel_id: novel_id.to_string(),
        job_type: "validate".to_string(),
        status: "running".to_string(),
        current_chapter: 0,
        total_chapters: total,
        message: "开始验证章节...".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    
    conn.execute(
        "INSERT INTO jobs (id, novel_id, job_type, status, current_chapter, total_chapters, message, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            job.id, job.novel_id, job.job_type, job.status,
            job.current_chapter, job.total_chapters, job.message,
            job.created_at, job.updated_at,
        ],
    ).map_err(|e| e.to_string())?;
    
    Ok(job)
}

pub(crate) async fn validate_chapter(
    client: &Client,
    profile: &ModelProfile,
    api_key: &str,
    chapter: &Chapter,
) -> Result<(bool, Option<String>), String> {
    let prompt = build_chapter_validation_prompt(chapter);
    let system = "你是一位专业的小说内容分析师，专注于判断章节是否为有效的小说内容。";
    
    let output = crate::ai::common::generate_text(
        client,
        None,
        profile,
        api_key,
        system,
        &prompt,
        true,
    ).await?;
    
    let value: serde_json::Value = serde_json::from_str(&output.text)
        .map_err(|e| format!("无法解析AI响应：{}", e))?;
    
    let is_valid = value["is_valid"].as_bool().unwrap_or(true);
    let reason = value["reason"].as_str().map(|s| s.to_string());
    
    Ok((is_valid, reason))
}

pub(crate) fn update_chapter_validation(
    conn: &Connection,
    chapter_id: &str,
    is_valid: bool,
    reason: Option<&str>,
) -> Result<(), String> {
    crate::repositories::chapters::update_chapter_validation(
        conn,
        chapter_id,
        is_valid,
        reason,
        "completed",
    )
}
