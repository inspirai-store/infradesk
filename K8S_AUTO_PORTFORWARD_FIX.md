# K8s 连接自动端口转发修复

## 问题描述

当从 Kubernetes 集群导入连接配置时，连接使用的是集群内部 DNS 名称（如 `game-mysql.backup.svc.cluster.local`），本地应用无法解析这些域名，导致连接失败：

```
Failed to load resource: the server responded with a status of 500 (Internal Server Error)
{"error":"dial tcp: lookup game-mysql.backup.svc.cluster.local: no such host"}
```

## 解决方案

### 1. 数据库结构更新

在 `connections` 表中添加了以下字段来存储 K8s 服务信息：

- `k8s_namespace`: K8s 命名空间
- `k8s_service_name`: K8s 服务名称
- `k8s_service_port`: K8s 服务端口

### 2. 导入逻辑修改

修改了 `ImportConnections` API（`k8s.go`），在导入 K8s 服务时：

- 使用 `localhost` 作为 Host 占位符
- Port 设为 0（将在端口转发时分配）
- 保存完整的 K8s 服务信息（namespace, service name, port）
- 标记 `forward_status` 为 `pending`

### 3. 自动端口转发

修改了 `MySQLHandler` 和 `RedisHandler` 的 `getConnection` 方法：

- 检查连接是否包含 K8s 服务信息
- 如果包含，自动创建或重用端口转发
- 更新连接信息为本地转发地址（localhost:随机端口）

## 工作流程

1. **导入阶段**：
   ```
   K8s Service Discovery -> Import -> 保存为 localhost:0 + K8s 信息
   ```

2. **首次访问**：
   ```
   前端请求 -> getConnection() -> 检测需要端口转发 -> 
   创建端口转发 -> 更新连接为 localhost:随机端口 -> 返回连接
   ```

3. **后续访问**：
   ```
   前端请求 -> getConnection() -> 检测到活跃的端口转发 -> 
   更新使用时间 -> 直接返回连接
   ```

## 代码修改清单

### 后端修改

1. **store/sqlite.go**:
   - 添加 K8s 相关字段到 `Connection` 结构体
   - 更新所有 SQL 查询以包含新字段
   - 添加数据库迁移

2. **api/k8s.go**:
   - 修改 `ImportServiceItem` 添加 `ServiceName` 字段
   - 修改 `ImportConnections` 使用 localhost 并保存 K8s 信息
   - 更新存在性检查基于 namespace + service name

3. **api/mysql.go** 和 **api/redis.go**:
   - 添加 `pfManager` 字段到 handler
   - 修改 `getConnection()` 实现自动端口转发
   - 检查并维护端口转发生命周期

4. **api/router.go**:
   - 将 `pfManager` 传递给 MySQL 和 Redis handlers

## 使用说明

### 导入 K8s 服务

1. 在前端使用 K8s 服务发现功能
2. 选择要导入的服务
3. 点击导入（可选择 force_override 模式）

### 访问数据库

1. 在连接列表中选择导入的 K8s 连接
2. 首次访问时会自动创建端口转发（约 3-5 秒）
3. 后续访问会重用现有的端口转发

### 端口转发管理

端口转发会自动管理：
- **创建**: 首次访问时自动创建
- **维护**: 每次访问更新最后使用时间
- **清理**: 空闲超时后自动清理（由 ForwardMonitor 负责）
- **重连**: 如果端口转发失败，下次访问会自动重建

## 注意事项

1. **超时设置**: 端口转发创建有 30 秒超时
2. **错误处理**: 如果端口转发失败，会返回详细错误信息
3. **兼容性**: 非 K8s 连接不受影响，继续使用原有逻辑
4. **性能**: 端口转发创建是异步的，不会阻塞其他操作

## 故障排查

### 端口转发失败

1. 检查 kubeconfig 配置是否正确
2. 确认有足够的 K8s 权限
3. 查看后端日志：`.dev/backend.log`
4. 尝试手动创建端口转发测试

### 连接超时

1. 首次访问可能需要 3-5 秒建立端口转发
2. 如果超过 30 秒仍未连接，检查 K8s 集群状态
3. 尝试删除并重新导入连接

### 数据库迁移问题

如果遇到数据库字段缺失错误：
```bash
rm -f backend/data/zeni-x.db
# 重启应用，会自动创建新的数据库
```

## 测试验证

启动应用后测试：

```bash
# 1. 启动服务
make dev-start

# 2. 访问前端
open http://localhost:15073

# 3. 导入 K8s 服务
# 4. 选择导入的连接并访问数据库
# 5. 检查后端日志确认端口转发创建
tail -f .dev/backend.log | grep "port forward"
```

## 相关文档

- [K8S_DISCOVERY_DEPLOYMENT.md](./K8S_DISCOVERY_DEPLOYMENT.md) - K8s 服务发现部署
- [PORT_FORWARD_IMPLEMENTATION_SUMMARY.md](./PORT_FORWARD_IMPLEMENTATION_SUMMARY.md) - 端口转发实现
- [K8S_IMPORT_FORCE_OVERRIDE.md](./K8S_IMPORT_FORCE_OVERRIDE.md) - 强制覆盖导入功能

