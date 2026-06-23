use crate::domain::{Chapter, ModelOutput, ModelProfile, Job};
use crate::ai::prompts::build_batch_validation_prompt;
use rusqlite::Connection;
use uuid::Uuid;
use reqwest::Client;

pub(crate) fn log_ai_call(
    conn: &Connection,
    novel_id: &str,
    profile_id: &str,
    action: &str,
    chapter_title: Option<&str>,
    status: &str,
    content: &str,
    reasoning: Option<&str>,
    raw_response: &str,
) {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO ai_logs (id, novel_id, profile_id, action, chapter_title, status, content, reasoning, raw_response, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, novel_id, profile_id, action, chapter_title, status, content, reasoning, raw_response, now],
    );
}

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
) -> Result<(Vec<(String, bool, Option<String>)>, ModelOutput), String> {
    let prompt = build_batch_validation_prompt(chapters);
    let system = "你是一位小说目录分析专家。按JSON数组格式输出，每个元素只有is_valid和reason两个字段。只输出JSON，不要其他文字。";

    let output = crate::ai::common::generate_text(
        client,
        None,
        profile,
        api_key,
        system,
        &prompt,
        true,
    ).await?;

    eprintln!("[validate_batch] AI response: {}", &output.text[..output.text.len().min(500)]);

    let json_str = extract_json_from_response(&output.text);
    let value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("无法解析AI响应：{}", e))?;

    // Get the results array
    let results_arr = value.as_array()
        .ok_or_else(|| {
            eprintln!("[validate_batch] AI response is not an array: {:?}", value);
            "AI响应不是JSON数组".to_string()
        })?;

    eprintln!("[validate_batch] Got {} results from AI, expected {} chapters", results_arr.len(), chapters.len());

    let mut results = Vec::new();

    // Match by index - AI returns results in the same order as input
    for (i, chapter) in chapters.iter().enumerate() {
        if let Some(item) = results_arr.get(i) {
            let is_valid = item.get("is_valid").and_then(|v| v.as_bool()).unwrap_or(true);
            let reason = item.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
            eprintln!("[validate_batch] Chapter {}: is_valid={}, reason={:?}", chapter.index, is_valid, reason);
            results.push((chapter.id.clone(), is_valid, reason));
        } else {
            // If AI returned fewer results, default remaining to valid
            eprintln!("[validate_batch] Chapter {}: no result from AI, defaulting to valid", chapter.index);
            results.push((chapter.id.clone(), true, None));
        }
    }

    Ok((results, output))
}

fn extract_json_from_response(text: &str) -> String {
    let trimmed = text.trim();

    if let Some(start) = trimmed.find("```json") {
        let rest = &trimmed[start + 7..];
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }

    if let Some(start) = trimmed.find("```") {
        let rest = &trimmed[start + 3..];
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }

    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            return trimmed[start..=end].to_string();
        }
    }

    trimmed.to_string()
}

pub(crate) fn update_chapter_validation(
    conn: &Connection,
    chapter_id: &str,
    is_valid: bool,
    reason: Option<&str>,
) -> Result<(), String> {
    eprintln!("[update_chapter_validation] chapter_id={}, is_valid={}", chapter_id, is_valid);
    crate::repositories::chapters::update_chapter_validation(
        conn,
        chapter_id,
        is_valid,
        reason,
        "completed",
    )
}
