package api

import (
	"fmt"
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/service"
	"github.com/zeni-x/backend/internal/store"
)

// K8sHandler K8s 服务发现处理器
type K8sHandler struct {
	discoverySvc *service.DiscoveryService
	db           *store.SQLite
}

// NewK8sHandler 创建 K8s 处理器
func NewK8sHandler(discoverySvc *service.DiscoveryService, db *store.SQLite) *K8sHandler {
	return &K8sHandler{
		discoverySvc: discoverySvc,
		db:           db,
	}
}

// DiscoverServicesRequest 发现服务请求
type DiscoverServicesRequest struct {
	Kubeconfig string `json:"kubeconfig"` // 可选的 kubeconfig 内容
	Context    string `json:"context"`    // 可选的上下文名称
}

// DiscoverServices 发现集群中的中间件服务
// @Summary 自动发现 K8s 集群中的中间件服务
// @Tags k8s
// @Accept json
// @Produce json
// @Param request body DiscoverServicesRequest false "发现请求（可选）"
// @Success 200 {array} service.DiscoveredService
// @Failure 500 {object} map[string]string
// @Router /api/k8s/discover [post]
func (h *K8sHandler) DiscoverServices(c *gin.Context) {
	var req DiscoverServicesRequest
	
	// Try to bind JSON body (optional)
	_ = c.ShouldBindJSON(&req)

	ctx := c.Request.Context()

	// If kubeconfig is provided, use it; otherwise use default discovery service
	var discoverySvc *service.DiscoveryService
	var err error

	if req.Kubeconfig != "" {
		// Create temporary discovery service with provided kubeconfig and context
		discoverySvc, err = service.NewDiscoveryServiceWithConfig(req.Kubeconfig, req.Context)
		if err != nil {
			log.Printf("Failed to create discovery service with kubeconfig: %v", err)
			c.JSON(http.StatusBadRequest, gin.H{
				"error": fmt.Sprintf("Invalid kubeconfig or context: %v", err),
			})
			return
		}
	} else {
		// Use default discovery service
		if h.discoverySvc == nil {
			c.JSON(http.StatusServiceUnavailable, gin.H{
				"error": "K8s discovery service is not available. Please provide a kubeconfig or ensure the application is running in a Kubernetes cluster with proper RBAC permissions.",
			})
			return
		}
		discoverySvc = h.discoverySvc
	}

	discovered, err := discoverySvc.DiscoverServices(ctx)
	if err != nil {
		log.Printf("Failed to discover services: %v", err)
		c.JSON(http.StatusInternalServerError, gin.H{
			"error": err.Error(),
		})
		return
	}

	c.JSON(http.StatusOK, discovered)
}

// ListClustersRequest 列出集群请求
type ListClustersRequest struct {
	Kubeconfig string `json:"kubeconfig" binding:"required"`
}

// ListClustersResponse 列出集群响应
type ListClustersResponse struct {
	Clusters []string `json:"clusters"`
}

// ListClusters 列出 kubeconfig 中的所有集群上下文
// @Summary 列出 kubeconfig 中的集群
// @Tags k8s
// @Accept json
// @Produce json
// @Param request body ListClustersRequest true "列出集群请求"
// @Success 200 {object} ListClustersResponse
// @Failure 400 {object} map[string]string
// @Router /api/k8s/clusters [post]
func (h *K8sHandler) ListClusters(c *gin.Context) {
	var req ListClustersRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	clusters, err := service.ListClustersFromKubeconfig(req.Kubeconfig)
	if err != nil {
		log.Printf("Failed to list clusters: %v", err)
		c.JSON(http.StatusBadRequest, gin.H{
			"error": fmt.Sprintf("Failed to parse kubeconfig: %v", err),
		})
		return
	}

	c.JSON(http.StatusOK, ListClustersResponse{
		Clusters: clusters,
	})
}

// ImportConnectionsRequest 批量导入连接请求
type ImportConnectionsRequest struct {
	Services      []ImportServiceItem `json:"services" binding:"required"`
	ForceOverride bool                `json:"force_override"` // 是否强制覆盖已存在的连接
	Kubeconfig    string              `json:"kubeconfig"`     // kubeconfig 内容，用于端口转发
	Context       string              `json:"context"`        // kubeconfig 上下文
	ClusterName   string              `json:"cluster_name"`   // 集群名称
}

// ImportServiceItem 单个导入服务项
type ImportServiceItem struct {
	Name        string `json:"name" binding:"required"`
	Type        string `json:"type" binding:"required"`
	Namespace   string `json:"namespace" binding:"required"`
	Host        string `json:"host" binding:"required"`
	Port        int    `json:"port" binding:"required"`
	Username    string `json:"username"`
	Password    string `json:"password"`
	Database    string `json:"database"`
	ServiceName string `json:"service_name"` // K8s service 名称，用于端口转发
}

// ImportConnectionsResponse 批量导入响应
type ImportConnectionsResponse struct {
	Success  int                      `json:"success"`
	Failed   int                      `json:"failed"`
	Updated  int                      `json:"updated"`  // 新增：覆盖更新的数量
	Skipped  int                      `json:"skipped"`  // 新增：跳过的数量
	Results  []ImportConnectionResult `json:"results"`
}

// ImportConnectionResult 单个导入结果
type ImportConnectionResult struct {
	Name     string `json:"name"`
	Success  bool   `json:"success"`
	Updated  bool   `json:"updated,omitempty"`  // 新增：是否是更新操作
	Skipped  bool   `json:"skipped,omitempty"`  // 新增：是否被跳过
	Error    string `json:"error,omitempty"`
	ID       int64  `json:"id,omitempty"`
}

// ImportConnections 批量导入发现的服务为连接配置
// @Summary 批量导入发现的服务
// @Tags k8s
// @Accept json
// @Produce json
// @Param request body ImportConnectionsRequest true "导入请求"
// @Success 200 {object} ImportConnectionsResponse
// @Failure 400 {object} map[string]string
// @Router /api/k8s/import [post]
func (h *K8sHandler) ImportConnections(c *gin.Context) {
	// Note: Import doesn't require discovery service (just imports provided data)
	var req ImportConnectionsRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	response := ImportConnectionsResponse{
		Success: 0,
		Failed:  0,
		Updated: 0,
		Skipped: 0,
		Results: make([]ImportConnectionResult, 0, len(req.Services)),
	}

	// 如果提供了集群名称，尝试获取或创建集群记录
	var clusterID *int64
	if req.ClusterName != "" {
		cluster, err := h.db.GetClusterByName(req.ClusterName)
		if err != nil {
			// 集群不存在，创建新集群
			newCluster := store.Cluster{
				Name:        req.ClusterName,
				Context:     req.Context,
				Environment: "unknown", // 可以从名称推断或让用户后续设置
				IsActive:    true,
			}
			if err := h.db.CreateCluster(&newCluster); err != nil {
				log.Printf("Warning: failed to create cluster record: %v", err)
			} else {
				clusterID = &newCluster.ID
				log.Printf("Created cluster record: %s (ID: %d)", req.ClusterName, newCluster.ID)
			}
		} else {
			clusterID = &cluster.ID
			log.Printf("Using existing cluster: %s (ID: %d)", req.ClusterName, cluster.ID)
		}
	}

	for _, svc := range req.Services {
		result := ImportConnectionResult{
			Name:    svc.Name,
			Success: false,
		}

		// 构建连接名称（包含命名空间信息）
		connName := svc.Name
		if svc.Namespace != "" {
			connName = svc.Namespace + "/" + svc.Name
		}

		// 确定服务名称（用于端口转发）
		serviceName := svc.ServiceName
		if serviceName == "" {
			serviceName = svc.Name // 默认使用名称作为服务名
		}

		// 创建连接配置
		// 对于 K8s 服务，我们使用占位符地址并保存 K8s 信息用于后续端口转发
		conn := store.Connection{
			Name:           connName,
			Type:           svc.Type,
			Host:           "localhost", // 使用 localhost 作为占位符，需要端口转发才能访问
			Port:           0,            // 端口将在端口转发时分配
			Username:       svc.Username,
			Password:       svc.Password,
			DatabaseName:   svc.Database,
			IsDefault:      false,
			K8sNamespace:   svc.Namespace,
			K8sServiceName: serviceName,
			K8sServicePort: svc.Port,
			ForwardStatus:  "pending", // 标记为需要端口转发
			ClusterID:      clusterID, // 关联集群
			Source:         "k8s",     // 标记来源为 k8s
		}

		// 检查是否已存在相同的连接（基于 K8s 服务信息）
		existingConns, err := h.db.GetConnectionsByType(svc.Type)
		if err == nil {
			var existingConn *store.Connection
			for _, existing := range existingConns {
				// 对于 K8s 服务，基于 namespace + service name 判断是否相同
				if existing.K8sNamespace == conn.K8sNamespace && 
				   existing.K8sServiceName == conn.K8sServiceName {
					existingConn = &existing
					break
				}
			}

			if existingConn != nil {
				// 连接已存在
				if req.ForceOverride {
					// 强制覆盖：更新现有连接
					conn.ID = existingConn.ID
					conn.IsDefault = existingConn.IsDefault // 保留默认状态
					// 保留现有的端口转发信息（如果有）
					if existingConn.ForwardID != "" {
						conn.ForwardID = existingConn.ForwardID
						conn.ForwardLocalPort = existingConn.ForwardLocalPort
						conn.ForwardStatus = existingConn.ForwardStatus
						conn.Host = existingConn.Host
						conn.Port = existingConn.Port
					}
					
					if err := h.db.UpdateConnection(&conn); err != nil {
						log.Printf("Failed to update connection %s: %v", connName, err)
						result.Error = err.Error()
						response.Failed++
					} else {
						result.Success = true
						result.Updated = true
						result.ID = conn.ID
						response.Success++
						response.Updated++
						log.Printf("Updated existing connection: %s (ID: %d)", connName, conn.ID)
					}
				} else {
					// 不覆盖：跳过
					result.Skipped = true
					result.Error = "connection already exists"
					response.Skipped++
					log.Printf("Skipped existing connection: %s", connName)
				}
				
				response.Results = append(response.Results, result)
				continue
			}
		}

		// 保存到数据库（新建）
		if err := h.db.CreateConnection(&conn); err != nil {
			log.Printf("Failed to import connection %s: %v", connName, err)
			result.Error = err.Error()
			response.Failed++
		} else {
			result.Success = true
			result.ID = conn.ID
			response.Success++
			log.Printf("Created new connection: %s (ID: %d, requires port-forward)", connName, conn.ID)
		}

		response.Results = append(response.Results, result)
	}

	c.JSON(http.StatusOK, response)
}

