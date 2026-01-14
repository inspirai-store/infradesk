//! Database models for the Tauri backend
//!
//! These models are used for both SQLite storage and IPC communication.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Connection {
    /// Unique identifier (auto-generated)
    pub id: Option<i64>,

    /// Display name for the connection
    pub name: String,

    /// Connection type: mysql, redis, mongodb, minio
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub conn_type: String,

    /// Database host address
    pub host: String,

    /// Database port
    pub port: i32,

    /// Username for authentication
    pub username: Option<String>,

    /// Encrypted password stored in SQLite
    /// This field stores the encrypted password, decrypted when reading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Default database name (for MySQL/MongoDB)
    pub database_name: Option<String>,

    /// Whether this is the default connection for its type
    #[serde(default)]
    pub is_default: bool,

    /// Connection source: local, k8s
    pub source: Option<String>,

    /// Kubernetes namespace (for k8s connections)
    pub k8s_namespace: Option<String>,

    /// Kubernetes service name (for k8s connections)
    pub k8s_service_name: Option<String>,

    /// Kubernetes service port (for k8s connections)
    pub k8s_service_port: Option<i32>,

    /// Associated cluster ID (for k8s connections)
    pub cluster_id: Option<i64>,

    /// Preferred local port for port forwarding (0 = auto-assign)
    #[serde(default)]
    #[sqlx(default)]
    pub forward_local_port: Option<i32>,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Result of testing a database connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    /// Whether the connection was successful
    pub success: bool,

    /// Error message if connection failed
    pub error: Option<String>,

    /// Success message or additional info
    pub message: Option<String>,
}

impl TestConnectionResult {
    /// Create a successful test result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            error: None,
            message: Some(message.into()),
        }
    }

    /// Create a failed test result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
            message: None,
        }
    }
}

/// Request to update a connection (partial update support)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConnectionRequest {
    /// Display name for the connection
    pub name: Option<String>,

    /// Connection type: mysql, redis, mongodb, minio
    #[serde(rename = "type")]
    pub conn_type: Option<String>,

    /// Database host address
    pub host: Option<String>,

    /// Database port
    pub port: Option<i32>,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Default database name (for MySQL/MongoDB)
    pub database_name: Option<String>,

    /// Whether this is the default connection for its type
    pub is_default: Option<bool>,

    /// Connection source: local, k8s
    pub source: Option<String>,

    /// Kubernetes namespace (for k8s connections)
    pub k8s_namespace: Option<String>,

    /// Kubernetes service name (for k8s connections)
    pub k8s_service_name: Option<String>,

    /// Kubernetes service port (for k8s connections)
    pub k8s_service_port: Option<i32>,

    /// Cluster ID this connection belongs to
    pub cluster_id: Option<i64>,

    /// Local port for port forwarding
    pub forward_local_port: Option<i32>,
}

/// Request to test a connection (no name required)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionRequest {
    /// Connection type: mysql, redis
    #[serde(rename = "type")]
    pub conn_type: String,

    /// Database host address
    pub host: String,

    /// Database port
    pub port: i32,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Default database name (for MySQL)
    pub database_name: Option<String>,
}

impl TestConnectionRequest {
    /// Convert to Connection for testing
    pub fn to_connection(&self) -> Connection {
        Connection {
            id: None,
            name: String::new(), // Not needed for testing
            conn_type: self.conn_type.clone(),
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            password: self.password.clone(),
            database_name: self.database_name.clone(),
            is_default: false,
            source: None,
            k8s_namespace: None,
            k8s_service_name: None,
            k8s_service_port: None,
            cluster_id: None,
            forward_local_port: None,
            created_at: None,
            updated_at: None,
        }
    }
}

/// Request to test a K8s connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestK8sConnectionRequest {
    /// Connection type: mysql, redis
    #[serde(rename = "type")]
    pub conn_type: String,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Default database name (for MySQL)
    pub database_name: Option<String>,

    /// Kubeconfig content (optional if cluster_id is provided)
    pub kubeconfig: Option<String>,

    /// Kubernetes context name
    pub context: Option<String>,

    /// Kubernetes namespace
    pub k8s_namespace: String,

    /// Kubernetes service name
    pub k8s_service_name: String,

    /// Kubernetes service port
    pub k8s_service_port: i32,

    /// Cluster ID for looking up kubeconfig from database
    pub cluster_id: Option<i64>,
}

// ==================== MySQL Models ====================

/// MySQL server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlServerInfo {
    pub version: String,
    pub host: String,
    pub port: i32,
    pub connected: bool,
}

/// MySQL database info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlDatabase {
    pub name: String,
    pub table_count: i64,
    pub size: String,
}

/// MySQL table info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlTable {
    pub name: String,
    pub engine: Option<String>,
    pub row_count: i64,
    pub data_size: i64,
    pub index_size: i64,
    pub comment: Option<String>,
}

/// MySQL column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub nullable: bool,
    pub key: Option<String>,
    pub default: Option<String>,
    pub extra: Option<String>,
    pub comment: Option<String>,
}

/// MySQL index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    #[serde(rename = "type")]
    pub index_type: String,
}

/// MySQL table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlTableSchema {
    pub name: String,
    pub columns: Vec<MysqlColumn>,
    pub indexes: Vec<MysqlIndex>,
}

/// MySQL query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlQueryResult {
    /// Column names
    pub columns: Vec<String>,
    /// Row data as JSON objects with column names as keys
    pub rows: Vec<std::collections::HashMap<String, serde_json::Value>>,
    /// Number of affected rows (for INSERT/UPDATE/DELETE)
    pub affected_rows: u64,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Query type (select, insert, update, delete, etc.)
    pub query_type: String,
}

/// MySQL table data with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlTableData {
    pub columns: Vec<String>,
    pub rows: Vec<std::collections::HashMap<String, serde_json::Value>>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// MySQL user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlUserInfo {
    pub user: String,
    pub host: String,
}

/// Request to create a MySQL database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
    pub charset: Option<String>,
    pub collation: Option<String>,
}

/// Request to alter a MySQL database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlterDatabaseRequest {
    pub charset: Option<String>,
    pub collation: Option<String>,
}

/// Request to create a MySQL user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub host: String,
    pub password: String,
}

/// Request to grant privileges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantPrivilegesRequest {
    pub username: String,
    pub host: String,
    pub privileges: Vec<String>,
}

/// Request to alter user password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlterUserPasswordRequest {
    pub username: String,
    pub host: String,
    pub new_password: String,
}

/// Request to drop a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropUserRequest {
    pub username: String,
    pub host: String,
}

/// Request to revoke privileges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokePrivilegesRequest {
    pub username: String,
    pub host: String,
    /// Privileges to revoke (e.g., SELECT, INSERT, ALL PRIVILEGES)
    pub privileges: Vec<String>,
    /// Database scope (e.g., "mydb" for mydb.*, "*" for *.*)
    pub database: String,
}

/// User grant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGrantInfo {
    /// The full GRANT statement
    pub grant_statement: String,
}

/// User grants response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGrantsResponse {
    pub username: String,
    pub host: String,
    pub grants: Vec<UserGrantInfo>,
}

// ==================== MySQL Table Management Models ====================

/// Column definition for creating/altering tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    /// Column name
    pub name: String,
    /// Data type (e.g., INT, VARCHAR(255), TEXT, DATETIME, JSON)
    pub data_type: String,
    /// Whether the column allows NULL
    #[serde(default)]
    pub nullable: bool,
    /// Default value (as SQL expression)
    pub default: Option<String>,
    /// Whether this column is auto-increment
    #[serde(default)]
    pub auto_increment: bool,
    /// Column comment
    pub comment: Option<String>,
}

/// Index definition for creating/altering tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    /// Index name
    pub name: String,
    /// Columns included in the index
    pub columns: Vec<String>,
    /// Whether this is a unique index
    #[serde(default)]
    pub unique: bool,
    /// Index type (BTREE, HASH, FULLTEXT)
    pub index_type: Option<String>,
}

/// Request to create a MySQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableRequest {
    /// Table name
    pub name: String,
    /// Column definitions
    pub columns: Vec<ColumnDefinition>,
    /// Primary key columns
    pub primary_key: Option<Vec<String>>,
    /// Index definitions
    pub indexes: Option<Vec<IndexDefinition>>,
    /// Storage engine (InnoDB, MyISAM)
    pub engine: Option<String>,
    /// Character set (utf8mb4, utf8, latin1)
    pub charset: Option<String>,
    /// Collation (utf8mb4_unicode_ci)
    pub collation: Option<String>,
    /// Table comment
    pub comment: Option<String>,
}

/// Request to rename a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameColumnRequest {
    pub old_name: String,
    pub new_name: String,
}

/// Request to alter a MySQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlterTableRequest {
    /// Columns to add
    pub add_columns: Option<Vec<ColumnDefinition>>,
    /// Columns to drop (by name)
    pub drop_columns: Option<Vec<String>>,
    /// Columns to modify
    pub modify_columns: Option<Vec<ColumnDefinition>>,
    /// Column to rename
    pub rename_column: Option<RenameColumnRequest>,
    /// Indexes to add
    pub add_indexes: Option<Vec<IndexDefinition>>,
    /// Indexes to drop (by name)
    pub drop_indexes: Option<Vec<String>>,
}

/// Request to rename a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameTableRequest {
    pub new_name: String,
}

/// Request to copy a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTableRequest {
    /// Target table name
    pub target_name: String,
    /// Whether to copy data (true) or just structure (false)
    #[serde(default)]
    pub with_data: bool,
}

// ==================== Index Management Models ====================

/// Index information from MySQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    /// Column names in the index
    pub columns: Vec<String>,
    /// Whether this is a unique index
    pub unique: bool,
    /// Index type (BTREE, HASH, FULLTEXT, SPATIAL)
    pub index_type: String,
    /// Whether this is the primary key
    pub is_primary: bool,
    /// Index comment
    pub comment: Option<String>,
}

/// Request to create a new index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIndexRequest {
    /// Index name
    pub name: String,
    /// Column names to include in the index
    pub columns: Vec<String>,
    /// Whether this is a unique index
    #[serde(default)]
    pub unique: bool,
    /// Index type (BTREE, HASH, FULLTEXT)
    pub index_type: Option<String>,
    /// Index comment
    pub comment: Option<String>,
}

// ==================== Foreign Key Management Models ====================

/// Foreign key information from MySQL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    /// Constraint name
    pub name: String,
    /// Source column name(s)
    pub columns: Vec<String>,
    /// Referenced table name
    pub ref_table: String,
    /// Referenced column name(s)
    pub ref_columns: Vec<String>,
    /// ON DELETE action (CASCADE, SET NULL, RESTRICT, NO ACTION)
    pub on_delete: String,
    /// ON UPDATE action (CASCADE, SET NULL, RESTRICT, NO ACTION)
    pub on_update: String,
}

/// Request to create a new foreign key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateForeignKeyRequest {
    /// Constraint name (optional, auto-generated if not provided)
    pub name: Option<String>,
    /// Source column name(s)
    pub columns: Vec<String>,
    /// Referenced table name
    pub ref_table: String,
    /// Referenced column name(s)
    pub ref_columns: Vec<String>,
    /// ON DELETE action (default: RESTRICT)
    #[serde(default = "default_fk_action")]
    pub on_delete: String,
    /// ON UPDATE action (default: RESTRICT)
    #[serde(default = "default_fk_action")]
    pub on_update: String,
}

fn default_fk_action() -> String {
    "RESTRICT".to_string()
}

// ==================== Data Export/Import Models ====================

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
    Sql,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Csv
    }
}

/// Request to export table data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTableRequest {
    /// Export format (csv, json, sql)
    #[serde(default)]
    pub format: ExportFormat,
    /// Columns to export (None = all columns)
    pub columns: Option<Vec<String>>,
    /// WHERE clause for filtering (optional)
    pub where_clause: Option<String>,
    /// Limit number of rows (optional)
    pub limit: Option<u32>,
    /// Include column headers (for CSV, default: true)
    #[serde(default = "default_true")]
    pub include_headers: bool,
}

fn default_true() -> bool {
    true
}

/// Response containing exported data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTableResponse {
    /// Exported data as string
    pub data: String,
    /// Export format used
    pub format: String,
    /// Number of rows exported
    pub row_count: usize,
}

/// Request to import data into a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDataRequest {
    /// Data to import (CSV or JSON string)
    pub data: String,
    /// Import format (csv, json)
    pub format: String,
    /// Column mapping (optional, maps source columns to target columns)
    pub column_mapping: Option<std::collections::HashMap<String, String>>,
    /// Skip first N rows (useful for CSV with headers)
    #[serde(default)]
    pub skip_rows: usize,
    /// On duplicate key action: ignore, update, error
    #[serde(default = "default_duplicate_action")]
    pub on_duplicate: String,
}

fn default_duplicate_action() -> String {
    "error".to_string()
}

/// Result of import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of rows successfully imported
    pub imported: usize,
    /// Number of rows skipped (due to duplicates, errors)
    pub skipped: usize,
    /// Number of rows failed
    pub failed: usize,
    /// Error messages for failed rows
    pub errors: Vec<String>,
}

// ==================== Cluster Models ====================

/// Kubernetes cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct Cluster {
    /// Unique identifier (auto-generated)
    pub id: Option<i64>,

    /// Display name for the cluster
    pub name: String,

    /// Kubernetes context name
    pub context: Option<String>,

    /// Environment: dev, staging, prod
    pub environment: Option<String>,

    /// Whether this cluster is active
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// Kubeconfig content (sensitive, not returned in responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubeconfig: Option<String>,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Discovered K8s service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Service name
    pub name: String,

    /// Kubernetes namespace
    pub namespace: String,

    /// Service type: mysql, redis, mongodb, etc.
    #[serde(rename = "type")]
    pub service_type: String,

    /// Service host (cluster DNS name)
    pub host: String,

    /// Service port
    pub port: i32,

    /// Whether credentials were detected
    pub has_credentials: bool,

    /// Detected username (if available)
    pub username: Option<String>,

    /// Detected password (if available from secrets)
    pub password: Option<String>,

    /// Default database name (if available)
    pub database: Option<String>,

    /// K8s service name for port forwarding
    pub service_name: String,
}

/// Port forward info
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct PortForward {
    /// Unique identifier
    pub id: Option<String>,

    /// Associated connection ID
    pub connection_id: i64,

    /// Kubernetes namespace
    pub namespace: String,

    /// Kubernetes service name
    pub service_name: String,

    /// Remote port on the service
    pub remote_port: i32,

    /// Local port assigned for forwarding
    pub local_port: i32,

    /// Forward status: active, stopped, error
    pub status: String,

    /// Error message if status is error
    pub error: Option<String>,

    /// Last used timestamp
    pub last_used: Option<String>,

    /// Creation timestamp
    pub created_at: Option<String>,
}

/// Import connections request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConnectionsRequest {
    /// Services to import
    pub services: Vec<ImportServiceItem>,

    /// Whether to force override existing connections
    #[serde(default)]
    pub force_override: bool,

    /// Kubeconfig content for port forwarding
    pub kubeconfig: Option<String>,

    /// Kubeconfig context
    pub context: Option<String>,

    /// Cluster name
    pub cluster_name: Option<String>,
}

/// Single service to import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportServiceItem {
    pub name: String,
    #[serde(rename = "type")]
    pub service_type: String,
    pub namespace: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub service_name: Option<String>,
}

/// Import connections response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConnectionsResponse {
    pub success: i32,
    pub failed: i32,
    pub updated: i32,
    pub skipped: i32,
    pub results: Vec<ImportConnectionResult>,
}

/// Single import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConnectionResult {
    pub name: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
}

/// List clusters response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListClustersResponse {
    pub clusters: Vec<String>,
}

/// Forward list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardListResponse {
    pub forwards: Vec<PortForward>,
}

// ==================== Redis Models ====================

/// Redis server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisServerInfo {
    pub version: String,
    pub host: String,
    pub port: i32,
    pub connected: bool,
    pub used_memory: Option<String>,
    pub connected_clients: Option<i64>,
    pub uptime_seconds: Option<i64>,
    pub db_count: i64,
}

/// Redis key info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisKeyInfo {
    pub key: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub ttl: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

/// Redis key list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisKeyListResponse {
    pub keys: Vec<RedisKeyInfo>,
    pub cursor: u64,
    pub has_more: bool,
}

/// Redis key value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisKeyValue {
    pub key: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub ttl: i64,
    pub value: serde_json::Value,
}

/// Request to set a Redis key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetKeyRequest {
    pub key: String,
    #[serde(rename = "type")]
    pub key_type: String,
    pub value: serde_json::Value,
    pub ttl: Option<i64>,
}

/// Redis export data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisExportData {
    pub keys: Vec<RedisKeyValue>,
}

// ==================== Query History Models ====================

/// Query history entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct QueryHistory {
    /// Unique identifier
    pub id: Option<i64>,

    /// Associated connection ID
    pub connection_id: i64,

    /// Database name
    pub database: String,

    /// Query type: select, insert, update, delete, etc.
    pub query_type: String,

    /// SQL query text
    pub query_text: String,

    /// Execution timestamp
    pub executed_at: Option<String>,

    /// Query duration in milliseconds
    pub duration_ms: i64,

    /// Number of rows affected/returned
    pub row_count: i64,

    /// Status: success, error
    pub status: String,

    /// Error message if status is error
    pub error_message: Option<String>,
}

/// Query history list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryListResponse {
    pub history: Vec<QueryHistory>,
    pub total: i64,
}

/// Add query history request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddQueryHistoryRequest {
    pub connection_id: i64,
    pub database: String,
    pub query_type: String,
    pub query_text: String,
    pub duration_ms: i64,
    pub row_count: i64,
    pub status: String,
    pub error_message: Option<String>,
}

// ==================== Saved Query Models ====================

/// Saved query entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct SavedQuery {
    /// Unique identifier
    pub id: Option<i64>,

    /// Associated connection ID
    pub connection_id: i64,

    /// Database name
    pub database: String,

    /// Display name for the saved query
    pub name: String,

    /// SQL query text
    pub query_text: String,

    /// Description
    pub description: Option<String>,

    /// Category for grouping
    pub category: Option<String>,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Create saved query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedQueryRequest {
    pub connection_id: i64,
    pub database: String,
    pub name: String,
    pub query_text: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

/// Update saved query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedQueryRequest {
    pub name: Option<String>,
    pub query_text: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

// ==================== K8s Resource Models ====================

/// K8s Deployment info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sDeployment {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub labels: std::collections::HashMap<String, String>,
    pub created_at: Option<String>,
}

/// K8s Pod info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sPod {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ready: String,
    pub restarts: i32,
    pub node: Option<String>,
    pub ip: Option<String>,
    pub created_at: Option<String>,
}

/// K8s ConfigMap info (metadata only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub data_keys: Vec<String>,
    pub created_at: Option<String>,
}

/// K8s Secret info (metadata only, no values exposed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sSecretInfo {
    pub name: String,
    pub namespace: String,
    pub secret_type: String,
    pub data_keys: Vec<String>,
    pub created_at: Option<String>,
}

/// K8s Service info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sServiceInfo {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: Option<String>,
    pub external_ip: Option<String>,
    pub ports: Vec<String>,
    pub created_at: Option<String>,
    /// ExternalName service target (only for ExternalName type)
    pub external_name: Option<String>,
}

/// Proxy Pod info for ExternalName service connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPodInfo {
    pub name: String,
    pub namespace: String,
    /// Target type: mysql, redis, etc.
    pub target_type: Option<String>,
    /// Target host being proxied
    pub target_host: Option<String>,
    /// Target port being proxied
    pub target_port: Option<i32>,
    /// Pod status: Running, Pending, etc.
    pub status: String,
    /// Associated Service name
    pub service_name: Option<String>,
}

/// K8s Ingress info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sIngressInfo {
    pub name: String,
    pub namespace: String,
    pub hosts: Vec<String>,
    pub address: Option<String>,
    pub created_at: Option<String>,
}

/// K8s Pod detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sPodDetail {
    /// Pod name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Pod status (Running, Pending, etc.)
    pub status: String,
    /// Pod phase
    pub phase: String,
    /// Node name
    pub node: Option<String>,
    /// Pod IP
    pub ip: Option<String>,
    /// Host IP
    pub host_ip: Option<String>,
    /// Start time
    pub start_time: Option<String>,
    /// Container info list
    pub containers: Vec<K8sContainerInfo>,
    /// Init container info list
    pub init_containers: Vec<K8sContainerInfo>,
    /// Pod conditions
    pub conditions: Vec<K8sPodCondition>,
    /// Labels
    pub labels: std::collections::HashMap<String, String>,
}

/// K8s Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sContainerInfo {
    /// Container name
    pub name: String,
    /// Container image
    pub image: String,
    /// Image pull policy
    pub image_pull_policy: Option<String>,
    /// Container ports
    pub ports: Vec<K8sContainerPort>,
    /// Environment variables
    pub env: Vec<K8sEnvVar>,
    /// Resource requirements
    pub resources: Option<K8sResourceRequirements>,
    /// Container state (running, waiting, terminated)
    pub state: String,
    /// Ready status
    pub ready: bool,
    /// Restart count
    pub restart_count: i32,
}

/// K8s Container port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sContainerPort {
    /// Port name
    pub name: Option<String>,
    /// Container port number
    pub container_port: i32,
    /// Protocol (TCP, UDP)
    pub protocol: String,
}

/// K8s Environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sEnvVar {
    /// Variable name
    pub name: String,
    /// Variable value (if directly set)
    pub value: Option<String>,
    /// Value source (configMapKeyRef, secretKeyRef, etc.)
    pub value_from: Option<String>,
}

/// K8s Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sResourceRequirements {
    /// CPU request
    pub cpu_request: Option<String>,
    /// Memory request
    pub memory_request: Option<String>,
    /// CPU limit
    pub cpu_limit: Option<String>,
    /// Memory limit
    pub memory_limit: Option<String>,
}

/// K8s Pod condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sPodCondition {
    /// Condition type
    pub condition_type: String,
    /// Status (True, False, Unknown)
    pub status: String,
    /// Last transition time
    pub last_transition_time: Option<String>,
    /// Reason
    pub reason: Option<String>,
    /// Message
    pub message: Option<String>,
}

// ==================== Extended K8s Workload Models ====================

/// K8s Job resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sJob {
    /// Job name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Desired completions
    pub completions: Option<i32>,
    /// Number of pods which reached phase Succeeded
    pub succeeded: i32,
    /// Number of pods which reached phase Failed
    pub failed: i32,
    /// Number of actively running pods
    pub active: i32,
    /// Represents time when the job was acknowledged by the job controller
    pub start_time: Option<String>,
    /// Represents time when the job was completed
    pub completion_time: Option<String>,
    /// Created timestamp
    pub created_at: Option<String>,
}

/// K8s CronJob resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sCronJob {
    /// CronJob name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Cron schedule expression
    pub schedule: String,
    /// Whether the job is suspended
    pub suspend: bool,
    /// Number of active jobs
    pub active: i32,
    /// Last time the job was scheduled
    pub last_schedule_time: Option<String>,
    /// Last time the job was successfully scheduled
    pub last_successful_time: Option<String>,
    /// Created timestamp
    pub created_at: Option<String>,
}

/// K8s StatefulSet resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sStatefulSet {
    /// StatefulSet name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Desired number of pods
    pub replicas: i32,
    /// Total number of pods with ready condition
    pub ready_replicas: i32,
    /// Total number of pods created by the controller
    pub current_replicas: i32,
    /// Total number of pods with updated template spec
    pub updated_replicas: i32,
    /// Created timestamp
    pub created_at: Option<String>,
}

/// K8s DaemonSet resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sDaemonSet {
    /// DaemonSet name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Number of nodes that should be running the daemon pod
    pub desired_number_scheduled: i32,
    /// Number of nodes that are running the daemon pod
    pub current_number_scheduled: i32,
    /// Number of nodes that should be running the daemon pod and have a Ready condition
    pub number_ready: i32,
    /// Number of nodes that should be running the daemon pod and have available
    pub number_available: i32,
    /// Created timestamp
    pub created_at: Option<String>,
}

/// K8s ReplicaSet resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sReplicaSet {
    /// ReplicaSet name
    pub name: String,
    /// Namespace
    pub namespace: String,
    /// Desired number of replicas
    pub replicas: i32,
    /// Number of pods that have labels matching pod template labels
    pub ready_replicas: i32,
    /// Number of available replicas
    pub available_replicas: i32,
    /// Created timestamp
    pub created_at: Option<String>,
}

// ==================== User Settings Models ====================

/// User setting entry (key-value store)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct UserSetting {
    /// Unique identifier
    pub id: Option<i64>,

    /// Setting key (unique)
    pub key: String,

    /// Setting value (JSON string)
    pub value: String,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Create or update setting request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertSettingRequest {
    pub key: String,
    pub value: serde_json::Value,
}

/// Batch get settings request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetSettingsRequest {
    pub keys: Vec<String>,
}

/// Batch settings response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSettingsResponse {
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

// ==================== LLM Config Models ====================

/// LLM configuration entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Default)]
pub struct LLMConfig {
    /// Unique identifier
    pub id: Option<i64>,

    /// Display name for the config
    pub name: String,

    /// Provider: openai, anthropic, custom
    pub provider: String,

    /// Encrypted API key (stored encrypted, not returned in responses)
    #[serde(skip_serializing)]
    pub api_key_encrypted: Option<String>,

    /// Custom base URL for API calls
    pub base_url: Option<String>,

    /// Model name (e.g., gpt-4, claude-3-opus)
    pub model: String,

    /// Maximum tokens for responses
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,

    /// Temperature for response randomness
    #[serde(default = "default_temperature")]
    pub temperature: f64,

    /// Whether this is the default config
    #[serde(default)]
    pub is_default: bool,

    /// Creation timestamp
    pub created_at: Option<String>,

    /// Last update timestamp
    pub updated_at: Option<String>,
}

fn default_max_tokens() -> i32 {
    2000
}

fn default_temperature() -> f64 {
    0.7
}

/// Create LLM config request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLLMConfigRequest {
    pub name: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f64>,
    pub is_default: Option<bool>,
}

/// Update LLM config request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLLMConfigRequest {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f64>,
    pub is_default: Option<bool>,
}

/// LLM config response (with masked API key indicator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfigResponse {
    pub id: i64,
    pub name: String,
    pub provider: String,
    pub has_api_key: bool,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: i32,
    pub temperature: f64,
    pub is_default: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<LLMConfig> for LLMConfigResponse {
    fn from(config: LLMConfig) -> Self {
        Self {
            id: config.id.unwrap_or(0),
            name: config.name,
            provider: config.provider,
            has_api_key: config.api_key_encrypted.is_some(),
            base_url: config.base_url,
            model: config.model,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            is_default: config.is_default,
            created_at: config.created_at,
            updated_at: config.updated_at,
        }
    }
}

// ==================== Database Object Management Types ====================

/// View information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInfo {
    pub name: String,
    pub definer: Option<String>,
    pub security_type: Option<String>,
    pub check_option: Option<String>,
    pub is_updatable: bool,
}

/// View definition (for viewing the CREATE VIEW statement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub name: String,
    pub definition: String,
}

/// Request to create a view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateViewRequest {
    pub name: String,
    pub definition: String,  // The SELECT statement
    pub or_replace: Option<bool>,
    pub algorithm: Option<String>,  // UNDEFINED, MERGE, TEMPTABLE
    pub security: Option<String>,   // DEFINER, INVOKER
}

/// Stored procedure/function information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureInfo {
    pub name: String,
    pub routine_type: String,  // PROCEDURE or FUNCTION
    pub definer: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub security_type: Option<String>,
    pub comment: Option<String>,
}

/// Stored procedure/function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureDefinition {
    pub name: String,
    pub routine_type: String,
    pub definition: String,
}

/// Trigger information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerInfo {
    pub name: String,
    pub event: String,      // INSERT, UPDATE, DELETE
    pub timing: String,     // BEFORE, AFTER
    pub table_name: String,
    pub definer: Option<String>,
    pub created: Option<String>,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDefinition {
    pub name: String,
    pub definition: String,
}

/// Event information (for scheduled events)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub name: String,
    pub definer: Option<String>,
    pub status: String,         // ENABLED, DISABLED, SLAVESIDE_DISABLED
    pub event_type: String,     // ONE TIME, RECURRING
    pub execute_at: Option<String>,
    pub interval_value: Option<String>,
    pub interval_field: Option<String>,
    pub starts: Option<String>,
    pub ends: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub name: String,
    pub definition: String,
}

// ==================== Server Monitoring Types ====================

/// Server variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVariable {
    pub name: String,
    pub value: String,
}

/// Process information from SHOW PROCESSLIST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: u64,
    pub user: String,
    pub host: String,
    pub db: Option<String>,
    pub command: String,
    pub time: u64,
    pub state: Option<String>,
    pub info: Option<String>,
}

/// Query explain result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResult {
    pub query: String,
    pub rows: Vec<serde_json::Value>,
}

/// Table check/repair result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMaintenanceResult {
    pub table_name: String,
    pub operation: String,
    pub msg_type: String,
    pub msg_text: String,
}
