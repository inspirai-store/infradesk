package api

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/k8s"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// MySQLHandler MySQL API 处理器
type MySQLHandler struct {
	svc       *service.MySQLService
	db        *store.SQLite
	pfManager *k8s.PortForwardManager
}

// NewMySQLHandler 创建 MySQL 处理器
func NewMySQLHandler(svc *service.MySQLService, db *store.SQLite, pfManager *k8s.PortForwardManager) *MySQLHandler {
	return &MySQLHandler{
		svc:       svc,
		db:        db,
		pfManager: pfManager,
	}
}

// getConnection 从请求头获取连接配置，并确保端口转发已建立
func (h *MySQLHandler) getConnection(c *gin.Context) (*store.Connection, error) {
	connIDStr := c.GetHeader("X-Connection-ID")
	if connIDStr == "" {
		return nil, nil
	}
	connID, err := strconv.ParseInt(connIDStr, 10, 64)
	if err != nil {
		return nil, err
	}
	
	conn, err := h.db.GetConnectionByID(connID)
	if err != nil {
		return nil, err
	}
	
	// 检查是否需要端口转发
	if h.pfManager != nil && conn.K8sNamespace != "" && conn.K8sServiceName != "" {
		// 检查端口转发是否已存在且活跃
		if conn.ForwardID != "" {
			forward, err := h.pfManager.GetForward(conn.ForwardID)
			if err == nil && forward.Status == k8s.StatusActive {
				// 端口转发活跃，更新最后使用时间
				h.pfManager.UpdateLastUsed(conn.ForwardID)
				return conn, nil
			}
		}
		
		// 需要创建或重新创建端口转发
		log.Printf("Creating port forward for connection %d (%s/%s)", 
			conn.ID, conn.K8sNamespace, conn.K8sServiceName)
		
		var kubeconfig, k8sContext string
		if conn.ClusterID != nil {
			cluster, err := h.db.GetClusterByID(*conn.ClusterID)
			if err == nil {
				kubeconfig = cluster.Kubeconfig
				k8sContext = cluster.Context
			}
		}

		ctx, cancel := context.WithTimeout(c.Request.Context(), 30*time.Second)
		defer cancel()
		
		forward, err := h.pfManager.CreateForward(
			ctx,
			conn.ID,
			conn.K8sNamespace,
			conn.K8sServiceName,
			int32(conn.K8sServicePort),
			kubeconfig,
			k8sContext,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to create port forward: %w", err)
		}
		
		// 更新连接信息
		conn.ForwardID = forward.ID
		conn.ForwardLocalPort = forward.LocalPort
		conn.ForwardStatus = string(forward.Status)
		conn.Host = "localhost"
		conn.Port = forward.LocalPort
		
		if err := h.db.UpdateConnection(conn); err != nil {
			log.Printf("Warning: failed to update connection with forward info: %v", err)
		}
	}
	
	return conn, nil
}

// GetInfo 获取服务器信息
func (h *MySQLHandler) GetInfo(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	info, err := h.svc.GetInfo(conn)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, info)
}

// ListDatabases 列出数据库
func (h *MySQLHandler) ListDatabases(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	databases, err := h.svc.ListDatabases(conn)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, databases)
}

// CreateDatabase 创建数据库
func (h *MySQLHandler) CreateDatabase(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var req service.CreateDatabaseRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.CreateDatabase(conn, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "database created", "name": req.Name})
}

// AlterDatabase 修改数据库属性
func (h *MySQLHandler) AlterDatabase(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	name := c.Param("db")
	var req service.AlterDatabaseRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// 至少需要修改一个属性
	if req.CharSet == "" && req.Collate == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "charset or collate is required"})
		return
	}

	if err := h.svc.AlterDatabase(conn, name, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "database altered", "name": name})
}

// GrantPrivileges 授予用户数据库权限
func (h *MySQLHandler) GrantPrivileges(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	database := c.Param("db")
	var req service.GrantPrivilegesRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.GrantPrivileges(conn, database, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "privileges granted", "database": database, "user": req.UserName})
}

// DropDatabase 删除数据库
func (h *MySQLHandler) DropDatabase(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	name := c.Param("db")
	if err := h.svc.DropDatabase(conn, name); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "database dropped", "name": name})
}

// ListTables 列出表
func (h *MySQLHandler) ListTables(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	tables, err := h.svc.ListTables(conn, dbName)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, tables)
}

// CreateTable 创建表
func (h *MySQLHandler) CreateTable(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	var req service.CreateTableRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.CreateTable(conn, dbName, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "table created", "name": req.Name})
}

// DropTable 删除表
func (h *MySQLHandler) DropTable(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")
	if err := h.svc.DropTable(conn, dbName, table); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "table dropped", "name": table})
}

// GetTableSchema 获取表结构
func (h *MySQLHandler) GetTableSchema(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")
	schema, err := h.svc.GetTableSchema(conn, dbName, table)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, schema)
}

// AlterTable 修改表结构
func (h *MySQLHandler) AlterTable(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")
	var req service.AlterTableRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.AlterTable(conn, dbName, table, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "table altered"})
}

// GetRows 获取表数据
func (h *MySQLHandler) GetRows(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	size, _ := strconv.Atoi(c.DefaultQuery("size", "50"))

	if page < 1 {
		page = 1
	}
	if size < 1 || size > 1000 {
		size = 50
	}

	result, err := h.svc.GetRows(conn, dbName, table, page, size)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, result)
}

// InsertRow 插入数据
func (h *MySQLHandler) InsertRow(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")

	var data map[string]interface{}
	if err := c.ShouldBindJSON(&data); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.InsertRow(conn, dbName, table, data); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "row inserted"})
}

// UpdateRow 更新数据
func (h *MySQLHandler) UpdateRow(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")

	var req service.UpdateRowRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.UpdateRow(conn, dbName, table, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "row updated"})
}

// DeleteRow 删除数据
func (h *MySQLHandler) DeleteRow(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	dbName := c.Param("db")
	table := c.Param("table")

	var where map[string]interface{}
	if err := c.ShouldBindJSON(&where); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.DeleteRow(conn, dbName, table, where); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "row deleted"})
}

// ExecuteQuery 执行 SQL 查询
func (h *MySQLHandler) ExecuteQuery(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var req struct {
		Database string `json:"database"`
		Query    string `json:"query" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	start := time.Now()
	result, err := h.svc.ExecuteQuery(conn, req.Database, req.Query)
	duration := time.Since(start).Milliseconds()

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result.Duration = duration

	// 记录查询历史
	var rowCount int64
	if result.Rows != nil {
		rowCount = int64(len(result.Rows))
	}
	h.db.AddQueryHistory(&store.QueryHistory{
		ConnectionID: conn.ID,
		QueryType:    "mysql",
		QueryText:    req.Query,
		DurationMs:   duration,
		RowCount:     rowCount,
	})

	c.JSON(http.StatusOK, result)
}

// Export 导出数据
func (h *MySQLHandler) Export(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var req struct {
		Database string `json:"database" binding:"required"`
		Table    string `json:"table" binding:"required"`
		Format   string `json:"format"` // csv, json, sql
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// 获取所有数据
	result, err := h.svc.GetRows(conn, req.Database, req.Table, 1, 10000)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	format := req.Format
	if format == "" {
		format = "json"
	}

	switch format {
	case "json":
		c.JSON(http.StatusOK, gin.H{
			"columns": result.Columns,
			"rows":    result.Rows,
			"total":   result.Total,
		})
	case "csv":
		// 简单 CSV 格式
		var csv string
		// Header
		for i, col := range result.Columns {
			if i > 0 {
				csv += ","
			}
			csv += col
		}
		csv += "\n"
		// Rows
		for _, row := range result.Rows {
			for i, col := range result.Columns {
				if i > 0 {
					csv += ","
				}
				val := row[col]
				if val != nil {
					csv += formatCSVValue(val)
				}
			}
			csv += "\n"
		}
		c.Header("Content-Type", "text/csv")
		c.Header("Content-Disposition", "attachment; filename="+req.Table+".csv")
		c.String(http.StatusOK, csv)
	default:
		c.JSON(http.StatusBadRequest, gin.H{"error": "unsupported format"})
	}
}

// formatCSVValue 格式化 CSV 值
func formatCSVValue(val interface{}) string {
	switch v := val.(type) {
	case string:
		return "\"" + v + "\""
	case nil:
		return ""
	default:
		return formatValue(v)
	}
}

// formatValue 格式化值
func formatValue(val interface{}) string {
	switch v := val.(type) {
	case string:
		return v
	case int, int64, float64:
		return strconv.FormatFloat(toFloat64(v), 'f', -1, 64)
	case bool:
		if v {
			return "1"
		}
		return "0"
	default:
		return ""
	}
}

func toFloat64(v interface{}) float64 {
	switch n := v.(type) {
	case int:
		return float64(n)
	case int64:
		return float64(n)
	case float64:
		return n
	default:
		return 0
	}
}

// Import 导入数据
func (h *MySQLHandler) Import(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var req struct {
		Database string                   `json:"database" binding:"required"`
		Table    string                   `json:"table" binding:"required"`
		Rows     []map[string]interface{} `json:"rows" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	var successCount, errorCount int
	for _, row := range req.Rows {
		if err := h.svc.InsertRow(conn, req.Database, req.Table, row); err != nil {
			errorCount++
		} else {
			successCount++
		}
	}

	c.JSON(http.StatusOK, gin.H{
		"message":       "import completed",
		"success_count": successCount,
		"error_count":   errorCount,
	})
}

// ListUsers 列出所有用户
func (h *MySQLHandler) ListUsers(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	users, err := h.svc.ListUsers(conn)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, users)
}

// CreateUser 创建新用户
func (h *MySQLHandler) CreateUser(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var req service.CreateUserRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.CreateUser(conn, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "user created", "username": req.UserName})
}

// ListUserGrants 列出用户的权限
func (h *MySQLHandler) ListUserGrants(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	username := c.Query("username")
	host := c.Query("host")

	if username == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "username is required"})
		return
	}
	if host == "" {
		host = "%"
	}

	grants, err := h.svc.ListUserGrants(conn, username, host)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, grants)
}
