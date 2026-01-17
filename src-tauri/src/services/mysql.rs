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
    AlterDatabaseRequest, AlterTableRequest, AlterUserPasswordRequest, Connection,
    CreateDatabaseRequest, CreateForeignKeyRequest, CreateIndexRequest, CreateTableRequest,
    CreateUserRequest, CreateViewRequest, DropUserRequest, ExplainResult, ExportFormat,
    ExportTableRequest, ExportTableResponse, ForeignKeyInfo, GrantPrivilegesRequest,
    ImportDataRequest, ImportResult, IndexInfo, MysqlColumn, MysqlDatabase, MysqlIndex,
    MysqlQueryResult, MysqlServerInfo, MysqlTable, MysqlTableData, MysqlTableSchema,
    MysqlUserInfo, ProcedureDefinition, ProcedureInfo, ProcessInfo, RevokePrivilegesRequest,
    ServerVariable, TableMaintenanceResult, TriggerDefinition, TriggerInfo, UserGrantInfo,
    UserGrantsResponse, ViewDefinition, ViewInfo,
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
        use urlencoding::encode;

        let password = conn.password.as_deref().unwrap_or("");
        let username = conn.username.as_deref().unwrap_or("root");

        // For K8s connections, use forward_local_port if available (port forwarding active)
        // Otherwise fall back to the original port
        let effective_port = conn
            .forward_local_port
            .filter(|&p| p > 0)
            .unwrap_or(conn.port);

        log::info!(
            "MysqlService::connect - connection_id: {:?}, username: {}, password provided: {}, password length: {}, host: {}, port: {} (forward_local_port: {:?})",
            conn.id,
            username,
            !password.is_empty(),
            password.len(),
            conn.host,
            effective_port,
            conn.forward_local_port
        );

        // URL-encode username and password to handle special characters like / @ :
        let encoded_username = encode(username);
        let encoded_password = encode(password);

        let url = format!(
            "mysql://{}:{}@{}:{}",
            encoded_username, encoded_password, conn.host, effective_port
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
        // Use raw query to handle VARBINARY return type from some MySQL configurations
        let rows = sqlx::query("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut databases = Vec::new();

        for row in rows {
            // Try to get as String first, then as bytes if that fails
            let name: String = row
                .try_get::<String, _>(0)
                .or_else(|_| {
                    row.try_get::<Vec<u8>, _>(0)
                        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                })
                .map_err(|e| AppError::Database(e.to_string()))?;
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

    /// Helper function to extract string from row that might be VARCHAR or VARBINARY
    fn get_string_from_row(row: &MySqlRow, column: &str) -> String {
        row.try_get::<String, _>(column)
            .or_else(|_| {
                row.try_get::<Vec<u8>, _>(column)
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            })
            .unwrap_or_default()
    }

    /// Helper function to extract optional string from row
    fn get_optional_string_from_row(row: &MySqlRow, column: &str) -> Option<String> {
        row.try_get::<Option<String>, _>(column)
            .ok()
            .flatten()
            .or_else(|| {
                row.try_get::<Option<Vec<u8>>, _>(column)
                    .ok()
                    .flatten()
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            })
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
                name: Self::get_string_from_row(row, "name"),
                engine: Self::get_optional_string_from_row(row, "engine"),
                row_count: row.try_get::<Option<i64>, _>("row_count").ok().flatten().unwrap_or(0),
                data_size: row.try_get::<Option<i64>, _>("data_size").ok().flatten().unwrap_or(0),
                index_size: row.try_get::<Option<i64>, _>("index_size").ok().flatten().unwrap_or(0),
                comment: Self::get_optional_string_from_row(row, "comment"),
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
                name: Self::get_string_from_row(row, "name"),
                column_type: Self::get_string_from_row(row, "type"),
                nullable: Self::get_string_from_row(row, "nullable") == "YES",
                key: Self::get_optional_string_from_row(row, "key"),
                default: Self::get_optional_string_from_row(row, "default"),
                extra: Self::get_optional_string_from_row(row, "extra"),
                comment: Self::get_optional_string_from_row(row, "comment"),
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
            let name = Self::get_string_from_row(row, "name");
            let column_name = Self::get_string_from_row(row, "column_name");
            let non_unique: i32 = row.try_get("non_unique").unwrap_or(1);
            let index_type = Self::get_string_from_row(row, "index_type");

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
    /// Uses raw_sql to avoid prepared statements, which some MySQL proxies don't support
    pub async fn execute_query(&self, database: &str, query: &str) -> AppResult<MysqlQueryResult> {
        let start = Instant::now();
        let query_type = detect_query_type(query);

        // Combine USE and query into a single raw_sql call to ensure same connection
        // For SELECT queries, we need special handling to get rows back
        if query_type == "select" || query_type == "show" || query_type == "describe" {
            // For queries that return rows, use USE statement first, then the query
            // Build the full query with USE prepended
            let full_query = format!("USE `{}`; {}", database, query);

            // SELECT query - return rows using raw_sql to avoid prepared statements
            let rows = sqlx::raw_sql(&full_query)
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
            let full_query = format!("USE `{}`; {}", database, query);
            let result = sqlx::raw_sql(&full_query)
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
        // Use CAST to convert BINARY fields to CHAR for MySQL 8.0 compatibility
        // MySQL 8.0 uses utf8mb3_bin collation for User and Host fields
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT CAST(User AS CHAR) as user, CAST(Host AS CHAR) as host \
             FROM mysql.user ORDER BY User, Host",
        )
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

    /// Alter user password
    pub async fn alter_user_password(&self, req: &AlterUserPasswordRequest) -> AppResult<()> {
        // MySQL 5.7+ uses ALTER USER syntax
        let query = format!(
            "ALTER USER '{}'@'{}' IDENTIFIED BY '{}'",
            req.username.replace('\'', "''"),
            req.host.replace('\'', "''"),
            req.new_password.replace('\'', "''")
        );

        log::info!("Altering password for user {}@{}", req.username, req.host);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Flush privileges to ensure changes take effect
        sqlx::query("FLUSH PRIVILEGES")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Drop a MySQL user
    pub async fn drop_user(&self, req: &DropUserRequest) -> AppResult<()> {
        let query = format!(
            "DROP USER '{}'@'{}'",
            req.username.replace('\'', "''"),
            req.host.replace('\'', "''")
        );

        log::info!("Dropping user {}@{}", req.username, req.host);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Show grants for a user
    pub async fn show_grants(&self, username: &str, host: &str) -> AppResult<UserGrantsResponse> {
        let query = format!(
            "SHOW GRANTS FOR '{}'@'{}'",
            username.replace('\'', "''"),
            host.replace('\'', "''")
        );

        log::info!("Showing grants for {}@{}", username, host);

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let grants: Vec<UserGrantInfo> = rows
            .iter()
            .map(|row| {
                // SHOW GRANTS returns a single column with the grant statement
                let grant_statement: String = row.try_get::<String, _>(0)
                    .or_else(|_| {
                        row.try_get::<Vec<u8>, _>(0)
                            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                    })
                    .unwrap_or_default();
                UserGrantInfo { grant_statement }
            })
            .collect();

        Ok(UserGrantsResponse {
            username: username.to_string(),
            host: host.to_string(),
            grants,
        })
    }

    /// Revoke privileges from a user
    pub async fn revoke_privileges(&self, req: &RevokePrivilegesRequest) -> AppResult<()> {
        let privileges = req.privileges.join(", ");

        // Handle database scope: "*" means all databases (*.*), otherwise use db.*
        let db_scope = if req.database == "*" {
            "*.*".to_string()
        } else {
            format!("`{}`.*", req.database)
        };

        let query = format!(
            "REVOKE {} ON {} FROM '{}'@'{}'",
            privileges,
            db_scope,
            req.username.replace('\'', "''"),
            req.host.replace('\'', "''")
        );

        log::info!("Revoking privileges: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Table Management Methods ====================

    /// Create a new table
    pub async fn create_table(&self, database: &str, req: &CreateTableRequest) -> AppResult<()> {
        // Build column definitions
        let column_defs: Vec<String> = req
            .columns
            .iter()
            .map(|col| {
                let mut def = format!("`{}` {}", col.name, col.data_type);
                if !col.nullable {
                    def.push_str(" NOT NULL");
                }
                if col.auto_increment {
                    def.push_str(" AUTO_INCREMENT");
                }
                if let Some(default) = &col.default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
                if let Some(comment) = &col.comment {
                    def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
                }
                def
            })
            .collect();

        // Build primary key definition
        let pk_def = req.primary_key.as_ref().map(|pk_cols| {
            format!(
                "PRIMARY KEY ({})",
                pk_cols.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", ")
            )
        });

        // Build index definitions
        let index_defs: Vec<String> = req
            .indexes
            .as_ref()
            .map(|indexes| {
                indexes
                    .iter()
                    .map(|idx| {
                        let idx_cols = idx
                            .columns
                            .iter()
                            .map(|c| format!("`{}`", c))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let idx_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
                        format!("{} `{}` ({})", idx_type, idx.name, idx_cols)
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Combine all definitions
        let mut all_defs = column_defs;
        if let Some(pk) = pk_def {
            all_defs.push(pk);
        }
        all_defs.extend(index_defs);

        // Build table options
        let mut options = vec![];
        if let Some(engine) = &req.engine {
            options.push(format!("ENGINE={}", engine));
        }
        if let Some(charset) = &req.charset {
            options.push(format!("DEFAULT CHARSET={}", charset));
        }
        if let Some(collation) = &req.collation {
            options.push(format!("COLLATE={}", collation));
        }
        if let Some(comment) = &req.comment {
            options.push(format!("COMMENT='{}'", comment.replace('\'', "''")));
        }

        let options_str = if options.is_empty() {
            String::new()
        } else {
            format!(" {}", options.join(" "))
        };

        let query = format!(
            "CREATE TABLE `{}`.`{}` ({}){}",
            database,
            req.name,
            all_defs.join(", "),
            options_str
        );

        log::info!("Creating table: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Alter an existing table
    pub async fn alter_table(
        &self,
        database: &str,
        table: &str,
        req: &AlterTableRequest,
    ) -> AppResult<()> {
        let mut alterations = vec![];

        // Add columns
        if let Some(add_cols) = &req.add_columns {
            for col in add_cols {
                let mut def = format!("ADD COLUMN `{}` {}", col.name, col.data_type);
                if !col.nullable {
                    def.push_str(" NOT NULL");
                }
                if col.auto_increment {
                    def.push_str(" AUTO_INCREMENT");
                }
                if let Some(default) = &col.default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
                if let Some(comment) = &col.comment {
                    def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
                }
                alterations.push(def);
            }
        }

        // Drop columns
        if let Some(drop_cols) = &req.drop_columns {
            for col_name in drop_cols {
                alterations.push(format!("DROP COLUMN `{}`", col_name));
            }
        }

        // Modify columns
        if let Some(modify_cols) = &req.modify_columns {
            for col in modify_cols {
                let mut def = format!("MODIFY COLUMN `{}` {}", col.name, col.data_type);
                if !col.nullable {
                    def.push_str(" NOT NULL");
                } else {
                    def.push_str(" NULL");
                }
                if col.auto_increment {
                    def.push_str(" AUTO_INCREMENT");
                }
                if let Some(default) = &col.default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
                if let Some(comment) = &col.comment {
                    def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
                }
                alterations.push(def);
            }
        }

        // Rename column
        if let Some(rename) = &req.rename_column {
            alterations.push(format!(
                "RENAME COLUMN `{}` TO `{}`",
                rename.old_name, rename.new_name
            ));
        }

        // Add indexes
        if let Some(add_indexes) = &req.add_indexes {
            for idx in add_indexes {
                let idx_cols = idx
                    .columns
                    .iter()
                    .map(|c| format!("`{}`", c))
                    .collect::<Vec<_>>()
                    .join(", ");
                let idx_type = if idx.unique { "ADD UNIQUE INDEX" } else { "ADD INDEX" };
                alterations.push(format!("{} `{}` ({})", idx_type, idx.name, idx_cols));
            }
        }

        // Drop indexes
        if let Some(drop_indexes) = &req.drop_indexes {
            for idx_name in drop_indexes {
                alterations.push(format!("DROP INDEX `{}`", idx_name));
            }
        }

        if alterations.is_empty() {
            return Ok(());
        }

        let query = format!(
            "ALTER TABLE `{}`.`{}` {}",
            database,
            table,
            alterations.join(", ")
        );

        log::info!("Altering table: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Rename a table
    pub async fn rename_table(
        &self,
        database: &str,
        old_name: &str,
        new_name: &str,
    ) -> AppResult<()> {
        let query = format!(
            "RENAME TABLE `{}`.`{}` TO `{}`.`{}`",
            database, old_name, database, new_name
        );

        log::info!("Renaming table: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Truncate a table (delete all rows, reset auto-increment)
    pub async fn truncate_table(&self, database: &str, table: &str) -> AppResult<()> {
        let query = format!("TRUNCATE TABLE `{}`.`{}`", database, table);

        log::info!("Truncating table: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Copy a table (structure only or with data)
    pub async fn copy_table(
        &self,
        database: &str,
        source_table: &str,
        target_table: &str,
        with_data: bool,
    ) -> AppResult<()> {
        // First, create the table structure
        let create_query = format!(
            "CREATE TABLE `{}`.`{}` LIKE `{}`.`{}`",
            database, target_table, database, source_table
        );

        log::info!("Copying table structure: {}", create_query);

        sqlx::query(&create_query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // If with_data, also copy the data
        if with_data {
            let insert_query = format!(
                "INSERT INTO `{}`.`{}` SELECT * FROM `{}`.`{}`",
                database, target_table, database, source_table
            );

            log::info!("Copying table data: {}", insert_query);

            sqlx::query(&insert_query)
                .execute(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
        }

        Ok(())
    }

    // ==================== Index Management ====================

    /// List all indexes on a table
    pub async fn list_indexes(&self, database: &str, table: &str) -> AppResult<Vec<IndexInfo>> {
        let query = format!(
            r#"
            SELECT
                INDEX_NAME as name,
                GROUP_CONCAT(COLUMN_NAME ORDER BY SEQ_IN_INDEX) as columns,
                NOT NON_UNIQUE as is_unique,
                CAST(INDEX_TYPE AS CHAR) as index_type,
                INDEX_NAME = 'PRIMARY' as is_primary,
                INDEX_COMMENT as comment
            FROM INFORMATION_SCHEMA.STATISTICS
            WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
            GROUP BY INDEX_NAME, NON_UNIQUE, INDEX_TYPE, INDEX_COMMENT
            ORDER BY INDEX_NAME = 'PRIMARY' DESC, INDEX_NAME
            "#
        );

        let rows = sqlx::query(&query)
            .bind(database)
            .bind(table)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let indexes: Vec<IndexInfo> = rows
            .iter()
            .map(|row| {
                let columns_str: String = row.get("columns");
                IndexInfo {
                    name: row.get("name"),
                    columns: columns_str.split(',').map(|s| s.to_string()).collect(),
                    unique: row.get::<i32, _>("is_unique") != 0,
                    index_type: row.get("index_type"),
                    is_primary: row.get::<i32, _>("is_primary") != 0,
                    comment: row.try_get("comment").ok(),
                }
            })
            .collect();

        Ok(indexes)
    }

    /// Create an index on a table
    pub async fn create_index(
        &self,
        database: &str,
        table: &str,
        req: &CreateIndexRequest,
    ) -> AppResult<()> {
        let index_type = if req.unique { "UNIQUE INDEX" } else { "INDEX" };
        let columns = req
            .columns
            .iter()
            .map(|c| format!("`{}`", c))
            .collect::<Vec<_>>()
            .join(", ");

        let mut query = format!(
            "CREATE {} `{}` ON `{}`.`{}` ({})",
            index_type, req.name, database, table, columns
        );

        // Add index type if specified (BTREE, HASH)
        if let Some(ref idx_type) = req.index_type {
            query.push_str(&format!(" USING {}", idx_type));
        }

        // Add comment if specified
        if let Some(ref comment) = req.comment {
            query.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }

        log::info!("Creating index: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Drop an index from a table
    pub async fn drop_index(&self, database: &str, table: &str, index_name: &str) -> AppResult<()> {
        let query = format!("DROP INDEX `{}` ON `{}`.`{}`", index_name, database, table);

        log::info!("Dropping index: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Foreign Key Management ====================

    /// List all foreign keys on a table
    pub async fn list_foreign_keys(
        &self,
        database: &str,
        table: &str,
    ) -> AppResult<Vec<ForeignKeyInfo>> {
        let query = r#"
            SELECT
                kcu.CONSTRAINT_NAME as name,
                GROUP_CONCAT(DISTINCT kcu.COLUMN_NAME ORDER BY kcu.ORDINAL_POSITION) as columns,
                kcu.REFERENCED_TABLE_NAME as ref_table,
                GROUP_CONCAT(DISTINCT kcu.REFERENCED_COLUMN_NAME ORDER BY kcu.ORDINAL_POSITION) as ref_columns,
                rc.DELETE_RULE as on_delete,
                rc.UPDATE_RULE as on_update
            FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
            JOIN INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS rc
                ON kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA
                AND kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME
            WHERE kcu.TABLE_SCHEMA = ?
                AND kcu.TABLE_NAME = ?
                AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
            GROUP BY kcu.CONSTRAINT_NAME, kcu.REFERENCED_TABLE_NAME, rc.DELETE_RULE, rc.UPDATE_RULE
            ORDER BY kcu.CONSTRAINT_NAME
        "#;

        let rows = sqlx::query(query)
            .bind(database)
            .bind(table)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let foreign_keys: Vec<ForeignKeyInfo> = rows
            .iter()
            .map(|row| {
                let columns_str: String = row.get("columns");
                let ref_columns_str: String = row.get("ref_columns");
                ForeignKeyInfo {
                    name: row.get("name"),
                    columns: columns_str.split(',').map(|s| s.to_string()).collect(),
                    ref_table: row.get("ref_table"),
                    ref_columns: ref_columns_str.split(',').map(|s| s.to_string()).collect(),
                    on_delete: row.get("on_delete"),
                    on_update: row.get("on_update"),
                }
            })
            .collect();

        Ok(foreign_keys)
    }

    /// Create a foreign key on a table
    pub async fn create_foreign_key(
        &self,
        database: &str,
        table: &str,
        req: &CreateForeignKeyRequest,
    ) -> AppResult<()> {
        let columns = req
            .columns
            .iter()
            .map(|c| format!("`{}`", c))
            .collect::<Vec<_>>()
            .join(", ");
        let ref_columns = req
            .ref_columns
            .iter()
            .map(|c| format!("`{}`", c))
            .collect::<Vec<_>>()
            .join(", ");

        // Generate constraint name if not provided
        let constraint_name = req.name.clone().unwrap_or_else(|| {
            format!(
                "fk_{}_{}",
                table,
                req.columns.join("_")
            )
        });

        let query = format!(
            "ALTER TABLE `{}`.`{}` ADD CONSTRAINT `{}` FOREIGN KEY ({}) REFERENCES `{}` ({}) ON DELETE {} ON UPDATE {}",
            database,
            table,
            constraint_name,
            columns,
            req.ref_table,
            ref_columns,
            req.on_delete,
            req.on_update
        );

        log::info!("Creating foreign key: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Drop a foreign key from a table
    pub async fn drop_foreign_key(
        &self,
        database: &str,
        table: &str,
        fk_name: &str,
    ) -> AppResult<()> {
        let query = format!(
            "ALTER TABLE `{}`.`{}` DROP FOREIGN KEY `{}`",
            database, table, fk_name
        );

        log::info!("Dropping foreign key: {}", query);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Data Export ====================

    /// Export table data to CSV format
    pub async fn export_table_csv(
        &self,
        database: &str,
        table: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<u32>,
        include_headers: bool,
    ) -> AppResult<ExportTableResponse> {
        let (data, row_count) = self.fetch_export_data(database, table, columns, where_clause, limit).await?;

        let mut csv_output = String::new();

        if data.is_empty() {
            return Ok(ExportTableResponse {
                data: csv_output,
                format: "csv".to_string(),
                row_count: 0,
            });
        }

        // Get column names from first row
        let column_names: Vec<&String> = data[0].keys().collect();

        // Add headers
        if include_headers {
            csv_output.push_str(&column_names.iter().map(|c| escape_csv_field(c)).collect::<Vec<_>>().join(","));
            csv_output.push('\n');
        }

        // Add data rows
        for row in &data {
            let values: Vec<String> = column_names.iter().map(|col| {
                match row.get(*col) {
                    Some(v) => escape_csv_field(&json_to_string(v)),
                    None => String::new(),
                }
            }).collect();
            csv_output.push_str(&values.join(","));
            csv_output.push('\n');
        }

        Ok(ExportTableResponse {
            data: csv_output,
            format: "csv".to_string(),
            row_count,
        })
    }

    /// Export table data to JSON format
    pub async fn export_table_json(
        &self,
        database: &str,
        table: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<u32>,
    ) -> AppResult<ExportTableResponse> {
        let (data, row_count) = self.fetch_export_data(database, table, columns, where_clause, limit).await?;

        let json_output = serde_json::to_string_pretty(&data)
            .map_err(|e| AppError::Database(format!("JSON serialization error: {}", e)))?;

        Ok(ExportTableResponse {
            data: json_output,
            format: "json".to_string(),
            row_count,
        })
    }

    /// Export table data to SQL INSERT statements
    pub async fn export_table_sql(
        &self,
        database: &str,
        table: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<u32>,
    ) -> AppResult<ExportTableResponse> {
        let (data, row_count) = self.fetch_export_data(database, table, columns, where_clause, limit).await?;

        if data.is_empty() {
            return Ok(ExportTableResponse {
                data: String::new(),
                format: "sql".to_string(),
                row_count: 0,
            });
        }

        let mut sql_output = String::new();

        // Get column names from first row
        let column_names: Vec<&String> = data[0].keys().collect();
        let columns_str = column_names.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", ");

        for row in &data {
            let values: Vec<String> = column_names.iter().map(|col| {
                match row.get(*col) {
                    Some(JsonValue::Null) => "NULL".to_string(),
                    Some(JsonValue::String(s)) => format!("'{}'", s.replace('\'', "\\'")),
                    Some(JsonValue::Number(n)) => n.to_string(),
                    Some(JsonValue::Bool(b)) => if *b { "1".to_string() } else { "0".to_string() },
                    Some(v) => format!("'{}'", v.to_string().replace('\'', "\\'")),
                    None => "NULL".to_string(),
                }
            }).collect();

            sql_output.push_str(&format!(
                "INSERT INTO `{}`.`{}` ({}) VALUES ({});\n",
                database, table, columns_str, values.join(", ")
            ));
        }

        Ok(ExportTableResponse {
            data: sql_output,
            format: "sql".to_string(),
            row_count,
        })
    }

    /// Export table data (unified method that dispatches based on format)
    pub async fn export_table(
        &self,
        database: &str,
        table: &str,
        request: &ExportTableRequest,
    ) -> AppResult<ExportTableResponse> {
        let columns = request.columns.as_deref();
        let where_clause = request.where_clause.as_deref();
        let limit = request.limit;

        match request.format {
            ExportFormat::Csv => {
                self.export_table_csv(database, table, columns, where_clause, limit, request.include_headers).await
            }
            ExportFormat::Json => {
                self.export_table_json(database, table, columns, where_clause, limit).await
            }
            ExportFormat::Sql => {
                self.export_table_sql(database, table, columns, where_clause, limit).await
            }
        }
    }

    /// Helper: Fetch data for export
    async fn fetch_export_data(
        &self,
        database: &str,
        table: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<u32>,
    ) -> AppResult<(Vec<HashMap<String, JsonValue>>, usize)> {
        let cols = match columns {
            Some(cols) if !cols.is_empty() => cols.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", "),
            _ => "*".to_string(),
        };

        let mut query = format!("SELECT {} FROM `{}`.`{}`", cols, database, table);

        if let Some(where_cl) = where_clause {
            if !where_cl.trim().is_empty() {
                query.push_str(&format!(" WHERE {}", where_cl));
            }
        }

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        log::info!("Export query: {}", query);

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let (_, data) = mysql_rows_to_json(&rows);
        let row_count = data.len();

        Ok((data, row_count))
    }

    // ==================== Data Import ====================

    /// Import data from CSV format
    pub async fn import_csv(
        &self,
        database: &str,
        table: &str,
        csv_data: &str,
        skip_rows: usize,
        on_duplicate: &str,
    ) -> AppResult<ImportResult> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());

        let headers: Vec<String> = reader.headers()
            .map_err(|e| AppError::Database(format!("CSV header error: {}", e)))?
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut imported = 0;
        let mut skipped = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        let mut row_num = 0;
        for result in reader.records() {
            row_num += 1;

            // Skip rows if requested
            if row_num <= skip_rows {
                skipped += 1;
                continue;
            }

            match result {
                Ok(record) => {
                    let values: Vec<String> = record.iter().map(|s| s.to_string()).collect();

                    match self.insert_import_row(database, table, &headers, &values, on_duplicate).await {
                        Ok(true) => imported += 1,
                        Ok(false) => skipped += 1,
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("Row {}: {}", row_num, e));
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Row {}: CSV parse error: {}", row_num, e));
                }
            }
        }

        Ok(ImportResult {
            imported,
            skipped,
            failed,
            errors,
        })
    }

    /// Import data from JSON format
    pub async fn import_json(
        &self,
        database: &str,
        table: &str,
        json_data: &str,
        on_duplicate: &str,
    ) -> AppResult<ImportResult> {
        let rows: Vec<HashMap<String, JsonValue>> = serde_json::from_str(json_data)
            .map_err(|e| AppError::Database(format!("JSON parse error: {}", e)))?;

        let mut imported = 0;
        let mut skipped = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (idx, row) in rows.iter().enumerate() {
            let columns: Vec<String> = row.keys().cloned().collect();
            let values: Vec<String> = columns.iter().map(|col| {
                json_to_sql_value(row.get(col))
            }).collect();

            match self.insert_import_row_sql(database, table, &columns, &values, on_duplicate).await {
                Ok(true) => imported += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("Row {}: {}", idx + 1, e));
                }
            }
        }

        Ok(ImportResult {
            imported,
            skipped,
            failed,
            errors,
        })
    }

    /// Import data (unified method that dispatches based on format)
    pub async fn import_data(
        &self,
        database: &str,
        table: &str,
        request: &ImportDataRequest,
    ) -> AppResult<ImportResult> {
        let on_duplicate = &request.on_duplicate;

        match request.format.to_lowercase().as_str() {
            "csv" => {
                self.import_csv(database, table, &request.data, request.skip_rows, on_duplicate).await
            }
            "json" => {
                self.import_json(database, table, &request.data, on_duplicate).await
            }
            _ => Err(AppError::Validation(format!("Unsupported import format: {}", request.format))),
        }
    }

    /// Helper: Insert a row during import (from CSV string values)
    async fn insert_import_row(
        &self,
        database: &str,
        table: &str,
        columns: &[String],
        values: &[String],
        on_duplicate: &str,
    ) -> AppResult<bool> {
        let cols = columns.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", ");
        let vals = values.iter().map(|v| {
            if v.is_empty() || v.to_lowercase() == "null" {
                "NULL".to_string()
            } else {
                format!("'{}'", v.replace('\'', "\\'"))
            }
        }).collect::<Vec<_>>().join(", ");

        let query = match on_duplicate {
            "ignore" => format!(
                "INSERT IGNORE INTO `{}`.`{}` ({}) VALUES ({})",
                database, table, cols, vals
            ),
            "update" => {
                let updates = columns.iter().zip(values.iter())
                    .map(|(c, v)| {
                        let val = if v.is_empty() || v.to_lowercase() == "null" {
                            "NULL".to_string()
                        } else {
                            format!("'{}'", v.replace('\'', "\\'"))
                        };
                        format!("`{}` = {}", c, val)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "INSERT INTO `{}`.`{}` ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
                    database, table, cols, vals, updates
                )
            }
            _ => format!(
                "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
                database, table, cols, vals
            ),
        };

        match sqlx::query(&query).execute(&self.pool).await {
            Ok(result) => Ok(result.rows_affected() > 0),
            Err(e) => {
                if on_duplicate == "ignore" {
                    Ok(false)
                } else {
                    Err(AppError::Database(e.to_string()))
                }
            }
        }
    }

    /// Helper: Insert a row during import (from pre-formatted SQL values)
    async fn insert_import_row_sql(
        &self,
        database: &str,
        table: &str,
        columns: &[String],
        values: &[String],
        on_duplicate: &str,
    ) -> AppResult<bool> {
        let cols = columns.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", ");
        let vals = values.join(", ");

        let query = match on_duplicate {
            "ignore" => format!(
                "INSERT IGNORE INTO `{}`.`{}` ({}) VALUES ({})",
                database, table, cols, vals
            ),
            "update" => {
                let updates = columns.iter().zip(values.iter())
                    .map(|(c, v)| format!("`{}` = {}", c, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "INSERT INTO `{}`.`{}` ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
                    database, table, cols, vals, updates
                )
            }
            _ => format!(
                "INSERT INTO `{}`.`{}` ({}) VALUES ({})",
                database, table, cols, vals
            ),
        };

        match sqlx::query(&query).execute(&self.pool).await {
            Ok(result) => Ok(result.rows_affected() > 0),
            Err(e) => {
                if on_duplicate == "ignore" {
                    Ok(false)
                } else {
                    Err(AppError::Database(e.to_string()))
                }
            }
        }
    }

    // ==================== View Management ====================

    /// List all views in a database
    pub async fn list_views(&self, database: &str) -> AppResult<Vec<ViewInfo>> {
        let query = format!(
            "SELECT TABLE_NAME, DEFINER, SECURITY_TYPE, CHECK_OPTION, IS_UPDATABLE \
             FROM information_schema.VIEWS WHERE TABLE_SCHEMA = '{}'",
            database.replace('\'', "''")
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let views = rows
            .iter()
            .map(|row| ViewInfo {
                name: row.try_get::<String, _>("TABLE_NAME").unwrap_or_default(),
                definer: row.try_get::<Option<String>, _>("DEFINER").ok().flatten(),
                security_type: row.try_get::<Option<String>, _>("SECURITY_TYPE").ok().flatten(),
                check_option: row.try_get::<Option<String>, _>("CHECK_OPTION").ok().flatten(),
                is_updatable: row.try_get::<String, _>("IS_UPDATABLE").unwrap_or_default() == "YES",
            })
            .collect();

        Ok(views)
    }

    /// Get view definition (CREATE VIEW statement)
    pub async fn get_view_definition(&self, database: &str, view: &str) -> AppResult<ViewDefinition> {
        let query = format!(
            "SHOW CREATE VIEW `{}`.`{}`",
            database, view
        );

        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // SHOW CREATE VIEW returns columns: View, Create View, character_set_client, collation_connection
        let definition = row.try_get::<String, _>(1)
            .or_else(|_| row.try_get::<String, _>("Create View"))
            .unwrap_or_default();

        Ok(ViewDefinition {
            name: view.to_string(),
            definition,
        })
    }

    /// Create a view
    pub async fn create_view(&self, database: &str, req: &CreateViewRequest) -> AppResult<()> {
        let or_replace = if req.or_replace.unwrap_or(false) { "OR REPLACE " } else { "" };
        let algorithm = req.algorithm.as_deref().map(|a| format!("ALGORITHM = {} ", a)).unwrap_or_default();
        let security = req.security.as_deref().map(|s| format!("SQL SECURITY {} ", s)).unwrap_or_default();

        let query = format!(
            "CREATE {}{}{}VIEW `{}`.`{}` AS {}",
            or_replace, algorithm, security, database, req.name, req.definition
        );

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Drop a view
    pub async fn drop_view(&self, database: &str, view: &str) -> AppResult<()> {
        let query = format!("DROP VIEW IF EXISTS `{}`.`{}`", database, view);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Stored Procedure Management ====================

    /// List all stored procedures and functions in a database
    pub async fn list_procedures(&self, database: &str) -> AppResult<Vec<ProcedureInfo>> {
        let query = format!(
            "SELECT ROUTINE_NAME, ROUTINE_TYPE, DEFINER, CREATED, LAST_ALTERED, SECURITY_TYPE, ROUTINE_COMMENT \
             FROM information_schema.ROUTINES WHERE ROUTINE_SCHEMA = '{}'",
            database.replace('\'', "''")
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let procedures = rows
            .iter()
            .map(|row| ProcedureInfo {
                name: row.try_get::<String, _>("ROUTINE_NAME").unwrap_or_default(),
                routine_type: row.try_get::<String, _>("ROUTINE_TYPE").unwrap_or_default(),
                definer: row.try_get::<Option<String>, _>("DEFINER").ok().flatten(),
                created: row.try_get::<Option<String>, _>("CREATED").ok().flatten(),
                modified: row.try_get::<Option<String>, _>("LAST_ALTERED").ok().flatten(),
                security_type: row.try_get::<Option<String>, _>("SECURITY_TYPE").ok().flatten(),
                comment: row.try_get::<Option<String>, _>("ROUTINE_COMMENT").ok().flatten(),
            })
            .collect();

        Ok(procedures)
    }

    /// Get stored procedure/function definition
    pub async fn get_procedure_definition(
        &self,
        database: &str,
        name: &str,
        routine_type: &str,
    ) -> AppResult<ProcedureDefinition> {
        let show_cmd = if routine_type.to_uppercase() == "FUNCTION" {
            "SHOW CREATE FUNCTION"
        } else {
            "SHOW CREATE PROCEDURE"
        };

        let query = format!("{} `{}`.`{}`", show_cmd, database, name);

        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Column index 2 contains the definition
        let definition = row.try_get::<String, _>(2)
            .unwrap_or_default();

        Ok(ProcedureDefinition {
            name: name.to_string(),
            routine_type: routine_type.to_string(),
            definition,
        })
    }

    /// Drop a stored procedure
    pub async fn drop_procedure(&self, database: &str, name: &str) -> AppResult<()> {
        let query = format!("DROP PROCEDURE IF EXISTS `{}`.`{}`", database, name);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Drop a function
    pub async fn drop_function(&self, database: &str, name: &str) -> AppResult<()> {
        let query = format!("DROP FUNCTION IF EXISTS `{}`.`{}`", database, name);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Trigger Management ====================

    /// List all triggers in a database
    pub async fn list_triggers(&self, database: &str) -> AppResult<Vec<TriggerInfo>> {
        let query = format!(
            "SELECT TRIGGER_NAME, EVENT_MANIPULATION, ACTION_TIMING, EVENT_OBJECT_TABLE, DEFINER, CREATED \
             FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = '{}'",
            database.replace('\'', "''")
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let triggers = rows
            .iter()
            .map(|row| TriggerInfo {
                name: row.try_get::<String, _>("TRIGGER_NAME").unwrap_or_default(),
                event: row.try_get::<String, _>("EVENT_MANIPULATION").unwrap_or_default(),
                timing: row.try_get::<String, _>("ACTION_TIMING").unwrap_or_default(),
                table_name: row.try_get::<String, _>("EVENT_OBJECT_TABLE").unwrap_or_default(),
                definer: row.try_get::<Option<String>, _>("DEFINER").ok().flatten(),
                created: row.try_get::<Option<String>, _>("CREATED").ok().flatten(),
            })
            .collect();

        Ok(triggers)
    }

    /// Get trigger definition
    pub async fn get_trigger_definition(&self, database: &str, name: &str) -> AppResult<TriggerDefinition> {
        let query = format!("SHOW CREATE TRIGGER `{}`.`{}`", database, name);

        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // SHOW CREATE TRIGGER returns: Trigger, sql_mode, SQL Original Statement, ...
        let definition = row.try_get::<String, _>(2)
            .unwrap_or_default();

        Ok(TriggerDefinition {
            name: name.to_string(),
            definition,
        })
    }

    /// Drop a trigger
    pub async fn drop_trigger(&self, database: &str, name: &str) -> AppResult<()> {
        let query = format!("DROP TRIGGER IF EXISTS `{}`.`{}`", database, name);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Server Monitoring ====================

    /// Get server variables
    pub async fn get_server_variables(&self, filter: Option<&str>) -> AppResult<Vec<ServerVariable>> {
        let query = match filter {
            Some(f) => format!("SHOW VARIABLES LIKE '%{}%'", f.replace('\'', "''")),
            None => "SHOW VARIABLES".to_string(),
        };

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let variables = rows
            .iter()
            .map(|row| ServerVariable {
                name: row.try_get::<String, _>(0).unwrap_or_default(),
                value: row.try_get::<String, _>(1).unwrap_or_default(),
            })
            .collect();

        Ok(variables)
    }

    /// Get process list
    pub async fn get_process_list(&self) -> AppResult<Vec<ProcessInfo>> {
        // Use information_schema.processlist for better compatibility with MySQL 8.0
        // Using column indices to avoid case-sensitivity issues with column names
        let rows = sqlx::query(
            "SELECT ID, USER, HOST, DB, COMMAND, TIME, STATE, INFO \
             FROM information_schema.processlist",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let processes = rows
            .iter()
            .map(|row| {
                // Use column indices (0-based) to avoid column name case issues
                // MySQL 8.0 may return different column name cases
                // ID and TIME fields are BIGINT UNSIGNED, use u64 directly
                ProcessInfo {
                    id: row.try_get::<u64, _>(0).unwrap_or(0),
                    user: row.try_get::<String, _>(1).unwrap_or_default(),
                    host: row.try_get::<String, _>(2).unwrap_or_default(),
                    db: row.try_get::<Option<String>, _>(3).ok().flatten(),
                    command: row.try_get::<String, _>(4).unwrap_or_default(),
                    time: row.try_get::<u64, _>(5).unwrap_or(0),
                    state: row.try_get::<Option<String>, _>(6).ok().flatten(),
                    info: row.try_get::<Option<String>, _>(7).ok().flatten(),
                }
            })
            .collect();

        Ok(processes)
    }

    /// Kill a process
    pub async fn kill_process(&self, process_id: u64) -> AppResult<()> {
        let query = format!("KILL {}", process_id);

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Explain a query
    pub async fn explain_query(&self, database: &str, query: &str) -> AppResult<ExplainResult> {
        let explain_query = format!("EXPLAIN {}", query);

        // Use the specified database
        sqlx::query(&format!("USE `{}`", database))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = sqlx::query(&explain_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let explain_rows: Vec<JsonValue> = rows
            .iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for col in row.columns() {
                    let value = mysql_value_to_json(row, col);
                    obj.insert(col.name().to_string(), value);
                }
                JsonValue::Object(obj)
            })
            .collect();

        Ok(ExplainResult {
            query: query.to_string(),
            rows: explain_rows,
        })
    }

    /// Optimize a table
    pub async fn optimize_table(&self, database: &str, table: &str) -> AppResult<TableMaintenanceResult> {
        let query = format!("OPTIMIZE TABLE `{}`.`{}`", database, table);

        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(TableMaintenanceResult {
            table_name: row.try_get::<String, _>("Table").unwrap_or_default(),
            operation: row.try_get::<String, _>("Op").unwrap_or_default(),
            msg_type: row.try_get::<String, _>("Msg_type").unwrap_or_default(),
            msg_text: row.try_get::<String, _>("Msg_text").unwrap_or_default(),
        })
    }

    /// Analyze a table
    pub async fn analyze_table(&self, database: &str, table: &str) -> AppResult<TableMaintenanceResult> {
        let query = format!("ANALYZE TABLE `{}`.`{}`", database, table);

        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(TableMaintenanceResult {
            table_name: row.try_get::<String, _>("Table").unwrap_or_default(),
            operation: row.try_get::<String, _>("Op").unwrap_or_default(),
            msg_type: row.try_get::<String, _>("Msg_type").unwrap_or_default(),
            msg_text: row.try_get::<String, _>("Msg_text").unwrap_or_default(),
        })
    }

    /// Check a table
    pub async fn check_table(&self, database: &str, table: &str) -> AppResult<TableMaintenanceResult> {
        let query = format!("CHECK TABLE `{}`.`{}`", database, table);

        let row = sqlx::query(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(TableMaintenanceResult {
            table_name: row.try_get::<String, _>("Table").unwrap_or_default(),
            operation: row.try_get::<String, _>("Op").unwrap_or_default(),
            msg_type: row.try_get::<String, _>("Msg_type").unwrap_or_default(),
            msg_text: row.try_get::<String, _>("Msg_text").unwrap_or_default(),
        })
    }
}

/// Escape a field for CSV output
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Convert JSON value to string for CSV
fn json_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        _ => value.to_string(),
    }
}

/// Convert JSON value to SQL value string
fn json_to_sql_value(value: Option<&JsonValue>) -> String {
    match value {
        None | Some(JsonValue::Null) => "NULL".to_string(),
        Some(JsonValue::String(s)) => format!("'{}'", s.replace('\'', "\\'")),
        Some(JsonValue::Number(n)) => n.to_string(),
        Some(JsonValue::Bool(b)) => if *b { "1".to_string() } else { "0".to_string() },
        Some(v) => format!("'{}'", v.to_string().replace('\'', "\\'")),
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

/// Convert MySQL rows to JSON format (returns objects with column names as keys)
fn mysql_rows_to_json(rows: &[MySqlRow]) -> (Vec<String>, Vec<HashMap<String, JsonValue>>) {
    if rows.is_empty() {
        return (vec![], vec![]);
    }

    // Get column names from the first row
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    // Convert each row to JSON object with column names as keys
    let json_rows: Vec<HashMap<String, JsonValue>> = rows
        .iter()
        .map(|row| {
            row.columns()
                .iter()
                .map(|col| (col.name().to_string(), mysql_value_to_json(row, col)))
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
