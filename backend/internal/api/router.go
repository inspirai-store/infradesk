package api

import (
	"net/http"
	"strconv"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/config"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// NewRouter 创建 Gin 路由
func NewRouter(cfg *config.Config, db *store.SQLite) *gin.Engine {
	// 设置运行模式
	if cfg.Server.Mode == "release" {
		gin.SetMode(gin.ReleaseMode)
	}

	r := gin.Default()

	// CORS 配置
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowHeaders:     []string{"Origin", "Content-Type", "Accept", "Authorization"},
		ExposeHeaders:    []string{"Content-Length"},
		AllowCredentials: true,
	}))

	// 健康检查
	r.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status":  "healthy",
			"service": "zeni-x",
		})
	})

	r.GET("/ready", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status": "ready",
		})
	})

	// 创建服务实例
	mysqlSvc := service.NewMySQLService(cfg)
	redisSvc := service.NewRedisService(cfg)

	// 创建处理器
	mysqlHandler := NewMySQLHandler(mysqlSvc, db)
	redisHandler := NewRedisHandler(redisSvc, db)

	// API 路由组
	api := r.Group("/api")
	{
		// 系统 API
		api.GET("/connections", func(c *gin.Context) {
			connections, err := db.GetConnections()
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, connections)
		})

		api.POST("/connections", func(c *gin.Context) {
			var conn store.Connection
			if err := c.ShouldBindJSON(&conn); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			if err := db.CreateConnection(&conn); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusCreated, conn)
		})

		api.GET("/history", func(c *gin.Context) {
			queryType := c.Query("type")
			limit := 100
			history, err := db.GetQueryHistory(queryType, limit)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, history)
		})

		api.GET("/saved-queries", func(c *gin.Context) {
			queries, err := db.GetSavedQueries()
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, queries)
		})

		api.POST("/saved-queries", func(c *gin.Context) {
			var query store.SavedQuery
			if err := c.ShouldBindJSON(&query); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			if err := db.CreateSavedQuery(&query); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusCreated, query)
		})

		api.DELETE("/saved-queries/:id", func(c *gin.Context) {
			id := c.Param("id")
			idInt, err := strconv.ParseInt(id, 10, 64)
			if err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
				return
			}
			if err := db.DeleteSavedQuery(idInt); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "deleted"})
		})

		// MySQL API
		mysql := api.Group("/mysql")
		{
			// 连接信息
			mysql.GET("/info", mysqlHandler.GetInfo)

			// 数据库操作
			mysql.GET("/databases", mysqlHandler.ListDatabases)
			mysql.POST("/databases", mysqlHandler.CreateDatabase)
			// IMPORTANT: param name must be consistent with other /databases/:db/... routes to avoid gin wildcard conflicts
			mysql.DELETE("/databases/:db", mysqlHandler.DropDatabase)

			// 表操作
			mysql.GET("/databases/:db/tables", mysqlHandler.ListTables)
			mysql.POST("/databases/:db/tables", mysqlHandler.CreateTable)
			mysql.DELETE("/databases/:db/tables/:table", mysqlHandler.DropTable)

			// 表结构
			mysql.GET("/databases/:db/tables/:table/schema", mysqlHandler.GetTableSchema)
			mysql.PUT("/databases/:db/tables/:table/schema", mysqlHandler.AlterTable)

			// 数据操作
			mysql.GET("/databases/:db/tables/:table/rows", mysqlHandler.GetRows)
			mysql.POST("/databases/:db/tables/:table/rows", mysqlHandler.InsertRow)
			mysql.PUT("/databases/:db/tables/:table/rows", mysqlHandler.UpdateRow)
			mysql.DELETE("/databases/:db/tables/:table/rows", mysqlHandler.DeleteRow)

			// SQL 查询
			mysql.POST("/query", mysqlHandler.ExecuteQuery)

			// 导入导出
			mysql.POST("/export", mysqlHandler.Export)
			mysql.POST("/import", mysqlHandler.Import)
		}

		// Redis API
		redis := api.Group("/redis")
		{
			// 连接信息
			redis.GET("/info", redisHandler.GetInfo)

			// Key 操作
			redis.GET("/keys", redisHandler.ListKeys)
			redis.GET("/keys/*key", redisHandler.GetKey)
			redis.POST("/keys", redisHandler.SetKey)
			redis.PUT("/keys/*key", redisHandler.UpdateKey)
			redis.DELETE("/keys/*key", redisHandler.DeleteKey)

			// TTL 操作
			// NOTE: gin does not allow registering both /keys/*key and /keys/*key/ttl (wildcard conflict)
			redis.PUT("/ttl/*key", redisHandler.SetTTL)

			// 导入导出
			redis.POST("/export", redisHandler.Export)
			redis.POST("/import", redisHandler.Import)
		}
	}

	return r
}

