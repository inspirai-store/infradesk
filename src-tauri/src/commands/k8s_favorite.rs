//! K8s favorites Tauri commands
//!
//! This module provides IPC commands for managing K8s cluster/namespace favorites.

use tauri::State;

use crate::db::models::{
    CreateK8sFavoriteRequest, K8sFavorite, K8sFavoriteWithCluster, UpdateK8sFavoriteRequest,
};
use crate::db::SqlitePool;

/// Get all K8s favorites with cluster info
#[tauri::command]
pub async fn get_k8s_favorites(
    pool: State<'_, SqlitePool>,
    category: Option<String>,
) -> Result<Vec<K8sFavoriteWithCluster>, String> {
    pool.get_k8s_favorites(category.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// Get a single K8s favorite by ID
#[tauri::command]
pub async fn get_k8s_favorite(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<K8sFavorite, String> {
    pool.get_k8s_favorite(id)
        .await
        .map_err(|e| e.to_string())
}

/// Check if a K8s favorite exists for the given cluster and namespace
#[tauri::command]
pub async fn k8s_favorite_exists(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Option<K8sFavorite>, String> {
    pool.k8s_favorite_exists(cluster_id, &namespace)
        .await
        .map_err(|e| e.to_string())
}

/// Create a new K8s favorite
#[tauri::command]
pub async fn create_k8s_favorite(
    pool: State<'_, SqlitePool>,
    request: CreateK8sFavoriteRequest,
) -> Result<K8sFavorite, String> {
    pool.create_k8s_favorite(&request)
        .await
        .map_err(|e| e.to_string())
}

/// Update an existing K8s favorite
#[tauri::command]
pub async fn update_k8s_favorite(
    pool: State<'_, SqlitePool>,
    id: i64,
    request: UpdateK8sFavoriteRequest,
) -> Result<K8sFavorite, String> {
    pool.update_k8s_favorite(id, &request)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a K8s favorite
#[tauri::command]
pub async fn delete_k8s_favorite(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    pool.delete_k8s_favorite(id)
        .await
        .map_err(|e| e.to_string())
}
