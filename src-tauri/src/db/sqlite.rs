//! SQLite database operations for local data storage
//!
//! This module handles all SQLite database operations including:
//! - Database initialization and migration
//! - Connection CRUD operations
//! - Connection pool management

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;

use crate::db::models::{Cluster, Connection, PortForward};
use crate::error::{AppError, AppResult};

/// SQLite connection pool wrapper
#[derive(Clone)]
pub struct SqlitePool {
    pool: Pool<Sqlite>,
}

impl SqlitePool {
    /// Create a new SQLite pool with the given database path
    pub async fn new(db_path: &Path) -> AppResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let sqlite_pool = Self { pool };
        sqlite_pool.initialize().await?;

        Ok(sqlite_pool)
    }

    /// Initialize database schema
    async fn initialize(&self) -> AppResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER NOT NULL,
                username TEXT,
                database_name TEXT,
                is_default INTEGER DEFAULT 0,
                source TEXT DEFAULT 'local',
                k8s_namespace TEXT,
                k8s_service_name TEXT,
                k8s_service_port INTEGER,
                cluster_id INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index on type for faster queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_connections_type ON connections(type)
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create clusters table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS clusters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                context TEXT,
                environment TEXT,
                is_active INTEGER DEFAULT 1,
                kubeconfig TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create port_forwards table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS port_forwards (
                id TEXT PRIMARY KEY,
                connection_id INTEGER NOT NULL,
                namespace TEXT NOT NULL,
                service_name TEXT NOT NULL,
                remote_port INTEGER NOT NULL,
                local_port INTEGER NOT NULL,
                status TEXT DEFAULT 'stopped',
                error TEXT,
                last_used TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (connection_id) REFERENCES connections(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index on cluster_id for connections
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_connections_cluster ON connections(cluster_id)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all connections
    pub async fn get_all_connections(&self) -> AppResult<Vec<Connection>> {
        let connections = sqlx::query_as::<_, Connection>(
            r#"
            SELECT id, name, type, host, port, username, database_name,
                   is_default, source, k8s_namespace, k8s_service_name,
                   k8s_service_port, cluster_id, created_at, updated_at
            FROM connections
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    /// Get connection by ID
    pub async fn get_connection(&self, id: i64) -> AppResult<Connection> {
        let connection = sqlx::query_as::<_, Connection>(
            r#"
            SELECT id, name, type, host, port, username, database_name,
                   is_default, source, k8s_namespace, k8s_service_name,
                   k8s_service_port, cluster_id, created_at, updated_at
            FROM connections
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(connection)
    }

    /// Get connections by type
    pub async fn get_connections_by_type(&self, conn_type: &str) -> AppResult<Vec<Connection>> {
        let connections = sqlx::query_as::<_, Connection>(
            r#"
            SELECT id, name, type, host, port, username, database_name,
                   is_default, source, k8s_namespace, k8s_service_name,
                   k8s_service_port, cluster_id, created_at, updated_at
            FROM connections
            WHERE type = ?
            ORDER BY name
            "#,
        )
        .bind(conn_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    /// Create a new connection
    pub async fn create_connection(&self, conn: &Connection) -> AppResult<Connection> {
        let result = sqlx::query(
            r#"
            INSERT INTO connections (name, type, host, port, username, database_name,
                                    is_default, source, k8s_namespace, k8s_service_name,
                                    k8s_service_port, cluster_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&conn.name)
        .bind(&conn.conn_type)
        .bind(&conn.host)
        .bind(conn.port)
        .bind(&conn.username)
        .bind(&conn.database_name)
        .bind(conn.is_default)
        .bind(&conn.source)
        .bind(&conn.k8s_namespace)
        .bind(&conn.k8s_service_name)
        .bind(conn.k8s_service_port)
        .bind(conn.cluster_id)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_connection(id).await
    }

    /// Update an existing connection
    pub async fn update_connection(&self, id: i64, conn: &Connection) -> AppResult<Connection> {
        sqlx::query(
            r#"
            UPDATE connections
            SET name = ?, type = ?, host = ?, port = ?, username = ?,
                database_name = ?, is_default = ?, source = ?,
                k8s_namespace = ?, k8s_service_name = ?, k8s_service_port = ?,
                cluster_id = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(&conn.name)
        .bind(&conn.conn_type)
        .bind(&conn.host)
        .bind(conn.port)
        .bind(&conn.username)
        .bind(&conn.database_name)
        .bind(conn.is_default)
        .bind(&conn.source)
        .bind(&conn.k8s_namespace)
        .bind(&conn.k8s_service_name)
        .bind(conn.k8s_service_port)
        .bind(conn.cluster_id)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_connection(id).await
    }

    /// Delete a connection
    pub async fn delete_connection(&self, id: i64) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Connection {} not found", id)));
        }

        Ok(())
    }

    /// Get connections by cluster ID
    pub async fn get_connections_by_cluster(&self, cluster_id: i64) -> AppResult<Vec<Connection>> {
        let connections = sqlx::query_as::<_, Connection>(
            r#"
            SELECT id, name, type, host, port, username, database_name,
                   is_default, source, k8s_namespace, k8s_service_name,
                   k8s_service_port, cluster_id, created_at, updated_at
            FROM connections
            WHERE cluster_id = ?
            ORDER BY name
            "#,
        )
        .bind(cluster_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    // ==================== Cluster Operations ====================

    /// Get all clusters
    pub async fn get_all_clusters(&self) -> AppResult<Vec<Cluster>> {
        let clusters = sqlx::query_as::<_, Cluster>(
            r#"
            SELECT id, name, context, environment, is_active, kubeconfig,
                   created_at, updated_at
            FROM clusters
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(clusters)
    }

    /// Get cluster by ID
    pub async fn get_cluster(&self, id: i64) -> AppResult<Cluster> {
        let cluster = sqlx::query_as::<_, Cluster>(
            r#"
            SELECT id, name, context, environment, is_active, kubeconfig,
                   created_at, updated_at
            FROM clusters
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(cluster)
    }

    /// Get cluster by name
    pub async fn get_cluster_by_name(&self, name: &str) -> AppResult<Cluster> {
        let cluster = sqlx::query_as::<_, Cluster>(
            r#"
            SELECT id, name, context, environment, is_active, kubeconfig,
                   created_at, updated_at
            FROM clusters
            WHERE name = ?
            "#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(cluster)
    }

    /// Create a new cluster
    pub async fn create_cluster(&self, cluster: &Cluster) -> AppResult<Cluster> {
        let result = sqlx::query(
            r#"
            INSERT INTO clusters (name, context, environment, is_active, kubeconfig)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&cluster.name)
        .bind(&cluster.context)
        .bind(&cluster.environment)
        .bind(cluster.is_active)
        .bind(&cluster.kubeconfig)
        .execute(&self.pool)
        .await?;

        let id = result.last_insert_rowid();
        self.get_cluster(id).await
    }

    /// Update an existing cluster
    pub async fn update_cluster(&self, id: i64, cluster: &Cluster) -> AppResult<Cluster> {
        sqlx::query(
            r#"
            UPDATE clusters
            SET name = ?, context = ?, environment = ?, is_active = ?,
                kubeconfig = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(&cluster.name)
        .bind(&cluster.context)
        .bind(&cluster.environment)
        .bind(cluster.is_active)
        .bind(&cluster.kubeconfig)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_cluster(id).await
    }

    /// Delete a cluster
    pub async fn delete_cluster(&self, id: i64) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM clusters WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Cluster {} not found", id)));
        }

        Ok(())
    }

    // ==================== Port Forward Operations ====================

    /// Get all port forwards
    pub async fn get_all_port_forwards(&self) -> AppResult<Vec<PortForward>> {
        let forwards = sqlx::query_as::<_, PortForward>(
            r#"
            SELECT id, connection_id, namespace, service_name, remote_port,
                   local_port, status, error, last_used, created_at
            FROM port_forwards
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(forwards)
    }

    /// Get port forward by ID
    pub async fn get_port_forward(&self, id: &str) -> AppResult<PortForward> {
        let forward = sqlx::query_as::<_, PortForward>(
            r#"
            SELECT id, connection_id, namespace, service_name, remote_port,
                   local_port, status, error, last_used, created_at
            FROM port_forwards
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(forward)
    }

    /// Get port forward by connection ID
    pub async fn get_port_forward_by_connection(&self, connection_id: i64) -> AppResult<PortForward> {
        let forward = sqlx::query_as::<_, PortForward>(
            r#"
            SELECT id, connection_id, namespace, service_name, remote_port,
                   local_port, status, error, last_used, created_at
            FROM port_forwards
            WHERE connection_id = ?
            "#,
        )
        .bind(connection_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(forward)
    }

    /// Create a new port forward record
    pub async fn create_port_forward(&self, forward: &PortForward) -> AppResult<PortForward> {
        let id = forward.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        sqlx::query(
            r#"
            INSERT INTO port_forwards (id, connection_id, namespace, service_name,
                                       remote_port, local_port, status, error, last_used)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(forward.connection_id)
        .bind(&forward.namespace)
        .bind(&forward.service_name)
        .bind(forward.remote_port)
        .bind(forward.local_port)
        .bind(&forward.status)
        .bind(&forward.error)
        .bind(&forward.last_used)
        .execute(&self.pool)
        .await?;

        self.get_port_forward(&id).await
    }

    /// Update port forward status
    pub async fn update_port_forward_status(&self, id: &str, status: &str, error: Option<&str>) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE port_forwards
            SET status = ?, error = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update port forward last used time
    pub async fn touch_port_forward(&self, id: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE port_forwards
            SET last_used = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete a port forward
    pub async fn delete_port_forward(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM port_forwards WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_and_get_connection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::new(&db_path).await.unwrap();

        let conn = Connection {
            id: None,
            name: "Test MySQL".to_string(),
            conn_type: "mysql".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            username: Some("root".to_string()),
            database_name: Some("test".to_string()),
            ..Default::default()
        };

        let created = pool.create_connection(&conn).await.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, "Test MySQL");

        let fetched = pool.get_connection(created.id.unwrap()).await.unwrap();
        assert_eq!(fetched.name, "Test MySQL");
        assert_eq!(fetched.host, "localhost");
    }

    #[tokio::test]
    async fn test_get_connections_by_type() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::new(&db_path).await.unwrap();

        // Create MySQL connection
        let mysql_conn = Connection {
            name: "MySQL 1".to_string(),
            conn_type: "mysql".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            ..Default::default()
        };
        pool.create_connection(&mysql_conn).await.unwrap();

        // Create Redis connection
        let redis_conn = Connection {
            name: "Redis 1".to_string(),
            conn_type: "redis".to_string(),
            host: "localhost".to_string(),
            port: 6379,
            ..Default::default()
        };
        pool.create_connection(&redis_conn).await.unwrap();

        let mysql_conns = pool.get_connections_by_type("mysql").await.unwrap();
        assert_eq!(mysql_conns.len(), 1);
        assert_eq!(mysql_conns[0].name, "MySQL 1");

        let redis_conns = pool.get_connections_by_type("redis").await.unwrap();
        assert_eq!(redis_conns.len(), 1);
        assert_eq!(redis_conns[0].name, "Redis 1");
    }

    #[tokio::test]
    async fn test_update_connection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::new(&db_path).await.unwrap();

        let conn = Connection {
            name: "Original".to_string(),
            conn_type: "mysql".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            ..Default::default()
        };

        let created = pool.create_connection(&conn).await.unwrap();
        let id = created.id.unwrap();

        let updated_conn = Connection {
            name: "Updated".to_string(),
            conn_type: "mysql".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3307,
            ..Default::default()
        };

        let updated = pool.update_connection(id, &updated_conn).await.unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.host, "127.0.0.1");
        assert_eq!(updated.port, 3307);
    }

    #[tokio::test]
    async fn test_delete_connection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::new(&db_path).await.unwrap();

        let conn = Connection {
            name: "To Delete".to_string(),
            conn_type: "mysql".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            ..Default::default()
        };

        let created = pool.create_connection(&conn).await.unwrap();
        let id = created.id.unwrap();

        pool.delete_connection(id).await.unwrap();

        let result = pool.get_connection(id).await;
        assert!(result.is_err());
    }
}
