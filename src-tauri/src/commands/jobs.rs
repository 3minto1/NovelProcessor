use crate::domain::Job;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn get_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Job, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::services::progress::get_job(&conn, &job_id)
}
