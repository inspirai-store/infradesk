# K8s 服务发现 - 新功能使用指南

## 🎯 新增功能

### 1. **可取消的扫描**
- 扫描过程中可以随时点击"取消扫描"按钮中止操作
- 避免长时间等待，提升用户体验

### 2. **Kubeconfig 上传支持**
- 本地测试时可以上传 kubeconfig 文件
- 支持选择不同的集群进行扫描
- 不再依赖默认的集群配置

## 📖 使用方法

### 方式一：使用默认集群配置

适用场景：
- 应用部署在 K8s 集群内
- 本地开发环境已配置 `~/.kube/config`

步骤：
1. 点击"自动发现"按钮
2. 点击"扫描集群"
3. 等待扫描完成或点击"取消扫描"
4. 选择要导入的服务
5. 点击"导入选中的服务"

### 方式二：上传 Kubeconfig

适用场景：
- 本地测试不同集群
- 需要临时连接到其他集群
- 使用自定义的 kubeconfig

步骤：
1. 点击"自动发现"按钮
2. 看到提示："在本地测试时，可以上传 kubeconfig 文件..."
3. 点击"上传 Kubeconfig"按钮
4. 选择你的 kubeconfig 文件（通常是 `~/.kube/config`）
5. 上传成功后显示"已上传 Kubeconfig"标签
6. 点击"扫描集群"开始扫描
7. 扫描完成后选择要导入的服务

### 取消扫描

如果扫描时间过长或发现误操作：
1. 在扫描过程中点击"取消扫描"按钮
2. 系统会中止当前扫描
3. 可以重新配置后再次扫描

## 🔧 API 变更

### 1. 发现服务 API

**之前：**
```http
GET /api/k8s/discover
```

**现在：**
```http
POST /api/k8s/discover
Content-Type: application/json

{
  "kubeconfig": "可选的 kubeconfig 内容"
}
```

**示例：使用默认配置**
```bash
curl -X POST http://localhost:15080/api/k8s/discover \
  -H "Content-Type: application/json" \
  -d '{}'
```

**示例：使用自定义 kubeconfig**
```bash
curl -X POST http://localhost:15080/api/k8s/discover \
  -H "Content-Type: application/json" \
  -d "{\"kubeconfig\": \"$(cat ~/.kube/config)\"}"
```

### 2. 列出集群 API（新增）

```http
POST /api/k8s/clusters
Content-Type: application/json

{
  "kubeconfig": "必需的 kubeconfig 内容"
}
```

**响应：**
```json
{
  "clusters": [
    "docker-desktop",
    "minikube",
    "production-cluster"
  ]
}
```

**示例：**
```bash
curl -X POST http://localhost:15080/api/k8s/clusters \
  -H "Content-Type: application/json" \
  -d "{\"kubeconfig\": \"$(cat ~/.kube/config)\"}"
```

## 🎨 前端组件变化

### ServiceDiscovery.vue 组件

**新增功能：**
1. Kubeconfig 上传区域
   - 文件上传按钮
   - 上传状态显示
   - 清除按钮

2. 可取消扫描
   - AbortController 支持
   - "取消扫描"按钮
   - 取消后的友好提示

3. 改进的错误处理
   - 区分取消操作和真实错误
   - 针对不同错误场景的提示信息

## 🧪 测试场景

### 测试 1: 默认配置扫描
```bash
# 前置条件：已配置 ~/.kube/config
# 操作：直接点击"扫描集群"
# 预期：成功扫描当前集群的服务
```

### 测试 2: 上传 Kubeconfig
```bash
# 操作：
# 1. 点击"上传 Kubeconfig"
# 2. 选择 kubeconfig 文件
# 3. 点击"扫描集群"
# 预期：使用上传的配置扫描集群
```

### 测试 3: 取消扫描
```bash
# 操作：
# 1. 开始扫描
# 2. 立即点击"取消扫描"
# 预期：扫描中止，显示"扫描已取消"
```

### 测试 4: 切换集群
```bash
# 操作：
# 1. 上传包含多个集群的 kubeconfig
# 2. 扫描一个集群
# 3. 点击"清除"按钮
# 4. 重新上传不同集群的 kubeconfig
# 5. 再次扫描
# 预期：扫描不同集群的服务
```

## 📝 技术实现细节

### 后端变化

1. **K8s Client** (`internal/k8s/client.go`)
   - 新增 `NewClientWithConfig()` 方法
   - 支持从字符串内容创建客户端
   - 新增 `ListClustersFromKubeconfig()` 方法

2. **Discovery Service** (`internal/service/discovery.go`)
   - 新增 `NewDiscoveryServiceWithConfig()` 方法
   - 支持临时创建带自定义配置的服务

3. **API Handler** (`internal/api/k8s.go`)
   - 发现 API 从 GET 改为 POST
   - 支持接收 kubeconfig 参数
   - 新增 `/api/k8s/clusters` 端点

### 前端变化

1. **API Client** (`api/index.ts`)
   - discover 方法支持 kubeconfig 参数
   - 支持 AbortSignal 用于取消请求
   - 新增 listClusters 方法

2. **组件** (`components/ServiceDiscovery.vue`)
   - 集成文件上传功能
   - 实现 AbortController 取消机制
   - 改进 UI 显示和交互

## 🔒 安全考虑

1. **Kubeconfig 处理**
   - Kubeconfig 内容仅在内存中临时使用
   - 不会持久化到数据库或磁盘
   - 每次扫描完成后即释放

2. **权限控制**
   - 上传的 kubeconfig 需要有相应的集群访问权限
   - 建议使用只读权限的 kubeconfig
   - 避免使用 cluster-admin 权限

3. **错误处理**
   - 无效的 kubeconfig 会返回友好的错误信息
   - 不会暴露敏感的集群信息

## 💡 最佳实践

1. **本地开发**
   - 使用上传 kubeconfig 的方式
   - 方便切换不同的测试集群

2. **生产环境**
   - 使用默认配置（InCluster 或 ServiceAccount）
   - 确保 RBAC 权限正确配置

3. **大型集群**
   - 扫描可能需要较长时间（30-60秒）
   - 可以随时取消并调整策略
   - 建议在非高峰时段扫描

## 🐛 故障排查

### 问题：上传 kubeconfig 后仍然报错

**解决方法：**
1. 检查 kubeconfig 格式是否正确
2. 确认 kubeconfig 中的集群地址可访问
3. 验证证书和认证信息是否有效

### 问题：取消后再次扫描失败

**解决方法：**
1. 等待几秒后重试
2. 刷新页面重新打开弹窗
3. 检查后端日志确认没有残留连接

### 问题：扫描超时

**解决方法：**
1. 点击"取消扫描"
2. 考虑缩小扫描范围（未来功能）
3. 增加网络超时时间（目前为 60 秒）

