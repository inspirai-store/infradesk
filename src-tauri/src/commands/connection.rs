//! Tauri commands for connection management
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{Connection, TestConnectionResult};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::ConnectionService;

/// Get all connections
#[tauri::command]
pub async fn get_all_connections(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Connection>, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.get_all().await
}

/// Get a single connection by ID
#[tauri::command]
pub async fn get_connection(pool: State<'_, SqlitePool>, id: i64) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.get_by_id(id).await
}

/// Get connections by type
#[tauri::command]
pub async fn get_connections_by_type(
    pool: State<'_, SqlitePool>,
    conn_type: String,
) -> Result<Vec<Connection>, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.get_by_type(&conn_type).await
}

/// Create a new connection
#[tauri::command]
pub async fn create_connection(
    pool: State<'_, SqlitePool>,
    data: Connection,
) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.create(data).await
}

/// Update an existing connection
#[tauri::command]
pub async fn update_connection(
    pool: State<'_, SqlitePool>,
    id: i64,
    data: Connection,
) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.update(id, data).await
}

/// Delete a connection
#[tauri::command]
pub async fn delete_connection(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.delete(id).await
}

/// Test a connection without saving
#[tauri::command]
pub async fn test_connection(
    pool: State<'_, SqlitePool>,
    data: Connection,
) -> Result<TestConnectionResult, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.test(&data).await
}
