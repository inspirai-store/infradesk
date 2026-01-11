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

fn default_true() -> bool {
    true
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
