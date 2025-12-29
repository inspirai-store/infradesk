# Zeni-X Helm Chart

Zeni-X 是一个现代化的数据库管理平台，用于管理 K8s 集群中的 MySQL、Redis、MongoDB 和 MinIO 服务。

## 快速开始

### 前置要求

- Kubernetes 1.19+
- Helm 3.0+

### 安装 Chart

```bash
# 添加 Helm repository (如果已发布)
helm repo add zeni-x https://charts.yunizeni.com
helm repo update

# 安装到 dev 环境
helm install zeni-x zeni-x/zeni-x --values values-dev.yaml --namespace zeni-x-dev --create-namespace

# 从本地目录安装
helm install zeni-x ./helm/zeni-x --values helm/zeni-x/values-dev.yaml --namespace zeni-x-dev --create-namespace
```

### 升级 Chart

```bash
helm upgrade zeni-x ./helm/zeni-x --values helm/zeni-x/values-dev.yaml --namespace zeni-x-dev
```

### 卸载 Chart

```bash
helm uninstall zeni-x --namespace zeni-x-dev
```

## 配置参数

### 全局配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `global.environment` | 环境名称 | `dev` |
| `global.clusterType` | 集群类型 (dev/prod) | `dev` |

### 镜像配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `image.frontend.repository` | Frontend 镜像仓库 | `registry.local/zeni-x-frontend` |
| `image.frontend.tag` | Frontend 镜像标签 | `latest` |
| `image.backend.repository` | Backend 镜像仓库 | `registry.local/zeni-x-backend` |
| `image.backend.tag` | Backend 镜像标签 | `latest` |

### Namespace 配置

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `namespace.name` | Namespace 名称 | `zeni-x` |
| `namespace.create` | 是否创建 Namespace | `true` |

更多配置参数请参考 `values.yaml` 文件。

## 环境部署

### Dev 环境

```bash
helm install zeni-x ./helm/zeni-x \
  --values helm/zeni-x/values-dev.yaml \
  --namespace zeni-x-dev \
  --create-namespace
```

### Test 环境

```bash
helm install zeni-x ./helm/zeni-x \
  --values helm/zeni-x/values-test.yaml \
  --namespace zeni-x-test \
  --create-namespace
```

### UAT 环境

```bash
helm install zeni-x ./helm/zeni-x \
  --values helm/zeni-x/values-uat.yaml \
  --namespace zeni-x-uat \
  --create-namespace
```

### Prod 环境

```bash
helm install zeni-x ./helm/zeni-x \
  --values helm/zeni-x/values-prod.yaml \
  --namespace zeni-x-prod \
  --create-namespace
```

## 使用 Makefile

项目提供了 Makefile 快捷命令：

```bash
# Lint Chart
make helm-lint

# 模板渲染
make helm-template

# 部署到 dev 环境
make helm-dev

# 部署到 test 环境
make helm-test

# 部署到 uat 环境
make helm-uat

# 部署到 prod 环境
make helm-prod
```

## 从 Kustomize 迁移到 Helm

### 使用 Kustomize (旧方式)

```bash
kubectl apply -k k8s/overlays/test
```

### 使用 Helm (新方式)

```bash
helm install zeni-x ./helm/zeni-x \
  --values helm/zeni-x/values-test.yaml \
  --namespace zeni-x-test \
  --create-namespace
```

两种方式的部署结果是等价的。Helm 提供了更好的版本管理和回滚能力。

## 故障排查

### 查看 Pod 状态

```bash
kubectl get pods -n zeni-x-dev
kubectl describe pod <pod-name> -n zeni-x-dev
kubectl logs <pod-name> -n zeni-x-dev
```

### 查看 Helm Release 状态

```bash
helm status zeni-x -n zeni-x-dev
helm history zeni-x -n zeni-x-dev
```

### 回滚到上一版本

```bash
helm rollback zeni-x -n zeni-x-dev
```

### 渲染模板查看生成的 YAML

```bash
helm template zeni-x ./helm/zeni-x \
  --values helm/zeni-x/values-dev.yaml \
  --namespace zeni-x-dev
```

## License

MIT
