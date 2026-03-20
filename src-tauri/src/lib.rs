// Scaffold phase: utility modules are not yet called from commands.
#![allow(dead_code)]

mod checksum;
mod classify;
mod commands;
mod compress;
mod credentials;
mod db;
mod dump;
mod ensure_db;
mod error;
mod progress;
mod ssh;

use crate::db::{init_db, AppPaths, DbState};
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = fix_path_env::fix();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let conn = init_db(&app_data_dir).expect("failed to initialise database");

            app.manage(DbState(Mutex::new(conn)));
            app.manage(AppPaths {
                data_dir: app_data_dir.clone(),
                snapshots_dir: app_data_dir.join("snapshots"),
                tmp_dir: app_data_dir.join("tmp"),
            });

            // Ensure directories exist
            std::fs::create_dir_all(app_data_dir.join("snapshots"))?;
            std::fs::create_dir_all(app_data_dir.join("tmp"))?;

            // Clean tmp directory on startup
            if let Ok(entries) = std::fs::read_dir(app_data_dir.join("tmp")) {
                for entry in entries.flatten() {
                    let _ = std::fs::remove_file(entry.path());
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::profile_create,
            commands::profile_update,
            commands::profile_delete,
            commands::profile_list,
            commands::profile_test_connection,
            commands::snapshot_create,
            commands::snapshot_list,
            commands::snapshot_restore,
            commands::snapshot_delete,
            commands::snapshot_delete_by_project,
            commands::snapshot_verify_integrity,
            commands::restore_history_list,
            commands::get_snapshots_dir,
            commands::storage_usage,
            commands::settings_list,
            commands::settings_set,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
