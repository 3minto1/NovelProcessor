use crate::domain::{Chapter, ModelProfile, Job};
use crate::ai::prompts::build_batch_review_prompt;
use rusqlite::Connection;
use uuid::Uuid;
use reqwest::Client;

pub(crate) fn start_review(
    conn: &Connection,
    novel_id: &str,
    _profile_id: &str,
) -> Result<Job, String> {
    let chapters = crate::repositories::chapters::list_chapters(conn, novel_id)?;
    let valid_chapters: Vec<&Chapter> = chapters.iter().filter(|c| c.is_valid).collect();
    let total = valid_chapters.len() as i64;
    
    let job_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let job = Job {
        id: job_id.clone(),
        novel_id: novel_id.to_string(),
        job_type: "review".to_string(),
        status: "running".to_string(),
        current_chapter: 0,
        total_chapters: total,
        message: "开始审查章节...".to_string(),
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

pub(crate) async fn review_batch(
    client: &Client,
    profile: &ModelProfile,
    api_key: &str,
    chapters: &[Chapter],
) -> Result<Vec<(String, String)>, String> {
    let prompt = build_batch_review_prompt(chapters);
    let system = "你是一位专业的小说编辑和校对员，专注于修正错别字、删除无关内容和修复语法问题。";
    
    let output = crate::ai::common::generate_text(
        client,
        None,
        profile,
        api_key,
        system,
        &prompt,
        false,
    ).await?;
    
    parse_review_output(&output.text, chapters)
}

fn parse_review_output(
    output: &str,
    expected_chapters: &[Chapter],
) -> Result<Vec<(String, String)>, String> {
    let mut results = Vec::new();
    let mut remaining = output.to_string();
    
    for chapter in expected_chapters {
        let start_marker = format!("<<<CHAPTER_START index={}>>>", chapter.index);
        let end_marker = format!("<<<CHAPTER_END index={}>>>", chapter.index);
        
        let start_pos = remaining.find(&start_marker)
            .ok_or_else(|| format!("AI输出缺少章节{}的开始标记", chapter.index))?;
        let after_start = &remaining[start_pos + start_marker.len()..];
        
        let end_pos = after_start.find(&end_marker)
            .ok_or_else(|| format!("AI输出缺少章节{}的结束标记", chapter.index))?;
        
        let corrected_text = after_start[..end_pos].trim().to_string();
        results.push((chapter.id.clone(), corrected_text));
        
        remaining = after_start[end_pos + end_marker.len()..].to_string();
    }
    
    Ok(results)
}

pub(crate) fn update_chapter_review(
    conn: &Connection,
    chapter_id: &str,
    corrected_text: &str,
) -> Result<(), String> {
    crate::repositories::chapters::update_chapter_review(
        conn,
        chapter_id,
        corrected_text,
        "completed",
    )
}
