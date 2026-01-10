//! Zeni-X Tauri Backend
//!
//! This is the Rust backend for the Zeni-X desktop application.
//! It provides local database storage and direct database connections.

mod commands;
mod db;
mod error;
mod services;

use std::path::PathBuf;
use tauri::Manager;

use db::SqlitePool;

/// Get the application data directory for database storage
fn get_app_data_dir(app: &tauri::App) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Failed to get app data directory")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize logging in debug mode
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize SQLite database synchronously
            let app_data_dir = get_app_data_dir(app);
            let db_path = app_data_dir.join("zeni-x.db");

            log::info!("Database path: {:?}", db_path);

            // Use block_on to initialize database synchronously
            // This ensures the pool is available before any commands are called
            let pool = tauri::async_runtime::block_on(async {
                SqlitePool::new(&db_path).await
            });

            match pool {
                Ok(pool) => {
                    log::info!("SQLite database initialized successfully");
                    app.manage(pool);
                }
                Err(e) => {
                    log::error!("Failed to initialize SQLite database: {}", e);
                    return Err(e.to_string().into());
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection management
            commands::get_all_connections,
            commands::get_connection,
            commands::get_connections_by_type,
            commands::create_connection,
            commands::update_connection,
            commands::delete_connection,
            commands::test_connection,
            // Cluster management
            commands::get_all_clusters,
            commands::get_cluster,
            commands::create_cluster,
            commands::update_cluster,
            commands::delete_cluster,
            commands::get_cluster_connections,
            // K8s operations
            commands::k8s_discover,
            commands::k8s_list_clusters,
            commands::k8s_import_connections,
            // MySQL operations
            commands::mysql_get_info,
            commands::mysql_list_databases,
            commands::mysql_create_database,
            commands::mysql_alter_database,
            commands::mysql_drop_database,
            commands::mysql_list_tables,
            commands::mysql_drop_table,
            commands::mysql_get_table_schema,
            commands::mysql_get_table_primary_key,
            commands::mysql_execute_query,
            commands::mysql_get_rows,
            commands::mysql_insert_row,
            commands::mysql_update_record,
            commands::mysql_delete_row,
            commands::mysql_list_users,
            commands::mysql_create_user,
            commands::mysql_grant_privileges,
            // Redis operations
            commands::redis_get_info,
            commands::redis_list_keys,
            commands::redis_get_key,
            commands::redis_set_key,
            commands::redis_update_key,
            commands::redis_delete_key,
            commands::redis_set_ttl,
            commands::redis_export_keys,
            commands::redis_import_keys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
