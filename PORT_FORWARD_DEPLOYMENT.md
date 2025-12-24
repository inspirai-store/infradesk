# 端口转发高级方案 - 部署指南

## 概述

端口转发功能已完全实现，支持：
- ✅ 按需创建端口转发（使用时创建，空闲自动清理）
- ✅ 独立转发（每个连接有独立的本地端口）
- ✅ 手动重连（断开时显示错误，用户手动重连）
- ✅ 双界面管理（连接页面 + 独立管理页面）
- ✅ 健康检查和自动清理

## 部署步骤

### 1. 更新 RBAC 权限

端口转发功能需要访问 Pod 资源。首先更新 RBAC 权限：

```bash
# 应用更新的 RBAC 配置
kubectl apply -f zeni-x/deploy/k8s/base/rbac.yaml
```

验证 ServiceAccount 权限：

```bash
kubectl auth can-i list pods --as=system:serviceaccount:zeni-x:zeni-x-reader
kubectl auth can-i get pods/portforward --as=system:serviceaccount:zeni-x:zeni-x-reader
```

### 2. 重新构建镜像

由于添加了新的后端功能，需要重新构建镜像：

```bash
cd zeni-x

# 构建后端镜像
docker build -t zeni-x-backend:latest -f backend/Dockerfile backend/

# 构建前端镜像
docker build -t zeni-x-frontend:latest -f frontend/Dockerfile frontend/

# 如果使用远程仓库，推送镜像
# docker tag zeni-x-backend:latest your-registry/zeni-x-backend:latest
# docker push your-registry/zeni-x-backend:latest
# docker tag zeni-x-frontend:latest your-registry/zeni-x-frontend:latest
# docker push your-registry/zeni-x-frontend:latest
```

### 3. 部署到集群

```bash
# Test 环境
kubectl apply -k zeni-x/deploy/k8s/overlays/test

# UAT 环境
kubectl apply -k zeni-x/deploy/k8s/overlays/uat
```

### 4. 验证部署

```bash
# 检查 Pod 状态
kubectl get pods -n zeni-x

# 查看后端日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend -f

# 检查端口转发监控服务是否启动
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep "Starting port forward monitor"
```

## 使用指南

### 通过 K8s 服务发现

1. **打开连接管理页面**
   - 点击"自动发现"按钮
   - 选择或上传 kubeconfig 文件
   - 选择目标集群
   - 点击"扫描集群"

2. **导入 ClusterIP 服务**
   - 扫描完成后，系统会自动标识 ClusterIP 服务
   - 勾选需要导入的服务
   - 点击"导入选中的服务"
   - 系统会自动创建端口转发（如果需要）

3. **查看端口转发状态**
   - 在连接卡片上会显示端口转发状态指示器
   - 绿色：活跃
   - 红色：错误
   - 黄色：空闲

### 通过端口转发管理页面

1. **访问管理页面**
   - 在连接管理页面点击"端口转发"按钮
   - 或直接访问 `/port-forward` 路由

2. **查看所有转发**
   - 查看转发ID、服务地址、本地端口、状态等信息
   - 实时显示统计信息（总计、活跃、错误、空闲）

3. **管理转发**
   - **重连**：如果转发出现错误，点击"重连"按钮
   - **停止**：点击"停止"按钮终止端口转发

### API 端点

端口转发功能提供以下 API 端点：

```bash
# 创建端口转发
POST /api/port-forward
{
  "connection_id": 1,
  "namespace": "dev-services",
  "service_name": "mysql",
  "remote_port": 3306
}

# 列出所有转发
GET /api/port-forward

# 获取单个转发状态
GET /api/port-forward/:id

# 通过连接ID查询
GET /api/port-forward/by-connection?connection_id=1

# 重新连接
POST /api/port-forward/:id/reconnect

# 停止转发
DELETE /api/port-forward/:id

# 更新使用时间
PUT /api/port-forward/:id/touch
```

## 配置说明

端口转发相关配置在 `configs/*.yaml` 文件中：

```yaml
port_forward:
  enabled: true                    # 是否启用端口转发
  local_port_range:
    min: 40000                     # 本地端口范围起始
    max: 50000                     # 本地端口范围结束
  idle_timeout: 10m                # 空闲超时时间
  health_check_interval: 30s       # 健康检查间隔
  cleanup_interval: 5m             # 清理任务间隔
```

## 监控和维护

### 后台任务

系统会自动启动两个后台任务：

1. **健康检查任务**（每30秒）
   - 检查所有端口转发是否正常
   - 自动标记失败的转发
   - 更新数据库中的连接状态

2. **空闲清理任务**（每5分钟）
   - 检查超过10分钟未使用的转发
   - 自动停止空闲转发
   - 释放端口资源

### 日志监控

```bash
# 查看端口转发相关日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep -i "port forward"

# 查看健康检查日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep "Health check"

# 查看清理日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep "Cleaning up idle"
```

## 故障排查

### 端口转发创建失败

**可能原因**：
1. 没有 Pod 资源权限
2. Service 没有对应的 Pod
3. Pod 不在运行状态
4. 端口范围已耗尽

**解决方法**：
```bash
# 检查 RBAC 权限
kubectl auth can-i list pods --as=system:serviceaccount:zeni-x:zeni-x-reader
kubectl auth can-i get pods/portforward --as=system:serviceaccount:zeni-x:zeni-x-reader

# 检查 Service 对应的 Pod
kubectl get pods -n <namespace> -l <service-selector>

# 检查后端日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend | tail -100
```

### 端口转发状态显示错误

**可能原因**：
1. Pod 重启导致连接断开
2. 网络问题
3. 目标服务不可用

**解决方法**：
1. 在连接卡片上点击"重连"按钮
2. 或在端口转发管理页面点击"重连"
3. 如果持续失败，检查目标服务状态

### 端口被占用

**可能原因**：
- 本地端口范围内的端口被其他进程占用

**解决方法**：
- 修改配置文件中的 `local_port_range`
- 重新部署应用

## 性能考虑

### 端口资源

- 默认端口范围：40000-50000（共10000个端口）
- 建议根据实际需求调整范围
- 如果端口不足，会返回错误

### 内存和网络

- 每个端口转发占用少量内存（通常<10MB）
- 实际流量由应用使用情况决定
- 空闲转发会被自动清理，释放资源

### 并发连接

- 系统支持同时维护大量端口转发
- 建议监控资源使用情况
- 可根据需要调整清理间隔和空闲超时

## 安全建议

1. **端口范围**
   - 使用高端口号（40000+）避免与系统服务冲突
   - 限制端口范围大小防止资源耗尽

2. **权限控制**
   - ServiceAccount 只有必要的 Pod 访问权限
   - 端口转发只绑定到 localhost

3. **审计日志**
   - 所有端口转发操作都有日志记录
   - 定期检查日志发现异常行为

4. **资源限制**
   - 设置合理的空闲超时避免长期占用
   - 监控活跃转发数量

## 已知限制

1. **集群访问**
   - 只支持从集群内部或通过 kubeconfig 访问
   - 不支持集群外直接访问 ClusterIP

2. **Pod 选择**
   - 自动选择 Service 关联的第一个运行中的 Pod
   - 如果 Pod 重启，需要手动重连

3. **并发限制**
   - 端口数量受配置的端口范围限制
   - 默认最多10000个同时转发

## 更新日志

### Version 1.0.0
- ✅ 实现端口转发管理器核心功能
- ✅ 实现端口转发 API 端点
- ✅ 更新数据库 schema
- ✅ 实现后台监控服务
- ✅ 实现前端 API 客户端
- ✅ 创建端口转发状态组件
- ✅ 创建端口转发管理页面
- ✅ 集成到连接管理页面
- ✅ 更新 RBAC 权限
- ✅ 添加配置项支持

## 技术支持

如有问题，请检查：
1. 后端日志
2. RBAC 权限配置
3. Service 和 Pod 状态
4. 网络连接

或查阅：
- `port_forward_advanced_baa1aeb6.plan.md` - 实现计划
- 后端代码：`backend/internal/k8s/portforward.go`
- 前端组件：`frontend/src/components/PortForwardStatus.vue`
- 管理页面：`frontend/src/views/PortForwardView.vue`

