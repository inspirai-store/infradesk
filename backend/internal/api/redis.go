package api

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/k8s"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// RedisHandler Redis API 处理器
type RedisHandler struct {
	svc       *service.RedisService
	db        *store.SQLite
	pfManager *k8s.PortForwardManager
}

// NewRedisHandler 创建 Redis 处理器
func NewRedisHandler(svc *service.RedisService, db *store.SQLite, pfManager *k8s.PortForwardManager) *RedisHandler {
	return &RedisHandler{
		svc:       svc,
		db:        db,
		pfManager: pfManager,
	}
}

// getConnection 从请求头获取连接配置，并确保端口转发已建立
func (h *RedisHandler) getConnection(c *gin.Context) (*store.Connection, error) {
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
		
		ctx, cancel := context.WithTimeout(c.Request.Context(), 30*time.Second)
		defer cancel()
		
		forward, err := h.pfManager.CreateForward(
			ctx,
			conn.ID,
			conn.K8sNamespace,
			conn.K8sServiceName,
			int32(conn.K8sServicePort),
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

// GetInfo 获取 Redis 信息
func (h *RedisHandler) GetInfo(c *gin.Context) {
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

// ListKeys 列出 Keys
func (h *RedisHandler) ListKeys(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	pattern := c.DefaultQuery("pattern", "*")
	cursor, _ := strconv.ParseUint(c.DefaultQuery("cursor", "0"), 10, 64)
	count, _ := strconv.ParseInt(c.DefaultQuery("count", "100"), 10, 64)

	if count < 1 || count > 1000 {
		count = 100
	}

	result, err := h.svc.ListKeys(conn, pattern, cursor, count)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, result)
}

// GetKey 获取 Key 详情
func (h *RedisHandler) GetKey(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	key := c.Param("key")
	// 移除前导斜杠
	key = strings.TrimPrefix(key, "/")

	info, err := h.svc.GetKey(conn, key)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, info)
}

// SetKey 创建 Key
func (h *RedisHandler) SetKey(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var req service.SetKeyRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.SetKey(conn, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "key created", "key": req.Key})
}

// UpdateKey 更新 Key
func (h *RedisHandler) UpdateKey(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	key := c.Param("key")
	key = strings.TrimPrefix(key, "/")

	var req service.SetKeyRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	req.Key = key

	if err := h.svc.SetKey(conn, &req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "key updated", "key": key})
}

// DeleteKey 删除 Key
func (h *RedisHandler) DeleteKey(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	key := c.Param("key")
	key = strings.TrimPrefix(key, "/")

	if err := h.svc.DeleteKey(conn, key); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "key deleted", "key": key})
}

// SetTTL 设置 TTL
func (h *RedisHandler) SetTTL(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	key := c.Param("key")
	key = strings.TrimPrefix(key, "/")

	var req struct {
		TTL int64 `json:"ttl"` // 秒，-1 表示移除过期时间
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.SetTTL(conn, key, req.TTL); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "TTL updated", "key": key, "ttl": req.TTL})
}

// Export 导出数据
func (h *RedisHandler) Export(c *gin.Context) {
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
		Keys   []string `json:"keys"`
		Format string   `json:"format"` // json
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if len(req.Keys) == 0 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no keys specified"})
		return
	}

	data, err := h.svc.Export(conn, req.Keys)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, data)
}

// Import 导入数据
func (h *RedisHandler) Import(c *gin.Context) {
	conn, err := h.getConnection(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection id"})
		return
	}
	if conn == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "no connection selected"})
		return
	}

	var data service.ExportData
	if err := c.ShouldBindJSON(&data); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.Import(conn, &data); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "import completed",
		"count":   len(data.Keys),
	})
}
