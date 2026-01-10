//! Tauri commands for MySQL operations
//!
//! These commands are exposed to the frontend via IPC.

use std::collections::HashMap;

use serde_json::Value as JsonValue;
use tauri::State;

use crate::db::models::{
    AlterDatabaseRequest, CreateDatabaseRequest, CreateUserRequest, GrantPrivilegesRequest,
    MysqlDatabase, MysqlQueryResult, MysqlServerInfo, MysqlTable, MysqlTableData,
    MysqlTableSchema, MysqlUserInfo,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{ConnectionService, MysqlService};

/// Helper to get connection and create MySQL service
async fn get_mysql_service(
    pool: &SqlitePool,
    connection_id: i64,
) -> Result<MysqlService, AppError> {
    let service = ConnectionService::new(pool.clone());
    let conn = service.get_by_id(connection_id).await?;

    if conn.conn_type != "mysql" {
        return Err(AppError::Validation(
            "Connection is not MySQL type".to_string(),
        ));
    }

    MysqlService::connect(&conn).await
}

/// Get MySQL server info
#[tauri::command]
pub async fn mysql_get_info(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<MysqlServerInfo, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.get_info().await
}

/// List all databases
#[tauri::command]
pub async fn mysql_list_databases(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<Vec<MysqlDatabase>, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.list_databases().await
}

/// Create a new database
#[tauri::command]
pub async fn mysql_create_database(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    data: CreateDatabaseRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.create_database(&data).await
}

/// Alter database settings
#[tauri::command]
pub async fn mysql_alter_database(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    name: String,
    data: AlterDatabaseRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.alter_database(&name, &data).await
}

/// Drop a database
#[tauri::command]
pub async fn mysql_drop_database(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.drop_database(&name).await
}

/// List tables in a database
#[tauri::command]
pub async fn mysql_list_tables(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
) -> Result<Vec<MysqlTable>, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.list_tables(&database).await
}

/// Drop a table
#[tauri::command]
pub async fn mysql_drop_table(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.drop_table(&database, &table).await
}

/// Get table schema
#[tauri::command]
pub async fn mysql_get_table_schema(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<MysqlTableSchema, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.get_table_schema(&database, &table).await
}

/// Get table primary key
#[tauri::command]
pub async fn mysql_get_table_primary_key(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<String, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.get_table_primary_key(&database, &table).await
}

/// Execute a SQL query
#[tauri::command]
pub async fn mysql_execute_query(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    query: String,
) -> Result<MysqlQueryResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.execute_query(&database, &query).await
}

/// Get table rows with pagination
#[tauri::command]
pub async fn mysql_get_rows(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<MysqlTableData, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(100);
    mysql.get_rows(&database, &table, page, page_size).await
}

/// Insert a row into a table
#[tauri::command]
pub async fn mysql_insert_row(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
    data: HashMap<String, JsonValue>,
) -> Result<u64, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.insert_row(&database, &table, &data).await
}

/// Update a record by primary key
#[tauri::command]
pub async fn mysql_update_record(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
    primary_key: String,
    primary_value: JsonValue,
    updates: HashMap<String, JsonValue>,
) -> Result<u64, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql
        .update_record(&database, &table, &primary_key, &primary_value, &updates)
        .await
}

/// Delete a row
#[tauri::command]
pub async fn mysql_delete_row(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    table: String,
    where_clause: HashMap<String, JsonValue>,
) -> Result<u64, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.delete_row(&database, &table, &where_clause).await
}

/// List MySQL users
#[tauri::command]
pub async fn mysql_list_users(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<Vec<MysqlUserInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.list_users().await
}

/// Create a MySQL user
#[tauri::command]
pub async fn mysql_create_user(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    data: CreateUserRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.create_user(&data).await
}

/// Grant privileges to a user
#[tauri::command]
pub async fn mysql_grant_privileges(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    database: String,
    data: GrantPrivilegesRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), connection_id).await?;
    mysql.grant_privileges(&database, &data).await
}
