# Requirements Document

## Introduction

本功能旨在为 Zeni-X 数据库管理平台实现 API 适配器模式，使前端能够在 Web 模式（HTTP API）和 Tauri 桌面模式（IPC Commands）之间无缝切换。采用 TDD 方式逐步实现，确保每个模块都有测试覆盖，并支持增量迁移。

核心目标：
- 前端代码无需修改即可在两种模式下运行
- Tauri 桌面应用成为真正独立的客户端，无需外部后端服务
- 采用适配器模式，保持代码可维护性和可测试性

## Requirements

### Requirement 1: API 适配器接口层

**User Story:** 作为开发者，我希望有一个统一的 API 接口层，使得前端代码无需关心底层是 HTTP 还是 IPC 调用。

#### Acceptance Criteria

1. WHEN 应用初始化时 THEN 系统 SHALL 根据运行环境（Tauri/Web）自动选择对应的适配器实现
2. WHEN 调用 API 方法时 THEN 适配器 SHALL 返回与当前 HTTP API 相同的数据结构
3. IF 适配器切换发生 THEN 系统 SHALL 确保所有现有的 store 和组件代码无需修改
4. WHEN 定义适配器接口时 THEN 接口 SHALL 覆盖所有现有 API 模块（connectionApi, mysqlApi, redisApi 等）

### Requirement 2: HTTP 适配器实现

**User Story:** 作为开发者，我希望现有的 HTTP API 调用被封装为适配器实现，以便在 Web 模式下继续使用。

#### Acceptance Criteria

1. WHEN 应用在 Web 模式运行时 THEN 系统 SHALL 使用 HTTP 适配器
2. WHEN HTTP 适配器被调用时 THEN 适配器 SHALL 通过 axios 发送请求到后端
3. WHEN HTTP 请求发生错误时 THEN 适配器 SHALL 抛出统一格式的错误对象
4. IF HTTP 适配器被使用 THEN 系统 SHALL 保持与现有行为完全一致（包括 X-Connection-ID header 注入）

### Requirement 3: Tauri IPC 适配器实现

**User Story:** 作为开发者，我希望有一个 Tauri IPC 适配器，通过 Tauri invoke 调用 Rust 后端命令。

#### Acceptance Criteria

1. WHEN 应用在 Tauri 模式运行时 THEN 系统 SHALL 使用 IPC 适配器
2. WHEN IPC 适配器被调用时 THEN 适配器 SHALL 通过 `@tauri-apps/api/core` 的 `invoke` 函数调用 Rust 命令
3. WHEN Rust 命令返回结果时 THEN 适配器 SHALL 将结果转换为与 HTTP API 相同的数据结构
4. WHEN Rust 命令返回错误时 THEN 适配器 SHALL 抛出统一格式的错误对象

### Requirement 4: Tauri Rust 后端 - 连接管理

**User Story:** 作为用户，我希望在 Tauri 桌面应用中能够管理数据库连接（创建、编辑、删除、测试）。

#### Acceptance Criteria

1. WHEN 用户请求获取所有连接时 THEN Rust 后端 SHALL 从本地 SQLite 数据库返回连接列表
2. WHEN 用户创建新连接时 THEN Rust 后端 SHALL 将连接信息保存到本地 SQLite 数据库
3. WHEN 用户更新连接时 THEN Rust 后端 SHALL 更新本地 SQLite 数据库中的连接信息
4. WHEN 用户删除连接时 THEN Rust 后端 SHALL 从本地 SQLite 数据库删除连接信息
5. WHEN 用户测试连接时 THEN Rust 后端 SHALL 尝试建立实际数据库连接并返回结果
6. IF 密码字段存在 THEN 系统 SHALL 使用 keyring 安全存储密码（不直接存储在 SQLite 中）

### Requirement 5: Tauri Rust 后端 - MySQL 操作

**User Story:** 作为用户，我希望在 Tauri 桌面应用中能够执行 MySQL 数据库操作。

#### Acceptance Criteria

1. WHEN 用户请求列出数据库时 THEN Rust 后端 SHALL 连接 MySQL 并返回数据库列表
2. WHEN 用户请求列出表时 THEN Rust 后端 SHALL 返回指定数据库的表列表
3. WHEN 用户执行 SQL 查询时 THEN Rust 后端 SHALL 执行查询并返回结果集
4. WHEN 查询返回大量数据时 THEN Rust 后端 SHALL 支持分页返回
5. WHEN 查询执行失败时 THEN Rust 后端 SHALL 返回详细的错误信息

### Requirement 6: Tauri Rust 后端 - Redis 操作

**User Story:** 作为用户，我希望在 Tauri 桌面应用中能够执行 Redis 操作。

#### Acceptance Criteria

1. WHEN 用户请求列出 keys 时 THEN Rust 后端 SHALL 使用 SCAN 命令返回 key 列表
2. WHEN 用户请求获取 key 值时 THEN Rust 后端 SHALL 根据 key 类型返回对应的值
3. WHEN 用户设置 key 时 THEN Rust 后端 SHALL 根据类型（string/hash/list/set/zset）正确设置值
4. WHEN 用户删除 key 时 THEN Rust 后端 SHALL 删除指定的 key
5. WHEN 用户设置 TTL 时 THEN Rust 后端 SHALL 为 key 设置过期时间

### Requirement 7: 增量迁移支持

**User Story:** 作为开发者，我希望能够逐步将 API 从 HTTP 迁移到 Tauri IPC，而不是一次性全部切换。

#### Acceptance Criteria

1. WHEN 适配器初始化时 THEN 系统 SHALL 支持配置哪些 API 模块使用 IPC，哪些仍使用 HTTP
2. IF 某个 API 模块的 IPC 实现尚未完成 THEN 系统 SHALL 回退到 HTTP 适配器（需要后端运行）
3. WHEN 新的 IPC 命令实现完成时 THEN 开发者 SHALL 能够通过配置启用该模块的 IPC 调用
4. IF 在 Tauri 环境下使用 HTTP 回退 THEN vite.config.ts SHALL 启用代理配置

### Requirement 8: 测试覆盖

**User Story:** 作为开发者，我希望所有适配器和 Rust 命令都有测试覆盖，确保代码质量。

#### Acceptance Criteria

1. WHEN 适配器接口定义完成时 THEN 开发者 SHALL 为接口编写类型测试
2. WHEN HTTP 适配器实现完成时 THEN 开发者 SHALL 编写单元测试（使用 mock axios）
3. WHEN IPC 适配器实现完成时 THEN 开发者 SHALL 编写单元测试（使用 mock invoke）
4. WHEN Rust 命令实现完成时 THEN 开发者 SHALL 编写 Rust 单元测试
5. IF 所有测试通过 THEN 系统 SHALL 能够在 CI 环境中自动运行测试
