package api

import (
	"fmt"
	"log"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/k8s"
	"github.com/zeni-x/backend/internal/store"
)

// PortForwardHandler 端口转发处理器
type PortForwardHandler struct {
	manager *k8s.PortForwardManager
	db      *store.SQLite
}

// NewPortForwardHandler 创建端口转发处理器
func NewPortForwardHandler(manager *k8s.PortForwardManager, db *store.SQLite) *PortForwardHandler {
	return &PortForwardHandler{
		manager: manager,
		db:      db,
	}
}

// CreateForwardRequest 创建端口转发请求
type CreateForwardRequest struct {
	ConnectionID int64  `json:"connection_id" binding:"required"`
	Namespace    string `json:"namespace" binding:"required"`
	ServiceName  string `json:"service_name" binding:"required"`
	RemotePort   int32  `json:"remote_port" binding:"required"`
}

// ForwardResponse 端口转发响应
type ForwardResponse struct {
	ID           string `json:"id"`
	ConnectionID int64  `json:"connection_id"`
	LocalHost    string `json:"local_host"`
	LocalPort    int    `json:"local_port"`
	RemoteHost   string `json:"remote_host"`
	RemotePort   int32  `json:"remote_port"`
	Status       string `json:"status"`
	CreatedAt    string `json:"created_at"`
	LastUsedAt   string `json:"last_used_at"`
	ErrorMessage string `json:"error_message,omitempty"`
}

// ForwardListResponse 转发列表响应
type ForwardListResponse struct {
	Forwards []ForwardResponse `json:"forwards"`
	Total    int               `json:"total"`
}

// CreateForward 创建端口转发
// @Summary 创建端口转发
// @Tags port-forward
// @Accept json
// @Produce json
// @Param request body CreateForwardRequest true "创建请求"
// @Success 200 {object} ForwardResponse
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/port-forward [post]
func (h *PortForwardHandler) CreateForward(c *gin.Context) {
	var req CreateForwardRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	ctx := c.Request.Context()

	// 获取连接信息以获取 kubeconfig
	var kubeconfig, k8sContext string
	conn, err := h.db.GetConnectionByID(req.ConnectionID)
	if err == nil && conn.ClusterID != nil {
		cluster, err := h.db.GetClusterByID(*conn.ClusterID)
		if err == nil {
			kubeconfig = cluster.Kubeconfig
			k8sContext = cluster.Context
		}
	}

	forward, err := h.manager.CreateForward(ctx, req.ConnectionID, req.Namespace, req.ServiceName, req.RemotePort, kubeconfig, k8sContext)
	if err != nil {
		log.Printf("Failed to create port forward: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": fmt.Sprintf("Failed to create port forward: %v", err),
		})
		return
	}

	// 更新连接信息
	if err == nil {
		conn.ForwardID = forward.ID
		conn.ForwardLocalPort = forward.LocalPort
		conn.ForwardStatus = string(forward.Status)
		// 更新主机和端口为本地转发地址
		conn.Host = "localhost"
		conn.Port = forward.LocalPort
		h.db.UpdateConnection(conn)
	}

	c.JSON(http.StatusOK, h.toForwardResponse(forward))
}

// ListForwards 列出所有端口转发
// @Summary 列出所有端口转发
// @Tags port-forward
// @Produce json
// @Success 200 {object} ForwardListResponse
// @Router /api/port-forward [get]
func (h *PortForwardHandler) ListForwards(c *gin.Context) {
	forwards := h.manager.ListForwards()

	responses := make([]ForwardResponse, 0, len(forwards))
	for _, forward := range forwards {
		responses = append(responses, h.toForwardResponse(forward))
	}

	c.JSON(http.StatusOK, ForwardListResponse{
		Forwards: responses,
		Total:    len(responses),
	})
}

// GetForward 获取单个端口转发状态
// @Summary 获取端口转发状态
// @Tags port-forward
// @Produce json
// @Param id path string true "转发ID"
// @Success 200 {object} ForwardResponse
// @Failure 404 {object} map[string]string
// @Router /api/port-forward/:id [get]
func (h *PortForwardHandler) GetForward(c *gin.Context) {
	id := c.Param("id")

	forward, err := h.manager.GetForward(id)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, h.toForwardResponse(forward))
}

// StopForward 停止端口转发
// @Summary 停止端口转发
// @Tags port-forward
// @Param id path string true "转发ID"
// @Success 200 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/port-forward/:id [delete]
func (h *PortForwardHandler) StopForward(c *gin.Context) {
	id := c.Param("id")

	// 获取转发信息以更新连接
	forward, err := h.manager.GetForward(id)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	// 停止转发
	if err := h.manager.StopForward(id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// 更新连接状态
	if forward.ConnectionID > 0 {
		conn, err := h.db.GetConnectionByID(forward.ConnectionID)
		if err == nil {
			conn.ForwardID = ""
			conn.ForwardLocalPort = 0
			conn.ForwardStatus = ""
			// 恢复原始主机和端口（从服务信息）
			conn.Host = fmt.Sprintf("%s.%s.svc.cluster.local", forward.ServiceName, forward.Namespace)
			conn.Port = int(forward.RemotePort)
			h.db.UpdateConnection(conn)
		}
	}

	c.JSON(http.StatusOK, gin.H{"status": "stopped"})
}

// ReconnectForward 重新连接端口转发
// @Summary 重新连接端口转发
// @Tags port-forward
// @Param id path string true "转发ID"
// @Success 200 {object} ForwardResponse
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/port-forward/:id/reconnect [post]
func (h *PortForwardHandler) ReconnectForward(c *gin.Context) {
	id := c.Param("id")
	ctx := c.Request.Context()

	forward, err := h.manager.Reconnect(ctx, id)
	if err != nil {
		log.Printf("Failed to reconnect port forward: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": fmt.Sprintf("Failed to reconnect: %v", err),
		})
		return
	}

	// 更新连接信息
	if forward.ConnectionID > 0 {
		conn, err := h.db.GetConnectionByID(forward.ConnectionID)
		if err == nil {
			conn.ForwardID = forward.ID
			conn.ForwardLocalPort = forward.LocalPort
			conn.ForwardStatus = string(forward.Status)
			conn.Host = "localhost"
			conn.Port = forward.LocalPort
			h.db.UpdateConnection(conn)
		}
	}

	c.JSON(http.StatusOK, h.toForwardResponse(forward))
}

// TouchForward 更新端口转发使用时间
// @Summary 更新使用时间
// @Tags port-forward
// @Param id path string true "转发ID"
// @Success 200 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Router /api/port-forward/:id/touch [put]
func (h *PortForwardHandler) TouchForward(c *gin.Context) {
	id := c.Param("id")

	if err := h.manager.UpdateLastUsed(id); err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "updated"})
}

// GetForwardByConnection 通过连接ID获取端口转发
// @Summary 通过连接ID获取端口转发
// @Tags port-forward
// @Param connection_id query int true "连接ID"
// @Success 200 {object} ForwardResponse
// @Failure 404 {object} map[string]string
// @Router /api/port-forward/by-connection [get]
func (h *PortForwardHandler) GetForwardByConnection(c *gin.Context) {
	connectionIDStr := c.Query("connection_id")
	connectionID, err := strconv.ParseInt(connectionIDStr, 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid connection_id"})
		return
	}

	forward, err := h.manager.GetForwardByConnectionID(connectionID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, h.toForwardResponse(forward))
}

// toForwardResponse 转换为响应格式
func (h *PortForwardHandler) toForwardResponse(forward *k8s.PortForward) ForwardResponse {
	return ForwardResponse{
		ID:           forward.ID,
		ConnectionID: forward.ConnectionID,
		LocalHost:    "localhost",
		LocalPort:    forward.LocalPort,
		RemoteHost:   fmt.Sprintf("%s.%s.svc.cluster.local", forward.ServiceName, forward.Namespace),
		RemotePort:   forward.RemotePort,
		Status:       string(forward.Status),
		CreatedAt:    forward.CreatedAt.Format("2006-01-02T15:04:05Z07:00"),
		LastUsedAt:   forward.LastUsedAt.Format("2006-01-02T15:04:05Z07:00"),
		ErrorMessage: forward.ErrorMessage,
	}
}

