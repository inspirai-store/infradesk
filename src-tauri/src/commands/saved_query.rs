//! Tauri commands for saved query management
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{SavedQuery, CreateSavedQueryRequest, UpdateSavedQueryRequest};
use crate::db::SqlitePool;
use crate::error::AppError;

/// Get saved queries with optional category filter
#[tauri::command]
pub async fn get_saved_queries(
    pool: State<'_, SqlitePool>,
    category: Option<String>,
) -> Result<Vec<SavedQuery>, AppError> {
    pool.get_saved_queries(category.as_deref()).await
}

/// Create a saved query
#[tauri::command]
pub async fn create_saved_query(
    pool: State<'_, SqlitePool>,
    data: CreateSavedQueryRequest,
) -> Result<SavedQuery, AppError> {
    pool.create_saved_query(&data).await
}

/// Update a saved query
#[tauri::command]
pub async fn update_saved_query(
    pool: State<'_, SqlitePool>,
    id: i64,
    data: UpdateSavedQueryRequest,
) -> Result<SavedQuery, AppError> {
    pool.update_saved_query(id, &data).await
}

/// Delete a saved query
#[tauri::command]
pub async fn delete_saved_query(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), AppError> {
    pool.delete_saved_query(id).await
}
