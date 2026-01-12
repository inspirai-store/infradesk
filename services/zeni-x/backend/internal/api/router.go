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
		pfManager = k8s.NewPortForwardManager()
		portForwardHandler = NewPortForwardHandler(pfManager, db)

		// 启动监控服务
		forwardMonitor = service.NewForwardMonitor(pfManager, db)
		forwardMonitor.Start()
	}

	// 创建处理器（传递 pfManager 用于自动端口转发）
	mysqlHandler := NewMySQLHandler(mysqlSvc, db, pfManager)
	redisHandler := NewRedisHandler(redisSvc, db, pfManager)

	// API 路由组
	apiGroup := r.Group("/api")
	{
		// ==================== 连接管理 API ====================
		// 注册 K8s 临时测试连接路由
		RegisterTestK8sRoute(apiGroup, mysqlSvc, redisSvc, pfManager)

		// 获取所有连接
		apiGroup.GET("/connections", func(c *gin.Context) {
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
		apiGroup.POST("/connections/test", func(c *gin.Context) {
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
		apiGroup.GET("/connections/types/:type", func(c *gin.Context) {
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
		apiGroup.POST("/connections", func(c *gin.Context) {
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
		apiGroup.GET("/connections/:id", func(c *gin.Context) {
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
		apiGroup.PUT("/connections/:id", func(c *gin.Context) {
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
		apiGroup.DELETE("/connections/:id", func(c *gin.Context) {
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
		// 查询历史记录（支持过滤和分页）
		apiGroup.GET("/history", mysqlHandler.GetQueryHistory)
		// 添加查询历史记录
		apiGroup.POST("/history", mysqlHandler.AddQueryHistory)
		// 删除指定历史记录
		apiGroup.DELETE("/history/:id", mysqlHandler.DeleteQueryHistory)
		// 清理旧的历史记录
		apiGroup.POST("/history/cleanup", mysqlHandler.CleanupOldHistory)

		// 获取收藏的查询（支持分类过滤）
		apiGroup.GET("/saved-queries", mysqlHandler.GetSavedQueries)
		// 创建收藏的查询
		apiGroup.POST("/saved-queries", mysqlHandler.CreateSavedQuery)
		// 更新收藏的查询
		apiGroup.PUT("/saved-queries/:id", mysqlHandler.UpdateSavedQuery)
		// 删除收藏的查询
		apiGroup.DELETE("/saved-queries/:id", mysqlHandler.DeleteSavedQuery)

		// ==================== MySQL API ====================
		mysql := apiGroup.Group("/mysql")
		{
			// 连接信息
			mysql.GET("/info", mysqlHandler.GetInfo)

			// 数据库操作
			mysql.GET("/databases", mysqlHandler.ListDatabases)
			mysql.POST("/databases", mysqlHandler.CreateDatabase)
			// IMPORTANT: param name must be consistent with other /databases/:db/... routes to avoid gin wildcard conflicts
			mysql.PUT("/databases/:db", mysqlHandler.AlterDatabase)
			mysql.POST("/databases/:db/grant", mysqlHandler.GrantPrivileges)
			mysql.DELETE("/databases/:db", mysqlHandler.DropDatabase)
			// 数据库 Schema（用于自动补全）
			mysql.GET("/databases/:db/schema", mysqlHandler.GetDatabaseSchema)

			// 表操作
			mysql.GET("/databases/:db/tables", mysqlHandler.ListTables)
			mysql.POST("/databases/:db/tables", mysqlHandler.CreateTable)
			mysql.DELETE("/databases/:db/tables/:table", mysqlHandler.DropTable)

			// 表结构
			mysql.GET("/databases/:db/tables/:table/schema", mysqlHandler.GetTableSchema)
			mysql.PUT("/databases/:db/tables/:table/schema", mysqlHandler.AlterTable)
			// 表主键（用于编辑功能）
			mysql.GET("/databases/:db/tables/:table/primary-key", mysqlHandler.GetTablePrimaryKey)

			// 数据操作
			mysql.GET("/databases/:db/tables/:table/rows", mysqlHandler.GetRows)
			mysql.POST("/databases/:db/tables/:table/rows", mysqlHandler.InsertRow)
			mysql.PUT("/databases/:db/tables/:table/rows", mysqlHandler.UpdateRow)
			mysql.DELETE("/databases/:db/tables/:table/rows", mysqlHandler.DeleteRow)
			// 单条记录更新（通过主键）
			mysql.PUT("/databases/:db/tables/:table/record", mysqlHandler.UpdateRecord)

			// SQL 查询
			mysql.POST("/query", mysqlHandler.ExecuteQuery)

			// 导入导出
			mysql.POST("/export", mysqlHandler.Export)
			mysql.POST("/import", mysqlHandler.Import)

			// 用户管理
			mysql.GET("/users", mysqlHandler.ListUsers)
			mysql.POST("/users", mysqlHandler.CreateUser)
			mysql.GET("/users/grants", mysqlHandler.ListUserGrants)
		}

		// ==================== Redis API ====================
		redis := apiGroup.Group("/redis")
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
		k8sGroup := apiGroup.Group("/k8s")
		{
			// 发现集群中的中间件服务
			k8sGroup.POST("/discover", k8sHandler.DiscoverServices)
			// 列出 kubeconfig 中的集群
			k8sGroup.POST("/clusters", k8sHandler.ListClusters)
			// 批量导入服务为连接配置
			k8sGroup.POST("/import", k8sHandler.ImportConnections)

			// K8s 资源查看 API
			k8sGroup.GET("/clusters/:id/namespaces", k8sHandler.ListNamespaces)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/deployments", k8sHandler.ListDeployments)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/pods", k8sHandler.ListPods)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/configmaps", k8sHandler.ListConfigMaps)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/configmaps/:name", k8sHandler.GetConfigMapData)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/secrets", k8sHandler.ListSecrets)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/services", k8sHandler.ListServices)
			k8sGroup.GET("/clusters/:id/namespaces/:namespace/ingresses", k8sHandler.ListIngresses)
		}

		// ==================== 端口转发 API ====================
		if portForwardHandler != nil {
			pf := apiGroup.Group("/port-forward")
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
		clusters := apiGroup.Group("/clusters")
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
