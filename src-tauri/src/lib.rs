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
    pub validation_task: task_control::TaskControl,
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

            let _ = conn.execute(
                "UPDATE jobs SET status = 'failed', message = message || ' [应用重启后终止]'
                 WHERE status = 'running'",
                [],
            );

            let state = AppState {
                db: Mutex::new(conn),
                db_path: db_path_str,
                validation_task: task_control::TaskControl::new(),
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
            commands::novels::delete_chapters_batch,
            commands::novels::toggle_chapter_validity,
            commands::novels::export_chapter_directory,
            commands::validate::start_validation,
            commands::validate::cancel_validation,
            commands::validate::is_validation_active,
            commands::validate::is_task_paused,
            commands::review::start_review,
            commands::review::cancel_review,
            commands::review::pause_review,
            commands::review::resume_review,
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
