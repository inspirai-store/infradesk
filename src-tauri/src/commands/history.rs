//! Tauri commands for query history management
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{QueryHistory, QueryHistoryListResponse, AddQueryHistoryRequest};
use crate::db::SqlitePool;
use crate::error::AppError;

/// Get query history with optional filters
#[tauri::command]
pub async fn get_history(
    pool: State<'_, SqlitePool>,
    conn_type: Option<String>,
    database: Option<String>,
    status: Option<String>,
    keyword: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<QueryHistoryListResponse, AppError> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let (history, total) = pool.get_query_history(
        conn_type.as_deref(),
        database.as_deref(),
        status.as_deref(),
        keyword.as_deref(),
        limit,
        offset,
    ).await?;

    Ok(QueryHistoryListResponse { history, total })
}

/// Add a query history entry
#[tauri::command]
pub async fn add_history(
    pool: State<'_, SqlitePool>,
    data: AddQueryHistoryRequest,
) -> Result<QueryHistory, AppError> {
    pool.add_query_history(&data).await
}

/// Delete a query history entry
#[tauri::command]
pub async fn delete_history(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), AppError> {
    pool.delete_query_history(id).await
}

/// Cleanup old query history entries
#[tauri::command]
pub async fn cleanup_history(
    pool: State<'_, SqlitePool>,
    days: i64,
) -> Result<i64, AppError> {
    pool.cleanup_query_history(days).await
}
