use crate::domain::{Chapter, ModelProfile, Job};
use crate::ai::prompts::build_batch_validation_prompt;
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

pub(crate) async fn validate_batch(
    client: &Client,
    profile: &ModelProfile,
    api_key: &str,
    chapters: &[Chapter],
) -> Result<Vec<(String, bool, Option<String>)>, String> {
    let prompt = build_batch_validation_prompt(chapters);
    let system = "你是一位专业的小说内容分析师，专注于批量判断章节是否为有效的小说内容。你必须严格按JSON数组格式输出结果。";

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

    // Try to parse as array first, then as object with "results" key
    let results_arr = if let Some(arr) = value.as_array() {
        arr.clone()
    } else if let Some(arr) = value.get("results").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        // If AI returned a single object, wrap it in an array
        vec![value.clone()]
    };

    let mut results = Vec::new();

    // First try to match by chapter_id if present
    for chapter in chapters {
        let mut found = false;
        for item in &results_arr {
            let item_id = item.get("chapter_id").and_then(|v| v.as_str());
            if item_id == Some(&chapter.id) {
                let is_valid = item.get("is_valid").and_then(|v| v.as_bool()).unwrap_or(true);
                let reason = item.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
                results.push((chapter.id.clone(), is_valid, reason));
                found = true;
                break;
            }
        }
        // If not found by ID, default to valid
        if !found {
            results.push((chapter.id.clone(), true, None));
        }
    }

    Ok(results)
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
