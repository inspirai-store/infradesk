package store

import (
	"database/sql"
	"os"
	"path/filepath"

	_ "github.com/mattn/go-sqlite3"
)

// SQLite SQLite 数据库封装
type SQLite struct {
	db *sql.DB
}

// NewSQLite 创建 SQLite 连接
func NewSQLite(dbPath string) (*SQLite, error) {
	// 确保目录存在
	dir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return nil, err
	}

	db, err := sql.Open("sqlite3", dbPath)
	if err != nil {
		return nil, err
	}

	// 测试连接
	if err := db.Ping(); err != nil {
		return nil, err
	}

	s := &SQLite{db: db}

	// 初始化表结构
	if err := s.initTables(); err != nil {
		return nil, err
	}

	return s, nil
}

// Close 关闭连接
func (s *SQLite) Close() error {
	return s.db.Close()
}

// DB 获取底层数据库连接
func (s *SQLite) DB() *sql.DB {
	return s.db
}

// initTables 初始化表结构
func (s *SQLite) initTables() error {
	schema := `
	-- 连接配置（预留多连接支持）
	CREATE TABLE IF NOT EXISTS connections (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL,
		type TEXT NOT NULL,
		host TEXT NOT NULL,
		port INTEGER NOT NULL,
		username TEXT,
		password TEXT,
		database_name TEXT,
		is_default BOOLEAN DEFAULT 0,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);

	-- SQL 查询历史
	CREATE TABLE IF NOT EXISTS query_history (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		connection_id INTEGER,
		query_type TEXT,
		query_text TEXT NOT NULL,
		executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		duration_ms INTEGER,
		row_count INTEGER,
		FOREIGN KEY (connection_id) REFERENCES connections(id)
	);

	-- 收藏的查询
	CREATE TABLE IF NOT EXISTS saved_queries (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		connection_id INTEGER,
		name TEXT NOT NULL,
		query_text TEXT NOT NULL,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (connection_id) REFERENCES connections(id)
	);

	-- 创建索引
	CREATE INDEX IF NOT EXISTS idx_query_history_type ON query_history(query_type);
	CREATE INDEX IF NOT EXISTS idx_query_history_executed_at ON query_history(executed_at);
	CREATE INDEX IF NOT EXISTS idx_connections_type ON connections(type);
	`

	_, err := s.db.Exec(schema)
	return err
}

// Connection 连接配置
type Connection struct {
	ID           int64  `json:"id"`
	Name         string `json:"name"`
	Type         string `json:"type"`
	Host         string `json:"host"`
	Port         int    `json:"port"`
	Username     string `json:"username,omitempty"`
	Password     string `json:"password,omitempty"` // 允许接收密码，但在返回时需要手动清空
	DatabaseName string `json:"database_name,omitempty"`
	IsDefault    bool   `json:"is_default"`
	CreatedAt    string `json:"created_at"`
	UpdatedAt    string `json:"updated_at"`
}

// QueryHistory 查询历史
type QueryHistory struct {
	ID           int64  `json:"id"`
	ConnectionID int64  `json:"connection_id"`
	QueryType    string `json:"query_type"`
	QueryText    string `json:"query_text"`
	ExecutedAt   string `json:"executed_at"`
	DurationMs   int64  `json:"duration_ms"`
	RowCount     int64  `json:"row_count"`
}

// SavedQuery 收藏的查询
type SavedQuery struct {
	ID           int64  `json:"id"`
	ConnectionID int64  `json:"connection_id"`
	Name         string `json:"name"`
	QueryText    string `json:"query_text"`
	CreatedAt    string `json:"created_at"`
}

// GetConnections 获取所有连接配置
func (s *SQLite) GetConnections() ([]Connection, error) {
	rows, err := s.db.Query(`
		SELECT id, name, type, host, port, username, database_name, is_default, created_at, updated_at 
		FROM connections ORDER BY created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var connections []Connection
	for rows.Next() {
		var c Connection
		var username, dbName sql.NullString
		if err := rows.Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &dbName, &c.IsDefault, &c.CreatedAt, &c.UpdatedAt); err != nil {
			return nil, err
		}
		c.Username = username.String
		c.DatabaseName = dbName.String
		connections = append(connections, c)
	}

	return connections, nil
}

// CreateConnection 创建连接配置
func (s *SQLite) CreateConnection(c *Connection) error {
	result, err := s.db.Exec(`
		INSERT INTO connections (name, type, host, port, username, password, database_name, is_default)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?)
	`, c.Name, c.Type, c.Host, c.Port, c.Username, c.Password, c.DatabaseName, c.IsDefault)
	if err != nil {
		return err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return err
	}
	c.ID = id

	return nil
}

// GetQueryHistory 获取查询历史
func (s *SQLite) GetQueryHistory(queryType string, limit int) ([]QueryHistory, error) {
	query := `
		SELECT id, connection_id, query_type, query_text, executed_at, duration_ms, row_count 
		FROM query_history 
		WHERE query_type = ? OR ? = ''
		ORDER BY executed_at DESC 
		LIMIT ?
	`
	rows, err := s.db.Query(query, queryType, queryType, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var history []QueryHistory
	for rows.Next() {
		var h QueryHistory
		var connID, durationMs, rowCount sql.NullInt64
		if err := rows.Scan(&h.ID, &connID, &h.QueryType, &h.QueryText, &h.ExecutedAt, &durationMs, &rowCount); err != nil {
			return nil, err
		}
		h.ConnectionID = connID.Int64
		h.DurationMs = durationMs.Int64
		h.RowCount = rowCount.Int64
		history = append(history, h)
	}

	return history, nil
}

// AddQueryHistory 添加查询历史
func (s *SQLite) AddQueryHistory(h *QueryHistory) error {
	_, err := s.db.Exec(`
		INSERT INTO query_history (connection_id, query_type, query_text, duration_ms, row_count)
		VALUES (?, ?, ?, ?, ?)
	`, h.ConnectionID, h.QueryType, h.QueryText, h.DurationMs, h.RowCount)
	return err
}

// GetSavedQueries 获取收藏的查询
func (s *SQLite) GetSavedQueries() ([]SavedQuery, error) {
	rows, err := s.db.Query(`
		SELECT id, connection_id, name, query_text, created_at 
		FROM saved_queries 
		ORDER BY created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var queries []SavedQuery
	for rows.Next() {
		var q SavedQuery
		var connID sql.NullInt64
		if err := rows.Scan(&q.ID, &connID, &q.Name, &q.QueryText, &q.CreatedAt); err != nil {
			return nil, err
		}
		q.ConnectionID = connID.Int64
		queries = append(queries, q)
	}

	return queries, nil
}

// CreateSavedQuery 保存查询
func (s *SQLite) CreateSavedQuery(q *SavedQuery) error {
	result, err := s.db.Exec(`
		INSERT INTO saved_queries (connection_id, name, query_text)
		VALUES (?, ?, ?)
	`, q.ConnectionID, q.Name, q.QueryText)
	if err != nil {
		return err
	}

	id, err := result.LastInsertId()
	if err != nil {
		return err
	}
	q.ID = id

	return nil
}

// DeleteSavedQuery 删除收藏的查询
func (s *SQLite) DeleteSavedQuery(id int64) error {
	_, err := s.db.Exec(`DELETE FROM saved_queries WHERE id = ?`, id)
	return err
}

// GetConnectionByID 根据 ID 获取连接配置（包含密码，用于服务端连接）
func (s *SQLite) GetConnectionByID(id int64) (*Connection, error) {
	var c Connection
	var username, password, dbName sql.NullString
	err := s.db.QueryRow(`
		SELECT id, name, type, host, port, username, password, database_name, is_default, created_at, updated_at 
		FROM connections WHERE id = ?
	`, id).Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &password, &dbName, &c.IsDefault, &c.CreatedAt, &c.UpdatedAt)
	if err != nil {
		return nil, err
	}
	c.Username = username.String
	c.Password = password.String
	c.DatabaseName = dbName.String
	return &c, nil
}

// UpdateConnection 更新连接配置
func (s *SQLite) UpdateConnection(c *Connection) error {
	_, err := s.db.Exec(`
		UPDATE connections 
		SET name = ?, type = ?, host = ?, port = ?, username = ?, password = ?, database_name = ?, is_default = ?, updated_at = CURRENT_TIMESTAMP
		WHERE id = ?
	`, c.Name, c.Type, c.Host, c.Port, c.Username, c.Password, c.DatabaseName, c.IsDefault, c.ID)
	return err
}

// DeleteConnection 删除连接配置
func (s *SQLite) DeleteConnection(id int64) error {
	_, err := s.db.Exec(`DELETE FROM connections WHERE id = ?`, id)
	return err
}

// GetConnectionsByType 按类型获取连接配置列表
func (s *SQLite) GetConnectionsByType(connType string) ([]Connection, error) {
	rows, err := s.db.Query(`
		SELECT id, name, type, host, port, username, database_name, is_default, created_at, updated_at 
		FROM connections WHERE type = ? ORDER BY created_at DESC
	`, connType)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var connections []Connection
	for rows.Next() {
		var c Connection
		var username, dbName sql.NullString
		if err := rows.Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &dbName, &c.IsDefault, &c.CreatedAt, &c.UpdatedAt); err != nil {
			return nil, err
		}
		c.Username = username.String
		c.DatabaseName = dbName.String
		connections = append(connections, c)
	}

	return connections, nil
}

