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
    crate::repositories::chapters::update_chapter_text(&conn, &chapter_id, &title, &original_text)
}

#[tauri::command]
pub(crate) async fn delete_chapter(
    state: State<'_, AppState>,
    chapter_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::repositories::chapters::delete_chapter(&conn, &chapter_id)
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
