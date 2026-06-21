use crate::domain::Chapter;
use rusqlite::Connection;
use std::fs;
use std::path::Path;

pub(crate) fn export_novel(
    conn: &Connection,
    novel_id: &str,
    output_dir: &str,
) -> Result<String, String> {
    let chapters = crate::repositories::chapters::list_chapters(conn, novel_id)?;
    let novel = crate::repositories::novels::get_novel(conn, novel_id)?;
    
    let valid_chapters: Vec<&Chapter> = chapters
        .iter()
        .filter(|c| c.is_valid)
        .collect();
    
    if valid_chapters.is_empty() {
        return Err("没有可导出的有效章节".to_string());
    }
    
    let mut content = String::new();
    for chapter in &valid_chapters {
        let text = chapter
            .corrected_text
            .as_deref()
            .unwrap_or(&chapter.original_text);
        content.push_str(&chapter.title);
        content.push_str("\n\n");
        content.push_str(text);
        content.push_str("\n\n");
    }
    
    let output_path = Path::new(output_dir).join(format!("{}.txt", novel.title));
    fs::write(&output_path, &content)
        .map_err(|error| format!("无法写入导出文件：{}", error))?;
    
    Ok(output_path.to_string_lossy().to_string())
}
