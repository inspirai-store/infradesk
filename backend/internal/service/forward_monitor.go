package service

import (
	"fmt"
	"log"
	"net"
	"time"

	"github.com/zeni-x/backend/internal/k8s"
	"github.com/zeni-x/backend/internal/store"
)

// ForwardMonitor 端口转发监控服务
type ForwardMonitor struct {
	manager          *k8s.PortForwardManager
	db               *store.SQLite
	cleanupInterval  time.Duration
	healthInterval   time.Duration
	idleTimeout      time.Duration
	stopChan         chan struct{}
}

// NewForwardMonitor 创建监控服务
func NewForwardMonitor(manager *k8s.PortForwardManager, db *store.SQLite) *ForwardMonitor {
	return &ForwardMonitor{
		manager:         manager,
		db:              db,
		cleanupInterval: 5 * time.Minute,
		healthInterval:  30 * time.Second,
		idleTimeout:     10 * time.Minute,
		stopChan:        make(chan struct{}),
	}
}

// Start 启动监控服务
func (m *ForwardMonitor) Start() {
	log.Println("Starting port forward monitor service")

	// 启动空闲清理任务
	go m.startCleanupTask()

	// 启动健康检查任务
	go m.startHealthCheckTask()
}

// Stop 停止监控服务
func (m *ForwardMonitor) Stop() {
	log.Println("Stopping port forward monitor service")
	close(m.stopChan)
}

// startCleanupTask 启动空闲清理任务
func (m *ForwardMonitor) startCleanupTask() {
	ticker := time.NewTicker(m.cleanupInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			m.cleanupIdle()
		case <-m.stopChan:
			return
		}
	}
}

// startHealthCheckTask 启动健康检查任务
func (m *ForwardMonitor) startHealthCheckTask() {
	ticker := time.NewTicker(m.healthInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			m.healthCheck()
		case <-m.stopChan:
			return
		}
	}
}

// cleanupIdle 清理空闲的端口转发
func (m *ForwardMonitor) cleanupIdle() {
	forwards := m.manager.ListForwards()
	now := time.Now()
	cleaned := 0

	for _, fwd := range forwards {
		if now.Sub(fwd.LastUsedAt) > m.idleTimeout {
			log.Printf("Cleaning up idle forward: %s (service: %s/%s, last used: %v ago)",
				fwd.ID, fwd.Namespace, fwd.ServiceName, now.Sub(fwd.LastUsedAt))

			// 停止转发
			if err := m.manager.StopForward(fwd.ID); err != nil {
				log.Printf("Failed to stop idle forward %s: %v", fwd.ID, err)
				continue
			}

			// 更新数据库中的连接状态
			if fwd.ConnectionID > 0 {
				conn, err := m.db.GetConnectionByID(fwd.ConnectionID)
				if err == nil {
					conn.ForwardID = ""
					conn.ForwardLocalPort = 0
					conn.ForwardStatus = ""
					// 恢复原始服务地址
					conn.Host = fmt.Sprintf("%s.%s.svc.cluster.local", fwd.ServiceName, fwd.Namespace)
					conn.Port = int(fwd.RemotePort)
					m.db.UpdateConnection(conn)
				}
			}

			cleaned++
		}
	}

	if cleaned > 0 {
		log.Printf("Cleaned up %d idle port forward(s)", cleaned)
	}
}

// healthCheck 健康检查所有端口转发
func (m *ForwardMonitor) healthCheck() {
	forwards := m.manager.ListForwards()
	if len(forwards) == 0 {
		return
	}

	errorCount := 0
	recoveredCount := 0

	for _, fwd := range forwards {
		// 尝试连接本地端口
		conn, err := net.DialTimeout("tcp", fmt.Sprintf("localhost:%d", fwd.LocalPort), 2*time.Second)
		
		previousStatus := fwd.Status
		
		if err != nil {
			// 连接失败
			if fwd.Status != k8s.StatusError {
				log.Printf("Port forward health check failed: %s (service: %s/%s, local port: %d): %v",
					fwd.ID, fwd.Namespace, fwd.ServiceName, fwd.LocalPort, err)
				fwd.Status = k8s.StatusError
				fwd.ErrorMessage = fmt.Sprintf("Health check failed: %v", err)
				errorCount++

				// 更新数据库中的连接状态
				if fwd.ConnectionID > 0 {
					if conn, err := m.db.GetConnectionByID(fwd.ConnectionID); err == nil {
						conn.ForwardStatus = string(k8s.StatusError)
						m.db.UpdateConnection(conn)
					}
				}
			}
		} else {
			// 连接成功
			conn.Close()
			
			if fwd.Status == k8s.StatusError {
				log.Printf("Port forward recovered: %s (service: %s/%s, local port: %d)",
					fwd.ID, fwd.Namespace, fwd.ServiceName, fwd.LocalPort)
				fwd.Status = k8s.StatusActive
				fwd.ErrorMessage = ""
				recoveredCount++

				// 更新数据库中的连接状态
				if fwd.ConnectionID > 0 {
					if conn, err := m.db.GetConnectionByID(fwd.ConnectionID); err == nil {
						conn.ForwardStatus = string(k8s.StatusActive)
						m.db.UpdateConnection(conn)
					}
				}
			}
		}

		// 检查是否空闲
		if time.Since(fwd.LastUsedAt) > 5*time.Minute && fwd.Status == k8s.StatusActive && previousStatus == k8s.StatusActive {
			fwd.Status = k8s.StatusIdle
		}
	}

	if errorCount > 0 || recoveredCount > 0 {
		log.Printf("Health check completed: %d error(s), %d recovered", errorCount, recoveredCount)
	}
}

// GetStats 获取统计信息
func (m *ForwardMonitor) GetStats() map[string]int {
	forwards := m.manager.ListForwards()
	stats := map[string]int{
		"total":  len(forwards),
		"active": 0,
		"error":  0,
		"idle":   0,
	}

	for _, fwd := range forwards {
		switch fwd.Status {
		case k8s.StatusActive:
			stats["active"]++
		case k8s.StatusError:
			stats["error"]++
		case k8s.StatusIdle:
			stats["idle"]++
		}
	}

	return stats
}

