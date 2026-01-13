//! Tauri commands for LLM configuration management
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{CreateLLMConfigRequest, UpdateLLMConfigRequest, LLMConfigResponse};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::LLMConfigService;

/// Get all LLM configs
#[tauri::command]
pub async fn get_all_llm_configs(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<LLMConfigResponse>, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.get_all().await
}

/// Get a single LLM config by ID
#[tauri::command]
pub async fn get_llm_config(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<LLMConfigResponse, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.get(id).await
}

/// Get the default LLM config
#[tauri::command]
pub async fn get_default_llm_config(
    pool: State<'_, SqlitePool>,
) -> Result<Option<LLMConfigResponse>, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.get_default().await
}

/// Create a new LLM config
#[tauri::command]
pub async fn create_llm_config(
    pool: State<'_, SqlitePool>,
    data: CreateLLMConfigRequest,
) -> Result<LLMConfigResponse, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.create(data).await
}

/// Update an existing LLM config
#[tauri::command]
pub async fn update_llm_config(
    pool: State<'_, SqlitePool>,
    id: i64,
    data: UpdateLLMConfigRequest,
) -> Result<LLMConfigResponse, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.update(id, data).await
}

/// Delete an LLM config
#[tauri::command]
pub async fn delete_llm_config(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.delete(id).await
}

/// Set an LLM config as default
#[tauri::command]
pub async fn set_default_llm_config(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<LLMConfigResponse, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.set_default(id).await
}

/// Get the API key for a config (for making LLM calls)
/// This returns the decrypted API key
#[tauri::command]
pub async fn get_llm_api_key(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<Option<String>, AppError> {
    let service = LLMConfigService::new(pool.inner().clone());
    service.get_api_key(id).await
}
