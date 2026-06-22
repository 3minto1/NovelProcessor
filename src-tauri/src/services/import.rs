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

    // Use a transaction for better performance
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    
    tx.execute(
        "INSERT INTO novels (id, title, source_path, encoding, status, detected_chapters, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            novel.id,
            novel.title,
            novel.source_path,
            novel.encoding,
            novel.status,
            novel.detected_chapters as i64,
            novel.created_at,
        ],
    )
    .map_err(|e| e.to_string())?;
    
    for chapter in &split_result.chapters {
        tx.execute(
            "INSERT INTO chapters (id, novel_id, chapter_index, title, original_text, is_valid,
                                  validation_reason, corrected_text, validation_status, review_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                chapter.id,
                chapter.novel_id,
                chapter.index,
                chapter.title,
                chapter.original_text,
                chapter.is_valid as i64,
                chapter.validation_reason,
                chapter.corrected_text,
                chapter.validation_status,
                chapter.review_status,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    create_batches_in_tx(&tx, &novel_id, &split_result.chapters)?;
    
    tx.commit().map_err(|e| e.to_string())?;
    
    Ok(novel)
}

fn create_batches_in_tx(
    tx: &rusqlite::Transaction,
    novel_id: &str,
    chapters: &[Chapter],
) -> Result<(), String> {
    if chapters.is_empty() {
        return Ok(());
    }

    let batch_size = 30;
    let now = chrono::Utc::now().to_rfc3339();
    let mut batch_index = 0i64;
    let mut start = 0;

    while start < chapters.len() {
        let end = (start + batch_size).min(chapters.len());
        let start_chapter = chapters[start].index;
        let end_chapter = chapters[end - 1].index;
        let label = format!("第{}-{}章", start_chapter, end_chapter);
        
        tx.execute(
            "INSERT INTO chapter_batches (id, novel_id, batch_index, label, start_chapter, end_chapter, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                novel_id,
                batch_index,
                label,
                start_chapter,
                end_chapter,
                "pending",
                now,
            ],
        )
        .map_err(|e| e.to_string())?;
        
        batch_index += 1;
        start = end;
    }

    Ok(())
}
