//! Tauri commands for cluster management
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{Cluster, Connection};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::ClusterService;

/// Get all clusters
#[tauri::command]
pub async fn get_all_clusters(pool: State<'_, SqlitePool>) -> Result<Vec<Cluster>, AppError> {
    let service = ClusterService::new(pool.inner().clone());
    service.get_all().await
}

/// Get cluster by ID
#[tauri::command]
pub async fn get_cluster(pool: State<'_, SqlitePool>, id: i64) -> Result<Cluster, AppError> {
    let service = ClusterService::new(pool.inner().clone());
    service.get_by_id(id).await
}

/// Create a new cluster
#[tauri::command]
pub async fn create_cluster(
    pool: State<'_, SqlitePool>,
    data: Cluster,
) -> Result<Cluster, AppError> {
    let service = ClusterService::new(pool.inner().clone());
    service.create(&data).await
}

/// Update an existing cluster
#[tauri::command]
pub async fn update_cluster(
    pool: State<'_, SqlitePool>,
    id: i64,
    data: Cluster,
) -> Result<Cluster, AppError> {
    let service = ClusterService::new(pool.inner().clone());
    service.update(id, &data).await
}

/// Delete a cluster
#[tauri::command]
pub async fn delete_cluster(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    let service = ClusterService::new(pool.inner().clone());
    service.delete(id).await
}

/// Get connections associated with a cluster
#[tauri::command]
pub async fn get_cluster_connections(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
) -> Result<Vec<Connection>, AppError> {
    let service = ClusterService::new(pool.inner().clone());
    service.get_connections(cluster_id).await
}
