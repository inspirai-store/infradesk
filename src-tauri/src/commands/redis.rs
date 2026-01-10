//! Tauri commands for Redis operations
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{
    RedisExportData, RedisKeyListResponse, RedisKeyValue, RedisServerInfo, SetKeyRequest,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{ConnectionService, RedisService};

/// Helper to get connection and create Redis service
async fn get_redis_service(
    pool: &SqlitePool,
    connection_id: i64,
) -> Result<RedisService, AppError> {
    let service = ConnectionService::new(pool.clone());
    let conn = service.get_by_id(connection_id).await?;

    if conn.conn_type != "redis" {
        return Err(AppError::Validation(
            "Connection is not Redis type".to_string(),
        ));
    }

    RedisService::connect(&conn).await
}

/// Get Redis server info
#[tauri::command]
pub async fn redis_get_info(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<RedisServerInfo, AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.get_info().await
}

/// List keys with pattern and cursor-based pagination
#[tauri::command]
pub async fn redis_list_keys(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    pattern: Option<String>,
    cursor: Option<u64>,
    count: Option<u64>,
) -> Result<RedisKeyListResponse, AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    let pattern = pattern.unwrap_or_default();
    let cursor = cursor.unwrap_or(0);
    let count = count.unwrap_or(100);
    redis.list_keys(&pattern, cursor, count).await
}

/// Get key value
#[tauri::command]
pub async fn redis_get_key(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    key: String,
) -> Result<RedisKeyValue, AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.get_key(&key).await
}

/// Set a key
#[tauri::command]
pub async fn redis_set_key(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    data: SetKeyRequest,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.set_key(&data).await
}

/// Update a key
#[tauri::command]
pub async fn redis_update_key(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    key: String,
    data: SetKeyRequest,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.update_key(&key, &data).await
}

/// Delete a key
#[tauri::command]
pub async fn redis_delete_key(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    key: String,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.delete_key(&key).await
}

/// Set TTL for a key
#[tauri::command]
pub async fn redis_set_ttl(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    key: String,
    ttl: i64,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.set_ttl(&key, ttl).await
}

/// Export keys
#[tauri::command]
pub async fn redis_export_keys(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    keys: Vec<String>,
) -> Result<RedisExportData, AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.export_keys(&keys).await
}

/// Import keys
#[tauri::command]
pub async fn redis_import_keys(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    data: RedisExportData,
) -> Result<i32, AppError> {
    let mut redis = get_redis_service(pool.inner(), connection_id).await?;
    redis.import_keys(&data).await
}
