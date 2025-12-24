package k8s

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"sync"
	"time"

	"github.com/google/uuid"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/tools/portforward"
	"k8s.io/client-go/transport/spdy"
)

// ForwardStatus 端口转发状态
type ForwardStatus string

const (
	StatusActive ForwardStatus = "active"
	StatusError  ForwardStatus = "error"
	StatusIdle   ForwardStatus = "idle"
)

// PortForward 单个端口转发实例
type PortForward struct {
	ID           string        `json:"id"`
	ConnectionID int64         `json:"connection_id"`
	ServiceName  string        `json:"service_name"`
	Namespace    string        `json:"namespace"`
	RemotePort   int32         `json:"remote_port"`
	LocalPort    int           `json:"local_port"`
	Status       ForwardStatus `json:"status"`
	CreatedAt    time.Time     `json:"created_at"`
	LastUsedAt   time.Time     `json:"last_used_at"`
	ErrorMessage string        `json:"error_message,omitempty"`
	StopChan     chan struct{} `json:"-"`
	ReadyChan    chan struct{} `json:"-"`
}

// PortForwardManager 管理端口转发
type PortForwardManager struct {
	client       *Client
	forwards     map[string]*PortForward // key: forward ID
	mu           sync.RWMutex
	localPortMin int
	localPortMax int
	idleTimeout  time.Duration
	usedPorts    map[int]bool
}

// NewPortForwardManager 创建端口转发管理器
func NewPortForwardManager(client *Client) *PortForwardManager {
	return &PortForwardManager{
		client:       client,
		forwards:     make(map[string]*PortForward),
		localPortMin: 40000,
		localPortMax: 50000,
		idleTimeout:  10 * time.Minute,
		usedPorts:    make(map[int]bool),
	}
}

// CreateForward 为指定服务创建端口转发
func (m *PortForwardManager) CreateForward(ctx context.Context, connectionID int64, namespace, serviceName string, remotePort int32) (*PortForward, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	// 检查是否已存在相同的转发
	for _, fwd := range m.forwards {
		if fwd.ConnectionID == connectionID && fwd.Namespace == namespace && 
		   fwd.ServiceName == serviceName && fwd.RemotePort == remotePort {
			// 更新最后使用时间
			fwd.LastUsedAt = time.Now()
			return fwd, nil
		}
	}

	// 分配本地端口
	localPort, err := m.findAvailablePort()
	if err != nil {
		return nil, fmt.Errorf("failed to find available port: %w", err)
	}

	// 获取服务对应的 Pod
	pods, err := m.getPodsForService(ctx, namespace, serviceName)
	if err != nil || len(pods) == 0 {
		return nil, fmt.Errorf("no pods found for service %s/%s: %w", namespace, serviceName, err)
	}

	// 选择第一个运行中的 Pod
	var podName string
	for _, pod := range pods {
		if pod.Status.Phase == corev1.PodRunning {
			podName = pod.Name
			break
		}
	}
	if podName == "" {
		return nil, fmt.Errorf("no running pods found for service %s/%s", namespace, serviceName)
	}

	// 创建转发实例
	forward := &PortForward{
		ID:           uuid.New().String(),
		ConnectionID: connectionID,
		ServiceName:  serviceName,
		Namespace:    namespace,
		RemotePort:   remotePort,
		LocalPort:    localPort,
		Status:       StatusActive,
		CreatedAt:    time.Now(),
		LastUsedAt:   time.Now(),
		StopChan:     make(chan struct{}, 1),
		ReadyChan:    make(chan struct{}),
	}

	// 启动端口转发
	go func() {
		if err := m.startPortForward(ctx, podName, forward); err != nil {
			m.mu.Lock()
			forward.Status = StatusError
			forward.ErrorMessage = err.Error()
			m.mu.Unlock()
		}
	}()

	// 等待就绪或超时
	select {
	case <-forward.ReadyChan:
		// 端口转发已就绪
	case <-time.After(10 * time.Second):
		return nil, fmt.Errorf("timeout waiting for port forward to be ready")
	}

	// 标记端口为已使用
	m.usedPorts[localPort] = true
	m.forwards[forward.ID] = forward

	return forward, nil
}

// startPortForward 启动端口转发
func (m *PortForwardManager) startPortForward(ctx context.Context, podName string, forward *PortForward) error {
	// 构建 port-forward 请求
	req := m.client.clientset.CoreV1().RESTClient().Post().
		Resource("pods").
		Namespace(forward.Namespace).
		Name(podName).
		SubResource("portforward")

	transport, upgrader, err := spdy.RoundTripperFor(m.client.config)
	if err != nil {
		return fmt.Errorf("failed to create round tripper: %w", err)
	}

	dialer := spdy.NewDialer(upgrader, &http.Client{Transport: transport}, "POST", req.URL())

	ports := []string{fmt.Sprintf("%d:%d", forward.LocalPort, forward.RemotePort)}

	pf, err := portforward.New(dialer, ports, forward.StopChan, forward.ReadyChan, nil, nil)
	if err != nil {
		return fmt.Errorf("failed to create port forwarder: %w", err)
	}

	// ForwardPorts 会阻塞直到 StopChan 关闭
	if err := pf.ForwardPorts(); err != nil {
		return fmt.Errorf("port forward failed: %w", err)
	}

	return nil
}

// GetForward 获取现有的端口转发
func (m *PortForwardManager) GetForward(id string) (*PortForward, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	forward, exists := m.forwards[id]
	if !exists {
		return nil, fmt.Errorf("forward not found: %s", id)
	}

	return forward, nil
}

// GetForwardByConnectionID 通过连接ID获取端口转发
func (m *PortForwardManager) GetForwardByConnectionID(connectionID int64) (*PortForward, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, forward := range m.forwards {
		if forward.ConnectionID == connectionID {
			return forward, nil
		}
	}

	return nil, fmt.Errorf("no forward found for connection ID: %d", connectionID)
}

// StopForward 停止端口转发
func (m *PortForwardManager) StopForward(id string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	forward, exists := m.forwards[id]
	if !exists {
		return fmt.Errorf("forward not found: %s", id)
	}

	// 关闭停止通道
	close(forward.StopChan)

	// 释放端口
	delete(m.usedPorts, forward.LocalPort)

	// 从映射中删除
	delete(m.forwards, id)

	return nil
}

// ListForwards 列出所有转发
func (m *PortForwardManager) ListForwards() []*PortForward {
	m.mu.RLock()
	defer m.mu.RUnlock()

	forwards := make([]*PortForward, 0, len(m.forwards))
	for _, forward := range m.forwards {
		forwards = append(forwards, forward)
	}

	return forwards
}

// UpdateLastUsed 更新最后使用时间
func (m *PortForwardManager) UpdateLastUsed(id string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	forward, exists := m.forwards[id]
	if !exists {
		return fmt.Errorf("forward not found: %s", id)
	}

	forward.LastUsedAt = time.Now()
	return nil
}

// CleanupIdle 清理空闲的端口转发
func (m *PortForwardManager) CleanupIdle() int {
	m.mu.Lock()
	defer m.mu.Unlock()

	cleaned := 0
	now := time.Now()

	for id, forward := range m.forwards {
		if now.Sub(forward.LastUsedAt) > m.idleTimeout {
			close(forward.StopChan)
			delete(m.usedPorts, forward.LocalPort)
			delete(m.forwards, id)
			cleaned++
		}
	}

	return cleaned
}

// HealthCheck 健康检查所有端口转发
func (m *PortForwardManager) HealthCheck() {
	m.mu.Lock()
	defer m.mu.Unlock()

	for _, forward := range m.forwards {
		// 尝试连接本地端口
		conn, err := net.DialTimeout("tcp", fmt.Sprintf("localhost:%d", forward.LocalPort), 2*time.Second)
		if err != nil {
			if forward.Status != StatusError {
				forward.Status = StatusError
				forward.ErrorMessage = fmt.Sprintf("Health check failed: %v", err)
			}
		} else {
			conn.Close()
			if forward.Status == StatusError {
				forward.Status = StatusActive
				forward.ErrorMessage = ""
			}
		}
	}
}

// findAvailablePort 找到可用的本地端口
func (m *PortForwardManager) findAvailablePort() (int, error) {
	for port := m.localPortMin; port <= m.localPortMax; port++ {
		// 检查是否已被管理器使用
		if m.usedPorts[port] {
			continue
		}

		// 检查端口是否被系统占用
		listener, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
		if err == nil {
			listener.Close()
			return port, nil
		}
	}

	return 0, fmt.Errorf("no available ports in range %d-%d", m.localPortMin, m.localPortMax)
}

// getPodsForService 获取服务对应的 Pods
func (m *PortForwardManager) getPodsForService(ctx context.Context, namespace, serviceName string) ([]corev1.Pod, error) {
	// 获取 Service
	svc, err := m.client.clientset.CoreV1().Services(namespace).Get(ctx, serviceName, metav1.GetOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to get service: %w", err)
	}

	// 使用 selector 查找 Pods
	labelSelector := metav1.FormatLabelSelector(&metav1.LabelSelector{
		MatchLabels: svc.Spec.Selector,
	})

	pods, err := m.client.clientset.CoreV1().Pods(namespace).List(ctx, metav1.ListOptions{
		LabelSelector: labelSelector,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to list pods: %w", err)
	}

	return pods.Items, nil
}

// Reconnect 重新连接端口转发
func (m *PortForwardManager) Reconnect(ctx context.Context, id string) (*PortForward, error) {
	m.mu.Lock()
	
	forward, exists := m.forwards[id]
	if !exists {
		m.mu.Unlock()
		return nil, fmt.Errorf("forward not found: %s", id)
	}

	// 保存连接信息
	connectionID := forward.ConnectionID
	namespace := forward.Namespace
	serviceName := forward.ServiceName
	remotePort := forward.RemotePort
	
	// 停止旧的转发
	close(forward.StopChan)
	delete(m.usedPorts, forward.LocalPort)
	delete(m.forwards, id)
	
	m.mu.Unlock()

	// 创建新的转发
	return m.CreateForward(ctx, connectionID, namespace, serviceName, remotePort)
}

