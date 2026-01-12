package api

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/k8s"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// TestK8sConnectionRequest K8s 临时测试连接请求
type TestK8sConnectionRequest struct {
	Type           string `json:"type" binding:"required"` // mysql, redis
	Username       string `json:"username"`
	Password       string `json:"password"`
	DatabaseName   string `json:"database_name"`
	Kubeconfig     string `json:"kubeconfig"`
	Context        string `json:"context"`
	K8sNamespace   string `json:"k8s_namespace" binding:"required"`
	K8sServiceName string `json:"k8s_service_name" binding:"required"`
	K8sServicePort int    `json:"k8s_service_port" binding:"required"`
}

// RegisterTestK8sRoute 注册 K8s 临时测试连接路由
func RegisterTestK8sRoute(r *gin.RouterGroup, mysqlSvc *service.MySQLService, redisSvc *service.RedisService, pfManager *k8s.PortForwardManager) {
	r.POST("/connections/test-k8s", func(c *gin.Context) {
		var req TestK8sConnectionRequest
		if err := c.ShouldBindJSON(&req); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}

		if pfManager == nil {
			c.JSON(http.StatusServiceUnavailable, gin.H{"error": "Port forward manager not available"})
			return
		}

		ctx, cancel := context.WithTimeout(c.Request.Context(), 45*time.Second)
		defer cancel()

		// 1. 建立临时端口转发 (connectionID 使用 0 标识临时转发)
		// 注意：这里会使用传入的 kubeconfig 建立临时连接
		forward, err := pfManager.CreateForward(
			ctx,
			0, // 0 means temporary/test connection
			req.K8sNamespace,
			req.K8sServiceName,
			int32(req.K8sServicePort),
			req.Kubeconfig,
			req.Context,
		)
		if err != nil {
			c.JSON(http.StatusOK, gin.H{"success": false, "error": fmt.Sprintf("K8s port-forward failed: %v", err)})
			return
		}

		// 确保结束时回收端口
		defer pfManager.StopForward(forward.ID)

		// 2. 构造临时连接配置
		testConn := &store.Connection{
			Type:         req.Type,
			Host:         "localhost",
			Port:         forward.LocalPort,
			Username:     req.Username,
			Password:     req.Password,
			DatabaseName: req.DatabaseName,
		}

		// 3. 测试数据库连接
		var testErr error
		switch req.Type {
		case "mysql":
			testErr = mysqlSvc.TestConnection(testConn)
		case "redis":
			testErr = redisSvc.TestConnection(testConn)
		default:
			c.JSON(http.StatusBadRequest, gin.H{"error": "unsupported connection type"})
			return
		}

		if testErr != nil {
			c.JSON(http.StatusOK, gin.H{"success": false, "error": testErr.Error()})
			return
		}

		c.JSON(http.StatusOK, gin.H{"success": true, "message": "K8s connection successful"})
	})
}

