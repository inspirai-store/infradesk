//! Tauri commands for Redis operations
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::commands::PortForwardState;
use crate::db::models::{
    Connection, RedisExportData, RedisKeyListResponse, RedisKeyValue, RedisServerInfo,
    SetKeyRequest,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{ConnectionService, RedisService};

/// Helper to get connection and create Redis service
/// For K8s connections, this will automatically start or use existing port forward
async fn get_redis_service(
    pool: &SqlitePool,
    pf_state: &PortForwardState,
    connection_id: i64,
) -> Result<RedisService, AppError> {
    let service = ConnectionService::new(pool.clone());
    let mut conn = service.get_by_id(connection_id).await?;

    if conn.conn_type != "redis" {
        return Err(AppError::Validation(
            "Connection is not Redis type".to_string(),
        ));
    }

    // For K8s connections, we need to ensure port forward is active
    if conn.source.as_deref() == Some("k8s") {
        conn = ensure_port_forward(pool, pf_state, conn).await?;
    }

    RedisService::connect(&conn).await
}

/// Ensure port forward is active for K8s connection
async fn ensure_port_forward(
    pool: &SqlitePool,
    pf_state: &PortForwardState,
    mut conn: Connection,
) -> Result<Connection, AppError> {
    let connection_id = conn.id.ok_or_else(|| {
        AppError::Validation("Connection ID is required".to_string())
    })?;

    let service_arc = pf_state.get_or_init(pool.clone()).await;
    let guard = service_arc.read().await;
    let pf_service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;

    // Try to get existing port forward for this connection
    let pf = match pf_service.get_by_connection(connection_id).await {
        Ok(existing) => {
            if existing.status == "active" {
                log::info!(
                    "Using existing port forward for Redis connection {}: localhost:{}",
                    connection_id,
                    existing.local_port
                );
                existing
            } else {
                log::info!(
                    "Reconnecting port forward for Redis connection {}",
                    connection_id
                );
                let local_port = conn.forward_local_port.map(|p| p as u16);
                pf_service.reconnect(&existing.id.unwrap_or_default(), local_port).await?
            }
        }
        Err(_) => {
            log::info!("Starting new port forward for Redis connection {}", connection_id);
            let local_port = conn.forward_local_port.map(|p| p as u16);
            pf_service.start(connection_id, local_port).await?
        }
    };

    conn.host = "127.0.0.1".to_string();
    conn.port = pf.local_port;

    Ok(conn)
}

/// Get Redis server info
#[tauri::command]
pub async fn redis_get_info(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
) -> Result<RedisServerInfo, AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.get_info().await
}

/// List keys with pattern and cursor-based pagination
#[tauri::command]
pub async fn redis_list_keys(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    pattern: Option<String>,
    cursor: Option<u64>,
    count: Option<u64>,
) -> Result<RedisKeyListResponse, AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    let pattern = pattern.unwrap_or_default();
    let cursor = cursor.unwrap_or(0);
    let count = count.unwrap_or(100);
    redis.list_keys(&pattern, cursor, count).await
}

/// Get key value
#[tauri::command]
pub async fn redis_get_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    key: String,
) -> Result<RedisKeyValue, AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.get_key(&key).await
}

/// Set a key
#[tauri::command]
pub async fn redis_set_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: SetKeyRequest,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.set_key(&data).await
}

/// Update a key
#[tauri::command]
pub async fn redis_update_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    key: String,
    data: SetKeyRequest,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.update_key(&key, &data).await
}

/// Delete a key
#[tauri::command]
pub async fn redis_delete_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    key: String,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.delete_key(&key).await
}

/// Set TTL for a key
#[tauri::command]
pub async fn redis_set_ttl(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    key: String,
    ttl: i64,
) -> Result<(), AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.set_ttl(&key, ttl).await
}

/// Export keys
#[tauri::command]
pub async fn redis_export_keys(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    keys: Vec<String>,
) -> Result<RedisExportData, AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.export_keys(&keys).await
}

/// Import keys
#[tauri::command]
pub async fn redis_import_keys(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: RedisExportData,
) -> Result<i32, AppError> {
    let mut redis = get_redis_service(pool.inner(), &pf_state, connection_id).await?;
    redis.import_keys(&data).await
}
