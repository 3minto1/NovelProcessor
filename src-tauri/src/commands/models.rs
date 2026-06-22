use crate::domain::{AiLog, ModelDiagnosis, ModelProfile};
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
    input: ModelProfileInput,
) -> Result<ModelProfile, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let api_key = input.api_key.as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "********")
        .map(str::to_string);
    
    conn.execute(
        "INSERT INTO model_profiles (id, name, provider, base_url, model, temperature, top_p, thinking_mode, api_key, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            provider = excluded.provider,
            base_url = excluded.base_url,
            model = excluded.model,
            temperature = excluded.temperature,
            top_p = excluded.top_p,
            thinking_mode = excluded.thinking_mode,
            api_key = CASE
                WHEN ?9 IS NOT NULL THEN excluded.api_key
                ELSE model_profiles.api_key
            END,
            updated_at = excluded.updated_at",
        rusqlite::params![
            id,
            input.name,
            input.provider,
            input.base_url,
            input.model,
            input.temperature,
            input.top_p,
            input.thinking_mode,
            api_key,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;
    
    Ok(ModelProfile {
        id,
        name: input.name,
        provider: input.provider,
        base_url: input.base_url,
        model: input.model,
        temperature: input.temperature,
        top_p: input.top_p,
        thinking_mode: input.thinking_mode,
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

#[tauri::command]
pub(crate) async fn diagnose_model_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<ModelDiagnosis, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    let profile = conn.query_row(
        "SELECT id, name, provider, base_url, model, temperature, top_p, thinking_mode,
                CASE WHEN api_key IS NOT NULL AND api_key != '' THEN 1 ELSE 0 END as has_api_key,
                'database' as api_key_storage, updated_at
         FROM model_profiles WHERE id = ?1",
        rusqlite::params![profile_id],
        |row| {
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
        },
    ).map_err(|e| e.to_string())?;
    
    let mut checks = Vec::new();
    
    if profile.has_api_key {
        checks.push(crate::domain::DiagnosisCheck {
            name: "API Key".to_string(),
            status: "ok".to_string(),
            message: "API Key 已配置".to_string(),
        });
    } else {
        checks.push(crate::domain::DiagnosisCheck {
            name: "API Key".to_string(),
            status: "warning".to_string(),
            message: "API Key 未配置".to_string(),
        });
    }
    
    if !profile.base_url.is_empty() {
        checks.push(crate::domain::DiagnosisCheck {
            name: "Base URL".to_string(),
            status: "ok".to_string(),
            message: format!("Base URL: {}", profile.base_url),
        });
    } else {
        checks.push(crate::domain::DiagnosisCheck {
            name: "Base URL".to_string(),
            status: "failed".to_string(),
            message: "Base URL 未配置".to_string(),
        });
    }
    
    if !profile.model.is_empty() && profile.model != "请填写模型名" {
        checks.push(crate::domain::DiagnosisCheck {
            name: "模型名称".to_string(),
            status: "ok".to_string(),
            message: format!("模型: {}", profile.model),
        });
    } else {
        checks.push(crate::domain::DiagnosisCheck {
            name: "模型名称".to_string(),
            status: "failed".to_string(),
            message: "模型名称未配置".to_string(),
        });
    }
    
    let status = if checks.iter().all(|c| c.status == "ok") {
        "ok".to_string()
    } else if checks.iter().any(|c| c.status == "failed") {
        "failed".to_string()
    } else {
        "warning".to_string()
    };
    
    Ok(ModelDiagnosis {
        status,
        recommended_thinking_mode: None,
        checks,
    })
}

#[tauri::command]
pub(crate) async fn list_ai_logs(
    state: State<'_, AppState>,
    novel_id: Option<String>,
) -> Result<Vec<AiLog>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut rows = Vec::new();
    
    if let Some(id) = &novel_id {
        let mut stmt = conn.prepare(
            "SELECT id, novel_id, profile_id, action, chapter_title, status, content, reasoning, raw_response, created_at
             FROM ai_logs WHERE novel_id = ?1 ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;
        
        let mapped = stmt.query_map(rusqlite::params![id], |row| {
            Ok(AiLog {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                profile_id: row.get(2)?,
                action: row.get(3)?,
                chapter_title: row.get(4)?,
                status: row.get(5)?,
                content: row.get(6)?,
                reasoning: row.get(7)?,
                raw_response: row.get(8)?,
                input_tokens: None,
                output_tokens: None,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
        rows = mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, novel_id, profile_id, action, chapter_title, status, content, reasoning, raw_response, created_at
             FROM ai_logs ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;
        
        let mapped = stmt.query_map([], |row| {
            Ok(AiLog {
                id: row.get(0)?,
                novel_id: row.get(1)?,
                profile_id: row.get(2)?,
                action: row.get(3)?,
                chapter_title: row.get(4)?,
                status: row.get(5)?,
                content: row.get(6)?,
                reasoning: row.get(7)?,
                raw_response: row.get(8)?,
                input_tokens: None,
                output_tokens: None,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;
        
        rows = mapped.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    }
    
    Ok(rows)
}

#[tauri::command]
pub(crate) async fn clear_ai_logs(
    state: State<'_, AppState>,
    novel_id: Option<String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    match &novel_id {
        Some(id) => {
            conn.execute("DELETE FROM ai_logs WHERE novel_id = ?1", rusqlite::params![id])
                .map_err(|e| e.to_string())?;
        }
        None => {
            conn.execute("DELETE FROM ai_logs", [])
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub temperature: f64,
    pub top_p: f64,
    pub thinking_mode: String,
    pub api_key: Option<String>,
}
