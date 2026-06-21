use crate::domain::{Novel, Chapter};
use crate::text::{chapter_split, encoding};
use rusqlite::Connection;
use std::fs;
use uuid::Uuid;

pub(crate) fn import_txt(conn: &Connection, file_path: &str) -> Result<Novel, String> {
    let bytes = fs::read(file_path).map_err(|error| format!("无法读取文件：{}", error))?;
    let (text, encoding_label) = encoding::decode_text(&bytes);
    
    let title = std::path::Path::new(file_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("未知小说")
        .to_string();

    let novel_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    let split_result = chapter_split::split_chapters(&novel_id, &text);
    
    let novel = Novel {
        id: novel_id.clone(),
        title,
        source_path: file_path.to_string(),
        encoding: encoding_label,
        status: "imported".to_string(),
        detected_chapters: split_result.detected_chapters,
        created_at: now,
    };

    crate::repositories::novels::insert_novel(conn, &novel)?;
    
    for chapter in &split_result.chapters {
        crate::repositories::chapters::insert_chapter(conn, chapter)?;
    }

    create_batches(conn, &novel_id, &split_result.chapters)?;
    
    Ok(novel)
}

fn create_batches(
    conn: &Connection,
    novel_id: &str,
    chapters: &[Chapter],
) -> Result<(), String> {
    if chapters.is_empty() {
        return Ok(());
    }

    let batch_size = get_batch_size(conn)? as usize;
    let now = chrono::Utc::now().to_rfc3339();
    let mut batch_index = 0i64;
    let mut start = 0;

    while start < chapters.len() {
        let end = (start + batch_size).min(chapters.len());
        let start_chapter = chapters[start].index;
        let end_chapter = chapters[end - 1].index;
        let label = format!("第{}-{}章", start_chapter, end_chapter);
        
        let batch = crate::domain::ChapterBatch {
            id: Uuid::new_v4().to_string(),
            novel_id: novel_id.to_string(),
            batch_index,
            label,
            start_chapter,
            end_chapter,
            status: "pending".to_string(),
            created_at: now.clone(),
        };
        
        crate::repositories::batches::insert_batch(conn, &batch)?;
        batch_index += 1;
        start = end;
    }

    Ok(())
}

fn get_batch_size(conn: &Connection) -> Result<i64, String> {
    let result = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'chapter_batch_size'",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(value) => value.parse::<i64>().map_err(|e| e.to_string()),
        Err(_) => Ok(30),
    }
}
