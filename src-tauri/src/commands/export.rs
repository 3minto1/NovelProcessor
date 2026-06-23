use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub(crate) struct ExportResult {
    pub path: String,
}

#[tauri::command]
pub(crate) async fn export_novel(
    state: State<'_, AppState>,
    novel_id: String,
    output_dir: String,
) -> Result<ExportResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let path = crate::services::export::export_novel(&conn, &novel_id, &output_dir)?;
    Ok(ExportResult { path })
}
