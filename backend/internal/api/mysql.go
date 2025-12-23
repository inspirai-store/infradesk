package api

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// MySQLHandler MySQL API 处理器
type MySQLHandler struct {
	svc *service.MySQLService
	db  *store.SQLite
}

// NewMySQLHandler 创建 MySQL 处理器
func NewMySQLHandler(svc *service.MySQLService, db *store.SQLite) *MySQLHandler {
	return &MySQLHandler{svc: svc, db: db}
}

// GetInfo 获取服务器信息
func (h *MySQLHandler) GetInfo(c *gin.Context) {
	info, err := h.svc.GetInfo()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, info)
}

// ListDatabases 列出数据库
func (h *MySQLHandler) ListDatabases(c *gin.Context) {
	databases, err := h.svc.ListDatabases()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, databases)
}

// CreateDatabase 创建数据库
func (h *MySQLHandler) CreateDatabase(c *gin.Context) {
	var req struct {
		Name string `json:"name" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.CreateDatabase(req.Name); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "database created", "name": req.Name})
}

// DropDatabase 删除数据库
func (h *MySQLHandler) DropDatabase(c *gin.Context) {
	name := c.Param("db")
	if err := h.svc.DropDatabase(name); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "database dropped", "name": name})
}

// ListTables 列出表
func (h *MySQLHandler) ListTables(c *gin.Context) {
	db := c.Param("db")
	tables, err := h.svc.ListTables(db)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, tables)
}

// CreateTable 创建表
func (h *MySQLHandler) CreateTable(c *gin.Context) {
	db := c.Param("db")
	var req service.CreateTableRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.CreateTable(db, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "table created", "name": req.Name})
}

// DropTable 删除表
func (h *MySQLHandler) DropTable(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")
	if err := h.svc.DropTable(db, table); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, gin.H{"message": "table dropped", "name": table})
}

// GetTableSchema 获取表结构
func (h *MySQLHandler) GetTableSchema(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")
	schema, err := h.svc.GetTableSchema(db, table)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, schema)
}

// AlterTable 修改表结构
func (h *MySQLHandler) AlterTable(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")
	var req service.AlterTableRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.AlterTable(db, table, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "table altered"})
}

// GetRows 获取表数据
func (h *MySQLHandler) GetRows(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	size, _ := strconv.Atoi(c.DefaultQuery("size", "50"))

	if page < 1 {
		page = 1
	}
	if size < 1 || size > 1000 {
		size = 50
	}

	result, err := h.svc.GetRows(db, table, page, size)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, result)
}

// InsertRow 插入数据
func (h *MySQLHandler) InsertRow(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")

	var data map[string]interface{}
	if err := c.ShouldBindJSON(&data); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.InsertRow(db, table, data); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "row inserted"})
}

// UpdateRow 更新数据
func (h *MySQLHandler) UpdateRow(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")

	var req service.UpdateRowRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.UpdateRow(db, table, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "row updated"})
}

// DeleteRow 删除数据
func (h *MySQLHandler) DeleteRow(c *gin.Context) {
	db := c.Param("db")
	table := c.Param("table")

	var where map[string]interface{}
	if err := c.ShouldBindJSON(&where); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.DeleteRow(db, table, where); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "row deleted"})
}

// ExecuteQuery 执行 SQL 查询
func (h *MySQLHandler) ExecuteQuery(c *gin.Context) {
	var req struct {
		Database string `json:"database"`
		Query    string `json:"query" binding:"required"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	start := time.Now()
	result, err := h.svc.ExecuteQuery(req.Database, req.Query)
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
		QueryType:  "mysql",
		QueryText:  req.Query,
		DurationMs: duration,
		RowCount:   rowCount,
	})

	c.JSON(http.StatusOK, result)
}

// Export 导出数据
func (h *MySQLHandler) Export(c *gin.Context) {
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
	result, err := h.svc.GetRows(req.Database, req.Table, 1, 10000)
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
		if err := h.svc.InsertRow(req.Database, req.Table, row); err != nil {
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

