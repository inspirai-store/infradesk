package api

import (
	"log"
	"net/http"
	"strconv"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/config"
	"github.com/zeni-x/backend/internal/k8s"
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
		AllowHeaders:     []string{"Origin", "Content-Type", "Accept", "Authorization", "X-Connection-ID"},
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

	// 创建服务实例 (no longer need config)
	mysqlSvc := service.NewMySQLService()
	redisSvc := service.NewRedisService()

	// 创建 K8s 服务发现服务（可选，如果失败则返回 nil）
	discoverySvc, err := service.NewDiscoveryService()
	if err != nil {
		log.Printf("Warning: K8s discovery service disabled: %v", err)
	}
	// Always create handler (will handle nil discoveryService gracefully)
	k8sHandler := NewK8sHandler(discoverySvc, db)

	// 创建端口转发管理器（如果 K8s 可用）
	var portForwardHandler *PortForwardHandler
	var forwardMonitor *service.ForwardMonitor
	var pfManager *k8s.PortForwardManager
	if discoverySvc != nil {
		// 从 discovery service 获取 K8s 客户端
		k8sClient, err := k8s.NewClient()
		if err == nil {
			pfManager = k8s.NewPortForwardManager(k8sClient)
			portForwardHandler = NewPortForwardHandler(pfManager, db)
			
			// 启动监控服务
			forwardMonitor = service.NewForwardMonitor(pfManager, db)
			forwardMonitor.Start()
		}
	}

	// 创建处理器（传递 pfManager 用于自动端口转发）
	mysqlHandler := NewMySQLHandler(mysqlSvc, db, pfManager)
	redisHandler := NewRedisHandler(redisSvc, db, pfManager)

	// API 路由组
	api := r.Group("/api")
	{
		// ==================== 连接管理 API ====================
		// 获取所有连接
		api.GET("/connections", func(c *gin.Context) {
			connections, err := db.GetConnections()
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			// 清空密码字段以保护安全
			for i := range connections {
				connections[i].Password = ""
			}
			c.JSON(http.StatusOK, connections)
		})

		// 测试连接（必须在 :id 路由之前）
		api.POST("/connections/test", func(c *gin.Context) {
			var conn store.Connection
			if err := c.ShouldBindJSON(&conn); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}

			// Debug: log received connection info (without password for security)
			log.Printf("Testing connection: type=%s, host=%s, port=%d, username=%s, password_length=%d",
				conn.Type, conn.Host, conn.Port, conn.Username, len(conn.Password))

			var testErr error
			switch conn.Type {
			case "mysql":
				testErr = mysqlSvc.TestConnection(&conn)
			case "redis":
				testErr = redisSvc.TestConnection(&conn)
			default:
				c.JSON(http.StatusBadRequest, gin.H{"error": "unsupported connection type"})
				return
			}

			if testErr != nil {
				c.JSON(http.StatusOK, gin.H{"success": false, "error": testErr.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"success": true, "message": "connection successful"})
		})

		// 按类型获取连接（必须在 :id 路由之前）
		api.GET("/connections/types/:type", func(c *gin.Context) {
			connType := c.Param("type")
			connections, err := db.GetConnectionsByType(connType)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			// 清空密码字段以保护安全
			for i := range connections {
				connections[i].Password = ""
			}
			c.JSON(http.StatusOK, connections)
		})

		// 创建连接
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
			// 清空密码字段
			conn.Password = ""
			c.JSON(http.StatusCreated, conn)
		})

		// 获取单个连接
		api.GET("/connections/:id", func(c *gin.Context) {
			id := c.Param("id")
			idInt, err := strconv.ParseInt(id, 10, 64)
			if err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
				return
			}
			conn, err := db.GetConnectionByID(idInt)
			if err != nil {
				c.JSON(http.StatusNotFound, gin.H{"error": "connection not found"})
				return
			}
			// 清空密码字段
			conn.Password = ""
			c.JSON(http.StatusOK, conn)
		})

		// 更新连接
		api.PUT("/connections/:id", func(c *gin.Context) {
			id := c.Param("id")
			idInt, err := strconv.ParseInt(id, 10, 64)
			if err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
				return
			}
			var conn store.Connection
			if err := c.ShouldBindJSON(&conn); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			conn.ID = idInt
			// 如果密码为空，从数据库获取原密码
			if conn.Password == "" {
				existingConn, err := db.GetConnectionByID(idInt)
				if err == nil {
					conn.Password = existingConn.Password
				}
			}
			if err := db.UpdateConnection(&conn); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			// 清空密码字段
			conn.Password = ""
			c.JSON(http.StatusOK, conn)
		})

		// 删除连接
		api.DELETE("/connections/:id", func(c *gin.Context) {
			id := c.Param("id")
			idInt, err := strconv.ParseInt(id, 10, 64)
			if err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
				return
			}
			if err := db.DeleteConnection(idInt); err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "deleted"})
		})

		// ==================== 历史和收藏 API ====================
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

		// ==================== MySQL API ====================
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

		// ==================== Redis API ====================
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

		// ==================== K8s 服务发现 API ====================
		k8s := api.Group("/k8s")
		{
			// 发现集群中的中间件服务
			k8s.POST("/discover", k8sHandler.DiscoverServices)
			// 列出 kubeconfig 中的集群
			k8s.POST("/clusters", k8sHandler.ListClusters)
			// 批量导入服务为连接配置
			k8s.POST("/import", k8sHandler.ImportConnections)
		}

		// ==================== 端口转发 API ====================
		if portForwardHandler != nil {
			pf := api.Group("/port-forward")
			{
				// 创建端口转发
				pf.POST("", portForwardHandler.CreateForward)
				// 列出所有转发
				pf.GET("", portForwardHandler.ListForwards)
				// 通过连接ID查询
				pf.GET("/by-connection", portForwardHandler.GetForwardByConnection)
				// 获取单个转发状态
				pf.GET("/:id", portForwardHandler.GetForward)
				// 停止端口转发
				pf.DELETE("/:id", portForwardHandler.StopForward)
				// 重新连接
				pf.POST("/:id/reconnect", portForwardHandler.ReconnectForward)
				// 更新使用时间
				pf.PUT("/:id/touch", portForwardHandler.TouchForward)
			}
		}

		// ==================== 集群管理 API ====================
		clusterHandler := NewClusterHandler(db)
		clusters := api.Group("/clusters")
		{
			// 获取所有集群
			clusters.GET("", clusterHandler.GetClusters)
			// 创建集群
			clusters.POST("", clusterHandler.CreateCluster)
			// 获取单个集群
			clusters.GET("/:id", clusterHandler.GetCluster)
			// 更新集群
			clusters.PUT("/:id", clusterHandler.UpdateCluster)
			// 删除集群
			clusters.DELETE("/:id", clusterHandler.DeleteCluster)
			// 获取集群下的所有连接
			clusters.GET("/:id/connections", clusterHandler.GetClusterConnections)
		}
	}

	return r
}
