package k8s

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

// Client Kubernetes 客户端封装
type Client struct {
	clientset *kubernetes.Clientset
	config    *rest.Config // 保存配置以供端口转发使用
}

// NewClient 创建 Kubernetes 客户端
// 优先使用 InCluster 配置，如果失败则尝试使用 kubeconfig
func NewClient() (*Client, error) {
	return NewClientWithConfig("", "")
}

// NewClientWithConfig 使用指定的 kubeconfig 内容创建客户端
// kubeconfigContent: kubeconfig 文件内容（YAML 格式）
// context: 可选的上下文名称，如果为空则使用 current-context
// 如果 kubeconfigContent 为空，则使用默认方式（InCluster 或 ~/.kube/config）
func NewClientWithConfig(kubeconfigContent string, context string) (*Client, error) {
	var config *rest.Config
	var err error

	if kubeconfigContent != "" {
		// 使用提供的 kubeconfig 内容
		clientConfig, err := clientcmd.NewClientConfigFromBytes([]byte(kubeconfigContent))
		if err != nil {
			return nil, fmt.Errorf("failed to parse kubeconfig: %w", err)
		}

		// 如果指定了 context，则使用指定的 context
		if context != "" {
			rawConfig, err := clientConfig.RawConfig()
			if err != nil {
				return nil, fmt.Errorf("failed to get raw config: %w", err)
			}

			// 检查 context 是否存在
			if _, ok := rawConfig.Contexts[context]; !ok {
				return nil, fmt.Errorf("context %s not found in kubeconfig", context)
			}

			// 创建带有指定 context 的 client config
			overrides := &clientcmd.ConfigOverrides{
				CurrentContext: context,
			}
			clientConfig = clientcmd.NewNonInteractiveClientConfig(rawConfig, context, overrides, nil)
		}

		config, err = clientConfig.ClientConfig()
		if err != nil {
			return nil, fmt.Errorf("failed to create client config: %w", err)
		}
	} else {
		// 首先尝试 InCluster 配置（在 Pod 内运行时）
		config, err = rest.InClusterConfig()
		if err != nil {
			// 如果 InCluster 失败，尝试从 kubeconfig 加载
			kubeconfig := os.Getenv("KUBECONFIG")
			if kubeconfig == "" {
				homeDir, _ := os.UserHomeDir()
				kubeconfig = filepath.Join(homeDir, ".kube", "config")
			}

			config, err = clientcmd.BuildConfigFromFlags("", kubeconfig)
			if err != nil {
				return nil, fmt.Errorf("failed to create kubernetes config: %w", err)
			}
		}
	}

	clientset, err := kubernetes.NewForConfig(config)
	if err != nil {
		return nil, fmt.Errorf("failed to create kubernetes client: %w", err)
	}

	return &Client{
		clientset: clientset,
		config:    config,
	}, nil
}

// ListClustersFromKubeconfig 从 kubeconfig 内容中列出所有集群
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

// SystemNamespaces 系统命名空间列表，自动发现时排除
var SystemNamespaces = []string{
	"kube-system",
	"kube-public",
	"kube-node-lease",
	"default",
	"ingress-nginx",
	"metallb-system",
	"cert-manager",
	"monitoring",
	"logging",
}

// isSystemNamespace 判断是否为系统命名空间
func isSystemNamespace(ns string) bool {
	for _, sysNs := range SystemNamespaces {
		if ns == sysNs {
			return true
		}
	}
	return false
}

// ListNamespaces 列出所有命名空间（排除系统命名空间）
func (c *Client) ListNamespaces(ctx context.Context) ([]string, error) {
	namespaces, err := c.clientset.CoreV1().Namespaces().List(ctx, metav1.ListOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to list namespaces: %w", err)
	}

	var result []string
	for _, ns := range namespaces.Items {
		if !isSystemNamespace(ns.Name) {
			result = append(result, ns.Name)
		}
	}

	return result, nil
}

// ListServices 列出指定命名空间的所有服务
func (c *Client) ListServices(ctx context.Context, namespace string) ([]corev1.Service, error) {
	services, err := c.clientset.CoreV1().Services(namespace).List(ctx, metav1.ListOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to list services in namespace %s: %w", namespace, err)
	}

	return services.Items, nil
}

// ListAllServices 列出所有命名空间的服务（排除系统命名空间）
func (c *Client) ListAllServices(ctx context.Context) ([]corev1.Service, error) {
	namespaces, err := c.ListNamespaces(ctx)
	if err != nil {
		return nil, err
	}

	var allServices []corev1.Service
	for _, ns := range namespaces {
		services, err := c.ListServices(ctx, ns)
		if err != nil {
			// 记录错误但继续处理其他命名空间
			fmt.Printf("Warning: failed to list services in namespace %s: %v\n", ns, err)
			continue
		}
		allServices = append(allServices, services...)
	}

	return allServices, nil
}

// GetSecret 获取指定命名空间的 Secret
func (c *Client) GetSecret(ctx context.Context, namespace, name string) (*corev1.Secret, error) {
	secret, err := c.clientset.CoreV1().Secrets(namespace).Get(ctx, name, metav1.GetOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to get secret %s/%s: %w", namespace, name, err)
	}

	return secret, nil
}

// ListSecrets 列出指定命名空间的所有 Secrets
func (c *Client) ListSecrets(ctx context.Context, namespace string) ([]corev1.Secret, error) {
	secrets, err := c.clientset.CoreV1().Secrets(namespace).List(ctx, metav1.ListOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to list secrets in namespace %s: %w", namespace, err)
	}

	return secrets.Items, nil
}

// FindSecretForService 查找与服务关联的 Secret
// 搜索策略：
// 1. 查找与服务同名的 Secret
// 2. 查找带有 -secret, -password, -credentials 后缀的 Secret
// 3. 查找带有相同标签的 Secret
func (c *Client) FindSecretForService(ctx context.Context, service *corev1.Service) (*corev1.Secret, error) {
	namespace := service.Namespace
	serviceName := service.Name

	// 策略 1: 查找同名 Secret
	candidateNames := []string{
		serviceName,
		serviceName + "-secret",
		serviceName + "-password",
		serviceName + "-credentials",
		serviceName + "-auth",
	}

	for _, name := range candidateNames {
		secret, err := c.GetSecret(ctx, namespace, name)
		if err == nil {
			return secret, nil
		}
	}

	// 策略 2: 查找所有 Secrets 并匹配标签或名称模式
	secrets, err := c.ListSecrets(ctx, namespace)
	if err != nil {
		return nil, err
	}

	for _, secret := range secrets {
		// 检查名称是否包含服务名
		if strings.Contains(secret.Name, serviceName) {
			return &secret, nil
		}

		// 检查标签是否匹配
		for key, value := range service.Spec.Selector {
			if secret.Labels != nil && secret.Labels[key] == value {
				return &secret, nil
			}
		}
	}

	return nil, fmt.Errorf("no secret found for service %s", serviceName)
}

