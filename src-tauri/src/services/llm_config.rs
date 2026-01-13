//! LLM Configuration service
//!
//! Manages LLM provider configurations with encrypted API key storage.

use crate::db::sqlite::SqlitePool;
use crate::db::models::{
    LLMConfig, CreateLLMConfigRequest, UpdateLLMConfigRequest, LLMConfigResponse,
};
use crate::error::AppResult;
use crate::services::crypto::CryptoService;

/// Service for managing LLM configurations
pub struct LLMConfigService {
    pool: SqlitePool,
}

impl LLMConfigService {
    /// Create a new LLM config service
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all LLM configs (without exposing API keys)
    pub async fn get_all(&self) -> AppResult<Vec<LLMConfigResponse>> {
        let configs = self.pool.get_all_llm_configs().await?;
        Ok(configs.into_iter().map(LLMConfigResponse::from).collect())
    }

    /// Get LLM config by ID (without exposing API key)
    pub async fn get(&self, id: i64) -> AppResult<LLMConfigResponse> {
        let config = self.pool.get_llm_config(id).await?;
        Ok(LLMConfigResponse::from(config))
    }

    /// Get default LLM config (without exposing API key)
    pub async fn get_default(&self) -> AppResult<Option<LLMConfigResponse>> {
        let config = self.pool.get_default_llm_config().await?;
        Ok(config.map(LLMConfigResponse::from))
    }

    /// Get the decrypted API key for a config
    /// This should only be used internally when making LLM API calls
    pub async fn get_api_key(&self, id: i64) -> AppResult<Option<String>> {
        let config = self.pool.get_llm_config(id).await?;
        match config.api_key_encrypted {
            Some(encrypted) => {
                let decrypted = CryptoService::decrypt(&encrypted)?;
                Ok(Some(decrypted))
            }
            None => Ok(None),
        }
    }

    /// Create a new LLM config
    pub async fn create(&self, request: CreateLLMConfigRequest) -> AppResult<LLMConfigResponse> {
        // Encrypt the API key if provided
        let api_key_encrypted = match request.api_key {
            Some(ref key) if !key.is_empty() => Some(CryptoService::encrypt(key)?),
            _ => None,
        };

        let config = LLMConfig {
            id: None,
            name: request.name,
            provider: request.provider,
            api_key_encrypted,
            base_url: request.base_url,
            model: request.model,
            max_tokens: request.max_tokens.unwrap_or(2000),
            temperature: request.temperature.unwrap_or(0.7),
            is_default: request.is_default.unwrap_or(false),
            created_at: None,
            updated_at: None,
        };

        let created = self.pool.create_llm_config(&config).await?;
        Ok(LLMConfigResponse::from(created))
    }

    /// Update an existing LLM config
    pub async fn update(&self, id: i64, request: UpdateLLMConfigRequest) -> AppResult<LLMConfigResponse> {
        // Get the existing config
        let mut existing = self.pool.get_llm_config(id).await?;

        // Update fields if provided
        if let Some(name) = request.name {
            existing.name = name;
        }
        if let Some(provider) = request.provider {
            existing.provider = provider;
        }
        if let Some(api_key) = request.api_key {
            // Encrypt the new API key
            existing.api_key_encrypted = if api_key.is_empty() {
                None
            } else {
                Some(CryptoService::encrypt(&api_key)?)
            };
        }
        if let Some(base_url) = request.base_url {
            existing.base_url = Some(base_url);
        }
        if let Some(model) = request.model {
            existing.model = model;
        }
        if let Some(max_tokens) = request.max_tokens {
            existing.max_tokens = max_tokens;
        }
        if let Some(temperature) = request.temperature {
            existing.temperature = temperature;
        }
        if let Some(is_default) = request.is_default {
            existing.is_default = is_default;
        }

        let updated = self.pool.update_llm_config(id, &existing).await?;
        Ok(LLMConfigResponse::from(updated))
    }

    /// Delete an LLM config
    pub async fn delete(&self, id: i64) -> AppResult<()> {
        self.pool.delete_llm_config(id).await
    }

    /// Set a config as default
    pub async fn set_default(&self, id: i64) -> AppResult<LLMConfigResponse> {
        let mut config = self.pool.get_llm_config(id).await?;
        config.is_default = true;
        let updated = self.pool.update_llm_config(id, &config).await?;
        Ok(LLMConfigResponse::from(updated))
    }
}
