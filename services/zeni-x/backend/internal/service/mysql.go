package service

import (
	"database/sql"
	"fmt"
	"strings"

	_ "github.com/go-sql-driver/mysql"
	"github.com/zeni-x/backend/internal/store"
)

// MySQLService MySQL 服务
type MySQLService struct {
	// No longer holds global config - connections are passed dynamically
}

// NewMySQLService 创建 MySQL 服务
func NewMySQLService() *MySQLService {
	return &MySQLService{}
}

// connect 创建数据库连接
func (s *MySQLService) connect(conn *store.Connection, database string) (*sql.DB, error) {
	dsn := fmt.Sprintf("%s:%s@tcp(%s:%d)/%s?parseTime=true&charset=utf8mb4",
		conn.Username,
		conn.Password,
		conn.Host,
		conn.Port,
		database,
	)

	db, err := sql.Open("mysql", dsn)
	if err != nil {
		return nil, err
	}

	if err := db.Ping(); err != nil {
		db.Close()
		return nil, err
	}

	return db, nil
}

// ServerInfo MySQL 服务器信息
type ServerInfo struct {
	Version   string `json:"version"`
	Host      string `json:"host"`
	Port      int    `json:"port"`
	Connected bool   `json:"connected"`
}

// GetInfo 获取服务器信息
func (s *MySQLService) GetInfo(conn *store.Connection) (*ServerInfo, error) {
	db, err := s.connect(conn, "")
	if err != nil {
		return &ServerInfo{
			Host:      conn.Host,
			Port:      conn.Port,
			Connected: false,
		}, nil
	}
	defer db.Close()

	var version string
	if err := db.QueryRow("SELECT VERSION()").Scan(&version); err != nil {
		return nil, err
	}

	return &ServerInfo{
		Version:   version,
		Host:      conn.Host,
		Port:      conn.Port,
		Connected: true,
	}, nil
}

// Database 数据库信息
type Database struct {
	Name       string `json:"name"`
	TableCount int    `json:"table_count"`
	Size       string `json:"size"`
}

// ListDatabases 列出所有数据库
func (s *MySQLService) ListDatabases(conn *store.Connection) ([]Database, error) {
	db, err := s.connect(conn, "")
	if err != nil {
		return nil, err
	}
	defer db.Close()

	rows, err := db.Query("SHOW DATABASES")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var databases []Database
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return nil, err
		}
		// 排除系统数据库
		if name == "information_schema" || name == "performance_schema" || name == "mysql" || name == "sys" {
			continue
		}
		databases = append(databases, Database{Name: name})
	}

	return databases, nil
}

// CreateDatabaseRequest 创建数据库请求
type CreateDatabaseRequest struct {
	Name         string `json:"name" binding:"required"`
	IfNotExists  bool   `json:"if_not_exists"`
	CharSet      string `json:"charset"`
	Collate      string `json:"collate"`
}

// CreateDatabase 创建数据库
func (s *MySQLService) CreateDatabase(conn *store.Connection, req *CreateDatabaseRequest) error {
	db, err := s.connect(conn, "")
	if err != nil {
		return err
	}
	defer db.Close()

	// 构建 SQL 语句
	var sqlParts []string
	sqlParts = append(sqlParts, "CREATE DATABASE")

	if req.IfNotExists {
		sqlParts = append(sqlParts, "IF NOT EXISTS")
	}

	sqlParts = append(sqlParts, fmt.Sprintf("`%s`", req.Name))

	// 添加 CHARACTER SET
	charset := req.CharSet
	if charset == "" {
		charset = "utf8mb4"
	}
	sqlParts = append(sqlParts, fmt.Sprintf("CHARACTER SET %s", charset))

	// 添加 COLLATE
	collate := req.Collate
	if collate == "" {
		// 使用默认的 utf8mb4 排序规则
		collate = "utf8mb4_unicode_ci"
	}
	sqlParts = append(sqlParts, fmt.Sprintf("COLLATE %s", collate))

	query := strings.Join(sqlParts, " ")
	_, err = db.Exec(query)
	return err
}

// AlterDatabaseRequest 修改数据库请求
type AlterDatabaseRequest struct {
	CharSet      string `json:"charset"`
	Collate      string `json:"collate"`
}

// AlterDatabase 修改数据库属性
func (s *MySQLService) AlterDatabase(conn *store.Connection, name string, req *AlterDatabaseRequest) error {
	db, err := s.connect(conn, "")
	if err != nil {
		return err
	}
	defer db.Close()

	// 构建 SQL 语句
	var sqlParts []string
	sqlParts = append(sqlParts, "ALTER DATABASE")
	sqlParts = append(sqlParts, fmt.Sprintf("`%s`", name))

	// 添加 CHARACTER SET（如果提供）
	if req.CharSet != "" {
		sqlParts = append(sqlParts, fmt.Sprintf("CHARACTER SET %s", req.CharSet))
	}

	// 添加 COLLATE（如果提供）
	if req.Collate != "" {
		sqlParts = append(sqlParts, fmt.Sprintf("COLLATE %s", req.Collate))
	}

	query := strings.Join(sqlParts, " ")
	_, err = db.Exec(query)
	return err
}

// GrantPrivilegesRequest 授权请求
type GrantPrivilegesRequest struct {
	UserName    string   `json:"username" binding:"required"`
	UserHost    string   `json:"user_host"`
	Password    string   `json:"password"`
	Privileges   []string `json:"privileges" binding:"required"`
	GrantOption  bool     `json:"grant_option"`
}

// GrantPrivileges 授予用户数据库权限
func (s *MySQLService) GrantPrivileges(conn *store.Connection, database string, req *GrantPrivilegesRequest) error {
	db, err := s.connect(conn, "")
	if err != nil {
		return err
	}
	defer db.Close()

	// 如果提供了密码，先创建或更新用户
	if req.Password != "" {
		userHost := req.UserHost
		if userHost == "" {
			userHost = "%"
		}

		// 使用 CREATE USER IF NOT EXISTS 或 ALTER USER
		createUserSQL := fmt.Sprintf("CREATE USER IF NOT EXISTS '%s'@'%s' IDENTIFIED BY '%s'",
			req.UserName, userHost, req.Password)
		_, err = db.Exec(createUserSQL)
		if err != nil {
			return err
		}
	}

	// 处理权限列表：如果包含 ALL PRIVILEGES，则只使用 ALL PRIVILEGES
	var privileges string
	hasAllPrivileges := false
	for _, priv := range req.Privileges {
		if priv == "ALL PRIVILEGES" {
			hasAllPrivileges = true
			break
		}
	}

	if hasAllPrivileges {
		privileges = "ALL PRIVILEGES"
	} else {
		privileges = strings.Join(req.Privileges, ", ")
	}

	userHost := req.UserHost
	if userHost == "" {
		userHost = "%"
	}

	var grantOption string
	if req.GrantOption {
		grantOption = " WITH GRANT OPTION"
	}

	query := fmt.Sprintf("GRANT %s ON `%s`.* TO '%s'@'%s'%s",
		privileges, database, req.UserName, userHost, grantOption)

	_, err = db.Exec(query)
	if err != nil {
		return err
	}

	// 刷新权限
	_, err = db.Exec("FLUSH PRIVILEGES")
	return err
}

// DropDatabase 删除数据库
func (s *MySQLService) DropDatabase(conn *store.Connection, name string) error {
	db, err := s.connect(conn, "")
	if err != nil {
		return err
	}
	defer db.Close()

	_, err = db.Exec(fmt.Sprintf("DROP DATABASE `%s`", name))
	return err
}

// Table 表信息
type Table struct {
	Name      string `json:"name"`
	Engine    string `json:"engine"`
	RowCount  int64  `json:"row_count"`
	DataSize  int64  `json:"data_size"`
	IndexSize int64  `json:"index_size"`
	Comment   string `json:"comment"`
}

// ListTables 列出数据库中的所有表
func (s *MySQLService) ListTables(conn *store.Connection, database string) ([]Table, error) {
	db, err := s.connect(conn, database)
	if err != nil {
		return nil, err
	}
	defer db.Close()

	query := `
		SELECT 
			TABLE_NAME, 
			ENGINE, 
			IFNULL(TABLE_ROWS, 0),
			IFNULL(DATA_LENGTH, 0),
			IFNULL(INDEX_LENGTH, 0),
			IFNULL(TABLE_COMMENT, '')
		FROM information_schema.TABLES 
		WHERE TABLE_SCHEMA = ?
	`

	rows, err := db.Query(query, database)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var tables []Table
	for rows.Next() {
		var t Table
		if err := rows.Scan(&t.Name, &t.Engine, &t.RowCount, &t.DataSize, &t.IndexSize, &t.Comment); err != nil {
			return nil, err
		}
		tables = append(tables, t)
	}

	return tables, nil
}

// Column 列信息
type Column struct {
	Name     string  `json:"name"`
	Type     string  `json:"type"`
	Nullable bool    `json:"nullable"`
	Key      string  `json:"key"`
	Default  *string `json:"default"`
	Extra    string  `json:"extra"`
	Comment  string  `json:"comment"`
}

// TableSchema 表结构
type TableSchema struct {
	Name    string   `json:"name"`
	Columns []Column `json:"columns"`
	Indexes []Index  `json:"indexes"`
}

// Index 索引信息
type Index struct {
	Name    string   `json:"name"`
	Columns []string `json:"columns"`
	Unique  bool     `json:"unique"`
	Type    string   `json:"type"`
}

// GetTableSchema 获取表结构
func (s *MySQLService) GetTableSchema(conn *store.Connection, database, table string) (*TableSchema, error) {
	db, err := s.connect(conn, database)
	if err != nil {
		return nil, err
	}
	defer db.Close()

	// 获取列信息
	columnsQuery := `
		SELECT 
			COLUMN_NAME,
			COLUMN_TYPE,
			IS_NULLABLE,
			IFNULL(COLUMN_KEY, ''),
			COLUMN_DEFAULT,
			IFNULL(EXTRA, ''),
			IFNULL(COLUMN_COMMENT, '')
		FROM information_schema.COLUMNS 
		WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
		ORDER BY ORDINAL_POSITION
	`

	rows, err := db.Query(columnsQuery, database, table)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var columns []Column
	for rows.Next() {
		var c Column
		var nullable string
		if err := rows.Scan(&c.Name, &c.Type, &nullable, &c.Key, &c.Default, &c.Extra, &c.Comment); err != nil {
			return nil, err
		}
		c.Nullable = nullable == "YES"
		columns = append(columns, c)
	}

	// 获取索引信息
	indexQuery := `
		SELECT 
			INDEX_NAME,
			COLUMN_NAME,
			NON_UNIQUE,
			INDEX_TYPE
		FROM information_schema.STATISTICS 
		WHERE TABLE_SCHEMA = ? AND TABLE_NAME = ?
		ORDER BY INDEX_NAME, SEQ_IN_INDEX
	`

	indexRows, err := db.Query(indexQuery, database, table)
	if err != nil {
		return nil, err
	}
	defer indexRows.Close()

	indexMap := make(map[string]*Index)
	for indexRows.Next() {
		var indexName, columnName, indexType string
		var nonUnique int
		if err := indexRows.Scan(&indexName, &columnName, &nonUnique, &indexType); err != nil {
			return nil, err
		}
		if idx, ok := indexMap[indexName]; ok {
			idx.Columns = append(idx.Columns, columnName)
		} else {
			indexMap[indexName] = &Index{
				Name:    indexName,
				Columns: []string{columnName},
				Unique:  nonUnique == 0,
				Type:    indexType,
			}
		}
	}

	var indexes []Index
	for _, idx := range indexMap {
		indexes = append(indexes, *idx)
	}

	return &TableSchema{
		Name:    table,
		Columns: columns,
		Indexes: indexes,
	}, nil
}

// CreateTableRequest 创建表请求
type CreateTableRequest struct {
	Name    string      `json:"name"`
	Columns []ColumnDef `json:"columns"`
	Engine  string      `json:"engine"`
	Comment string      `json:"comment"`
}

// ColumnDef 列定义
type ColumnDef struct {
	Name          string  `json:"name"`
	Type          string  `json:"type"`
	Length        int     `json:"length,omitempty"`
	Nullable      bool    `json:"nullable"`
	Default       *string `json:"default,omitempty"`
	PrimaryKey    bool    `json:"primary_key"`
	AutoIncrement bool    `json:"auto_increment"`
	Comment       string  `json:"comment,omitempty"`
}

// CreateTable 创建表
func (s *MySQLService) CreateTable(conn *store.Connection, database string, req *CreateTableRequest) error {
	db, err := s.connect(conn, database)
	if err != nil {
		return err
	}
	defer db.Close()

	var columnDefs []string
	var primaryKeys []string

	for _, col := range req.Columns {
		def := fmt.Sprintf("`%s` %s", col.Name, col.Type)
		if col.Length > 0 {
			def = fmt.Sprintf("`%s` %s(%d)", col.Name, col.Type, col.Length)
		}
		if !col.Nullable {
			def += " NOT NULL"
		}
		if col.Default != nil {
			def += fmt.Sprintf(" DEFAULT '%s'", *col.Default)
		}
		if col.AutoIncrement {
			def += " AUTO_INCREMENT"
		}
		if col.Comment != "" {
			def += fmt.Sprintf(" COMMENT '%s'", col.Comment)
		}
		if col.PrimaryKey {
			primaryKeys = append(primaryKeys, fmt.Sprintf("`%s`", col.Name))
		}
		columnDefs = append(columnDefs, def)
	}

	if len(primaryKeys) > 0 {
		columnDefs = append(columnDefs, fmt.Sprintf("PRIMARY KEY (%s)", strings.Join(primaryKeys, ", ")))
	}

	engine := req.Engine
	if engine == "" {
		engine = "InnoDB"
	}

	query := fmt.Sprintf("CREATE TABLE `%s` (%s) ENGINE=%s", req.Name, strings.Join(columnDefs, ", "), engine)
	if req.Comment != "" {
		query += fmt.Sprintf(" COMMENT='%s'", req.Comment)
	}

	_, err = db.Exec(query)
	return err
}

// DropTable 删除表
func (s *MySQLService) DropTable(conn *store.Connection, database, table string) error {
	db, err := s.connect(conn, database)
	if err != nil {
		return err
	}
	defer db.Close()

	_, err = db.Exec(fmt.Sprintf("DROP TABLE `%s`", table))
	return err
}

// AlterTableRequest 修改表请求
type AlterTableRequest struct {
	AddColumns    []ColumnDef `json:"add_columns,omitempty"`
	DropColumns   []string    `json:"drop_columns,omitempty"`
	ModifyColumns []ColumnDef `json:"modify_columns,omitempty"`
	RenameColumn  *RenameCol  `json:"rename_column,omitempty"`
}

// RenameCol 重命名列
type RenameCol struct {
	OldName string `json:"old_name"`
	NewName string `json:"new_name"`
}

// AlterTable 修改表结构
func (s *MySQLService) AlterTable(conn *store.Connection, database, table string, req *AlterTableRequest) error {
	db, err := s.connect(conn, database)
	if err != nil {
		return err
	}
	defer db.Close()

	var alterParts []string

	// 添加列
	for _, col := range req.AddColumns {
		def := fmt.Sprintf("ADD COLUMN `%s` %s", col.Name, col.Type)
		if !col.Nullable {
			def += " NOT NULL"
		}
		if col.Default != nil {
			def += fmt.Sprintf(" DEFAULT '%s'", *col.Default)
		}
		alterParts = append(alterParts, def)
	}

	// 删除列
	for _, colName := range req.DropColumns {
		alterParts = append(alterParts, fmt.Sprintf("DROP COLUMN `%s`", colName))
	}

	// 修改列
	for _, col := range req.ModifyColumns {
		def := fmt.Sprintf("MODIFY COLUMN `%s` %s", col.Name, col.Type)
		if !col.Nullable {
			def += " NOT NULL"
		}
		if col.Default != nil {
			def += fmt.Sprintf(" DEFAULT '%s'", *col.Default)
		}
		alterParts = append(alterParts, def)
	}

	// 重命名列
	if req.RenameColumn != nil {
		alterParts = append(alterParts, fmt.Sprintf("RENAME COLUMN `%s` TO `%s`", req.RenameColumn.OldName, req.RenameColumn.NewName))
	}

	if len(alterParts) == 0 {
		return nil
	}

	query := fmt.Sprintf("ALTER TABLE `%s` %s", table, strings.Join(alterParts, ", "))
	_, err = db.Exec(query)
	return err
}

// RowsResult 行查询结果
type RowsResult struct {
	Columns []string                 `json:"columns"`
	Rows    []map[string]interface{} `json:"rows"`
	Total   int64                    `json:"total"`
	Page    int                      `json:"page"`
	Size    int                      `json:"size"`
}

// GetRows 获取表数据
func (s *MySQLService) GetRows(conn *store.Connection, database, table string, page, size int) (*RowsResult, error) {
	db, err := s.connect(conn, database)
	if err != nil {
		return nil, err
	}
	defer db.Close()

	// 获取总数
	var total int64
	if err := db.QueryRow(fmt.Sprintf("SELECT COUNT(*) FROM `%s`", table)).Scan(&total); err != nil {
		return nil, err
	}

	// 获取数据
	offset := (page - 1) * size
	rows, err := db.Query(fmt.Sprintf("SELECT * FROM `%s` LIMIT %d OFFSET %d", table, size, offset))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	columns, err := rows.Columns()
	if err != nil {
		return nil, err
	}

	var result []map[string]interface{}
	for rows.Next() {
		values := make([]interface{}, len(columns))
		valuePtrs := make([]interface{}, len(columns))
		for i := range values {
			valuePtrs[i] = &values[i]
		}

		if err := rows.Scan(valuePtrs...); err != nil {
			return nil, err
		}

		row := make(map[string]interface{})
		for i, col := range columns {
			val := values[i]
			if b, ok := val.([]byte); ok {
				row[col] = string(b)
			} else {
				row[col] = val
			}
		}
		result = append(result, row)
	}

	return &RowsResult{
		Columns: columns,
		Rows:    result,
		Total:   total,
		Page:    page,
		Size:    size,
	}, nil
}

// InsertRow 插入行
func (s *MySQLService) InsertRow(conn *store.Connection, database, table string, data map[string]interface{}) error {
	db, err := s.connect(conn, database)
	if err != nil {
		return err
	}
	defer db.Close()

	var columns []string
	var placeholders []string
	var values []interface{}

	for col, val := range data {
		columns = append(columns, fmt.Sprintf("`%s`", col))
		placeholders = append(placeholders, "?")
		values = append(values, val)
	}

	query := fmt.Sprintf("INSERT INTO `%s` (%s) VALUES (%s)",
		table,
		strings.Join(columns, ", "),
		strings.Join(placeholders, ", "),
	)

	_, err = db.Exec(query, values...)
	return err
}

// UpdateRowRequest 更新行请求
type UpdateRowRequest struct {
	Where map[string]interface{} `json:"where"`
	Data  map[string]interface{} `json:"data"`
}

// UpdateRow 更新行
func (s *MySQLService) UpdateRow(conn *store.Connection, database, table string, req *UpdateRowRequest) error {
	db, err := s.connect(conn, database)
	if err != nil {
		return err
	}
	defer db.Close()

	var setClauses []string
	var values []interface{}

	for col, val := range req.Data {
		setClauses = append(setClauses, fmt.Sprintf("`%s` = ?", col))
		values = append(values, val)
	}

	var whereClauses []string
	for col, val := range req.Where {
		whereClauses = append(whereClauses, fmt.Sprintf("`%s` = ?", col))
		values = append(values, val)
	}

	query := fmt.Sprintf("UPDATE `%s` SET %s WHERE %s",
		table,
		strings.Join(setClauses, ", "),
		strings.Join(whereClauses, " AND "),
	)

	_, err = db.Exec(query, values...)
	return err
}

// DeleteRow 删除行
func (s *MySQLService) DeleteRow(conn *store.Connection, database, table string, where map[string]interface{}) error {
	db, err := s.connect(conn, database)
	if err != nil {
		return err
	}
	defer db.Close()

	var whereClauses []string
	var values []interface{}

	for col, val := range where {
		whereClauses = append(whereClauses, fmt.Sprintf("`%s` = ?", col))
		values = append(values, val)
	}

	query := fmt.Sprintf("DELETE FROM `%s` WHERE %s", table, strings.Join(whereClauses, " AND "))
	_, err = db.Exec(query, values...)
	return err
}

// QueryResult SQL 查询结果
type QueryResult struct {
	Columns      []string                 `json:"columns"`
	Rows         []map[string]interface{} `json:"rows"`
	RowsAffected int64                    `json:"rows_affected"`
	Duration     int64                    `json:"duration_ms"`
}

// ExecuteQuery 执行 SQL 查询
func (s *MySQLService) ExecuteQuery(conn *store.Connection, database, query string) (*QueryResult, error) {
	db, err := s.connect(conn, database)
	if err != nil {
		return nil, err
	}
	defer db.Close()

	// 判断是否是 SELECT 查询
	queryUpper := strings.TrimSpace(strings.ToUpper(query))
	isSelect := strings.HasPrefix(queryUpper, "SELECT") || strings.HasPrefix(queryUpper, "SHOW") || strings.HasPrefix(queryUpper, "DESCRIBE")

	if isSelect {
		rows, err := db.Query(query)
		if err != nil {
			return nil, err
		}
		defer rows.Close()

		columns, err := rows.Columns()
		if err != nil {
			return nil, err
		}

		var result []map[string]interface{}
		for rows.Next() {
			values := make([]interface{}, len(columns))
			valuePtrs := make([]interface{}, len(columns))
			for i := range values {
				valuePtrs[i] = &values[i]
			}

			if err := rows.Scan(valuePtrs...); err != nil {
				return nil, err
			}

			row := make(map[string]interface{})
			for i, col := range columns {
				val := values[i]
				if b, ok := val.([]byte); ok {
					row[col] = string(b)
				} else {
					row[col] = val
				}
			}
			result = append(result, row)
		}

		return &QueryResult{
			Columns: columns,
			Rows:    result,
		}, nil
	}

	// 非 SELECT 查询
	result, err := db.Exec(query)
	if err != nil {
		return nil, err
	}

	affected, _ := result.RowsAffected()

	return &QueryResult{
		RowsAffected: affected,
	}, nil
}

// TestConnection 测试连接是否有效
func (s *MySQLService) TestConnection(conn *store.Connection) error {
	db, err := s.connect(conn, "")
	if err != nil {
		return err
	}
	defer db.Close()
	return nil
}

// UserInfo 用户信息
type UserInfo struct {
	Host     string `json:"host"`
	User     string `json:"user"`
	Password string `json:"password,omitempty"`
}

// ListUsers 列出所有用户
func (s *MySQLService) ListUsers(conn *store.Connection) ([]UserInfo, error) {
	db, err := s.connect(conn, "")
	if err != nil {
		return nil, err
	}
	defer db.Close()

	query := `
		SELECT host, user
		FROM mysql.user
		WHERE user NOT IN ('mysql.sys', 'mysql.session')
		ORDER BY user, host
	`

	rows, err := db.Query(query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var users []UserInfo
	for rows.Next() {
		var u UserInfo
		if err := rows.Scan(&u.Host, &u.User); err != nil {
			return nil, err
		}
		users = append(users, u)
	}

	return users, nil
}

// CreateUserRequest 创建用户请求
type CreateUserRequest struct {
	UserName string `json:"username" binding:"required"`
	UserHost string `json:"user_host"`
	Password string `json:"password" binding:"required"`
}

// CreateUser 创建新用户
func (s *MySQLService) CreateUser(conn *store.Connection, req *CreateUserRequest) error {
	db, err := s.connect(conn, "")
	if err != nil {
		return err
	}
	defer db.Close()

	userHost := req.UserHost
	if userHost == "" {
		userHost = "%"
	}

	query := fmt.Sprintf("CREATE USER '%s'@'%s' IDENTIFIED BY '%s'",
		req.UserName, userHost, req.Password)

	_, err = db.Exec(query)
	return err
}

// UserGrants 用户权限信息
type UserGrants struct {
	Grantee string `json:"grantee"`
	Table   string `json:"table"`
	Privileges string `json:"privileges"`
}

// ListUserGrants 列出用户的权限
func (s *MySQLService) ListUserGrants(conn *store.Connection, username, host string) ([]UserGrants, error) {
	db, err := s.connect(conn, "")
	if err != nil {
		return nil, err
	}
	defer db.Close()

	query := fmt.Sprintf("SHOW GRANTS FOR '%s'@'%s'", username, host)
	rows, err := db.Query(query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var grants []UserGrants
	for rows.Next() {
		var grantText string
		if err := rows.Scan(&grantText); err != nil {
			return nil, err
		}

		// 解析 GRANT 语句
		// 示例: GRANT SELECT, INSERT ON `database`.* TO 'user'@'%'
		grants = append(grants, UserGrants{
			Grantee: fmt.Sprintf("'%s'@'%s'", username, host),
			Table: grantText, // 简化处理，直接存储完整的 GRANT 语句
			Privileges: grantText,
		})
	}

	return grants, nil
}

// GetDatabaseSchema 获取数据库的完整 Schema（用于自动补全）
func (s *MySQLService) GetDatabaseSchema(conn *store.Connection, database string) (map[string]interface{}, error) {
	db, err := s.connect(conn, database)
	if err != nil {
		return nil, err
	}
	defer db.Close()

	// 获取所有表
	tables, err := s.ListTables(conn, database)
	if err != nil {
		return nil, err
	}

	result := make(map[string]interface{})
	result["database"] = database
	result["tables"] = tables

	// 为每个表获取列信息
	tablesMap := make([]map[string]interface{}, 0)
	for _, table := range tables {
		schema, err := s.GetTableSchema(conn, database, table.Name)
		if err != nil {
			continue
		}

		tableMap := map[string]interface{}{
			"name":      table.Name,
			"engine":    table.Engine,
			"comment":   table.Comment,
			"columns":   schema.Columns,
			"indexes":   schema.Indexes,
		}
		tablesMap = append(tablesMap, tableMap)
	}
	result["tables_detail"] = tablesMap

	return result, nil
}
