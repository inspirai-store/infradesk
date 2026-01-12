package api

import (
	"fmt"
	"log"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/zeni-x/backend/internal/k8s"
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

	// 过滤掉已经存在的 K8s 连接
	existingConns, err := h.db.GetConnections()
	if err == nil {
		// 构建已存在服务的映射 (namespace/service_name)
		existingMap := make(map[string]bool)
		for _, conn := range existingConns {
			if conn.K8sNamespace != "" && conn.K8sServiceName != "" {
				key := fmt.Sprintf("%s/%s", conn.K8sNamespace, conn.K8sServiceName)
				existingMap[key] = true
			}
		}

		// 过滤发现的服务
		filtered := make([]service.DiscoveredService, 0)
		for _, svc := range discovered {
			key := fmt.Sprintf("%s/%s", svc.Namespace, svc.Name)
			if !existingMap[key] {
				filtered = append(filtered, svc)
			}
		}
		discovered = filtered
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

// ==================== K8s Resource Response Types ====================

// K8sDeployment Deployment 信息
type K8sDeployment struct {
	Name              string            `json:"name"`
	Namespace         string            `json:"namespace"`
	Replicas          int32             `json:"replicas"`
	ReadyReplicas     int32             `json:"ready_replicas"`
	AvailableReplicas int32             `json:"available_replicas"`
	Labels            map[string]string `json:"labels"`
	CreatedAt         *string           `json:"created_at"`
}

// K8sPod Pod 信息
type K8sPod struct {
	Name      string  `json:"name"`
	Namespace string  `json:"namespace"`
	Status    string  `json:"status"`
	Ready     string  `json:"ready"`
	Restarts  int32   `json:"restarts"`
	Node      *string `json:"node"`
	IP        *string `json:"ip"`
	CreatedAt *string `json:"created_at"`
}

// K8sConfigMapInfo ConfigMap 信息
type K8sConfigMapInfo struct {
	Name      string   `json:"name"`
	Namespace string   `json:"namespace"`
	DataKeys  []string `json:"data_keys"`
	CreatedAt *string  `json:"created_at"`
}

// K8sSecretInfo Secret 信息
type K8sSecretInfo struct {
	Name       string   `json:"name"`
	Namespace  string   `json:"namespace"`
	SecretType string   `json:"secret_type"`
	DataKeys   []string `json:"data_keys"`
	CreatedAt  *string  `json:"created_at"`
}

// K8sServiceInfo Service 信息
type K8sServiceInfo struct {
	Name        string   `json:"name"`
	Namespace   string   `json:"namespace"`
	ServiceType string   `json:"service_type"`
	ClusterIP   *string  `json:"cluster_ip"`
	ExternalIP  *string  `json:"external_ip"`
	Ports       []string `json:"ports"`
	CreatedAt   *string  `json:"created_at"`
}

// K8sIngressInfo Ingress 信息
type K8sIngressInfo struct {
	Name      string   `json:"name"`
	Namespace string   `json:"namespace"`
	Hosts     []string `json:"hosts"`
	Address   *string  `json:"address"`
	CreatedAt *string  `json:"created_at"`
}

// getK8sClient 根据集群 ID 获取 K8s 客户端
func (h *K8sHandler) getK8sClient(clusterID int64) (*k8s.Client, error) {
	cluster, err := h.db.GetClusterByID(clusterID)
	if err != nil {
		return nil, fmt.Errorf("cluster not found: %w", err)
	}

	// 从集群配置创建客户端
	client, err := k8s.NewClientWithConfig(cluster.Kubeconfig, cluster.Context)
	if err != nil {
		return nil, fmt.Errorf("failed to create k8s client: %w", err)
	}

	return client, nil
}

// ListNamespaces 列出集群的所有命名空间
// @Router /api/k8s/clusters/:id/namespaces [get]
func (h *K8sHandler) ListNamespaces(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	namespaces, err := client.ListAllNamespaces(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, namespaces)
}

// ListDeployments 列出命名空间的 Deployments
// @Router /api/k8s/clusters/:id/namespaces/:namespace/deployments [get]
func (h *K8sHandler) ListDeployments(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	deployments, err := client.ListDeployments(c.Request.Context(), namespace)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result := make([]K8sDeployment, 0, len(deployments))
	for _, d := range deployments {
		var createdAt *string
		if !d.CreationTimestamp.IsZero() {
			t := d.CreationTimestamp.Format(time.RFC3339)
			createdAt = &t
		}

		var replicas, readyReplicas, availableReplicas int32
		if d.Spec.Replicas != nil {
			replicas = *d.Spec.Replicas
		}
		readyReplicas = d.Status.ReadyReplicas
		availableReplicas = d.Status.AvailableReplicas

		result = append(result, K8sDeployment{
			Name:              d.Name,
			Namespace:         d.Namespace,
			Replicas:          replicas,
			ReadyReplicas:     readyReplicas,
			AvailableReplicas: availableReplicas,
			Labels:            d.Labels,
			CreatedAt:         createdAt,
		})
	}

	c.JSON(http.StatusOK, result)
}

// ListPods 列出命名空间的 Pods
// @Router /api/k8s/clusters/:id/namespaces/:namespace/pods [get]
func (h *K8sHandler) ListPods(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	pods, err := client.ListPods(c.Request.Context(), namespace)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result := make([]K8sPod, 0, len(pods))
	for _, p := range pods {
		var createdAt *string
		if !p.CreationTimestamp.IsZero() {
			t := p.CreationTimestamp.Format(time.RFC3339)
			createdAt = &t
		}

		// Calculate ready containers
		var readyContainers, totalContainers int
		var restarts int32
		for _, cs := range p.Status.ContainerStatuses {
			totalContainers++
			if cs.Ready {
				readyContainers++
			}
			restarts += cs.RestartCount
		}
		ready := fmt.Sprintf("%d/%d", readyContainers, totalContainers)

		var node, ip *string
		if p.Spec.NodeName != "" {
			node = &p.Spec.NodeName
		}
		if p.Status.PodIP != "" {
			ip = &p.Status.PodIP
		}

		result = append(result, K8sPod{
			Name:      p.Name,
			Namespace: p.Namespace,
			Status:    string(p.Status.Phase),
			Ready:     ready,
			Restarts:  restarts,
			Node:      node,
			IP:        ip,
			CreatedAt: createdAt,
		})
	}

	c.JSON(http.StatusOK, result)
}

// ListConfigMaps 列出命名空间的 ConfigMaps
// @Router /api/k8s/clusters/:id/namespaces/:namespace/configmaps [get]
func (h *K8sHandler) ListConfigMaps(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	configmaps, err := client.ListConfigMaps(c.Request.Context(), namespace)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result := make([]K8sConfigMapInfo, 0, len(configmaps))
	for _, cm := range configmaps {
		var createdAt *string
		if !cm.CreationTimestamp.IsZero() {
			t := cm.CreationTimestamp.Format(time.RFC3339)
			createdAt = &t
		}

		keys := make([]string, 0, len(cm.Data))
		for k := range cm.Data {
			keys = append(keys, k)
		}

		result = append(result, K8sConfigMapInfo{
			Name:      cm.Name,
			Namespace: cm.Namespace,
			DataKeys:  keys,
			CreatedAt: createdAt,
		})
	}

	c.JSON(http.StatusOK, result)
}

// GetConfigMapData 获取 ConfigMap 数据
// @Router /api/k8s/clusters/:id/namespaces/:namespace/configmaps/:name [get]
func (h *K8sHandler) GetConfigMapData(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")
	name := c.Param("name")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	cm, err := client.GetConfigMap(c.Request.Context(), namespace, name)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, cm.Data)
}

// ListSecrets 列出命名空间的 Secrets
// @Router /api/k8s/clusters/:id/namespaces/:namespace/secrets [get]
func (h *K8sHandler) ListSecrets(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	secrets, err := client.ListSecrets(c.Request.Context(), namespace)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result := make([]K8sSecretInfo, 0, len(secrets))
	for _, s := range secrets {
		var createdAt *string
		if !s.CreationTimestamp.IsZero() {
			t := s.CreationTimestamp.Format(time.RFC3339)
			createdAt = &t
		}

		keys := make([]string, 0, len(s.Data))
		for k := range s.Data {
			keys = append(keys, k)
		}

		result = append(result, K8sSecretInfo{
			Name:       s.Name,
			Namespace:  s.Namespace,
			SecretType: string(s.Type),
			DataKeys:   keys,
			CreatedAt:  createdAt,
		})
	}

	c.JSON(http.StatusOK, result)
}

// ListServices 列出命名空间的 Services
// @Router /api/k8s/clusters/:id/namespaces/:namespace/services [get]
func (h *K8sHandler) ListServices(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	services, err := client.ListServices(c.Request.Context(), namespace)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result := make([]K8sServiceInfo, 0, len(services))
	for _, svc := range services {
		var createdAt *string
		if !svc.CreationTimestamp.IsZero() {
			t := svc.CreationTimestamp.Format(time.RFC3339)
			createdAt = &t
		}

		var clusterIP, externalIP *string
		if svc.Spec.ClusterIP != "" && svc.Spec.ClusterIP != "None" {
			clusterIP = &svc.Spec.ClusterIP
		}
		if len(svc.Status.LoadBalancer.Ingress) > 0 {
			ip := svc.Status.LoadBalancer.Ingress[0].IP
			if ip == "" {
				ip = svc.Status.LoadBalancer.Ingress[0].Hostname
			}
			if ip != "" {
				externalIP = &ip
			}
		}

		ports := make([]string, 0, len(svc.Spec.Ports))
		for _, p := range svc.Spec.Ports {
			portStr := fmt.Sprintf("%d", p.Port)
			if p.NodePort > 0 {
				portStr = fmt.Sprintf("%d:%d", p.Port, p.NodePort)
			}
			portStr = fmt.Sprintf("%s/%s", portStr, p.Protocol)
			ports = append(ports, portStr)
		}

		result = append(result, K8sServiceInfo{
			Name:        svc.Name,
			Namespace:   svc.Namespace,
			ServiceType: string(svc.Spec.Type),
			ClusterIP:   clusterIP,
			ExternalIP:  externalIP,
			Ports:       ports,
			CreatedAt:   createdAt,
		})
	}

	c.JSON(http.StatusOK, result)
}

// ListIngresses 列出命名空间的 Ingresses
// @Router /api/k8s/clusters/:id/namespaces/:namespace/ingresses [get]
func (h *K8sHandler) ListIngresses(c *gin.Context) {
	clusterID, err := strconv.ParseInt(c.Param("id"), 10, 64)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid cluster id"})
		return
	}
	namespace := c.Param("namespace")

	client, err := h.getK8sClient(clusterID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	ingresses, err := client.ListIngresses(c.Request.Context(), namespace)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	result := make([]K8sIngressInfo, 0, len(ingresses))
	for _, ing := range ingresses {
		var createdAt *string
		if !ing.CreationTimestamp.IsZero() {
			t := ing.CreationTimestamp.Format(time.RFC3339)
			createdAt = &t
		}

		hosts := make([]string, 0)
		for _, rule := range ing.Spec.Rules {
			if rule.Host != "" {
				hosts = append(hosts, rule.Host)
			}
		}

		var address *string
		if len(ing.Status.LoadBalancer.Ingress) > 0 {
			addr := ing.Status.LoadBalancer.Ingress[0].IP
			if addr == "" {
				addr = ing.Status.LoadBalancer.Ingress[0].Hostname
			}
			if addr != "" {
				address = &addr
			}
		}

		result = append(result, K8sIngressInfo{
			Name:      ing.Name,
			Namespace: ing.Namespace,
			Hosts:     hosts,
			Address:   address,
			CreatedAt: createdAt,
		})
	}

	c.JSON(http.StatusOK, result)
}

