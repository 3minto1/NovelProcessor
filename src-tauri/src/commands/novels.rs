use crate::domain::{Novel, NovelDetail};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn list_novels(state: State<'_, AppState>) -> Result<Vec<Novel>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::repositories::novels::list_novels(&conn)
}

#[tauri::command]
pub(crate) async fn get_novel_detail(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<NovelDetail, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::repositories::novels::get_novel_detail(&conn, &novel_id)
}

#[tauri::command]
pub(crate) async fn import_txt(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Novel, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::services::import::import_txt(&conn, &file_path)
}

#[tauri::command]
pub(crate) async fn delete_novel(
    state: State<'_, AppState>,
    novel_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::repositories::novels::delete_novel(&conn, &novel_id)
}

#[tauri::command]
pub(crate) async fn update_chapter_text(
    state: State<'_, AppState>,
    chapter_id: String,
    title: String,
    original_text: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let old_title: String = conn
        .query_row(
            "SELECT title FROM chapters WHERE id = ?1",
            rusqlite::params![chapter_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let synced_text = if old_title != title && !old_title.trim().is_empty() {
        original_text.replacen(old_title.trim(), title.trim(), 1)
    } else {
        original_text
    };
    crate::repositories::chapters::update_chapter_text(&conn, &chapter_id, &title, &synced_text)
}

#[tauri::command]
pub(crate) async fn delete_chapter(
    state: State<'_, AppState>,
    chapter_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let novel_id: String = conn.query_row(
        "SELECT novel_id FROM chapters WHERE id = ?1",
        rusqlite::params![chapter_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    crate::repositories::chapters::delete_chapter(&conn, &chapter_id)?;
    crate::repositories::chapters::reindex_chapters(&conn, &novel_id)?;
    Ok(())
}

#[tauri::command]
pub(crate) async fn delete_chapters_batch(
    state: State<'_, AppState>,
    chapter_ids: Vec<String>,
) -> Result<(), String> {
    if chapter_ids.is_empty() {
        return Ok(());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let novel_id: String = conn.query_row(
        "SELECT novel_id FROM chapters WHERE id = ?1",
        rusqlite::params![chapter_ids[0]],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    for id in &chapter_ids {
        let _ = crate::repositories::chapters::delete_chapter(&conn, id);
    }
    crate::repositories::chapters::reindex_chapters(&conn, &novel_id)?;
    Ok(())
}

#[tauri::command]
pub(crate) async fn toggle_chapter_validity(
    state: State<'_, AppState>,
    chapter_id: String,
    is_valid: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::repositories::chapters::toggle_chapter_validity(&conn, &chapter_id, is_valid)
}

#[tauri::command]
pub(crate) async fn export_chapter_directory(
    state: State<'_, AppState>,
    novel_id: String,
    output_path: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let chapters = crate::repositories::chapters::list_chapters(&conn, &novel_id)?;
    
    let content: String = chapters
        .iter()
        .map(|ch| {
            let title = ch.title.trim();
            if title.is_empty() {
                format!("第{}章", ch.index)
            } else {
                format!("{}. {}", ch.index, title)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    std::fs::write(&output_path, &content)
        .map_err(|error| format!("无法写入文件：{}", error))?;
    
    Ok(())
}
