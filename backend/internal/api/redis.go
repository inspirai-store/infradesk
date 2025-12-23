package api

import (
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// RedisHandler Redis API 处理器
type RedisHandler struct {
	svc *service.RedisService
	db  *store.SQLite
}

// NewRedisHandler 创建 Redis 处理器
func NewRedisHandler(svc *service.RedisService, db *store.SQLite) *RedisHandler {
	return &RedisHandler{svc: svc, db: db}
}

// GetInfo 获取 Redis 信息
func (h *RedisHandler) GetInfo(c *gin.Context) {
	info, err := h.svc.GetInfo()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, info)
}

// ListKeys 列出 Keys
func (h *RedisHandler) ListKeys(c *gin.Context) {
	pattern := c.DefaultQuery("pattern", "*")
	cursor, _ := strconv.ParseUint(c.DefaultQuery("cursor", "0"), 10, 64)
	count, _ := strconv.ParseInt(c.DefaultQuery("count", "100"), 10, 64)

	if count < 1 || count > 1000 {
		count = 100
	}

	result, err := h.svc.ListKeys(pattern, cursor, count)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, result)
}

// GetKey 获取 Key 详情
func (h *RedisHandler) GetKey(c *gin.Context) {
	key := c.Param("key")
	// 移除前导斜杠
	key = strings.TrimPrefix(key, "/")

	info, err := h.svc.GetKey(key)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, info)
}

// SetKey 创建 Key
func (h *RedisHandler) SetKey(c *gin.Context) {
	var req service.SetKeyRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.SetKey(&req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{"message": "key created", "key": req.Key})
}

// UpdateKey 更新 Key
func (h *RedisHandler) UpdateKey(c *gin.Context) {
	key := c.Param("key")
	key = strings.TrimPrefix(key, "/")

	var req service.SetKeyRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	req.Key = key

	if err := h.svc.SetKey(&req); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "key updated", "key": key})
}

// DeleteKey 删除 Key
func (h *RedisHandler) DeleteKey(c *gin.Context) {
	key := c.Param("key")
	key = strings.TrimPrefix(key, "/")

	if err := h.svc.DeleteKey(key); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "key deleted", "key": key})
}

// SetTTL 设置 TTL
func (h *RedisHandler) SetTTL(c *gin.Context) {
	key := c.Param("key")
	key = strings.TrimPrefix(key, "/")

	var req struct {
		TTL int64 `json:"ttl"` // 秒，-1 表示移除过期时间
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.SetTTL(key, req.TTL); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "TTL updated", "key": key, "ttl": req.TTL})
}

// Export 导出数据
func (h *RedisHandler) Export(c *gin.Context) {
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

	data, err := h.svc.Export(req.Keys)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, data)
}

// Import 导入数据
func (h *RedisHandler) Import(c *gin.Context) {
	var data service.ExportData
	if err := c.ShouldBindJSON(&data); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if err := h.svc.Import(&data); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "import completed",
		"count":   len(data.Keys),
	})
}

