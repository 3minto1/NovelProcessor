use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn export_novel(
    state: State<'_, AppState>,
    novel_id: String,
    output_dir: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::services::export::export_novel(&conn, &novel_id, &output_dir)
}
