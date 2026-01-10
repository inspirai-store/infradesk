# Implementation Plan

## Phase 1: 前端 API 接口层

- [ ] 1. 定义 API 接口类型
  - [ ] 1.1 创建 `src/api/types.ts` 定义适配器接口
    - 定义 `IConnectionApi` 接口，包含 getAll, getById, create, update, delete, test, getByType 方法
    - 定义 `IMysqlApi` 接口，包含现有 mysqlApi 的所有方法
    - 定义 `IRedisApi` 接口，包含现有 redisApi 的所有方法
    - 定义 `IApiAdapter` 统一接口
    - _Requirements: 1.1, 1.4_

  - [ ] 1.2 编写接口类型测试
    - 创建 `src/api/__tests__/types.test.ts`
    - 验证接口定义与现有 API 返回类型兼容
    - _Requirements: 8.1_

## Phase 2: HTTP 适配器

- [ ] 2. 实现 HTTP 适配器
  - [ ] 2.1 创建 HTTP Connection 适配器
    - 创建 `src/api/adapters/http/connection.ts`
    - 实现 `IConnectionApi` 接口，封装现有 axios 调用
    - 保持 X-Connection-ID header 注入逻辑
    - _Requirements: 2.1, 2.2, 2.4_

  - [ ] 2.2 编写 HTTP Connection 适配器测试
    - 创建 `src/api/adapters/http/__tests__/connection.test.ts`
    - Mock axios，测试所有方法
    - 测试错误处理逻辑
    - _Requirements: 8.2_

  - [ ] 2.3 创建 HTTP MySQL 适配器
    - 创建 `src/api/adapters/http/mysql.ts`
    - 实现 `IMysqlApi` 接口
    - _Requirements: 2.1, 2.2_

  - [ ] 2.4 创建 HTTP Redis 适配器
    - 创建 `src/api/adapters/http/redis.ts`
    - 实现 `IRedisApi` 接口
    - _Requirements: 2.1, 2.2_

  - [ ] 2.5 创建 HTTP 适配器工厂
    - 创建 `src/api/adapters/http/index.ts`
    - 导出 `createHttpAdapter()` 函数
    - _Requirements: 2.1_

## Phase 3: 适配器工厂

- [ ] 3. 实现适配器工厂和 API 导出
  - [ ] 3.1 创建适配器工厂
    - 创建 `src/api/adapter.ts`
    - 实现 `createApiAdapter()` 函数
    - 根据 `isTauri()` 选择适配器
    - 支持模块级别的 IPC 启用配置
    - _Requirements: 1.1, 7.1, 7.2_

  - [ ] 3.2 编写适配器工厂测试
    - 创建 `src/api/__tests__/adapter.test.ts`
    - Mock `isTauri()` 测试不同环境下的适配器选择
    - 测试增量迁移配置
    - _Requirements: 8.2, 8.3_

  - [ ] 3.3 重构 API 导出
    - 修改 `src/api/index.ts`
    - 使用适配器工厂创建 API 对象
    - 保持现有导出接口不变（connectionApi, mysqlApi 等）
    - _Requirements: 1.3_

  - [ ] 3.4 验证现有功能
    - 运行前端测试 `npm run test`
    - 启动应用验证 Web 模式正常工作
    - _Requirements: 2.4_

## Phase 4: Rust 基础设施

- [ ] 4. 搭建 Rust 后端基础架构
  - [ ] 4.1 创建错误处理模块
    - 创建 `src-tauri/src/error.rs`
    - 定义 `AppError` 枚举
    - 实现 `Into<tauri::InvokeError>`
    - _Requirements: 3.4, 5.5, 6.5_

  - [ ] 4.2 创建数据模型
    - 创建 `src-tauri/src/db/models.rs`
    - 定义 `Connection` 结构体
    - 定义 `TestConnectionResult` 结构体
    - _Requirements: 4.1_

  - [ ] 4.3 实现 SQLite 数据库层
    - 创建 `src-tauri/src/db/sqlite.rs`
    - 实现数据库初始化和连接池
    - 创建 connections 表 schema
    - _Requirements: 4.1, 4.2_

  - [ ] 4.4 编写数据库层测试
    - 在 `sqlite.rs` 中添加测试模块
    - 使用 tempfile 创建临时数据库
    - 测试 CRUD 操作
    - _Requirements: 8.4_

## Phase 5: Rust 连接管理服务

- [ ] 5. 实现连接管理服务
  - [ ] 5.1 实现 keyring 密码存储
    - 创建 `src-tauri/src/services/keyring.rs`
    - 实现 `save_password`, `get_password`, `delete_password`
    - 使用 connection id 作为 key
    - _Requirements: 4.6_

  - [ ] 5.2 编写 keyring 服务测试
    - 测试密码保存、获取、删除
    - _Requirements: 8.4_

  - [ ] 5.3 实现 ConnectionService
    - 创建 `src-tauri/src/services/connection.rs`
    - 实现 `get_all`, `get_by_id`, `create`, `update`, `delete` 方法
    - 实现 `test_connection` 方法（MySQL 和 Redis 连接测试）
    - 密码通过 keyring 存取
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [ ] 5.4 编写 ConnectionService 测试
    - 测试所有 CRUD 方法
    - 测试连接测试逻辑
    - _Requirements: 8.4_

## Phase 6: Rust Tauri Commands

- [ ] 6. 实现 Tauri 连接管理命令
  - [ ] 6.1 创建连接管理命令
    - 创建 `src-tauri/src/commands/connection.rs`
    - 实现 `get_all_connections`, `get_connection`, `create_connection`, `update_connection`, `delete_connection`, `test_connection` 命令
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [ ] 6.2 注册 Tauri 命令
    - 修改 `src-tauri/src/lib.rs`
    - 初始化 SQLite 数据库
    - 注册所有连接管理命令
    - _Requirements: 3.2_

  - [ ] 6.3 编写集成测试
    - 测试命令注册和调用
    - _Requirements: 8.4_

## Phase 7: IPC 适配器

- [ ] 7. 实现 IPC Connection 适配器
  - [ ] 7.1 创建 IPC Connection 适配器
    - 创建 `src/api/adapters/ipc/connection.ts`
    - 实现 `IConnectionApi` 接口
    - 使用 `@tauri-apps/api/core` 的 `invoke` 调用 Rust 命令
    - _Requirements: 3.1, 3.2, 3.3_

  - [ ] 7.2 编写 IPC Connection 适配器测试
    - 创建 `src/api/adapters/ipc/__tests__/connection.test.ts`
    - Mock `invoke` 函数
    - 测试所有方法和错误处理
    - _Requirements: 8.3_

  - [ ] 7.3 创建 IPC 适配器占位
    - 创建 `src/api/adapters/ipc/mysql.ts` 和 `redis.ts`
    - 暂时抛出 "Not implemented" 错误
    - _Requirements: 7.2_

  - [ ] 7.4 创建 IPC 适配器工厂
    - 创建 `src/api/adapters/ipc/index.ts`
    - 导出 `createIpcAdapter()` 函数
    - _Requirements: 3.1_

## Phase 8: 启用连接管理 IPC

- [ ] 8. 启用并测试连接管理 IPC
  - [ ] 8.1 更新适配器配置
    - 在 `src/api/adapter.ts` 中启用 connection 模块的 IPC
    - _Requirements: 7.1, 7.3_

  - [ ] 8.2 更新 vite 代理配置
    - 修改 `vite.config.ts` 始终启用代理（支持回退）
    - _Requirements: 7.4_

  - [ ] 8.3 端到端测试
    - 在 Tauri 模式下启动应用
    - 验证连接管理功能正常
    - 验证 HTTP 回退正常工作
    - _Requirements: 1.1, 1.2, 3.1, 3.2, 3.3, 3.4_

## Phase 9: MySQL IPC (后续扩展)

- [ ] 9. 实现 MySQL IPC 支持
  - [ ] 9.1 实现 MySQL Rust 服务
    - 创建 `src-tauri/src/services/mysql.rs`
    - 实现数据库连接、查询执行等方法
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ] 9.2 实现 MySQL Tauri 命令
    - 创建 `src-tauri/src/commands/mysql.rs`
    - 实现所有 MySQL 相关命令
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ] 9.3 实现 IPC MySQL 适配器
    - 更新 `src/api/adapters/ipc/mysql.ts`
    - 实现 `IMysqlApi` 接口
    - _Requirements: 3.1, 3.2, 3.3_

  - [ ] 9.4 启用 MySQL IPC
    - 在适配器配置中启用 mysql 模块
    - 测试所有 MySQL 功能
    - _Requirements: 7.1, 7.3_

## Phase 10: Redis IPC (后续扩展)

- [ ] 10. 实现 Redis IPC 支持
  - [ ] 10.1 实现 Redis Rust 服务
    - 创建 `src-tauri/src/services/redis.rs`
    - 实现 key 操作、TTL 等方法
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [ ] 10.2 实现 Redis Tauri 命令
    - 创建 `src-tauri/src/commands/redis.rs`
    - 实现所有 Redis 相关命令
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [ ] 10.3 实现 IPC Redis 适配器
    - 更新 `src/api/adapters/ipc/redis.ts`
    - 实现 `IRedisApi` 接口
    - _Requirements: 3.1, 3.2, 3.3_

  - [ ] 10.4 启用 Redis IPC
    - 在适配器配置中启用 redis 模块
    - 测试所有 Redis 功能
    - _Requirements: 7.1, 7.3_

## Phase 11: K8s 集群支持 (客户端模式)

- [ ] 11. 实现 K8s 集群管理支持
  - [ ] 11.1 实现 Cluster Rust 数据模型和存储
    - 在 `src-tauri/src/db/models.rs` 添加 `Cluster` 结构体
    - 在 SQLite 中创建 clusters 表
    - 添加 connections 表的 cluster_id 外键和 K8s 相关字段
    - _Requirements: 集群管理基础存储_

  - [ ] 11.2 实现 Cluster Rust 服务
    - 创建 `src-tauri/src/services/cluster.rs`
    - 实现 `get_all`, `get_by_id`, `create`, `update`, `delete` 方法
    - 实现 `get_connections_by_cluster` 方法
    - _Requirements: 集群 CRUD 操作_

  - [ ] 11.3 实现 Cluster Tauri 命令
    - 创建 `src-tauri/src/commands/cluster.rs`
    - 实现所有集群管理命令
    - 注册到 lib.rs
    - _Requirements: IPC 命令暴露_

  - [ ] 11.4 实现 K8s 服务发现 Rust 服务
    - 添加 `kube` 依赖到 Cargo.toml
    - 创建 `src-tauri/src/services/k8s.rs`
    - 实现 kubeconfig 解析和集群列表
    - 实现服务发现（MySQL/Redis pods）
    - _Requirements: K8s API 集成_

  - [ ] 11.5 实现端口转发 Rust 服务
    - 创建 `src-tauri/src/services/portforward.rs`
    - 使用 kube-rs 的端口转发 API
    - 实现 `create`, `list`, `stop`, `reconnect` 方法
    - 实现空闲超时监控
    - _Requirements: 本地端口转发_

  - [ ] 11.6 实现 K8s/PortForward Tauri 命令
    - 创建 `src-tauri/src/commands/k8s.rs`
    - 实现 k8s_discover, k8s_list_clusters, k8s_import_connections 命令
    - 创建 `src-tauri/src/commands/portforward.rs`
    - 实现端口转发管理命令
    - _Requirements: IPC 命令暴露_

  - [ ] 11.7 更新 IPC 适配器
    - 更新 `src/api/adapters/ipc/index.ts` 中的 IpcClusterApi
    - 实现 IpcK8sApi
    - 实现 IpcPortForwardApi
    - _Requirements: 前端 IPC 调用_

  - [ ] 11.8 端到端测试
    - 在 Tauri 模式下测试集群管理
    - 测试 K8s 服务发现
    - 测试端口转发功能
    - 验证 MySQL/Redis 通过端口转发连接
    - _Requirements: 功能验证_
