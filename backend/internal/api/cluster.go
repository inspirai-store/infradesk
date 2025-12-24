package api

import (
	"database/sql"
	"log"
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/store"
)

// ClusterHandler 集群管理处理器
type ClusterHandler struct {
	db *store.SQLite
}

// NewClusterHandler 创建集群处理器
func NewClusterHandler(db *store.SQLite) *ClusterHandler {
	return &ClusterHandler{db: db}
}

// GetClusters 获取所有集群
// @Summary 获取所有集群
// @Tags clusters
// @Produce json
// @Success 200 {array} store.Cluster
// @Failure 500 {object} map[string]string
// @Router /api/clusters [get]
func (h *ClusterHandler) GetClusters(c *gin.Context) {
	clusters, err := h.db.GetClusters()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}
	c.JSON(http.StatusOK, clusters)
}

// GetCluster 获取单个集群
// @Summary 获取单个集群
// @Tags clusters
// @Param id path int true "Cluster ID"
// @Produce json
// @Success 200 {object} store.Cluster
// @Failure 404 {object} map[string]string
// @Router /api/clusters/:id [get]
func (h *ClusterHandler) GetCluster(c *gin.Context) {
	id := c.Param("id")
	idInt, err := strconv.ParseInt(id, 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
		return
	}

	cluster, err := h.db.GetClusterByID(idInt)
	if err != nil {
		if err == sql.ErrNoRows {
			c.JSON(http.StatusNotFound, gin.H{"error": "cluster not found"})
		} else {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		}
		return
	}

	c.JSON(http.StatusOK, cluster)
}

// CreateCluster 创建集群
// @Summary 创建集群
// @Tags clusters
// @Accept json
// @Produce json
// @Param cluster body store.Cluster true "Cluster object"
// @Success 201 {object} store.Cluster
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/clusters [post]
func (h *ClusterHandler) CreateCluster(c *gin.Context) {
	var cluster store.Cluster
	if err := c.ShouldBindJSON(&cluster); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	// 检查集群名称是否已存在
	_, err := h.db.GetClusterByName(cluster.Name)
	if err == nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "cluster name already exists"})
		return
	}

	if err := h.db.CreateCluster(&cluster); err != nil {
		log.Printf("Failed to create cluster: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusCreated, cluster)
}

// UpdateCluster 更新集群
// @Summary 更新集群
// @Tags clusters
// @Accept json
// @Produce json
// @Param id path int true "Cluster ID"
// @Param cluster body store.Cluster true "Cluster object"
// @Success 200 {object} store.Cluster
// @Failure 400 {object} map[string]string
// @Failure 404 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/clusters/:id [put]
func (h *ClusterHandler) UpdateCluster(c *gin.Context) {
	id := c.Param("id")
	idInt, err := strconv.ParseInt(id, 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
		return
	}

	var cluster store.Cluster
	if err := c.ShouldBindJSON(&cluster); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	cluster.ID = idInt

	// 检查集群是否存在
	_, err = h.db.GetClusterByID(idInt)
	if err != nil {
		if err == sql.ErrNoRows {
			c.JSON(http.StatusNotFound, gin.H{"error": "cluster not found"})
		} else {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		}
		return
	}

	if err := h.db.UpdateCluster(&cluster); err != nil {
		log.Printf("Failed to update cluster: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, cluster)
}

// DeleteCluster 删除集群
// @Summary 删除集群
// @Tags clusters
// @Param id path int true "Cluster ID"
// @Success 200 {object} map[string]string
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/clusters/:id [delete]
func (h *ClusterHandler) DeleteCluster(c *gin.Context) {
	id := c.Param("id")
	idInt, err := strconv.ParseInt(id, 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
		return
	}

	// 检查是否有关联的连接
	connections, err := h.db.GetConnectionsByCluster(idInt)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	if len(connections) > 0 {
		c.JSON(http.StatusBadRequest, gin.H{
			"error":       "cluster has associated connections",
			"connections": len(connections),
		})
		return
	}

	if err := h.db.DeleteCluster(idInt); err != nil {
		log.Printf("Failed to delete cluster: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "cluster deleted"})
}

// GetClusterConnections 获取集群下的所有连接
// @Summary 获取集群下的所有连接
// @Tags clusters
// @Param id path int true "Cluster ID"
// @Produce json
// @Success 200 {array} store.Connection
// @Failure 400 {object} map[string]string
// @Failure 500 {object} map[string]string
// @Router /api/clusters/:id/connections [get]
func (h *ClusterHandler) GetClusterConnections(c *gin.Context) {
	id := c.Param("id")
	idInt, err := strconv.ParseInt(id, 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid id"})
		return
	}

	connections, err := h.db.GetConnectionsByCluster(idInt)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	// 清空密码字段以保护安全
	for i := range connections {
		connections[i].Password = ""
	}

	c.JSON(http.StatusOK, connections)
}

