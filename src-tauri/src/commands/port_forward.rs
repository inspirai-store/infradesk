//! Tauri commands for port forwarding management
//!
//! These commands are exposed to the frontend via IPC for managing
//! Kubernetes port forwarding to database services.

use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::db::models::PortForward;
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::PortForwardService;

/// Shared state for port forward service
/// This ensures the service state persists across all commands
pub struct PortForwardState {
    service: Arc<RwLock<Option<PortForwardService>>>,
    pool: Arc<RwLock<Option<SqlitePool>>>,
}

impl PortForwardState {
    pub fn new() -> Self {
        Self {
            service: Arc::new(RwLock::new(None)),
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// Get or create the port forward service
    pub async fn get_or_init(&self, pool: SqlitePool) -> Arc<RwLock<Option<PortForwardService>>> {
        {
            let mut pool_guard = self.pool.write().await;
            if pool_guard.is_none() {
                *pool_guard = Some(pool.clone());
            }
        }

        {
            let mut service_guard = self.service.write().await;
            if service_guard.is_none() {
                *service_guard = Some(PortForwardService::new(pool));
            }
        }

        self.service.clone()
    }
}

impl Default for PortForwardState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start a port forward for a connection
///
/// # Arguments
/// * `connection_id` - The connection ID to forward
/// * `local_port` - Optional preferred local port. If None or 0, auto-assign an available port.
#[tauri::command]
pub async fn start_port_forward(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    local_port: Option<u16>,
) -> Result<PortForward, AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.start(connection_id, local_port).await
}

/// Stop a port forward
#[tauri::command]
pub async fn stop_port_forward(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    id: String,
) -> Result<(), AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.stop(&id).await
}

/// Get all port forwards
#[tauri::command]
pub async fn list_port_forwards(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
) -> Result<Vec<PortForward>, AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.list().await
}

/// Get a port forward by ID
#[tauri::command]
pub async fn get_port_forward(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    id: String,
) -> Result<PortForward, AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.get(&id).await
}

/// Get port forward by connection ID
#[tauri::command]
pub async fn get_port_forward_by_connection(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
) -> Result<PortForward, AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.get_by_connection(connection_id).await
}

/// Reconnect a port forward
///
/// # Arguments
/// * `id` - The port forward ID to reconnect
/// * `local_port` - Optional preferred local port. If None, reuse the existing port.
#[tauri::command]
pub async fn reconnect_port_forward(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    id: String,
    local_port: Option<u16>,
) -> Result<PortForward, AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.reconnect(&id, local_port).await
}

/// Touch a port forward (update last used time)
#[tauri::command]
pub async fn touch_port_forward(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    id: String,
) -> Result<(), AppError> {
    let service_arc = pf_state.get_or_init(pool.inner().clone()).await;
    let guard = service_arc.read().await;
    let service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;
    service.touch(&id).await
}
