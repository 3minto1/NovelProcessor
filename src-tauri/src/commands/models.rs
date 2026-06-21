use crate::domain::ModelProfile;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub(crate) async fn list_model_profiles(
    state: State<'_, AppState>,
) -> Result<Vec<ModelProfile>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, provider, base_url, model, temperature, top_p, thinking_mode,
                    CASE WHEN api_key IS NOT NULL AND api_key != '' THEN 1 ELSE 0 END as has_api_key,
                    'database' as api_key_storage, updated_at
             FROM model_profiles ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ModelProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                base_url: row.get(3)?,
                model: row.get(4)?,
                temperature: row.get(5)?,
                top_p: row.get(6)?,
                thinking_mode: row.get(7)?,
                has_api_key: row.get::<_, i64>(8)? != 0,
                api_key_storage: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn save_model_profile(
    state: State<'_, AppState>,
    name: String,
    provider: String,
    base_url: String,
    model: String,
    temperature: f64,
    top_p: f64,
    thinking_mode: String,
    api_key: Option<String>,
) -> Result<ModelProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO model_profiles (id, name, provider, base_url, model, temperature, top_p, thinking_mode, api_key, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, name, provider, base_url, model, temperature, top_p, thinking_mode, api_key, now],
    ).map_err(|e| e.to_string())?;
    
    Ok(ModelProfile {
        id,
        name,
        provider,
        base_url,
        model,
        temperature,
        top_p,
        thinking_mode,
        has_api_key: api_key.is_some(),
        api_key_storage: "database".to_string(),
        updated_at: now,
    })
}

#[tauri::command]
pub(crate) async fn delete_model_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM model_profiles WHERE id = ?1", rusqlite::params![profile_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
