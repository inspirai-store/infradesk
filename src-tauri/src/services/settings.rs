//! Settings service for managing user preferences
//!
//! Provides a key-value store for user settings with JSON value support.

use crate::db::sqlite::SqlitePool;
use crate::db::models::{UserSetting, UpsertSettingRequest, BatchSettingsResponse};
use crate::error::AppResult;
use std::collections::HashMap;

/// Service for managing user settings
pub struct SettingsService {
    pool: SqlitePool,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all settings
    pub async fn get_all(&self) -> AppResult<Vec<UserSetting>> {
        self.pool.get_all_settings().await
    }

    /// Get a setting by key
    pub async fn get(&self, key: &str) -> AppResult<Option<serde_json::Value>> {
        let setting = self.pool.get_setting(key).await?;
        match setting {
            Some(s) => {
                let value: serde_json::Value = serde_json::from_str(&s.value)
                    .unwrap_or(serde_json::Value::String(s.value));
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get multiple settings by keys
    pub async fn get_batch(&self, keys: &[String]) -> AppResult<BatchSettingsResponse> {
        let settings = self.pool.get_settings_by_keys(keys).await?;
        let mut map = HashMap::new();

        for setting in settings {
            let value: serde_json::Value = serde_json::from_str(&setting.value)
                .unwrap_or(serde_json::Value::String(setting.value));
            map.insert(setting.key, value);
        }

        Ok(BatchSettingsResponse { settings: map })
    }

    /// Set a setting (upsert)
    pub async fn set(&self, request: &UpsertSettingRequest) -> AppResult<UserSetting> {
        let value_str = serde_json::to_string(&request.value)?;
        self.pool.upsert_setting(&request.key, &value_str).await
    }

    /// Delete a setting
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        self.pool.delete_setting(key).await
    }

    // ==================== Convenience Methods for Common Settings ====================

    /// Get active connections map
    pub async fn get_active_connections(&self) -> AppResult<HashMap<String, Option<i64>>> {
        let value = self.get("active_connections").await?;
        match value {
            Some(v) => {
                let map: HashMap<String, Option<i64>> = serde_json::from_value(v)?;
                Ok(map)
            }
            None => Ok(HashMap::new()),
        }
    }

    /// Set active connections map
    pub async fn set_active_connections(&self, connections: &HashMap<String, Option<i64>>) -> AppResult<()> {
        let request = UpsertSettingRequest {
            key: "active_connections".to_string(),
            value: serde_json::to_value(connections)?,
        };
        self.set(&request).await?;
        Ok(())
    }

    /// Get MySQL query limit
    pub async fn get_mysql_query_limit(&self) -> AppResult<i32> {
        let value = self.get("mysql_query_limit").await?;
        match value {
            Some(v) => Ok(v.as_i64().unwrap_or(100) as i32),
            None => Ok(100), // Default value
        }
    }

    /// Set MySQL query limit
    pub async fn set_mysql_query_limit(&self, limit: i32) -> AppResult<()> {
        let request = UpsertSettingRequest {
            key: "mysql_query_limit".to_string(),
            value: serde_json::Value::Number(limit.into()),
        };
        self.set(&request).await?;
        Ok(())
    }
}
