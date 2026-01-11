//! Redis database operations service
//!
//! This service handles all Redis-related operations including:
//! - Server info retrieval
//! - Key management (CRUD operations)
//! - TTL management
//! - Export/Import

use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, Value as RedisValue};
use serde_json::Value as JsonValue;

use crate::db::models::{
    Connection, RedisExportData, RedisKeyInfo, RedisKeyListResponse, RedisKeyValue,
    RedisServerInfo, SetKeyRequest,
};
use crate::error::{AppError, AppResult};

/// Redis service for database operations
pub struct RedisService {
    manager: ConnectionManager,
    connection: Connection,
}

impl RedisService {
    /// Create a new Redis service by connecting to the server
    pub async fn connect(conn: &Connection) -> AppResult<Self> {
        let password = conn.password.as_deref().unwrap_or("");
        let db = conn.database_name.as_deref().unwrap_or("0");

        // For K8s connections, use forward_local_port if available (port forwarding active)
        // Otherwise fall back to the original port
        let effective_port = conn
            .forward_local_port
            .filter(|&p| p > 0)
            .unwrap_or(conn.port);

        log::info!(
            "RedisService::connect - connection_id: {:?}, password provided: {}, password length: {}, host: {}, port: {} (forward_local_port: {:?})",
            conn.id,
            !password.is_empty(),
            password.len(),
            conn.host,
            effective_port,
            conn.forward_local_port
        );

        let url = if password.is_empty() {
            format!("redis://{}:{}/{}", conn.host, effective_port, db)
        } else {
            format!("redis://:{}@{}:{}/{}", password, conn.host, effective_port, db)
        };

        let client = Client::open(url).map_err(|e| AppError::Connection(e.to_string()))?;

        let manager = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Connection(e.to_string()))?;

        Ok(Self {
            manager,
            connection: conn.clone(),
        })
    }

    /// Get Redis server info
    pub async fn get_info(&mut self) -> AppResult<RedisServerInfo> {
        let info: String = redis::cmd("INFO")
            .query_async(&mut self.manager)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Parse INFO response
        let mut version = String::new();
        let mut used_memory = None;
        let mut connected_clients = None;
        let mut uptime_seconds = None;

        for line in info.lines() {
            if let Some(v) = line.strip_prefix("redis_version:") {
                version = v.to_string();
            } else if let Some(v) = line.strip_prefix("used_memory_human:") {
                used_memory = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("connected_clients:") {
                connected_clients = v.parse().ok();
            } else if let Some(v) = line.strip_prefix("uptime_in_seconds:") {
                uptime_seconds = v.parse().ok();
            }
        }

        // Get database count
        let db_count = self.get_db_count().await.unwrap_or(16);

        Ok(RedisServerInfo {
            version,
            host: self.connection.host.clone(),
            port: self.connection.port,
            connected: true,
            used_memory,
            connected_clients,
            uptime_seconds,
            db_count,
        })
    }

    /// Get number of databases
    async fn get_db_count(&mut self) -> AppResult<i64> {
        let config: Vec<String> = redis::cmd("CONFIG")
            .arg("GET")
            .arg("databases")
            .query_async(&mut self.manager)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if config.len() >= 2 {
            Ok(config[1].parse().unwrap_or(16))
        } else {
            Ok(16)
        }
    }

    /// List keys with pattern and cursor-based pagination
    pub async fn list_keys(
        &mut self,
        pattern: &str,
        cursor: u64,
        count: u64,
    ) -> AppResult<RedisKeyListResponse> {
        let pattern = if pattern.is_empty() { "*" } else { pattern };

        let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(count)
            .query_async(&mut self.manager)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut key_infos = Vec::new();

        for key in &keys {
            let key_type = self.get_key_type(key).await.unwrap_or_else(|_| "unknown".to_string());
            let ttl = self.get_ttl(key).await.unwrap_or(-1);

            key_infos.push(RedisKeyInfo {
                key: key.clone(),
                key_type,
                ttl,
                value: None,
                size: None,
            });
        }

        Ok(RedisKeyListResponse {
            keys: key_infos,
            cursor: new_cursor,
            has_more: new_cursor != 0,
        })
    }

    /// Get key type
    async fn get_key_type(&mut self, key: &str) -> AppResult<String> {
        let key_type: String = redis::cmd("TYPE")
            .arg(key)
            .query_async(&mut self.manager)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(key_type)
    }

    /// Get TTL for a key
    async fn get_ttl(&mut self, key: &str) -> AppResult<i64> {
        let ttl: i64 = self
            .manager
            .ttl(key)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(ttl)
    }

    /// Get key value
    pub async fn get_key(&mut self, key: &str) -> AppResult<RedisKeyValue> {
        let key_type = self.get_key_type(key).await?;
        let ttl = self.get_ttl(key).await.unwrap_or(-1);

        let value = match key_type.as_str() {
            "string" => {
                let v: String = self
                    .manager
                    .get(key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                JsonValue::String(v)
            }
            "list" => {
                let v: Vec<String> = self
                    .manager
                    .lrange(key, 0, -1)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                JsonValue::Array(v.into_iter().map(JsonValue::String).collect())
            }
            "set" => {
                let v: Vec<String> = self
                    .manager
                    .smembers(key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                JsonValue::Array(v.into_iter().map(JsonValue::String).collect())
            }
            "zset" => {
                let v: Vec<(String, f64)> = self
                    .manager
                    .zrange_withscores(key, 0, -1)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                let items: Vec<JsonValue> = v
                    .into_iter()
                    .map(|(member, score)| {
                        serde_json::json!({
                            "member": member,
                            "score": score
                        })
                    })
                    .collect();
                JsonValue::Array(items)
            }
            "hash" => {
                let v: Vec<(String, String)> = self
                    .manager
                    .hgetall(key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                let obj: serde_json::Map<String, JsonValue> = v
                    .into_iter()
                    .map(|(k, v)| (k, JsonValue::String(v)))
                    .collect();
                JsonValue::Object(obj)
            }
            _ => JsonValue::Null,
        };

        Ok(RedisKeyValue {
            key: key.to_string(),
            key_type,
            ttl,
            value,
        })
    }

    /// Set a key value
    pub async fn set_key(&mut self, req: &SetKeyRequest) -> AppResult<()> {
        match req.key_type.as_str() {
            "string" => {
                let value = match &req.value {
                    JsonValue::String(s) => s.clone(),
                    _ => req.value.to_string(),
                };

                if let Some(ttl) = req.ttl {
                    if ttl > 0 {
                        let _: () = self
                            .manager
                            .set_ex(&req.key, value, ttl as u64)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    } else {
                        let _: () = self
                            .manager
                            .set(&req.key, value)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                } else {
                    let _: () = self
                        .manager
                        .set(&req.key, value)
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;
                }
            }
            "list" => {
                // Delete existing key first
                let _: () = self
                    .manager
                    .del(&req.key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;

                if let JsonValue::Array(arr) = &req.value {
                    let values: Vec<String> = arr
                        .iter()
                        .map(|v| match v {
                            JsonValue::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .collect();

                    if !values.is_empty() {
                        let _: () = self
                            .manager
                            .rpush(&req.key, values)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }

                if let Some(ttl) = req.ttl {
                    if ttl > 0 {
                        let _: () = self
                            .manager
                            .expire(&req.key, ttl)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }
            }
            "set" => {
                // Delete existing key first
                let _: () = self
                    .manager
                    .del(&req.key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;

                if let JsonValue::Array(arr) = &req.value {
                    let values: Vec<String> = arr
                        .iter()
                        .map(|v| match v {
                            JsonValue::String(s) => s.clone(),
                            _ => v.to_string(),
                        })
                        .collect();

                    if !values.is_empty() {
                        let _: () = self
                            .manager
                            .sadd(&req.key, values)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }

                if let Some(ttl) = req.ttl {
                    if ttl > 0 {
                        let _: () = self
                            .manager
                            .expire(&req.key, ttl)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }
            }
            "zset" => {
                // Delete existing key first
                let _: () = self
                    .manager
                    .del(&req.key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;

                if let JsonValue::Array(arr) = &req.value {
                    for item in arr {
                        if let (Some(member), Some(score)) = (
                            item.get("member").and_then(|v| v.as_str()),
                            item.get("score").and_then(|v| v.as_f64()),
                        ) {
                            let _: () = self
                                .manager
                                .zadd(&req.key, member, score)
                                .await
                                .map_err(|e| AppError::Database(e.to_string()))?;
                        }
                    }
                }

                if let Some(ttl) = req.ttl {
                    if ttl > 0 {
                        let _: () = self
                            .manager
                            .expire(&req.key, ttl)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }
            }
            "hash" => {
                // Delete existing key first
                let _: () = self
                    .manager
                    .del(&req.key)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;

                if let JsonValue::Object(obj) = &req.value {
                    let fields: Vec<(String, String)> = obj
                        .iter()
                        .map(|(k, v)| {
                            let val = match v {
                                JsonValue::String(s) => s.clone(),
                                _ => v.to_string(),
                            };
                            (k.clone(), val)
                        })
                        .collect();

                    if !fields.is_empty() {
                        let _: () = self
                            .manager
                            .hset_multiple(&req.key, &fields)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }

                if let Some(ttl) = req.ttl {
                    if ttl > 0 {
                        let _: () = self
                            .manager
                            .expire(&req.key, ttl)
                            .await
                            .map_err(|e| AppError::Database(e.to_string()))?;
                    }
                }
            }
            _ => {
                return Err(AppError::Validation(format!(
                    "Unsupported key type: {}",
                    req.key_type
                )));
            }
        }

        Ok(())
    }

    /// Update a key (alias for set_key, but requires key to exist)
    pub async fn update_key(&mut self, key: &str, req: &SetKeyRequest) -> AppResult<()> {
        // Check if key exists
        let exists: bool = self
            .manager
            .exists(key)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if !exists {
            return Err(AppError::NotFound(format!("Key not found: {}", key)));
        }

        self.set_key(req).await
    }

    /// Delete a key
    pub async fn delete_key(&mut self, key: &str) -> AppResult<()> {
        let _: () = self
            .manager
            .del(key)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Set TTL for a key
    pub async fn set_ttl(&mut self, key: &str, ttl: i64) -> AppResult<()> {
        if ttl > 0 {
            let _: () = self
                .manager
                .expire(key, ttl)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        } else {
            // Remove TTL (persist)
            let _: () = self
                .manager
                .persist(key)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }
        Ok(())
    }

    /// Export keys
    pub async fn export_keys(&mut self, keys: &[String]) -> AppResult<RedisExportData> {
        let mut exported = Vec::new();

        for key in keys {
            match self.get_key(key).await {
                Ok(kv) => exported.push(kv),
                Err(e) => {
                    log::warn!("Failed to export key {}: {}", key, e);
                }
            }
        }

        Ok(RedisExportData { keys: exported })
    }

    /// Import keys
    pub async fn import_keys(&mut self, data: &RedisExportData) -> AppResult<i32> {
        let mut imported = 0;

        for kv in &data.keys {
            let req = SetKeyRequest {
                key: kv.key.clone(),
                key_type: kv.key_type.clone(),
                value: kv.value.clone(),
                ttl: if kv.ttl > 0 { Some(kv.ttl) } else { None },
            };

            match self.set_key(&req).await {
                Ok(_) => imported += 1,
                Err(e) => {
                    log::warn!("Failed to import key {}: {}", kv.key, e);
                }
            }
        }

        Ok(imported)
    }
}
