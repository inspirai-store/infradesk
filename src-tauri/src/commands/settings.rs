//! Tauri commands for user settings management
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{UserSetting, UpsertSettingRequest, BatchGetSettingsRequest, BatchSettingsResponse};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::SettingsService;

/// Get all settings
#[tauri::command]
pub async fn get_all_settings(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<UserSetting>, AppError> {
    let service = SettingsService::new(pool.inner().clone());
    service.get_all().await
}

/// Get a single setting by key
#[tauri::command]
pub async fn get_setting(
    pool: State<'_, SqlitePool>,
    key: String,
) -> Result<Option<serde_json::Value>, AppError> {
    let service = SettingsService::new(pool.inner().clone());
    service.get(&key).await
}

/// Get multiple settings by keys
#[tauri::command]
pub async fn get_settings_batch(
    pool: State<'_, SqlitePool>,
    request: BatchGetSettingsRequest,
) -> Result<BatchSettingsResponse, AppError> {
    let service = SettingsService::new(pool.inner().clone());
    service.get_batch(&request.keys).await
}

/// Set a setting (upsert)
#[tauri::command]
pub async fn set_setting(
    pool: State<'_, SqlitePool>,
    request: UpsertSettingRequest,
) -> Result<UserSetting, AppError> {
    let service = SettingsService::new(pool.inner().clone());
    service.set(&request).await
}

/// Delete a setting
#[tauri::command]
pub async fn delete_setting(
    pool: State<'_, SqlitePool>,
    key: String,
) -> Result<(), AppError> {
    let service = SettingsService::new(pool.inner().clone());
    service.delete(&key).await
}
