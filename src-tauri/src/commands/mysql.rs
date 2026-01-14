//! Tauri commands for MySQL operations
//!
//! These commands are exposed to the frontend via IPC.

use std::collections::HashMap;

use serde_json::Value as JsonValue;
use tauri::State;

use crate::commands::PortForwardState;
use crate::db::models::{
    AlterDatabaseRequest, AlterTableRequest, AlterUserPasswordRequest, Connection,
    CopyTableRequest, CreateDatabaseRequest, CreateForeignKeyRequest, CreateIndexRequest,
    CreateTableRequest, CreateUserRequest, CreateViewRequest, DropUserRequest, ExportFormat,
    ExportTableRequest, ExportTableResponse, ExplainResult, ForeignKeyInfo, GrantPrivilegesRequest,
    ImportDataRequest, ImportResult, IndexInfo, MysqlDatabase, MysqlQueryResult, MysqlServerInfo,
    MysqlTable, MysqlTableData, MysqlTableSchema, MysqlUserInfo, ProcedureDefinition, ProcedureInfo,
    ProcessInfo, RenameTableRequest, RevokePrivilegesRequest, ServerVariable, TableMaintenanceResult,
    TriggerDefinition, TriggerInfo, UserGrantsResponse, ViewDefinition, ViewInfo,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{ConnectionService, MysqlService};

/// Helper to get connection and create MySQL service
/// For K8s connections, this will automatically start or use existing port forward
async fn get_mysql_service(
    pool: &SqlitePool,
    pf_state: &PortForwardState,
    connection_id: i64,
) -> Result<MysqlService, AppError> {
    let service = ConnectionService::new(pool.clone());
    let mut conn = service.get_by_id(connection_id).await?;

    if conn.conn_type != "mysql" {
        return Err(AppError::Validation(
            "Connection is not MySQL type".to_string(),
        ));
    }

    // For K8s connections, we need to ensure port forward is active
    if conn.source.as_deref() == Some("k8s") {
        conn = ensure_port_forward(pool, pf_state, conn).await?;
    }

    MysqlService::connect(&conn).await
}

/// Ensure port forward is active for K8s connection
/// Returns the connection with updated host/port for the local forward
async fn ensure_port_forward(
    pool: &SqlitePool,
    pf_state: &PortForwardState,
    mut conn: Connection,
) -> Result<Connection, AppError> {
    let connection_id = conn.id.ok_or_else(|| {
        AppError::Validation("Connection ID is required".to_string())
    })?;

    let service_arc = pf_state.get_or_init(pool.clone()).await;
    let guard = service_arc.read().await;
    let pf_service = guard.as_ref().ok_or_else(|| {
        AppError::Internal("Port forward service not initialized".to_string())
    })?;

    // Try to get existing port forward for this connection
    let pf = match pf_service.get_by_connection(connection_id).await {
        Ok(existing) => {
            // Check if it's still active
            if existing.status == "active" {
                log::info!(
                    "Using existing port forward for connection {}: localhost:{}",
                    connection_id,
                    existing.local_port
                );
                existing
            } else {
                // Reconnect if not active
                log::info!(
                    "Reconnecting port forward for connection {}: localhost:{}",
                    connection_id,
                    existing.local_port
                );
                let local_port = conn.forward_local_port.map(|p| p as u16);
                pf_service.reconnect(&existing.id.unwrap_or_default(), local_port).await?
            }
        }
        Err(_) => {
            // No existing forward, create new one
            log::info!("Starting new port forward for connection {}", connection_id);
            let local_port = conn.forward_local_port.map(|p| p as u16);
            pf_service.start(connection_id, local_port).await?
        }
    };

    // Update connection with forwarded port
    conn.host = "127.0.0.1".to_string();
    conn.port = pf.local_port;

    Ok(conn)
}

/// Get MySQL server info
#[tauri::command]
pub async fn mysql_get_info(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
) -> Result<MysqlServerInfo, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_info().await
}

/// List all databases
#[tauri::command]
pub async fn mysql_list_databases(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
) -> Result<Vec<MysqlDatabase>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_databases().await
}

/// Create a new database
#[tauri::command]
pub async fn mysql_create_database(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: CreateDatabaseRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.create_database(&data).await
}

/// Alter database settings
#[tauri::command]
pub async fn mysql_alter_database(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    name: String,
    data: AlterDatabaseRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.alter_database(&name, &data).await
}

/// Drop a database
#[tauri::command]
pub async fn mysql_drop_database(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_database(&name).await
}

/// List tables in a database
#[tauri::command]
pub async fn mysql_list_tables(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
) -> Result<Vec<MysqlTable>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_tables(&database).await
}

/// Drop a table
#[tauri::command]
pub async fn mysql_drop_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_table(&database, &table).await
}

/// Get table schema
#[tauri::command]
pub async fn mysql_get_table_schema(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<MysqlTableSchema, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_table_schema(&database, &table).await
}

/// Get table primary key
#[tauri::command]
pub async fn mysql_get_table_primary_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<String, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_table_primary_key(&database, &table).await
}

/// Execute a SQL query
#[tauri::command]
pub async fn mysql_execute_query(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    query: String,
) -> Result<MysqlQueryResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.execute_query(&database, &query).await
}

/// Get table rows with pagination
#[tauri::command]
pub async fn mysql_get_rows(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<MysqlTableData, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(100);
    mysql.get_rows(&database, &table, page, page_size).await
}

/// Insert a row into a table
#[tauri::command]
pub async fn mysql_insert_row(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: HashMap<String, JsonValue>,
) -> Result<u64, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.insert_row(&database, &table, &data).await
}

/// Update a record by primary key
#[tauri::command]
pub async fn mysql_update_record(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    primary_key: String,
    primary_value: JsonValue,
    updates: HashMap<String, JsonValue>,
) -> Result<u64, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql
        .update_record(&database, &table, &primary_key, &primary_value, &updates)
        .await
}

/// Delete a row
#[tauri::command]
pub async fn mysql_delete_row(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    where_clause: HashMap<String, JsonValue>,
) -> Result<u64, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.delete_row(&database, &table, &where_clause).await
}

/// List MySQL users
#[tauri::command]
pub async fn mysql_list_users(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
) -> Result<Vec<MysqlUserInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_users().await
}

/// Create a MySQL user
#[tauri::command]
pub async fn mysql_create_user(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: CreateUserRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.create_user(&data).await
}

/// Grant privileges to a user
#[tauri::command]
pub async fn mysql_grant_privileges(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    data: GrantPrivilegesRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.grant_privileges(&database, &data).await
}

/// Alter user password
#[tauri::command]
pub async fn mysql_alter_user_password(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: AlterUserPasswordRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.alter_user_password(&data).await
}

/// Drop a MySQL user
#[tauri::command]
pub async fn mysql_drop_user(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: DropUserRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_user(&data).await
}

/// Show grants for a user
#[tauri::command]
pub async fn mysql_show_grants(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    username: String,
    host: String,
) -> Result<UserGrantsResponse, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.show_grants(&username, &host).await
}

/// Revoke privileges from a user
#[tauri::command]
pub async fn mysql_revoke_privileges(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    data: RevokePrivilegesRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.revoke_privileges(&data).await
}

// ==================== Table Management Commands ====================

/// Create a new table
#[tauri::command]
pub async fn mysql_create_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    data: CreateTableRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.create_table(&database, &data).await
}

/// Alter an existing table
#[tauri::command]
pub async fn mysql_alter_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: AlterTableRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.alter_table(&database, &table, &data).await
}

/// Rename a table
#[tauri::command]
pub async fn mysql_rename_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: RenameTableRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.rename_table(&database, &table, &data.new_name).await
}

/// Truncate a table (delete all rows, reset auto-increment)
#[tauri::command]
pub async fn mysql_truncate_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.truncate_table(&database, &table).await
}

/// Copy a table (structure only or with data)
#[tauri::command]
pub async fn mysql_copy_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: CopyTableRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql
        .copy_table(&database, &table, &data.target_name, data.with_data)
        .await
}

// ==================== Index Management ====================

/// List all indexes on a table
#[tauri::command]
pub async fn mysql_list_indexes(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<Vec<IndexInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_indexes(&database, &table).await
}

/// Create an index on a table
#[tauri::command]
pub async fn mysql_create_index(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: CreateIndexRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.create_index(&database, &table, &data).await
}

/// Drop an index from a table
#[tauri::command]
pub async fn mysql_drop_index(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    index_name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_index(&database, &table, &index_name).await
}

// ==================== Foreign Key Management ====================

/// List all foreign keys on a table
#[tauri::command]
pub async fn mysql_list_foreign_keys(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<Vec<ForeignKeyInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_foreign_keys(&database, &table).await
}

/// Create a foreign key on a table
#[tauri::command]
pub async fn mysql_create_foreign_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: CreateForeignKeyRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.create_foreign_key(&database, &table, &data).await
}

/// Drop a foreign key from a table
#[tauri::command]
pub async fn mysql_drop_foreign_key(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    fk_name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_foreign_key(&database, &table, &fk_name).await
}

// ==================== Data Export ====================

/// Export table data to specified format (CSV, JSON, SQL)
#[tauri::command]
pub async fn mysql_export_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: ExportTableRequest,
) -> Result<ExportTableResponse, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;

    let columns = data.columns.as_ref().map(|v| v.as_slice());
    let where_clause = data.where_clause.as_deref();

    match data.format {
        ExportFormat::Csv => {
            mysql.export_table_csv(
                &database,
                &table,
                columns,
                where_clause,
                data.limit,
                data.include_headers,
            ).await
        }
        ExportFormat::Json => {
            mysql.export_table_json(&database, &table, columns, where_clause, data.limit).await
        }
        ExportFormat::Sql => {
            mysql.export_table_sql(&database, &table, columns, where_clause, data.limit).await
        }
    }
}

// ==================== Data Import ====================

/// Import data into a table
#[tauri::command]
pub async fn mysql_import_data(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
    data: ImportDataRequest,
) -> Result<ImportResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;

    match data.format.as_str() {
        "csv" => {
            mysql.import_csv(&database, &table, &data.data, data.skip_rows, &data.on_duplicate).await
        }
        "json" => {
            mysql.import_json(&database, &table, &data.data, &data.on_duplicate).await
        }
        _ => Err(AppError::Validation(format!("Unsupported import format: {}", data.format))),
    }
}

// ==================== View Management ====================

/// List all views in a database
#[tauri::command]
pub async fn mysql_list_views(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
) -> Result<Vec<ViewInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_views(&database).await
}

/// Get view definition
#[tauri::command]
pub async fn mysql_get_view_definition(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    view: String,
) -> Result<ViewDefinition, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_view_definition(&database, &view).await
}

/// Create a new view
#[tauri::command]
pub async fn mysql_create_view(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    data: CreateViewRequest,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.create_view(&database, &data).await
}

/// Drop a view
#[tauri::command]
pub async fn mysql_drop_view(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    view: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_view(&database, &view).await
}

// ==================== Stored Procedure Management ====================

/// List all stored procedures and functions in a database
#[tauri::command]
pub async fn mysql_list_procedures(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
) -> Result<Vec<ProcedureInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_procedures(&database).await
}

/// Get stored procedure/function definition
#[tauri::command]
pub async fn mysql_get_procedure_definition(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    name: String,
    routine_type: String,
) -> Result<ProcedureDefinition, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_procedure_definition(&database, &name, &routine_type).await
}

/// Drop a stored procedure
#[tauri::command]
pub async fn mysql_drop_procedure(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_procedure(&database, &name).await
}

/// Drop a function
#[tauri::command]
pub async fn mysql_drop_function(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_function(&database, &name).await
}

// ==================== Trigger Management ====================

/// List all triggers in a database
#[tauri::command]
pub async fn mysql_list_triggers(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
) -> Result<Vec<TriggerInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.list_triggers(&database).await
}

/// Get trigger definition
#[tauri::command]
pub async fn mysql_get_trigger_definition(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    name: String,
) -> Result<TriggerDefinition, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_trigger_definition(&database, &name).await
}

/// Drop a trigger
#[tauri::command]
pub async fn mysql_drop_trigger(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    name: String,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.drop_trigger(&database, &name).await
}

// ==================== Server Monitoring ====================

/// Get server variables
#[tauri::command]
pub async fn mysql_get_server_variables(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    filter: Option<String>,
) -> Result<Vec<ServerVariable>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_server_variables(filter.as_deref()).await
}

/// Get process list
#[tauri::command]
pub async fn mysql_get_process_list(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
) -> Result<Vec<ProcessInfo>, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.get_process_list().await
}

/// Kill a process
#[tauri::command]
pub async fn mysql_kill_process(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    process_id: u64,
) -> Result<(), AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.kill_process(process_id).await
}

// ==================== Query Analysis ====================

/// Explain a query
#[tauri::command]
pub async fn mysql_explain_query(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    query: String,
) -> Result<ExplainResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.explain_query(&database, &query).await
}

// ==================== Table Maintenance ====================

/// Optimize a table
#[tauri::command]
pub async fn mysql_optimize_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<TableMaintenanceResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.optimize_table(&database, &table).await
}

/// Analyze a table
#[tauri::command]
pub async fn mysql_analyze_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<TableMaintenanceResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.analyze_table(&database, &table).await
}

/// Check a table
#[tauri::command]
pub async fn mysql_check_table(
    pool: State<'_, SqlitePool>,
    pf_state: State<'_, PortForwardState>,
    connection_id: i64,
    database: String,
    table: String,
) -> Result<TableMaintenanceResult, AppError> {
    let mysql = get_mysql_service(pool.inner(), &pf_state, connection_id).await?;
    mysql.check_table(&database, &table).await
}
