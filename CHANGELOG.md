# Zeni-X 更新日志

## 2024-12-24 - 多连接配置支持

### 主要功能
- ✅ 支持每种数据库类型配置多个不同的连接
- ✅ 用户可以自定义添加、编辑、删除连接配置
- ✅ 连接配置存储在 SQLite 数据库中
- ✅ 支持连接测试功能
- ✅ 支持在不同连接之间快速切换
- ✅ 活动连接状态保存到 localStorage

### 后端修改

#### 1. Store 层 (`internal/store/sqlite.go`)
- 新增 `GetConnectionByID()` - 获取单个连接（包含密码）
- 新增 `UpdateConnection()` - 更新连接配置
- 新增 `DeleteConnection()` - 删除连接配置
- 新增 `GetConnectionsByType()` - 按类型获取所有连接
- 修改 `Connection.Password` JSON 标签从 `json:"-"` 改为 `json:"password,omitempty"`，允许接收密码

#### 2. Service 层
**`internal/service/mysql.go`**
- 移除了对全局 `config.Config` 的依赖
- `connect()` 方法改为接受 `*store.Connection` 参数
- 所有公共方法改为接受 `*store.Connection` 作为第一个参数
- 新增 `TestConnection()` 方法用于测试连接

**`internal/service/redis.go`**
- 移除了对全局 `config.Config` 的依赖
- `connect()` 方法改为接受 `*store.Connection` 参数
- 所有公共方法改为接受 `*store.Connection` 作为第一个参数
- 新增 `TestConnection()` 方法用于测试连接
- Redis DB 索引从 `Connection.DatabaseName` 字段解析

#### 3. API 层
**`internal/api/mysql.go` & `internal/api/redis.go`**
- 新增 `getConnection()` 辅助方法，从 `X-Connection-ID` 请求头获取连接配置
- 所有 handler 方法改为动态获取连接配置

**`internal/api/router.go`**
- 新增 `POST /api/connections/test` - 测试连接
- 新增 `GET /api/connections/:id` - 获取单个连接
- 新增 `PUT /api/connections/:id` - 更新连接
- 新增 `DELETE /api/connections/:id` - 删除连接
- 新增 `GET /api/connections/types/:type` - 按类型获取连接列表
- 修复路由顺序：具体路径（如 `/connections/test`）在动态路由（如 `/connections/:id`）之前注册
- 所有返回连接信息的 API 自动清空密码字段以保护安全
- 更新连接时，如果密码为空则保留原密码

#### 4. 配置文件
**`configs/dev.yaml`**
- 将所有 `localhost` 改为 `127.0.0.1`，避免 MySQL Unix socket 连接导致的权限问题

### 前端修改

#### 1. Pinia Store (`src/stores/connections.ts`)
- 新增 `useConnectionStore` 状态管理
- 管理所有可用连接列表
- 管理当前活动连接 ID
- 提供连接获取和切换功能

#### 2. API 层 (`src/api/index.ts`)
- 新增 Axios 请求拦截器，自动添加 `X-Connection-ID` 请求头
- 新增 `systemApi.getConnection()` - 获取单个连接
- 新增 `systemApi.updateConnection()` - 更新连接
- 新增 `systemApi.deleteConnection()` - 删除连接
- 新增 `systemApi.testConnection()` - 测试连接
- 新增 `systemApi.getConnectionsByType()` - 按类型获取连接

#### 3. 新增页面和组件

**`src/views/connections/ConnectionsView.vue`**
- 连接管理主页面
- 按类型分组显示所有连接
- 支持添加、编辑、删除连接
- 支持测试连接功能
- 显示连接详细信息（主机、端口、用户名等）

**`src/components/ConnectionSelector.vue`**
- 可复用的连接选择器组件
- 下拉菜单选择活动连接
- 快速测试当前连接
- 快速跳转到连接管理页面
- 自动保存选择到 localStorage
- 自动加载默认连接或第一个可用连接

#### 4. 路由更新 (`src/router/index.ts`)
- 新增 `/connections` 路由，指向连接管理页面

#### 5. 布局更新 (`src/components/AppLayout.vue`)
- 侧边栏新增"连接管理"菜单项

#### 6. 视图更新
**`src/views/mysql/MySQLView.vue`**
- 集成 `ConnectionSelector` 组件
- 监听活动连接变化，自动刷新数据
- 无活动连接时清空数据显示

**`src/views/redis/RedisView.vue`**
- 集成 `ConnectionSelector` 组件
- 监听活动连接变化，自动刷新数据
- 无活动连接时清空数据显示

### 技术亮点

1. **解耦设计**：服务层与全局配置解耦，实现真正的多连接支持
2. **安全性**：
   - 密码在返回给前端时自动清空
   - 更新连接时，空密码不会覆盖原密码
   - 密码存储在 SQLite 中（生产环境建议加密）
3. **用户体验**：
   - 连接状态持久化到 localStorage
   - 自动选择默认连接或第一个可用连接
   - 快速连接切换和测试
   - 实时显示连接状态（在线/离线）
4. **架构改进**：
   - 使用 `X-Connection-ID` 请求头传递连接信息
   - 前端 Axios 拦截器自动注入请求头
   - 后端统一的连接获取逻辑

### 网络配置说明

**为什么使用 `127.0.0.1` 而不是 `localhost`？**

在 MySQL 客户端中：
- `localhost` 会优先尝试 Unix socket 连接（`/tmp/mysql.sock` 或 `/var/run/mysqld/mysqld.sock`）
- `127.0.0.1` 强制使用 TCP/IP 连接

MySQL 权限系统中，`root@localhost` 和 `root@127.0.0.1` 是不同的用户，可能有不同的权限配置。使用 `127.0.0.1` 可以：
- 确保使用 TCP 连接，权限更可控
- 避免 Unix socket 文件权限问题
- 统一连接方式，便于调试

### 下一步建议

1. **安全增强**：
   - 实现密码加密存储
   - 添加连接权限管理
   - 实现会话超时机制

2. **功能扩展**：
   - 支持 SSH 隧道连接
   - 支持 SSL/TLS 加密连接
   - 支持连接分组和标签
   - 支持连接导入/导出

3. **用户体验**：
   - 添加连接最近使用记录
   - 添加连接搜索和过滤
   - 添加连接健康检查
   - 添加连接性能监控

4. **完善其他数据源**：
   - MongoDB 多连接支持
   - MinIO 多连接支持

