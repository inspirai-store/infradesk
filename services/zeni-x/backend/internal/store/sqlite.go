package store

import (
	"database/sql"
	"fmt"
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
	-- 集群信息表
	CREATE TABLE IF NOT EXISTS clusters (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		name TEXT NOT NULL UNIQUE,
		description TEXT,
		environment TEXT,
		context TEXT,
		api_server TEXT,
		kubeconfig TEXT,
		is_active BOOLEAN DEFAULT 1,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
	);

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
		forward_id TEXT,
		forward_local_port INTEGER,
		forward_status TEXT,
		cluster_id INTEGER,
		source TEXT DEFAULT 'local',
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE SET NULL
	);

	-- SQL 查询历史
	CREATE TABLE IF NOT EXISTS query_history (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		connection_id INTEGER,
		database TEXT,
		query_type TEXT,
		query_text TEXT NOT NULL,
		executed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		duration_ms INTEGER,
		row_count INTEGER,
		status TEXT DEFAULT 'success',
		error_message TEXT,
		FOREIGN KEY (connection_id) REFERENCES connections(id)
	);

	-- 收藏的查询
	CREATE TABLE IF NOT EXISTS saved_queries (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		connection_id INTEGER,
		database TEXT,
		name TEXT NOT NULL,
		description TEXT,
		query_text TEXT NOT NULL,
		category TEXT,
		created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (connection_id) REFERENCES connections(id)
	);

	-- 创建索引
	CREATE INDEX IF NOT EXISTS idx_query_history_type ON query_history(query_type);
	CREATE INDEX IF NOT EXISTS idx_query_history_executed_at ON query_history(executed_at);
	CREATE INDEX IF NOT EXISTS idx_connections_type ON connections(type);
	CREATE INDEX IF NOT EXISTS idx_connections_cluster ON connections(cluster_id);
	CREATE INDEX IF NOT EXISTS idx_connections_source ON connections(source);
	`

	_, err := s.db.Exec(schema)
	if err != nil {
		return err
	}

	// 迁移：添加端口转发字段到现有数据库
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN forward_id TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN forward_local_port INTEGER`)
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN forward_status TEXT`)
	
	// 迁移：添加 K8s 相关字段
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN k8s_namespace TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN k8s_service_name TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN k8s_service_port INTEGER`)
	
	// 迁移：添加集群和来源字段
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN cluster_id INTEGER`)
	_, _ = s.db.Exec(`ALTER TABLE connections ADD COLUMN source TEXT DEFAULT 'local'`)
	
	// 迁移：为 clusters 添加 kubeconfig 字段
	_, _ = s.db.Exec(`ALTER TABLE clusters ADD COLUMN kubeconfig TEXT`)

	// 为已存在的 NULL source 设置默认值
	_, _ = s.db.Exec(`UPDATE connections SET source = 'local' WHERE source IS NULL`)

	// 迁移：为 query_history 添加新字段
	_, _ = s.db.Exec(`ALTER TABLE query_history ADD COLUMN database TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE query_history ADD COLUMN status TEXT DEFAULT 'success'`)
	_, _ = s.db.Exec(`ALTER TABLE query_history ADD COLUMN error_message TEXT`)

	// 迁移：为 saved_queries 添加新字段
	_, _ = s.db.Exec(`ALTER TABLE saved_queries ADD COLUMN database TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE saved_queries ADD COLUMN description TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE saved_queries ADD COLUMN category TEXT`)
	_, _ = s.db.Exec(`ALTER TABLE saved_queries ADD COLUMN updated_at DATETIME DEFAULT CURRENT_TIMESTAMP`)

	return nil
}

// Connection 连接配置
type Connection struct {
	ID               int64   `json:"id"`
	Name             string  `json:"name"`
	Type             string  `json:"type"`
	Host             string  `json:"host"`
	Port             int     `json:"port"`
	Username         string  `json:"username,omitempty"`
	Password         string  `json:"password,omitempty"` // 允许接收密码，但在返回时需要手动清空
	DatabaseName     string  `json:"database_name,omitempty"`
	IsDefault        bool    `json:"is_default"`
	ForwardID        string  `json:"forward_id,omitempty"`
	ForwardLocalPort int     `json:"forward_local_port,omitempty"`
	ForwardStatus    string  `json:"forward_status,omitempty"`
	K8sNamespace     string  `json:"k8s_namespace,omitempty"`    // K8s 命名空间
	K8sServiceName   string  `json:"k8s_service_name,omitempty"` // K8s 服务名
	K8sServicePort   int     `json:"k8s_service_port,omitempty"` // K8s 服务端口
	ClusterID        *int64  `json:"cluster_id,omitempty"`       // 所属集群ID（nullable）
	Source           string  `json:"source"`                     // 来源: local, k8s
	CreatedAt        string  `json:"created_at"`
	UpdatedAt        string  `json:"updated_at"`
}

// Cluster 集群信息
	type Cluster struct {
		ID          int64  `json:"id"`
		Name        string `json:"name"`                   // 集群名称
		Description string `json:"description,omitempty"`  // 集群描述
		Environment string `json:"environment,omitempty"`  // 环境: dev, test, uat, prod
		Context     string `json:"context,omitempty"`      // K8s context
		APIServer   string `json:"api_server,omitempty"`   // K8s API Server 地址
		Kubeconfig  string `json:"kubeconfig,omitempty"`    // Kubeconfig 内容
		IsActive    bool   `json:"is_active"`              // 是否激活
		CreatedAt   string `json:"created_at"`
		UpdatedAt   string `json:"updated_at"`
	}

// QueryHistory 查询历史
type QueryHistory struct {
	ID           int64  `json:"id"`
	ConnectionID int64  `json:"connection_id"`
	Database     string `json:"database"`
	QueryType    string `json:"query_type"`
	QueryText    string `json:"query_text"`
	ExecutedAt   string `json:"executed_at"`
	DurationMs   int64  `json:"duration_ms"`
	RowCount     int64  `json:"row_count"`
	Status       string `json:"status"`
	ErrorMessage string `json:"error_message"`
}

// SavedQuery 收藏的查询
type SavedQuery struct {
	ID           int64  `json:"id"`
	ConnectionID int64  `json:"connection_id"`
	Database     string `json:"database"`
	Name         string `json:"name"`
	Description  string `json:"description"`
	QueryText    string `json:"query_text"`
	Category     string `json:"category"`
	CreatedAt    string `json:"created_at"`
	UpdatedAt    string `json:"updated_at"`
}

// GetConnections 获取所有连接配置
func (s *SQLite) GetConnections() ([]Connection, error) {
	rows, err := s.db.Query(`
		SELECT id, name, type, host, port, username, database_name, is_default, 
		       COALESCE(forward_id, ''), COALESCE(forward_local_port, 0), COALESCE(forward_status, ''),
		       COALESCE(k8s_namespace, ''), COALESCE(k8s_service_name, ''), COALESCE(k8s_service_port, 0),
		       cluster_id, COALESCE(source, 'local'),
		       created_at, updated_at 
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
		if err := rows.Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &dbName, &c.IsDefault,
			&c.ForwardID, &c.ForwardLocalPort, &c.ForwardStatus,
			&c.K8sNamespace, &c.K8sServiceName, &c.K8sServicePort,
			&c.ClusterID, &c.Source,
			&c.CreatedAt, &c.UpdatedAt); err != nil {
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
	// 如果 source 为空，默认设置为 local
	if c.Source == "" {
		c.Source = "local"
	}
	
	result, err := s.db.Exec(`
		INSERT INTO connections (name, type, host, port, username, password, database_name, is_default,
		                        forward_id, forward_local_port, forward_status,
		                        k8s_namespace, k8s_service_name, k8s_service_port,
		                        cluster_id, source)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
	`, c.Name, c.Type, c.Host, c.Port, c.Username, c.Password, c.DatabaseName, c.IsDefault,
		c.ForwardID, c.ForwardLocalPort, c.ForwardStatus,
		c.K8sNamespace, c.K8sServiceName, c.K8sServicePort,
		c.ClusterID, c.Source)
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

// GetQueryHistory 获取查询历史（支持分页和筛选）
func (s *SQLite) GetQueryHistory(queryType, database, status, keyword string, limit, offset int) ([]QueryHistory, int, error) {
	// 构建查询条件
	whereClause := "WHERE 1=1"
	args := []interface{}{}
	argCount := 0

	if queryType != "" {
		argCount++
		whereClause += fmt.Sprintf(" AND query_type = @%d", argCount)
		args = append(args, queryType)
	}
	if database != "" {
		argCount++
		whereClause += fmt.Sprintf(" AND database = @%d", argCount)
		args = append(args, database)
	}
	if status != "" {
		argCount++
		whereClause += fmt.Sprintf(" AND status = @%d", argCount)
		args = append(args, status)
	}
	if keyword != "" {
		argCount++
		whereClause += fmt.Sprintf(" AND query_text LIKE @%d", argCount)
		args = append(args, "%"+keyword+"%")
	}

	// 获取总数
	countQuery := "SELECT COUNT(*) FROM query_history " + whereClause
	var total int
	err := s.db.QueryRow(countQuery, args...).Scan(&total)
	if err != nil {
		return nil, 0, err
	}

	// 获取分页数据
	query := `
		SELECT id, connection_id, COALESCE(database, ''), query_type, query_text,
		       executed_at, COALESCE(duration_ms, 0), COALESCE(row_count, 0),
		       COALESCE(status, 'success'), COALESCE(error_message, '')
		FROM query_history ` + whereClause + `
		ORDER BY executed_at DESC
		LIMIT ? OFFSET ?
	`
	args = append(args, limit, offset)

	rows, err := s.db.Query(query, args...)
	if err != nil {
		return nil, 0, err
	}
	defer rows.Close()

	var history []QueryHistory
	for rows.Next() {
		var h QueryHistory
		var connID sql.NullInt64
		if err := rows.Scan(&h.ID, &connID, &h.Database, &h.QueryType, &h.QueryText,
			&h.ExecutedAt, &h.DurationMs, &h.RowCount, &h.Status, &h.ErrorMessage); err != nil {
			return nil, 0, err
		}
		h.ConnectionID = connID.Int64
		history = append(history, h)
	}

	return history, total, nil
}

// AddQueryHistory 添加查询历史
func (s *SQLite) AddQueryHistory(h *QueryHistory) error {
	_, err := s.db.Exec(`
		INSERT INTO query_history (connection_id, database, query_type, query_text, duration_ms, row_count, status, error_message)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?)
	`, h.ConnectionID, h.Database, h.QueryType, h.QueryText, h.DurationMs, h.RowCount, h.Status, h.ErrorMessage)
	return err
}

// DeleteQueryHistory 删除查询历史
func (s *SQLite) DeleteQueryHistory(id int64) error {
	_, err := s.db.Exec(`DELETE FROM query_history WHERE id = ?`, id)
	return err
}

// CleanupOldHistory 清理旧的历史记录
func (s *SQLite) CleanupOldHistory(keepCount int) (int64, error) {
	result, err := s.db.Exec(`
		DELETE FROM query_history
		WHERE id NOT IN (
			SELECT id FROM query_history
			ORDER BY executed_at DESC
			LIMIT ?
		)
	`, keepCount)
	if err != nil {
		return 0, err
	}
	return result.RowsAffected()
}

// GetSavedQueries 获取收藏的查询（支持分类筛选）
func (s *SQLite) GetSavedQueries(category string) ([]SavedQuery, error) {
	var rows *sql.Rows
	var err error

	if category != "" {
		rows, err = s.db.Query(`
			SELECT id, connection_id, COALESCE(database, ''), name, COALESCE(description, ''),
			       query_text, COALESCE(category, ''), created_at, updated_at
			FROM saved_queries
			WHERE category = ?
			ORDER BY created_at DESC
		`, category)
	} else {
		rows, err = s.db.Query(`
			SELECT id, connection_id, COALESCE(database, ''), name, COALESCE(description, ''),
			       query_text, COALESCE(category, ''), created_at, updated_at
			FROM saved_queries
			ORDER BY created_at DESC
		`)
	}

	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var queries []SavedQuery
	for rows.Next() {
		var q SavedQuery
		var connID sql.NullInt64
		if err := rows.Scan(&q.ID, &connID, &q.Database, &q.Name, &q.Description,
			&q.QueryText, &q.Category, &q.CreatedAt, &q.UpdatedAt); err != nil {
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
		INSERT INTO saved_queries (connection_id, database, name, description, query_text, category)
		VALUES (?, ?, ?, ?, ?, ?)
	`, q.ConnectionID, q.Database, q.Name, q.Description, q.QueryText, q.Category)
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

// UpdateSavedQuery 更新收藏的查询
func (s *SQLite) UpdateSavedQuery(q *SavedQuery) error {
	_, err := s.db.Exec(`
		UPDATE saved_queries
		SET database = ?, name = ?, description = ?, query_text = ?, category = ?, updated_at = CURRENT_TIMESTAMP
		WHERE id = ?
	`, q.Database, q.Name, q.Description, q.QueryText, q.Category, q.ID)
	return err
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
		SELECT id, name, type, host, port, username, password, database_name, is_default,
		       COALESCE(forward_id, ''), COALESCE(forward_local_port, 0), COALESCE(forward_status, ''),
		       COALESCE(k8s_namespace, ''), COALESCE(k8s_service_name, ''), COALESCE(k8s_service_port, 0),
		       cluster_id, COALESCE(source, 'local'),
		       created_at, updated_at 
		FROM connections WHERE id = ?
	`, id).Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &password, &dbName, &c.IsDefault,
		&c.ForwardID, &c.ForwardLocalPort, &c.ForwardStatus,
		&c.K8sNamespace, &c.K8sServiceName, &c.K8sServicePort,
		&c.ClusterID, &c.Source,
		&c.CreatedAt, &c.UpdatedAt)
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
		SET name = ?, type = ?, host = ?, port = ?, username = ?, password = ?, database_name = ?, is_default = ?,
		    forward_id = ?, forward_local_port = ?, forward_status = ?,
		    k8s_namespace = ?, k8s_service_name = ?, k8s_service_port = ?,
		    cluster_id = ?, source = ?,
		    updated_at = CURRENT_TIMESTAMP
		WHERE id = ?
	`, c.Name, c.Type, c.Host, c.Port, c.Username, c.Password, c.DatabaseName, c.IsDefault,
		c.ForwardID, c.ForwardLocalPort, c.ForwardStatus,
		c.K8sNamespace, c.K8sServiceName, c.K8sServicePort,
		c.ClusterID, c.Source,
		c.ID)
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
		SELECT id, name, type, host, port, username, database_name, is_default,
		       COALESCE(forward_id, ''), COALESCE(forward_local_port, 0), COALESCE(forward_status, ''),
		       COALESCE(k8s_namespace, ''), COALESCE(k8s_service_name, ''), COALESCE(k8s_service_port, 0),
		       cluster_id, COALESCE(source, 'local'),
		       created_at, updated_at 
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
		if err := rows.Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &dbName, &c.IsDefault,
			&c.ForwardID, &c.ForwardLocalPort, &c.ForwardStatus,
			&c.K8sNamespace, &c.K8sServiceName, &c.K8sServicePort,
			&c.ClusterID, &c.Source,
			&c.CreatedAt, &c.UpdatedAt); err != nil {
			return nil, err
		}
		c.Username = username.String
		c.DatabaseName = dbName.String
		connections = append(connections, c)
	}

	return connections, nil
}

// ==================== Cluster Management ====================

// GetClusters 获取所有集群
func (s *SQLite) GetClusters() ([]Cluster, error) {
	rows, err := s.db.Query(`
		SELECT id, name, COALESCE(description, ''), COALESCE(environment, ''), 
		       COALESCE(context, ''), COALESCE(api_server, ''), COALESCE(kubeconfig, ''), is_active, 
		       created_at, updated_at 
		FROM clusters ORDER BY created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var clusters []Cluster
	for rows.Next() {
		var c Cluster
		if err := rows.Scan(&c.ID, &c.Name, &c.Description, &c.Environment,
			&c.Context, &c.APIServer, &c.Kubeconfig, &c.IsActive,
			&c.CreatedAt, &c.UpdatedAt); err != nil {
			return nil, err
		}
		clusters = append(clusters, c)
	}

	return clusters, nil
}

// GetClusterByID 根据 ID 获取集群
func (s *SQLite) GetClusterByID(id int64) (*Cluster, error) {
	var c Cluster
	err := s.db.QueryRow(`
		SELECT id, name, COALESCE(description, ''), COALESCE(environment, ''), 
		       COALESCE(context, ''), COALESCE(api_server, ''), COALESCE(kubeconfig, ''), is_active,
		       created_at, updated_at 
		FROM clusters WHERE id = ?
	`, id).Scan(&c.ID, &c.Name, &c.Description, &c.Environment,
		&c.Context, &c.APIServer, &c.Kubeconfig, &c.IsActive,
		&c.CreatedAt, &c.UpdatedAt)
	if err != nil {
		return nil, err
	}
	return &c, nil
}

// GetClusterByName 根据名称获取集群
func (s *SQLite) GetClusterByName(name string) (*Cluster, error) {
	var c Cluster
	err := s.db.QueryRow(`
		SELECT id, name, COALESCE(description, ''), COALESCE(environment, ''), 
		       COALESCE(context, ''), COALESCE(api_server, ''), COALESCE(kubeconfig, ''), is_active,
		       created_at, updated_at 
		FROM clusters WHERE name = ?
	`, name).Scan(&c.ID, &c.Name, &c.Description, &c.Environment,
		&c.Context, &c.APIServer, &c.Kubeconfig, &c.IsActive,
		&c.CreatedAt, &c.UpdatedAt)
	if err != nil {
		return nil, err
	}
	return &c, nil
}

// CreateCluster 创建集群
func (s *SQLite) CreateCluster(c *Cluster) error {
	result, err := s.db.Exec(`
		INSERT INTO clusters (name, description, environment, context, api_server, kubeconfig, is_active)
		VALUES (?, ?, ?, ?, ?, ?, ?)
	`, c.Name, c.Description, c.Environment, c.Context, c.APIServer, c.Kubeconfig, c.IsActive)
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

// UpdateCluster 更新集群
func (s *SQLite) UpdateCluster(c *Cluster) error {
	_, err := s.db.Exec(`
		UPDATE clusters 
		SET name = ?, description = ?, environment = ?, context = ?, api_server = ?, kubeconfig = ?, is_active = ?,
		    updated_at = CURRENT_TIMESTAMP
		WHERE id = ?
	`, c.Name, c.Description, c.Environment, c.Context, c.APIServer, c.Kubeconfig, c.IsActive, c.ID)
	return err
}

// DeleteCluster 删除集群（会将关联的连接的 cluster_id 设置为 NULL）
func (s *SQLite) DeleteCluster(id int64) error {
	_, err := s.db.Exec(`DELETE FROM clusters WHERE id = ?`, id)
	return err
}

// GetConnectionsByCluster 获取集群下的所有连接
func (s *SQLite) GetConnectionsByCluster(clusterID int64) ([]Connection, error) {
	rows, err := s.db.Query(`
		SELECT id, name, type, host, port, username, database_name, is_default,
		       COALESCE(forward_id, ''), COALESCE(forward_local_port, 0), COALESCE(forward_status, ''),
		       COALESCE(k8s_namespace, ''), COALESCE(k8s_service_name, ''), COALESCE(k8s_service_port, 0),
		       cluster_id, COALESCE(source, 'local'),
		       created_at, updated_at 
		FROM connections WHERE cluster_id = ? ORDER BY created_at DESC
	`, clusterID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var connections []Connection
	for rows.Next() {
		var c Connection
		var username, dbName sql.NullString
		if err := rows.Scan(&c.ID, &c.Name, &c.Type, &c.Host, &c.Port, &username, &dbName, &c.IsDefault,
			&c.ForwardID, &c.ForwardLocalPort, &c.ForwardStatus,
			&c.K8sNamespace, &c.K8sServiceName, &c.K8sServicePort,
			&c.ClusterID, &c.Source,
			&c.CreatedAt, &c.UpdatedAt); err != nil {
			return nil, err
		}
		c.Username = username.String
		c.DatabaseName = dbName.String
		connections = append(connections, c)
	}

	return connections, nil
}
