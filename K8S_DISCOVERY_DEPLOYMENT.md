# K8s 服务自动发现功能 - 部署说明

## 功能概述

zeni-x 现在支持自动发现 Kubernetes 集群中的中间件服务，包括：
- MySQL
- PostgreSQL  
- Redis
- MongoDB
- MinIO

系统会自动识别这些服务并尝试从相关的 Secrets 中提取凭据信息，支持一键批量导入。

## 部署步骤

### 1. 更新 Go 依赖

在 `zeni-x/backend` 目录下运行：

```bash
cd zeni-x/backend
go mod tidy
go mod download
```

这会下载必需的 Kubernetes 客户端库。

### 2. 部署 RBAC 资源

RBAC 资源定义了只读权限的 ServiceAccount，用于访问集群资源：

```bash
# 应用 RBAC 配置
kubectl apply -f zeni-x/deploy/k8s/base/rbac.yaml

# 验证 ServiceAccount 和 ClusterRole
kubectl get serviceaccount zeni-x-reader -n zeni-x
kubectl get clusterrole zeni-x-reader
kubectl get clusterrolebinding zeni-x-reader
```

### 3. 重新构建镜像

后端增加了新的依赖和代码，前端增加了新的组件，需要重新构建镜像：

```bash
cd zeni-x

# 构建前端镜像
docker build -t registry.local/zeni-x-frontend:latest -f frontend/Dockerfile frontend/

# 构建后端镜像  
docker build -t registry.local/zeni-x-backend:latest -f backend/Dockerfile backend/

# 推送到镜像仓库（如果使用）
docker push registry.local/zeni-x-frontend:latest
docker push registry.local/zeni-x-backend:latest
```

### 4. 部署/更新应用

使用 Kustomize 部署更新的应用：

```bash
# 应用基础配置
kubectl apply -k zeni-x/deploy/k8s/base/

# 或应用特定环境
kubectl apply -k zeni-x/deploy/k8s/overlays/test/
# kubectl apply -k zeni-x/deploy/k8s/overlays/uat/
```

### 5. 验证部署

```bash
# 检查 Pod 状态
kubectl get pods -n zeni-x

# 查看 Pod 日志（特别是后端日志）
kubectl logs -n zeni-x -l app=zeni-x -c backend

# 检查 ServiceAccount 挂载
kubectl describe pod -n zeni-x -l app=zeni-x
```

## 使用方法

### 通过 UI 使用

1. 访问 zeni-x Web 界面
2. 进入 "连接管理" 页面
3. 点击 "自动发现" 按钮
4. 系统会扫描集群并列出发现的中间件服务
5. 选择需要导入的服务（有凭据的服务会自动选中）
6. 点击 "导入选中的服务" 完成批量导入

### 通过 API 使用

发现服务：
```bash
curl http://<zeni-x-host>/api/k8s/discover
```

导入服务：
```bash
curl -X POST http://<zeni-x-host>/api/k8s/import \
  -H "Content-Type: application/json" \
  -d '{
    "services": [
      {
        "name": "mysql",
        "type": "mysql",
        "namespace": "dev-services",
        "host": "mysql.dev-services.svc.cluster.local",
        "port": 3306,
        "username": "root",
        "password": "***"
      }
    ]
  }'
```

## 权限说明

自动发现功能使用的 `zeni-x-reader` ServiceAccount 拥有以下只读权限：

- **Services**: 读取集群中的所有服务
- **Endpoints**: 读取服务端点信息
- **Namespaces**: 读取命名空间列表（用于过滤）
- **Secrets**: 读取密钥信息（用于提取凭据）
- **Pods**: 读取 Pod 信息（用于获取容器镜像辅助识别）

所有权限都是只读的（`get`, `list`），不会修改任何集群资源。

## 安全考虑

1. **最小权限原则**: ClusterRole 仅授予必需的只读权限
2. **命名空间过滤**: 自动排除系统命名空间（kube-system, kube-public 等）
3. **密码保护**: 
   - 密码在前端仅用于创建连接，不会显示
   - 导入后密码存储在 SQLite 数据库中
   - API 返回连接列表时会自动清空密码字段

## 故障排查

### 问题: "K8s discovery service disabled" 日志

**原因**: 后端无法连接到 Kubernetes API

**解决方法**:
1. 检查 Pod 是否使用了正确的 ServiceAccount
   ```bash
   kubectl get pod -n zeni-x -o yaml | grep serviceAccountName
   ```
2. 检查 RBAC 权限是否正确配置
   ```bash
   kubectl auth can-i list services --as=system:serviceaccount:zeni-x:zeni-x-reader
   ```
3. 查看详细日志
   ```bash
   kubectl logs -n zeni-x -l app=zeni-x -c backend --tail=100
   ```

### 问题: 发现不到预期的服务

**可能原因**:
1. 服务端口不匹配标准端口
2. 服务名称不包含识别关键字
3. 服务位于被排除的系统命名空间中

**解决方法**:
- 查看服务定义确认端口和名称
- 如果服务使用非标准端口，可能需要手动创建连接

### 问题: 无法获取 Secret 凭据

**可能原因**:
1. Secret 不存在或名称不匹配
2. Secret 字段名与预期不符
3. ServiceAccount 没有读取 Secret 的权限

**解决方法**:
- 检查 Secret 是否存在及字段名
  ```bash
  kubectl get secret <secret-name> -n <namespace> -o yaml
  ```
- 验证权限
  ```bash
  kubectl auth can-i get secrets --as=system:serviceaccount:zeni-x:zeni-x-reader -n <namespace>
  ```

## 开发测试

在本地开发环境测试（使用 kubeconfig）：

```bash
# 设置 KUBECONFIG 环境变量
export KUBECONFIG=~/.kube/config

# 运行后端
cd zeni-x/backend
go run cmd/server/main.go

# 测试发现 API
curl http://localhost:8080/api/k8s/discover
```

## 文件清单

新增文件：
- `zeni-x/deploy/k8s/base/rbac.yaml` - RBAC 资源定义
- `zeni-x/backend/internal/k8s/client.go` - K8s 客户端封装
- `zeni-x/backend/internal/service/discovery.go` - 服务发现逻辑
- `zeni-x/backend/internal/api/k8s.go` - K8s API 处理器
- `zeni-x/frontend/src/components/ServiceDiscovery.vue` - 服务发现 UI 组件

修改文件：
- `zeni-x/backend/go.mod` - 添加 K8s 依赖
- `zeni-x/backend/internal/api/router.go` - 添加 K8s API 路由
- `zeni-x/frontend/src/api/index.ts` - 添加 K8s API 接口
- `zeni-x/frontend/src/views/connections/ConnectionsView.vue` - 集成自动发现功能
- `zeni-x/deploy/k8s/base/deployment.yaml` - 添加 ServiceAccount 配置
- `zeni-x/deploy/k8s/base/kustomization.yaml` - 添加 rbac.yaml 资源

## 后续改进

可能的功能增强：
1. 支持更多中间件类型（Kafka, Elasticsearch 等）
2. 支持自定义识别规则配置
3. 支持测试连接后再导入
4. 支持导入前预览和编辑连接信息
5. 支持定期自动扫描和更新

