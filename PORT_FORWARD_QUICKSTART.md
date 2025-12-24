# 端口转发功能 - 快速开始

## 🚀 快速部署（3步）

### 1. 更新 RBAC 权限

```bash
kubectl apply -f zeni-x/deploy/k8s/base/rbac.yaml
```

### 2. 重新构建和部署

```bash
cd zeni-x

# 构建镜像
docker build -t zeni-x-backend:latest -f backend/Dockerfile backend/
docker build -t zeni-x-frontend:latest -f frontend/Dockerfile frontend/

# 部署到 Test 环境
kubectl apply -k deploy/k8s/overlays/test
```

### 3. 验证

```bash
kubectl get pods -n zeni-x
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep "Starting port forward monitor"
```

## 💡 快速使用

### 方式一：通过 K8s 服务发现

1. 打开连接管理页面
2. 点击"自动发现" → 上传 kubeconfig → 选择集群 → 扫描
3. 勾选 ClusterIP 服务 → 导入
4. 系统自动创建端口转发 ✨

### 方式二：通过端口转发管理页面

1. 点击"端口转发"按钮
2. 查看所有转发状态
3. 管理转发（重连/停止）

## 📊 监控

```bash
# 查看端口转发日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep -i "port forward"

# 查看健康检查
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep "Health check"

# 查看清理任务
kubectl logs -n zeni-x -l app=zeni-x,component=backend | grep "Cleaning up idle"
```

## 🔧 配置（可选）

编辑 `configs/*.yaml`：

```yaml
port_forward:
  local_port_range:
    min: 40000        # 调整端口范围
    max: 50000
  idle_timeout: 10m   # 调整空闲超时
```

## 📚 更多信息

- 完整部署指南：`PORT_FORWARD_DEPLOYMENT.md`
- 实现总结：`PORT_FORWARD_IMPLEMENTATION_SUMMARY.md`
- 实现计划：`.cursor/plans/port_forward_advanced_baa1aeb6.plan.md`

## ✅ 功能特性

- ✅ 按需创建，自动清理
- ✅ 独立端口管理
- ✅ 健康监控
- ✅ 手动重连
- ✅ 双界面管理

Happy coding! 🎉

