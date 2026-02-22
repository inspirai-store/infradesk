//! Standalone HTTP server for web debug mode
//!
//! This binary runs the HTTP API server independently without Tauri,
//! allowing it to start before the frontend for better startup sequencing.

use std::path::PathBuf;

use infradesk_lib::db::SqlitePool;
use infradesk_lib::http::create_router;
use infradesk_lib::services::PortForwardService;

fn get_db_path() -> PathBuf {
    // Use the same path as Tauri would use
    let home = dirs::home_dir().expect("Failed to get home directory");
    let app_data_dir = if cfg!(target_os = "macos") {
        home.join("Library/Application Support/com.infradesk.app")
    } else if cfg!(target_os = "windows") {
        home.join("AppData/Roaming/com.infradesk.app")
    } else {
        home.join(".local/share/com.infradesk.app")
    };

    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
    app_data_dir.join("infradesk.db")
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting InfraDesk Web Server...");

    // Initialize database
    let db_path = get_db_path();
    log::info!("Database path: {:?}", db_path);

    let pool = SqlitePool::new(&db_path)
        .await
        .expect("Failed to initialize database");
    log::info!("Database initialized successfully");

    // Initialize port forward service
    let pf_service = PortForwardService::new(pool.clone());

    // Create router
    let router = create_router(pool, pf_service);

    // Start HTTP server
    let addr = "127.0.0.1:12420";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind HTTP server");

    log::info!("HTTP server started: http://{}", addr);
    log::info!("Health check: http://{}/api/health", addr);
    log::info!("");
    log::info!("Waiting for frontend to start...");

    axum::serve(listener, router)
        .await
        .expect("Server error");
}
