package service

import (
	"context"
	"fmt"
	"strings"

	corev1 "k8s.io/api/core/v1"

	"github.com/zeni-x/backend/internal/k8s"
	"k8s.io/client-go/tools/clientcmd"
)

// DiscoveryService 服务发现服务
type DiscoveryService struct {
	k8sClient *k8s.Client
}

// NewDiscoveryService 创建服务发现服务
func NewDiscoveryService() (*DiscoveryService, error) {
	return NewDiscoveryServiceWithConfig("", "")
}

// NewDiscoveryServiceWithConfig 使用指定的 kubeconfig 创建服务发现服务
// context: 可选的上下文名称
func NewDiscoveryServiceWithConfig(kubeconfigContent string, context string) (*DiscoveryService, error) {
	client, err := k8s.NewClientWithConfig(kubeconfigContent, context)
	if err != nil {
		return nil, fmt.Errorf("failed to create kubernetes client: %w", err)
	}

	return &DiscoveryService{
		k8sClient: client,
	}, nil
}

// DiscoveredService 发现的服务信息
type DiscoveredService struct {
	Name           string `json:"name"`
	Type           string `json:"type"`
	Namespace      string `json:"namespace"`
	Host           string `json:"host"`
	Port           int32  `json:"port"`
	Username       string `json:"username,omitempty"`
	Password       string `json:"password,omitempty"`
	Database       string `json:"database,omitempty"`
	HasCredentials bool   `json:"has_credentials"`
}

// MiddlewareType 中间件类型定义
type MiddlewareType struct {
	Name          string
	Ports         []int32
	NamePatterns  []string
	ImagePatterns []string
}

// SupportedMiddlewares 支持的中间件类型
var SupportedMiddlewares = []MiddlewareType{
	{
		Name:          "mysql",
		Ports:         []int32{3306},
		NamePatterns:  []string{"mysql", "mariadb"},
		ImagePatterns: []string{"mysql", "mariadb"},
	},
	{
		Name:          "postgresql",
		Ports:         []int32{5432},
		NamePatterns:  []string{"postgres", "postgresql", "pg"},
		ImagePatterns: []string{"postgres", "postgresql"},
	},
	{
		Name:          "redis",
		Ports:         []int32{6379},
		NamePatterns:  []string{"redis"},
		ImagePatterns: []string{"redis"},
	},
	{
		Name:          "mongodb",
		Ports:         []int32{27017},
		NamePatterns:  []string{"mongo", "mongodb"},
		ImagePatterns: []string{"mongo"},
	},
	{
		Name:          "minio",
		Ports:         []int32{9000},
		NamePatterns:  []string{"minio"},
		ImagePatterns: []string{"minio"},
	},
}

// detectMiddlewareType 检测服务的中间件类型
// 仅基于 Service 的端口和名称进行识别，不查询 Pod 信息
func (s *DiscoveryService) detectMiddlewareType(ctx context.Context, service *corev1.Service) *MiddlewareType {
	serviceName := strings.ToLower(service.Name)

	// 获取服务的所有端口
	var servicePorts []int32
	for _, port := range service.Spec.Ports {
		servicePorts = append(servicePorts, port.Port)
	}

	// 对每种中间件类型进行匹配
	// 优先使用端口和名称匹配，不依赖 Pod 镜像信息
	for _, middleware := range SupportedMiddlewares {
		score := 0

		// 检查端口匹配（权重最高）
		for _, mwPort := range middleware.Ports {
			for _, svcPort := range servicePorts {
				if mwPort == svcPort {
					score += 15 // 提高端口匹配权重
					break
				}
			}
		}

		// 检查名称匹配
		for _, pattern := range middleware.NamePatterns {
			if strings.Contains(serviceName, pattern) {
				score += 10 // 提高名称匹配权重
			}
		}

		// 如果得分足够高，认为是该类型的中间件
		// 降低阈值，因为不再依赖镜像匹配
		if score >= 15 {
			return &middleware
		}
	}

	return nil
}

// extractCredentials 从 Secret 中提取凭据信息
func (s *DiscoveryService) extractCredentials(secret *corev1.Secret, middlewareType string) (username, password, database string) {
	if secret == nil {
		return "", "", ""
	}

	data := secret.Data

	// 定义不同中间件的凭据字段映射
	usernameFields := []string{
		"username", "user", "USER", "USERNAME",
		fmt.Sprintf("%s_USER", strings.ToUpper(middlewareType)),
		fmt.Sprintf("%s_USERNAME", strings.ToUpper(middlewareType)),
	}

	passwordFields := []string{
		"password", "PASSWORD",
		fmt.Sprintf("%s_PASSWORD", strings.ToUpper(middlewareType)),
		fmt.Sprintf("%s_ROOT_PASSWORD", strings.ToUpper(middlewareType)),
		"REDIS_PASSWORD",
		"MYSQL_ROOT_PASSWORD",
		"POSTGRES_PASSWORD",
		"MONGODB_ROOT_PASSWORD",
	}

	databaseFields := []string{
		"database", "DATABASE", "db", "DB",
		fmt.Sprintf("%s_DATABASE", strings.ToUpper(middlewareType)),
		"MYSQL_DATABASE",
		"POSTGRES_DB",
		"MONGODB_DATABASE",
	}

	// 提取用户名
	for _, field := range usernameFields {
		if val, ok := data[field]; ok {
			username = string(val)
			break
		}
	}

	// 提取密码
	for _, field := range passwordFields {
		if val, ok := data[field]; ok {
			password = string(val)
			break
		}
	}

	// 提取数据库名
	for _, field := range databaseFields {
		if val, ok := data[field]; ok {
			database = string(val)
			break
		}
	}

	// 为不同中间件设置默认用户名
	if username == "" {
		switch middlewareType {
		case "mysql":
			username = "root"
		case "postgresql":
			username = "postgres"
		case "mongodb":
			username = "root"
		case "redis":
			username = "" // Redis 默认无用户名
		}
	}

	return username, password, database
}

// getServiceHost 获取服务的主机地址
func (s *DiscoveryService) getServiceHost(service *corev1.Service) string {
	// 使用集群内 DNS 名称：<service-name>.<namespace>.svc.cluster.local
	return fmt.Sprintf("%s.%s.svc.cluster.local", service.Name, service.Namespace)
}

// getServicePort 获取服务的主端口
func (s *DiscoveryService) getServicePort(service *corev1.Service, middlewareType *MiddlewareType) int32 {
	// 优先返回中间件的标准端口
	for _, port := range service.Spec.Ports {
		for _, mwPort := range middlewareType.Ports {
			if port.Port == mwPort {
				return port.Port
			}
		}
	}

	// 如果没有标准端口，返回第一个端口
	if len(service.Spec.Ports) > 0 {
		return service.Spec.Ports[0].Port
	}

	return 0
}

// DiscoverServices 发现集群中的所有中间件服务
func (s *DiscoveryService) DiscoverServices(ctx context.Context) ([]DiscoveredService, error) {
	// 获取所有服务
	services, err := s.k8sClient.ListAllServices(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to list services: %w", err)
	}

	var discovered []DiscoveredService

	for _, service := range services {
		// 检测中间件类型
		middlewareType := s.detectMiddlewareType(ctx, &service)
		if middlewareType == nil {
			continue // 不是支持的中间件类型
		}

		// 查找关联的 Secret
		secret, err := s.k8sClient.FindSecretForService(ctx, &service)
		hasCredentials := err == nil && secret != nil

		// 提取凭据
		var username, password, database string
		if hasCredentials {
			username, password, database = s.extractCredentials(secret, middlewareType.Name)
		}

		// 构建发现的服务信息
		discovered = append(discovered, DiscoveredService{
			Name:           service.Name,
			Type:           middlewareType.Name,
			Namespace:      service.Namespace,
			Host:           s.getServiceHost(&service),
			Port:           s.getServicePort(&service, middlewareType),
			Username:       username,
			Password:       password,
			Database:       database,
			HasCredentials: hasCredentials && password != "",
		})
	}

	return discovered, nil
}

// ListClustersFromKubeconfig 从 kubeconfig 内容中列出所有集群上下文
func ListClustersFromKubeconfig(kubeconfigContent string) ([]string, error) {
	config, err := clientcmd.Load([]byte(kubeconfigContent))
	if err != nil {
		return nil, fmt.Errorf("failed to load kubeconfig: %w", err)
	}

	var clusters []string
	for name := range config.Contexts {
		clusters = append(clusters, name)
	}

	return clusters, nil
}

