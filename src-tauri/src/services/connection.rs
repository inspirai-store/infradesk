//! Connection management service
//!
//! This service handles all connection-related business logic including:
//! - CRUD operations with password encryption
//! - Connection testing for MySQL and Redis

use crate::db::models::{Connection, TestConnectionResult};
use crate::db::SqlitePool;
use crate::error::AppResult;
use crate::services::crypto::CryptoService;

/// Connection management service
pub struct ConnectionService {
    pool: SqlitePool,
}

impl ConnectionService {
    /// Create a new connection service
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Decrypt password in a connection
    fn decrypt_password(conn: &mut Connection) {
        if let Some(encrypted) = &conn.password {
            if !encrypted.is_empty() {
                match CryptoService::decrypt(encrypted) {
                    Ok(decrypted) => {
                        conn.password = Some(decrypted);
                    }
                    Err(e) => {
                        log::warn!("Failed to decrypt password for connection {:?}: {}", conn.id, e);
                        conn.password = None;
                    }
                }
            }
        }
    }

    /// Encrypt password for storage
    fn encrypt_password(password: Option<&str>) -> Option<String> {
        password.and_then(|pwd| {
            if pwd.is_empty() {
                None
            } else {
                match CryptoService::encrypt(pwd) {
                    Ok(encrypted) => Some(encrypted),
                    Err(e) => {
                        log::error!("Failed to encrypt password: {}", e);
                        None
                    }
                }
            }
        })
    }

    /// Get all connections with passwords decrypted
    pub async fn get_all(&self) -> AppResult<Vec<Connection>> {
        let mut connections = self.pool.get_all_connections().await?;

        // Decrypt passwords
        for conn in &mut connections {
            Self::decrypt_password(conn);
        }

        Ok(connections)
    }

    /// Get a connection by ID with password decrypted
    pub async fn get_by_id(&self, id: i64) -> AppResult<Connection> {
        let mut conn = self.pool.get_connection(id).await?;
        Self::decrypt_password(&mut conn);
        log::info!(
            "ConnectionService::get_by_id - id: {}, password present: {}, length: {}",
            id,
            conn.password.is_some(),
            conn.password.as_ref().map(|p| p.len()).unwrap_or(0)
        );
        Ok(conn)
    }

    /// Get connections by type with passwords decrypted
    pub async fn get_by_type(&self, conn_type: &str) -> AppResult<Vec<Connection>> {
        let mut connections = self.pool.get_connections_by_type(conn_type).await?;

        for conn in &mut connections {
            Self::decrypt_password(conn);
        }

        Ok(connections)
    }

    /// Create a new connection with password encrypted
    pub async fn create(&self, mut conn: Connection) -> AppResult<Connection> {
        // Store original password for return
        let original_password = conn.password.clone();

        // Encrypt password before saving
        conn.password = Self::encrypt_password(conn.password.as_deref());

        log::info!(
            "ConnectionService::create - encrypting password, original length: {}, encrypted length: {}",
            original_password.as_ref().map(|p| p.len()).unwrap_or(0),
            conn.password.as_ref().map(|p| p.len()).unwrap_or(0)
        );

        // Create connection in DB
        let mut created = self.pool.create_connection(&conn).await?;

        // Return connection with original (decrypted) password
        created.password = original_password;
        Ok(created)
    }

    /// Update an existing connection with password encrypted
    pub async fn update(&self, id: i64, mut conn: Connection) -> AppResult<Connection> {
        // Store original password for return
        let original_password = conn.password.clone();

        log::info!(
            "ConnectionService::update - id: {}, password provided: {}, password length: {}",
            id,
            original_password.is_some(),
            original_password.as_ref().map(|p| p.len()).unwrap_or(0)
        );

        // Encrypt password before saving
        conn.password = Self::encrypt_password(conn.password.as_deref());

        log::info!(
            "ConnectionService::update - encrypted password length: {}",
            conn.password.as_ref().map(|p| p.len()).unwrap_or(0)
        );

        // Update connection in DB
        let mut updated = self.pool.update_connection(id, &conn).await?;

        // Return connection with original (decrypted) password
        updated.password = original_password;
        Ok(updated)
    }

    /// Partial update - only update provided fields
    pub async fn partial_update(
        &self,
        id: i64,
        update: crate::db::models::UpdateConnectionRequest,
    ) -> AppResult<Connection> {
        // First get the existing connection
        let mut existing = self.pool.get_connection(id).await?;
        // Decrypt existing password
        Self::decrypt_password(&mut existing);

        // Apply partial updates (only if Some)
        if let Some(name) = update.name {
            existing.name = name;
        }
        if let Some(conn_type) = update.conn_type {
            existing.conn_type = conn_type;
        }
        if let Some(host) = update.host {
            existing.host = host;
        }
        if let Some(port) = update.port {
            existing.port = port;
        }
        if let Some(username) = update.username {
            existing.username = Some(username);
        }
        if let Some(password) = update.password {
            existing.password = Some(password);
        }
        if let Some(database_name) = update.database_name {
            existing.database_name = Some(database_name);
        }
        if let Some(is_default) = update.is_default {
            existing.is_default = is_default;
        }
        if let Some(source) = update.source {
            existing.source = Some(source);
        }
        if let Some(k8s_namespace) = update.k8s_namespace {
            existing.k8s_namespace = Some(k8s_namespace);
        }
        if let Some(k8s_service_name) = update.k8s_service_name {
            existing.k8s_service_name = Some(k8s_service_name);
        }
        if let Some(k8s_service_port) = update.k8s_service_port {
            existing.k8s_service_port = Some(k8s_service_port);
        }
        if let Some(cluster_id) = update.cluster_id {
            existing.cluster_id = Some(cluster_id);
        }
        if let Some(forward_local_port) = update.forward_local_port {
            existing.forward_local_port = Some(forward_local_port);
        }

        // Now use the full update method
        self.update(id, existing).await
    }

    /// Delete a connection
    pub async fn delete(&self, id: i64) -> AppResult<()> {
        self.pool.delete_connection(id).await
    }

    /// Update only the forward_local_port for a connection
    pub async fn update_forward_port(&self, id: i64, port: i32) -> AppResult<()> {
        use crate::error::AppError;

        sqlx::query("UPDATE connections SET forward_local_port = ? WHERE id = ?")
            .bind(port)
            .bind(id)
            .execute(self.pool.pool())
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        log::info!("ConnectionService::update_forward_port - id: {}, port: {}", id, port);
        Ok(())
    }

    /// Test a connection without saving
    pub async fn test(&self, conn: &Connection) -> AppResult<TestConnectionResult> {
        match conn.conn_type.as_str() {
            "mysql" => self.test_mysql(conn).await,
            "redis" => self.test_redis(conn).await,
            _ => Ok(TestConnectionResult::failure(format!(
                "Unsupported connection type: {}",
                conn.conn_type
            ))),
        }
    }

    /// Test MySQL connection
    async fn test_mysql(&self, conn: &Connection) -> AppResult<TestConnectionResult> {
        use sqlx::mysql::MySqlPoolOptions;
        use urlencoding::encode;

        let password = conn.password.as_deref().unwrap_or("");
        let username = conn.username.as_deref().unwrap_or("root");
        let database = conn.database_name.as_deref().unwrap_or("");

        // URL-encode username and password to handle special characters like / @ :
        let encoded_username = encode(username);
        let encoded_password = encode(password);

        let url = if database.is_empty() {
            format!(
                "mysql://{}:{}@{}:{}",
                encoded_username, encoded_password, conn.host, conn.port
            )
        } else {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                encoded_username, encoded_password, conn.host, conn.port, database
            )
        };

        let result = MySqlPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&url)
            .await;

        match result {
            Ok(pool) => {
                // Try a simple query to verify connection
                let version: Result<(String,), _> =
                    sqlx::query_as("SELECT VERSION()").fetch_one(&pool).await;

                match version {
                    Ok((ver,)) => Ok(TestConnectionResult::success(format!(
                        "Connected to MySQL {}",
                        ver
                    ))),
                    Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
                }
            }
            Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
        }
    }

    /// Test Redis connection
    async fn test_redis(&self, conn: &Connection) -> AppResult<TestConnectionResult> {
        let password = conn.password.as_deref();

        let url = if let Some(pwd) = password {
            if pwd.is_empty() {
                format!("redis://{}:{}", conn.host, conn.port)
            } else {
                format!("redis://:{}@{}:{}", pwd, conn.host, conn.port)
            }
        } else {
            format!("redis://{}:{}", conn.host, conn.port)
        };

        let result = redis::Client::open(url.as_str());

        match result {
            Ok(client) => {
                let con_result = client.get_multiplexed_tokio_connection().await;

                match con_result {
                    Ok(mut con) => {
                        // Try PING command
                        let pong: Result<String, _> = redis::cmd("PING")
                            .query_async(&mut con)
                            .await;

                        match pong {
                            Ok(_) => Ok(TestConnectionResult::success("Connected to Redis")),
                            Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
                        }
                    }
                    Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
                }
            }
            Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
        }
    }
}
