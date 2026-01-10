//! Connection management service
//!
//! This service handles all connection-related business logic including:
//! - CRUD operations with password management
//! - Connection testing for MySQL and Redis

use crate::db::models::{Connection, TestConnectionResult};
use crate::db::SqlitePool;
use crate::error::AppResult;
use crate::services::keyring::KeyringService;

/// Connection management service
pub struct ConnectionService {
    pool: SqlitePool,
}

impl ConnectionService {
    /// Create a new connection service
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all connections with passwords populated from keyring
    pub async fn get_all(&self) -> AppResult<Vec<Connection>> {
        let mut connections = self.pool.get_all_connections().await?;

        // Populate passwords from keyring
        for conn in &mut connections {
            if let Some(id) = conn.id {
                conn.password = KeyringService::get_password(id)?;
            }
        }

        Ok(connections)
    }

    /// Get a connection by ID with password populated
    pub async fn get_by_id(&self, id: i64) -> AppResult<Connection> {
        let mut conn = self.pool.get_connection(id).await?;
        conn.password = KeyringService::get_password(id)?;
        Ok(conn)
    }

    /// Get connections by type with passwords populated
    pub async fn get_by_type(&self, conn_type: &str) -> AppResult<Vec<Connection>> {
        let mut connections = self.pool.get_connections_by_type(conn_type).await?;

        for conn in &mut connections {
            if let Some(id) = conn.id {
                conn.password = KeyringService::get_password(id)?;
            }
        }

        Ok(connections)
    }

    /// Create a new connection
    pub async fn create(&self, mut conn: Connection) -> AppResult<Connection> {
        // Extract password before saving to DB
        let password = conn.password.take();

        // Create connection in DB
        let mut created = self.pool.create_connection(&conn).await?;

        // Save password to keyring if provided
        if let (Some(id), Some(pwd)) = (created.id, &password) {
            if !pwd.is_empty() {
                KeyringService::save_password(id, pwd)?;
            }
        }

        // Return connection with password
        created.password = password;
        Ok(created)
    }

    /// Update an existing connection
    pub async fn update(&self, id: i64, mut conn: Connection) -> AppResult<Connection> {
        // Extract password before saving to DB
        let password = conn.password.take();

        // Update connection in DB
        let mut updated = self.pool.update_connection(id, &conn).await?;

        // Update password in keyring
        if let Some(pwd) = &password {
            if !pwd.is_empty() {
                KeyringService::save_password(id, pwd)?;
            } else {
                // Empty password means delete it
                KeyringService::delete_password(id)?;
            }
        }

        // Return connection with password
        updated.password = password;
        Ok(updated)
    }

    /// Delete a connection
    pub async fn delete(&self, id: i64) -> AppResult<()> {
        // Delete from DB first
        self.pool.delete_connection(id).await?;

        // Delete password from keyring
        KeyringService::delete_password(id)?;

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

        let password = conn.password.as_deref().unwrap_or("");
        let username = conn.username.as_deref().unwrap_or("root");
        let database = conn.database_name.as_deref().unwrap_or("");

        let url = if database.is_empty() {
            format!(
                "mysql://{}:{}@{}:{}",
                username, password, conn.host, conn.port
            )
        } else {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, conn.host, conn.port, database
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
