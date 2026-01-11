//! Standalone HTTP server for Zeni-X
//!
//! This binary provides HTTP API endpoints for web-only development mode,
//! allowing frontend debugging without Tauri.
//!
//! Usage:
//!     cargo run --bin zeni-x-server
//!
//! Environment variables:
//!     SERVER_PORT     - HTTP server port (default: 15080)
//!     SQLITE_PATH     - SQLite database path (default: ./data/zeni-x.db)

use std::net::SocketAddr;
use std::path::PathBuf;

use zeni_x_lib::{db::SqlitePool, http::create_router, services::PortForwardService};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Get configuration from environment
    let port: u16 = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(15080);

    let db_path = std::env::var("SQLITE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data/zeni-x.db"));

    log::info!("Starting Zeni-X HTTP server...");
    log::info!("Database path: {:?}", db_path);
    log::info!("Server port: {}", port);

    // Ensure database directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    // Initialize SQLite database
    let pool = SqlitePool::new(&db_path)
        .await
        .expect("Failed to initialize SQLite database");

    log::info!("SQLite database initialized successfully");

    // Create port forward service
    let pf_service = PortForwardService::new(pool.clone());
    log::info!("Port forward service initialized");

    // Create router
    let app = create_router(pool, pf_service);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    log::info!("HTTP server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
