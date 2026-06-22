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
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
            
            let db_path = app_dir.join("novel_processor.db");
            let db_path_str = db_path.to_string_lossy().to_string();
            let conn = Connection::open(&db_path).expect("Failed to open database");
            db::schema::init_db(&conn).expect("Failed to initialize database");
            
            let state = AppState {
                db: Mutex::new(conn),
                db_path: db_path_str,
            };
            app.manage(state);
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::novels::list_novels,
            commands::novels::get_novel_detail,
            commands::novels::import_txt,
            commands::novels::delete_novel,
            commands::novels::update_chapter_text,
            commands::novels::delete_chapter,
            commands::novels::toggle_chapter_validity,
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
