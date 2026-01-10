//! MySQL database operations service
//!
//! This service handles all MySQL-related operations including:
//! - Server info retrieval
//! - Database and table management
//! - Query execution
//! - Schema inspection

use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value as JsonValue;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row, TypeInfo};

use crate::db::models::{
    AlterDatabaseRequest, Connection, CreateDatabaseRequest, CreateUserRequest,
    GrantPrivilegesRequest, MysqlColumn, MysqlDatabase, MysqlIndex, MysqlQueryResult,
    MysqlServerInfo, MysqlTable, MysqlTableData, MysqlTableSchema, MysqlUserInfo,
};
use crate::error::{AppError, AppResult};

/// MySQL service for database operations
pub struct MysqlService {
    pool: MySqlPool,
    connection: Connection,
}

impl MysqlService {
    /// Create a new MySQL service by connecting to the database
    pub async fn connect(conn: &Connection) -> AppResult<Self> {
        let password = conn.password.as_deref().unwrap_or("");
        let username = conn.username.as_deref().unwrap_or("root");

        let url = format!(
            "mysql://{}:{}@{}:{}",
            username, password, conn.host, conn.port
        );

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&url)
            .await
            .map_err(|e| AppError::Connection(e.to_string()))?;

        Ok(Self {
            pool,
            connection: conn.clone(),
        })
    }

    /// Get MySQL server info
    pub async fn get_info(&self) -> AppResult<MysqlServerInfo> {
        let version: (String,) = sqlx::query_as("SELECT VERSION()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(MysqlServerInfo {
            version: version.0,
            host: self.connection.host.clone(),
            port: self.connection.port,
            connected: true,
        })
    }

    /// List all databases
    pub async fn list_databases(&self) -> AppResult<Vec<MysqlDatabase>> {
        let rows: Vec<(String,)> = sqlx::query_as("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut databases = Vec::new();

        for (name,) in rows {
            // Skip system databases for cleaner display
            if name == "information_schema" || name == "performance_schema" || name == "sys" {
                continue;
            }

            // Get table count
            let table_count = self.get_table_count(&name).await.unwrap_or(0);

            // Get database size
            let size = self.get_database_size(&name).await.unwrap_or_default();

            databases.push(MysqlDatabase {
                name,
                table_count,
                size,
            });
        }

        Ok(databases)
    }

    /// Get table count for a database
    async fn get_table_count(&self, database: &str) -> AppResult<i64> {
        let query = format!(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = '{}'",
            database
        );
        let row: (i64,) = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.0)
    }

    /// Get database size
    async fn get_database_size(&self, database: &str) -> AppResult<String> {
        let query = format!(
            r#"SELECT CONCAT(ROUND(SUM(data_length + index_length) / 1024 / 1024, 2), ' MB') AS size
               FROM information_schema.tables
               WHERE table_schema = '{}'"#,
            database
        );
        let row: (Option<String>,) = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.0.unwrap_or_else(|| "0 MB".to_string()))
    }

    /// Create a new database
    pub async fn create_database(&self, req: &CreateDatabaseRequest) -> AppResult<()> {
        let mut query = format!("CREATE DATABASE `{}`", req.name);

        if let Some(charset) = &req.charset {
            query.push_str(&format!(" CHARACTER SET {}", charset));
        }
        if let Some(collation) = &req.collation {
            query.push_str(&format!(" COLLATE {}", collation));
        }

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Alter database settings
    pub async fn alter_database(&self, name: &str, req: &AlterDatabaseRequest) -> AppResult<()> {
        let mut query = format!("ALTER DATABASE `{}`", name);

        if let Some(charset) = &req.charset {
            query.push_str(&format!(" CHARACTER SET {}", charset));
        }
        if let Some(collation) = &req.collation {
            query.push_str(&format!(" COLLATE {}", collation));
        }

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Drop a database
    pub async fn drop_database(&self, name: &str) -> AppResult<()> {
        let query = format!("DROP DATABASE `{}`", name);
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// List tables in a database
    pub async fn list_tables(&self, database: &str) -> AppResult<Vec<MysqlTable>> {
        let query = format!(
            r#"SELECT
                TABLE_NAME as name,
                ENGINE as engine,
                TABLE_ROWS as row_count,
                DATA_LENGTH as data_size,
                INDEX_LENGTH as index_size,
                TABLE_COMMENT as comment
            FROM information_schema.tables
            WHERE table_schema = '{}'
            AND table_type = 'BASE TABLE'
            ORDER BY TABLE_NAME"#,
            database
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let tables = rows
            .iter()
            .map(|row| MysqlTable {
                name: row.get("name"),
                engine: row.get("engine"),
                row_count: row.get::<Option<i64>, _>("row_count").unwrap_or(0),
                data_size: row.get::<Option<i64>, _>("data_size").unwrap_or(0),
                index_size: row.get::<Option<i64>, _>("index_size").unwrap_or(0),
                comment: row.get("comment"),
            })
            .collect();

        Ok(tables)
    }

    /// Drop a table
    pub async fn drop_table(&self, database: &str, table: &str) -> AppResult<()> {
        let query = format!("DROP TABLE `{}`.`{}`", database, table);
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get table schema
    pub async fn get_table_schema(
        &self,
        database: &str,
        table: &str,
    ) -> AppResult<MysqlTableSchema> {
        // Get columns
        let columns = self.get_table_columns(database, table).await?;

        // Get indexes
        let indexes = self.get_table_indexes(database, table).await?;

        Ok(MysqlTableSchema {
            name: table.to_string(),
            columns,
            indexes,
        })
    }

    /// Get table columns
    async fn get_table_columns(&self, database: &str, table: &str) -> AppResult<Vec<MysqlColumn>> {
        let query = format!(
            r#"SELECT
                COLUMN_NAME as name,
                COLUMN_TYPE as type,
                IS_NULLABLE as nullable,
                COLUMN_KEY as `key`,
                COLUMN_DEFAULT as `default`,
                EXTRA as extra,
                COLUMN_COMMENT as comment
            FROM information_schema.columns
            WHERE table_schema = '{}' AND table_name = '{}'
            ORDER BY ORDINAL_POSITION"#,
            database, table
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let columns = rows
            .iter()
            .map(|row| MysqlColumn {
                name: row.get("name"),
                column_type: row.get("type"),
                nullable: row.get::<String, _>("nullable") == "YES",
                key: row.get("key"),
                default: row.get("default"),
                extra: row.get("extra"),
                comment: row.get("comment"),
            })
            .collect();

        Ok(columns)
    }

    /// Get table indexes
    async fn get_table_indexes(&self, database: &str, table: &str) -> AppResult<Vec<MysqlIndex>> {
        let query = format!(
            r#"SELECT
                INDEX_NAME as name,
                COLUMN_NAME as column_name,
                NON_UNIQUE as non_unique,
                INDEX_TYPE as index_type
            FROM information_schema.statistics
            WHERE table_schema = '{}' AND table_name = '{}'
            ORDER BY INDEX_NAME, SEQ_IN_INDEX"#,
            database, table
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Group columns by index name
        let mut index_map: HashMap<String, MysqlIndex> = HashMap::new();

        for row in &rows {
            let name: String = row.get("name");
            let column_name: String = row.get("column_name");
            let non_unique: i32 = row.get("non_unique");
            let index_type: String = row.get("index_type");

            index_map
                .entry(name.clone())
                .and_modify(|idx| idx.columns.push(column_name.clone()))
                .or_insert(MysqlIndex {
                    name,
                    columns: vec![column_name],
                    unique: non_unique == 0,
                    index_type,
                });
        }

        Ok(index_map.into_values().collect())
    }

    /// Get table primary key column
    pub async fn get_table_primary_key(&self, database: &str, table: &str) -> AppResult<String> {
        let query = format!(
            r#"SELECT COLUMN_NAME
            FROM information_schema.KEY_COLUMN_USAGE
            WHERE TABLE_SCHEMA = '{}'
            AND TABLE_NAME = '{}'
            AND CONSTRAINT_NAME = 'PRIMARY'
            ORDER BY ORDINAL_POSITION
            LIMIT 1"#,
            database, table
        );

        let row: Option<(String,)> = sqlx::query_as(&query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        row.map(|(name,)| name)
            .ok_or_else(|| AppError::NotFound("No primary key found".to_string()))
    }

    /// Execute a SQL query
    pub async fn execute_query(&self, database: &str, query: &str) -> AppResult<MysqlQueryResult> {
        // Use the specified database
        sqlx::query(&format!("USE `{}`", database))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let start = Instant::now();
        let query_type = detect_query_type(query);

        if query_type == "select" || query_type == "show" || query_type == "describe" {
            // SELECT query - return rows
            let rows = sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            let execution_time_ms = start.elapsed().as_millis() as u64;

            let (columns, json_rows) = mysql_rows_to_json(&rows);

            Ok(MysqlQueryResult {
                columns,
                rows: json_rows,
                affected_rows: rows.len() as u64,
                execution_time_ms,
                query_type,
            })
        } else {
            // Non-SELECT query - return affected rows count
            let result = sqlx::query(query)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            let execution_time_ms = start.elapsed().as_millis() as u64;

            Ok(MysqlQueryResult {
                columns: vec![],
                rows: vec![],
                affected_rows: result.rows_affected(),
                execution_time_ms,
                query_type,
            })
        }
    }

    /// Get table data with pagination
    pub async fn get_rows(
        &self,
        database: &str,
        table: &str,
        page: i32,
        page_size: i32,
    ) -> AppResult<MysqlTableData> {
        let offset = (page - 1) * page_size;

        // Get total count
        let count_query = format!("SELECT COUNT(*) FROM `{}`.`{}`", database, table);
        let total: (i64,) = sqlx::query_as(&count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Get rows
        let query = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
            database, table, page_size, offset
        );
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let (columns, json_rows) = mysql_rows_to_json(&rows);

        Ok(MysqlTableData {
            columns,
            rows: json_rows,
            total: total.0,
            page,
            page_size,
        })
    }

    /// Insert a row into a table
    pub async fn insert_row(
        &self,
        database: &str,
        table: &str,
        data: &HashMap<String, JsonValue>,
    ) -> AppResult<u64> {
        if data.is_empty() {
            return Err(AppError::Validation("No data provided".to_string()));
        }

        let columns: Vec<&String> = data.keys().collect();
        let placeholders: Vec<String> = (0..columns.len()).map(|_| "?".to_string()).collect();

        let query = format!(
            "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
            database,
            table,
            columns
                .iter()
                .map(|c| format!("`{}`", c))
                .collect::<Vec<_>>()
                .join(", "),
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&query);

        for col in &columns {
            q = bind_json_value(q, data.get(*col).unwrap());
        }

        let result = q
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.last_insert_id())
    }

    /// Update a record by primary key
    pub async fn update_record(
        &self,
        database: &str,
        table: &str,
        primary_key: &str,
        primary_value: &JsonValue,
        updates: &HashMap<String, JsonValue>,
    ) -> AppResult<u64> {
        if updates.is_empty() {
            return Err(AppError::Validation("No updates provided".to_string()));
        }

        let set_clauses: Vec<String> = updates.keys().map(|k| format!("`{}` = ?", k)).collect();

        let query = format!(
            "UPDATE `{}`.`{}` SET {} WHERE `{}` = ?",
            database,
            table,
            set_clauses.join(", "),
            primary_key
        );

        let mut q = sqlx::query(&query);

        // Bind update values
        for value in updates.values() {
            q = bind_json_value(q, value);
        }

        // Bind primary key value
        q = bind_json_value(q, primary_value);

        let result = q
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Delete a row by conditions
    pub async fn delete_row(
        &self,
        database: &str,
        table: &str,
        where_clause: &HashMap<String, JsonValue>,
    ) -> AppResult<u64> {
        if where_clause.is_empty() {
            return Err(AppError::Validation(
                "WHERE clause is required for delete".to_string(),
            ));
        }

        let conditions: Vec<String> = where_clause.keys().map(|k| format!("`{}` = ?", k)).collect();

        let query = format!(
            "DELETE FROM `{}`.`{}` WHERE {}",
            database,
            table,
            conditions.join(" AND ")
        );

        let mut q = sqlx::query(&query);

        for value in where_clause.values() {
            q = bind_json_value(q, value);
        }

        let result = q
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// List MySQL users
    pub async fn list_users(&self) -> AppResult<Vec<MysqlUserInfo>> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT User, Host FROM mysql.user ORDER BY User, Host")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|(user, host)| MysqlUserInfo { user, host })
            .collect())
    }

    /// Create a MySQL user
    pub async fn create_user(&self, req: &CreateUserRequest) -> AppResult<()> {
        let query = format!(
            "CREATE USER '{}'@'{}' IDENTIFIED BY '{}'",
            req.username, req.host, req.password
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// Grant privileges to a user
    pub async fn grant_privileges(
        &self,
        database: &str,
        req: &GrantPrivilegesRequest,
    ) -> AppResult<()> {
        let privileges = req.privileges.join(", ");
        let query = format!(
            "GRANT {} ON `{}`.* TO '{}'@'{}'",
            privileges, database, req.username, req.host
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}

/// Detect the type of SQL query
fn detect_query_type(query: &str) -> String {
    let trimmed = query.trim().to_lowercase();
    if trimmed.starts_with("select") {
        "select".to_string()
    } else if trimmed.starts_with("insert") {
        "insert".to_string()
    } else if trimmed.starts_with("update") {
        "update".to_string()
    } else if trimmed.starts_with("delete") {
        "delete".to_string()
    } else if trimmed.starts_with("show") {
        "show".to_string()
    } else if trimmed.starts_with("describe") || trimmed.starts_with("desc") {
        "describe".to_string()
    } else if trimmed.starts_with("create") {
        "create".to_string()
    } else if trimmed.starts_with("drop") {
        "drop".to_string()
    } else if trimmed.starts_with("alter") {
        "alter".to_string()
    } else {
        "other".to_string()
    }
}

/// Convert MySQL rows to JSON format
fn mysql_rows_to_json(rows: &[MySqlRow]) -> (Vec<String>, Vec<Vec<JsonValue>>) {
    if rows.is_empty() {
        return (vec![], vec![]);
    }

    // Get column names from the first row
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    // Convert each row to JSON values
    let json_rows: Vec<Vec<JsonValue>> = rows
        .iter()
        .map(|row| {
            row.columns()
                .iter()
                .map(|col| mysql_value_to_json(row, col))
                .collect()
        })
        .collect();

    (columns, json_rows)
}

/// Convert a single MySQL column value to JSON
fn mysql_value_to_json(row: &MySqlRow, col: &sqlx::mysql::MySqlColumn) -> JsonValue {
    let type_name = col.type_info().name();
    let idx = col.ordinal();

    // Try to get the value based on type
    match type_name {
        "BIGINT" | "BIGINT UNSIGNED" => row
            .try_get::<Option<i64>, _>(idx)
            .ok()
            .flatten()
            .map(JsonValue::from)
            .unwrap_or(JsonValue::Null),

        "INT" | "INT UNSIGNED" | "MEDIUMINT" | "SMALLINT" | "TINYINT" => row
            .try_get::<Option<i32>, _>(idx)
            .ok()
            .flatten()
            .map(JsonValue::from)
            .unwrap_or(JsonValue::Null),

        "FLOAT" | "DOUBLE" | "DECIMAL" => row
            .try_get::<Option<f64>, _>(idx)
            .ok()
            .flatten()
            .map(JsonValue::from)
            .unwrap_or(JsonValue::Null),

        "BOOLEAN" | "BOOL" => row
            .try_get::<Option<bool>, _>(idx)
            .ok()
            .flatten()
            .map(JsonValue::from)
            .unwrap_or(JsonValue::Null),

        "JSON" => row
            .try_get::<Option<JsonValue>, _>(idx)
            .ok()
            .flatten()
            .unwrap_or(JsonValue::Null),

        _ => {
            // Default to string for all other types
            row.try_get::<Option<String>, _>(idx)
                .ok()
                .flatten()
                .map(JsonValue::from)
                .unwrap_or(JsonValue::Null)
        }
    }
}

/// Bind a JSON value to a query
fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    value: &'q JsonValue,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match value {
        JsonValue::Null => query.bind(Option::<String>::None),
        JsonValue::Bool(b) => query.bind(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        JsonValue::String(s) => query.bind(s.as_str()),
        JsonValue::Array(_) | JsonValue::Object(_) => query.bind(value.to_string()),
    }
}
