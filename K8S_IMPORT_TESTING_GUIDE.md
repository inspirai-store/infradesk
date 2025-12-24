# 强制覆盖功能 - 测试指南

## 问题诊断

如果您看到：
```
14 个连接导入失败: game-mysql: connection already exists, ...
```

说明代码可能还没有重新编译部署。

## 验证步骤

### 1. 检查代码版本

查看后端日志，应该能看到类似的日志：

```bash
# 跳过重复连接时
Skipped existing connection: dev-services/mysql

# 强制覆盖时
Updated existing connection: dev-services/mysql (ID: 123)
```

### 2. 前端测试步骤

**步骤 1：扫描集群**
```
打开连接管理 → 点击"自动发现"
上传 kubeconfig → 选择集群 → 点击"扫描集群"
```

**步骤 2：选择要导入的服务**
```
勾选已存在的服务（如 mysql, redis 等）
点击"导入选中的服务"
```

**步骤 3：观察对话框**
```
应该弹出：
┌─────────────────────────────┐
│ ⚠️  发现重复连接             │
│                             │
│ 以下 14 个连接已存在：      │
│ game-mysql                  │
│ mysql                       │
│ redis                       │
│ ...                         │
│                             │
│ 是否要强制覆盖这些已有连接？ │
│                             │
│  [取消]     [强制覆盖]      │
└─────────────────────────────┘
```

**如果没有弹出对话框**，说明前端代码还没有更新。

### 3. 本地测试（推荐）

如果在部署环境中测试不方便，可以在本地测试：

```bash
# 1. 启动后端
cd zeni-x/backend
go run cmd/server/main.go

# 2. 在另一个终端启动前端
cd zeni-x/frontend
npm run dev

# 3. 在浏览器打开
http://localhost:5173
```

### 4. 验证对话框功能

**测试场景 A：点击"取消"**
```
1. 弹出对话框后点击"取消"
2. 应该看到提示：
   "跳过 14 个重复连接"
3. 不会有错误信息
```

**测试场景 B：点击"强制覆盖"**
```
1. 弹出对话框后点击"强制覆盖"
2. 应该看到提示：
   "成功导入连接：覆盖 14 个"
3. 连接信息会被更新
```

## 部署更新

如果测试时仍然看到旧的错误信息，需要重新部署：

### 方式一：重新构建镜像（生产环境）

```bash
cd zeni-x

# 1. 构建前端
cd frontend
npm run build

# 2. 构建镜像
cd ..
docker build -t zeni-x-frontend:latest -f frontend/Dockerfile frontend/
docker build -t zeni-x-backend:latest -f backend/Dockerfile backend/

# 3. 推送到镜像仓库（如果使用）
docker tag zeni-x-frontend:latest your-registry/zeni-x-frontend:latest
docker push your-registry/zeni-x-frontend:latest

docker tag zeni-x-backend:latest your-registry/zeni-x-backend:latest
docker push your-registry/zeni-x-backend:latest

# 4. 重新部署
kubectl rollout restart deployment/zeni-x-frontend -n zeni-x
kubectl rollout restart deployment/zeni-x-backend -n zeni-x

# 5. 等待部署完成
kubectl rollout status deployment/zeni-x-frontend -n zeni-x
kubectl rollout status deployment/zeni-x-backend -n zeni-x
```

### 方式二：本地端口转发（测试环境）

```bash
# 1. 转发后端端口
kubectl port-forward -n zeni-x svc/zeni-x-backend 15080:8080

# 2. 在本地启动前端开发服务器
cd frontend
npm run dev

# 3. 访问 http://localhost:5173
```

## 调试方法

### 1. 检查浏览器控制台

打开浏览器开发者工具（F12），查看：

**Network 标签页**：
```
查找 /api/k8s/import 请求
检查 Request Payload 是否包含：
{
  "services": [...],
  "force_override": false  // 第一次应该是 false
}

检查 Response 是否包含：
{
  "success": 0,
  "failed": 0,
  "updated": 0,
  "skipped": 14,  // 应该有这个字段
  "results": [...]
}
```

**Console 标签页**：
```
看是否有 JavaScript 错误
```

### 2. 检查后端日志

```bash
# 查看后端日志
kubectl logs -n zeni-x -l app=zeni-x,component=backend -f --tail=50

# 应该看到类似的日志：
# Skipped existing connection: dev-services/mysql
# Skipped existing connection: dev-services/redis
```

### 3. 检查 API 响应

使用 curl 测试 API：

```bash
# 测试导入（不强制覆盖）
curl -X POST http://localhost:15080/api/k8s/import \
  -H "Content-Type: application/json" \
  -d '{
    "services": [
      {
        "name": "mysql",
        "type": "mysql",
        "namespace": "dev-services",
        "host": "mysql.dev-services.svc.cluster.local",
        "port": 3306
      }
    ],
    "force_override": false
  }'

# 应该返回类似：
# {
#   "success": 0,
#   "failed": 0,
#   "updated": 0,
#   "skipped": 1,
#   "results": [
#     {
#       "name": "mysql",
#       "success": false,
#       "skipped": true,
#       "error": "connection already exists"
#     }
#   ]
# }
```

## 常见问题

### Q1: 对话框没有弹出
**A**: 可能前端代码还没有更新。检查：
1. 浏览器是否有缓存（Ctrl+Shift+R 强制刷新）
2. 前端镜像是否重新构建
3. Pod 是否重启完成

### Q2: 仍然看到 "connection already exists" 错误
**A**: 这是正常的！重点是：
- ❌ 如果显示为 "导入失败" → 代码未更新
- ✅ 如果弹出确认对话框 → 代码已更新

### Q3: 点击"强制覆盖"后仍然失败
**A**: 检查：
1. 后端日志是否有错误
2. API 响应中的 `force_override` 是否为 `true`
3. 数据库连接是否正常

## 成功标志

功能正常工作的标志：

✅ **第一次导入时**
- 不会立即显示错误信息
- 弹出确认对话框
- 列出所有重复的连接

✅ **点击"取消"后**
- 显示 "跳过 X 个重复连接"
- 不会有错误消息

✅ **点击"强制覆盖"后**
- 显示 "成功导入连接：覆盖 X 个"
- 连接列表中的信息已更新

## 下一步

如果按照以上步骤测试后仍有问题，请提供：

1. 浏览器控制台截图（Console 和 Network 标签页）
2. 后端日志（最近 50 行）
3. API 请求和响应的完整内容

这样我可以帮助您进一步诊断问题。

