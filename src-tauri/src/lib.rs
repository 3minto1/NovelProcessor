mod ai;
mod commands;
mod credentials;
mod db;
mod domain;
mod model_support;
mod rate_limit;
mod repositories;
mod services;
mod task_control;
mod text;

use rusqlite::Connection;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = "novel_processor.db";
    
    let conn = Connection::open(db_path).expect("Failed to open database");
    db::schema::init_db(&conn).expect("Failed to initialize database");
    
    let state = AppState {
        db: Mutex::new(conn),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::novels::list_novels,
            commands::novels::get_novel_detail,
            commands::novels::import_txt,
            commands::novels::delete_novel,
            commands::validate::start_validation,
            commands::review::start_review,
            commands::export::export_novel,
            commands::models::list_model_profiles,
            commands::models::save_model_profile,
            commands::models::delete_model_profile,
            commands::models::diagnose_model_profile,
            commands::models::list_ai_logs,
            commands::models::clear_ai_logs,
            commands::jobs::get_job,
            commands::settings::get_app_settings,
            commands::settings::save_app_settings,
            commands::settings::save_selected_profile_id,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
