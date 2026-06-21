use crate::domain::AppSettings;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub(crate) async fn get_app_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    let export_dir = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'export_dir'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .unwrap_or(None);
    
    let selected_profile_id = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'selected_profile_id'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .unwrap_or(None);
    
    let chapter_batch_size = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'chapter_batch_size'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .unwrap_or(None)
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(30);
    
    Ok(AppSettings {
        export_dir,
        selected_profile_id,
        chapter_batch_size,
    })
}

#[tauri::command]
pub(crate) async fn save_app_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    if let Some(export_dir) = &settings.export_dir {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('export_dir', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [export_dir],
        ).map_err(|e| e.to_string())?;
    }
    
    if let Some(profile_id) = &settings.selected_profile_id {
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('selected_profile_id', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [profile_id],
        ).map_err(|e| e.to_string())?;
    }
    
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('chapter_batch_size', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [settings.chapter_batch_size.to_string()],
    ).map_err(|e| e.to_string())?;
    
    get_app_settings_inner(&conn)
}

fn get_app_settings_inner(conn: &rusqlite::Connection) -> Result<AppSettings, String> {
    let export_dir = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'export_dir'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .unwrap_or(None);
    
    let selected_profile_id = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'selected_profile_id'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .unwrap_or(None);
    
    let chapter_batch_size = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'chapter_batch_size'",
            [],
            |row| row.get::<_, Option<String>>(0),
        )
        .unwrap_or(None)
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(30);
    
    Ok(AppSettings {
        export_dir,
        selected_profile_id,
        chapter_batch_size,
    })
}

#[tauri::command]
pub(crate) async fn save_selected_profile_id(
    state: State<'_, AppState>,
    profile_id: Option<String>,
) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('selected_profile_id', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [profile_id.as_deref().unwrap_or("")],
    ).map_err(|e| e.to_string())?;
    
    get_app_settings_inner(&conn)
}
