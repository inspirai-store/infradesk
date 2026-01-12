# Zeni-X Helm Chart

Zeni-X 是一个现代化的数据库管理平台，提供友好的 Web 界面用于管理和操作各种数据库。

## 简介

本 Helm Chart 用于在 Kubernetes 集群上部署 Zeni-X 应用。Zeni-X 采用前后端分离架构：
- **前端**: Vue 3 + TypeScript + Vite (Nginx 静态文件服务)
- **后端**: Go + Gin 框架

## 前置要求

- Kubernetes 1.19+
- Helm 3.0+
- PV provisioner (用于持久化存储)

## 安装

### 1. 添加 Chart 仓库（如果适用）

```bash
# 如果 Chart 已发布到仓库
helm repo add zeni-x https://charts.yunizeni.com
helm repo update
```

### 2. 创建敏感配置文件

在每个环境部署前，需要创建对应的敏感配置文件：

```bash
# 复制示例文件
cp helm/zeni-x/values-test.secret.example helm/zeni-x/values-test.secret.yaml

# 编辑并填入实际的敏感信息
vim helm/zeni-x/values-test.secret.yaml
```

### 3. 部署到指定环境

```bash
# 测试环境
make test

# UAT 环境
make uat

# 生产环境
make prod
```

或使用 Helm 直接部署：

```bash
# 部署到测试环境
helm upgrade --install zeni-x-test ./helm/zeni-x \
  --namespace zeni-x-test \
  --create-namespace \
  --values helm/zeni-x/values-test.yaml \
  --values helm/zeni-x/values-test.secret.yaml

# 部署到 UAT 环境
helm upgrade --install zeni-x-uat ./helm/zeni-x \
  --namespace zeni-x-uat \
  --create-namespace \
  --values helm/zeni-x/values-uat.yaml \
  --values helm/zeni-x/values-uat.secret.yaml

# 部署到生产环境
helm upgrade --install zeni-x-prod ./helm/zeni-x \
  --namespace zeni-x-prod \
  --create-namespace \
  --values helm/zeni-x/values-prod.yaml \
  --values helm/zeni-x/values-prod.secret.yaml
```

## 配置

### 环境特定配置

每个环境有独立的 values 文件：

| 文件 | 说明 |
|------|------|
| `values.yaml` | 默认配置（所有环境通用） |
| `values-dev.yaml` | 开发环境配置 |
| `values-test.yaml` | 测试环境配置 |
| `values-uat.yaml` | UAT 环境配置 |
| `values-prod.yaml` | 生产环境配置 |
| `values-{env}.secret.yaml` | 环境敏感配置（不提交到 Git） |

### 主要配置参数

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `namespace` | Kubernetes 命名空间 | `zeni-x` |
| `global.imageRegistry` | 镜像仓库地址 | `registry.local` |
| `global.environment` | 环境标识 | `dev` |
| `frontend.enabled` | 是否启用前端容器 | `true` |
| `frontend.image.repository` | 前端镜像名称 | `zeni-x-frontend` |
| `frontend.image.tag` | 前端镜像标签 | `latest` |
| `backend.enabled` | 是否启用后端容器 | `true` |
| `backend.image.repository` | 后端镜像名称 | `zeni-x-backend` |
| `backend.image.tag` | 后端镜像标签 | `latest` |
| `service.type` | Service 类型 | `NodePort` |
| `service.frontend.nodePort` | 前端 NodePort | `30080` |
| `service.backend.nodePort` | 后端 NodePort | `30088` |
| `ingress.enabled` | 是否启用 Ingress | `true` |
| `ingress.className` | Ingress Class | `traefik` |
| `persistence.enabled` | 是否启用持久化存储 | `true` |
| `persistence.size` | PVC 存储大小 | `1Gi` |

## 升级

```bash
# 测试环境
helm upgrade zeni-x-test ./helm/zeni-x \
  --namespace zeni-x-test \
  --values helm/zeni-x/values-test.yaml \
  --values helm/zeni-x/values-test.secret.yaml

# UAT 环境
helm upgrade zeni-x-uat ./helm/zeni-x \
  --namespace zeni-x-uat \
  --values helm/zeni-x/values-uat.yaml \
  --values helm/zeni-x/values-uat.secret.yaml
```

## 卸载

```bash
# 测试环境
helm uninstall zeni-x-test --namespace zeni-x-test

# UAT 环境
helm uninstall zeni-x-uat --namespace zeni-x-uat

# 生产环境
helm uninstall zeni-x-prod --namespace zeni-x-prod
```

或使用 Makefile：

```bash
make clean-k8s-test
make clean-k8s-uat
```

## 故障排查

### 查看 Pod 状态

```bash
kubectl get pods -n zeni-x-test
kubectl describe pod <pod-name> -n zeni-x-test
```

### 查看日志

```bash
# 前端日志
kubectl logs deployment/zeni-x -c frontend -n zeni-x-test

# 后端日志
kubectl logs deployment/zeni-x -c backend -n zeni-x-test
```

### 进入容器调试

```bash
# 前端容器
kubectl exec -it deployment/zeni-x -c frontend -n zeni-x-test -- /bin/sh

# 后端容器
kubectl exec -it deployment/zeni-x -c backend -n zeni-x-test -- /bin/sh
```

## 常见问题

**Q: 部署失败，提示 secret 文件不存在？**

A: 请确保已创建对应环境的 secret 文件：
```bash
cp helm/zeni-x/values-test.secret.example helm/zeni-x/values-test.secret.yaml
# 编辑并填入实际值
vim helm/zeni-x/values-test.secret.yaml
```

**Q: 如何修改 NodePort？**

A: 编辑对应环境的 values 文件，修改 `service.frontend.nodePort` 和 `service.backend.nodePort`。

**Q: 如何配置 Ingress 域名？**

A: 编辑对应环境的 values 文件，修改 `ingress.hosts[0].host`。

## 相关链接

- [Zeni-X 项目主页](https://github.com/inspirai/InterComponent)
- [Issue 跟踪](https://github.com/inspirai/InterComponent/issues)
