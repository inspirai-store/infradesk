# Kubeconfig 格式说明

## 基本结构

Kubeconfig 是 YAML 格式的配置文件，包含以下主要部分：

### 1. 完整示例

```yaml
apiVersion: v1
kind: Config
preferences: {}

# 集群列表
clusters:
- name: docker-desktop
  cluster:
    server: https://kubernetes.docker.internal:6443
    certificate-authority-data: LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t...
    # 或者使用文件路径
    # certificate-authority: /path/to/ca.crt

- name: production-cluster
  cluster:
    server: https://prod-k8s.example.com:6443
    certificate-authority-data: LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t...
    # 跳过 TLS 验证（不推荐用于生产）
    # insecure-skip-tls-verify: true

# 用户凭据列表
users:
- name: docker-desktop
  user:
    # 方式 1: 客户端证书认证
    client-certificate-data: LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t...
    client-key-data: LS0tLS1CRUdJTiBSU0EgUFJJVkFURSBLRVktLS0tLQ==...

- name: admin-user
  user:
    # 方式 2: Token 认证
    token: eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9.eyJpc3MiOiJrdWJlcm5ldGVz...

- name: service-account
  user:
    # 方式 3: 用户名密码（Basic Auth，已弃用）
    username: admin
    password: secret

- name: oidc-user
  user:
    # 方式 4: OIDC 认证
    auth-provider:
      name: oidc
      config:
        client-id: kubernetes
        client-secret: secret
        id-token: eyJhbGciOiJSUzI1NiIsImtpZCI6IiJ9...
        idp-issuer-url: https://accounts.google.com
        refresh-token: refresh-token-value

# 上下文列表（集群 + 用户 + 命名空间）
contexts:
- name: docker-desktop
  context:
    cluster: docker-desktop
    user: docker-desktop
    namespace: default

- name: production
  context:
    cluster: production-cluster
    user: admin-user
    namespace: production

- name: dev-environment
  context:
    cluster: production-cluster
    user: admin-user
    namespace: development

# 当前激活的上下文
current-context: docker-desktop
```

## 字段说明

### Clusters 部分
| 字段 | 说明 | 必需 |
|------|------|------|
| name | 集群名称（标识符） | ✅ |
| server | K8s API Server 地址 | ✅ |
| certificate-authority-data | Base64 编码的 CA 证书 | ❌ |
| certificate-authority | CA 证书文件路径 | ❌ |
| insecure-skip-tls-verify | 跳过 TLS 验证（不安全） | ❌ |

### Users 部分
| 字段 | 说明 | 必需 |
|------|------|------|
| name | 用户名称（标识符） | ✅ |
| client-certificate-data | Base64 编码的客户端证书 | ❌ |
| client-key-data | Base64 编码的客户端私钥 | ❌ |
| token | Bearer Token | ❌ |
| username | Basic Auth 用户名 | ❌ |
| password | Basic Auth 密码 | ❌ |

### Contexts 部分
| 字段 | 说明 | 必需 |
|------|------|------|
| name | 上下文名称 | ✅ |
| cluster | 引用的集群名称 | ✅ |
| user | 引用的用户名称 | ✅ |
| namespace | 默认命名空间 | ❌ |

## 如何获取 Kubeconfig

### 方法 1: 查看默认配置
```bash
# 显示当前 kubeconfig
cat ~/.kube/config

# 或使用 kubectl
kubectl config view
```

### 方法 2: 查看合并后的配置（包含所有来源）
```bash
kubectl config view --merge --flatten
```

### 方法 3: 从集群管理员获取
管理员可以为你生成专用的 kubeconfig：
```bash
# 生成 ServiceAccount 的 kubeconfig
kubectl create serviceaccount my-user
kubectl create clusterrolebinding my-user-binding \
  --clusterrole=view \
  --serviceaccount=default:my-user

# 获取 token 并生成 kubeconfig
```

### 方法 4: 云服务商提供
- **阿里云 ACK**: 容器服务控制台 → 集群 → 连接信息
- **腾讯云 TKE**: 集群管理 → 基本信息 → kubeconfig
- **AWS EKS**: `aws eks update-kubeconfig --name cluster-name`
- **GKE**: `gcloud container clusters get-credentials cluster-name`
- **Azure AKS**: `az aks get-credentials --name cluster-name --resource-group rg-name`

## 常用操作

### 查看当前上下文
```bash
kubectl config current-context
```

### 列出所有上下文
```bash
kubectl config get-contexts
```

### 切换上下文
```bash
kubectl config use-context docker-desktop
```

### 查看集群信息
```bash
kubectl cluster-info
```

### 验证连接
```bash
kubectl get nodes
```

## Zeni-X 使用场景

### 本地开发使用默认配置
如果你已经配置了 `~/.kube/config`，直接点击"扫描集群"即可。

### 上传自定义 Kubeconfig
1. 准备 kubeconfig 文件（可以是完整的 `~/.kube/config`）
2. 在 Zeni-X 中点击"上传 Kubeconfig"
3. 选择文件上传
4. 点击"扫描集群"

### 多集群场景
如果你的 kubeconfig 包含多个上下文：
```yaml
contexts:
  - name: dev-cluster
  - name: staging-cluster
  - name: prod-cluster
current-context: dev-cluster
```

系统会使用 `current-context` 指定的集群。如果需要切换，有两种方式：
1. 修改 kubeconfig 的 `current-context` 后重新上传
2. 使用 `kubectl config use-context` 切换后复制新的配置

## 安全最佳实践

### 1. 使用只读权限
为 Zeni-X 创建专用的只读 ServiceAccount：
```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: zeni-x-reader
  namespace: default
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: zeni-x-reader
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: view  # 只读权限
subjects:
- kind: ServiceAccount
  name: zeni-x-reader
  namespace: default
```

### 2. 最小权限原则
不要使用 `cluster-admin` 权限的 kubeconfig，只授予必需的权限：
- `get` 和 `list` Services
- `get` 和 `list` Secrets（仅用于读取凭据）
- `get` 和 `list` Namespaces

### 3. 临时凭据
使用有时效性的 token 而非长期证书：
```yaml
user:
  token: <short-lived-token>
```

### 4. 不要提交到版本控制
在 `.gitignore` 中添加：
```
*.kubeconfig
kubeconfig
.kube/
```

## 故障排查

### 问题：证书验证失败
```
x509: certificate signed by unknown authority
```
**解决**：检查 `certificate-authority-data` 是否正确，或临时使用 `insecure-skip-tls-verify: true`（仅测试）

### 问题：连接超时
```
Unable to connect to the server: dial tcp: i/o timeout
```
**解决**：检查 `server` 地址是否可访问，网络连接是否正常

### 问题：权限不足
```
User "system:serviceaccount:default:my-sa" cannot list services
```
**解决**：检查 RBAC 配置，确保用户有足够权限

### 问题：Token 过期
```
Unable to authenticate the request due to an error: invalid bearer token
```
**解决**：刷新 token 或重新获取 kubeconfig

## 示例文件

### Docker Desktop 的 kubeconfig
位置：`~/.kube/config`

### Minikube 的 kubeconfig  
位置：`~/.kube/config` 或 `~/.minikube/config`

### Kind 的 kubeconfig
```bash
kind get kubeconfig --name my-cluster > kind-config.yaml
```

### K3s 的 kubeconfig
位置：`/etc/rancher/k3s/k3s.yaml`（需要 root 权限）

## 更多资源

- [Kubernetes 官方文档 - kubeconfig](https://kubernetes.io/docs/concepts/configuration/organize-cluster-access-kubeconfig/)
- [kubectl 配置文档](https://kubernetes.io/docs/reference/kubectl/cheatsheet/#kubectl-context-and-configuration)

