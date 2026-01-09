# Implementation Plan

## Phase 1: 项目初始化与基础架构

- [ ] 1. 初始化 Tauri 项目结构
  - 在项目根目录运行 `pnpm create tauri-app --rc` 初始化 Tauri 2.0
  - 配置 `src-tauri/tauri.conf.json` 指向现有前端目录
  - 配置 Cargo.toml 添加必要依赖（sqlx, redis, keyring, serde, thiserror）
  - 验证 `pnpm tauri dev` 能正常启动空白应用
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 2. 配置前端 Tauri 集成
  - 安装 `@tauri-apps/api` 和 `@tauri-apps/plugin-*` 依赖
  - 修改 `vite.config.ts` 添加 Tauri 开发服务器配置
  - 创建 `src/utils/platform.ts` 检测运行环境（Tauri/Web）
  - 编写环境检测测试用例
  - _Requirements: 2.1, 2.5_

## Phase 2: Tauri API 适配层（TDD）

- [ ] 3. 创建 Tauri 适配层测试
  - 编写 `src/api/__tests__/tauri-adapter.spec.ts` 测试文件
  - 测试 `isTauri()` 环境检测函数
  - 测试 `resolveCommand()` 路由映射函数
  - 测试 `tauriRequest()` 请求封装函数
  - 运行测试确认全部失败
  - _Requirements: 2.2, 2.3, 4.1_

- [ ] 4. 实现 Tauri 适配层
  - 创建 `src/api/tauri-adapter.ts` 实现适配层
  - 实现 API 路由到 Tauri 命令的映射表
  - 实现 `isTauri()` 环境检测
  - 实现 `resolveCommand()` 路由解析
  - 实现 `tauriRequest()` 请求封装
  - 运行测试确认全部通过
  - _Requirements: 2.2, 2.3_

- [ ] 5. 改造 axios 拦截器
  - 修改 `src/api/index.ts` 添加 Tauri 拦截器
  - 在 Tauri 环境下拦截请求并转发到 IPC
  - 保持 Web 环境下的原有 HTTP 行为
  - 编写拦截器行为测试
  - _Requirements: 2.2, 2.3_

## Phase 3: Rust 数据模型（TDD）

- [ ] 6. 创建 Rust 数据模型测试
  - 创建 `src-tauri/src/models/mod.rs` 模块入口
  - 编写 Connection 模型序列化/反序列化测试
  - 编写 QueryResult 模型测试
  - 编写 DbType 枚举测试
  - 运行 `cargo test` 确认失败
  - _Requirements: 3.4, 4.3_

- [ ] 7. 实现 Rust 数据模型
  - 创建 `src-tauri/src/models/connection.rs`
  - 创建 `src-tauri/src/models/query.rs`
  - 创建 `src-tauri/src/models/result.rs`
  - 实现所有 Serialize/Deserialize trait
  - 运行测试确认通过
  - _Requirements: 3.4_

## Phase 4: 本地存储服务（TDD）

- [ ] 8. 创建存储服务测试
  - 编写 SQLite 连接初始化测试
  - 编写 connections 表 CRUD 测试
  - 编写 query_history 表操作测试
  - 编写 saved_queries 表操作测试
  - _Requirements: 3.4, 5.1, 7.5, 4.3_

- [ ] 9. 实现存储服务
  - 创建 `src-tauri/src/services/storage.rs`
  - 实现 SQLite 数据库初始化和迁移
  - 实现 Connection CRUD 操作
  - 实现 QueryHistory 操作
  - 实现 SavedQuery 操作
  - 运行测试确认通过
  - _Requirements: 3.4, 5.1_

- [ ] 10. 集成系统密钥链
  - 编写密钥链存取测试（使用 mock）
  - 创建 `src-tauri/src/utils/keychain.rs`
  - 实现跨平台密钥链访问（keyring crate）
  - 集成到 StorageService 的密码存取
  - _Requirements: 5.4, 7.1, 7.2_

## Phase 5: MySQL 服务层（TDD）

- [ ] 11. 创建 MySQL 服务测试
  - 编写 `apply_limit()` 函数测试
  - 编写连接池管理测试
  - 编写 `get_databases()` 测试（使用 mock）
  - 编写 `execute_query()` 测试（使用 mock）
  - _Requirements: 3.1, 3.3, 4.3_

- [ ] 12. 实现 MySQL 服务
  - 创建 `src-tauri/src/services/mysql.rs`
  - 实现连接池管理（HashMap + RwLock）
  - 实现 `apply_limit()` SQL 处理
  - 实现 `get_databases()` 查询
  - 实现 `get_tables()` 查询
  - 实现 `execute_query()` 通用查询
  - 运行测试确认通过
  - _Requirements: 3.1, 3.3_

- [ ] 13. 实现 MySQL 表数据操作
  - 编写 `get_table_data()` 测试
  - 编写 `update_record()` 测试
  - 编写 `insert_record()` 测试
  - 编写 `delete_record()` 测试
  - 实现对应功能
  - _Requirements: 3.1_

## Phase 6: Redis 服务层（TDD）

- [ ] 14. 创建 Redis 服务测试
  - 编写连接管理测试
  - 编写 `scan_keys()` 测试
  - 编写 `get_value()` 测试（支持多种数据类型）
  - 编写 `set_value()` 测试
  - _Requirements: 3.2, 4.3_

- [ ] 15. 实现 Redis 服务
  - 创建 `src-tauri/src/services/redis.rs`
  - 实现连接管理
  - 实现 `scan_keys()` 扫描键
  - 实现 `get_value()` 获取值（处理 String/List/Hash/Set/ZSet）
  - 实现 `set_value()` 设置值
  - 实现 `delete_key()` 删除键
  - _Requirements: 3.2_

## Phase 7: Tauri 命令层（TDD）

- [ ] 16. 创建连接管理命令测试
  - 编写 `get_connections` 命令测试
  - 编写 `create_connection` 命令测试
  - 编写 `test_connection` 命令测试
  - 编写 `delete_connection` 命令测试
  - _Requirements: 5.1, 5.2, 5.3, 4.3_

- [ ] 17. 实现连接管理命令
  - 创建 `src-tauri/src/commands/connection.rs`
  - 实现所有连接管理 Tauri 命令
  - 在 `main.rs` 注册命令到 invoke_handler
  - 运行测试确认通过
  - _Requirements: 5.1, 5.2, 5.3_

- [ ] 18. 创建 MySQL 命令测试
  - 编写 `get_mysql_databases` 命令测试
  - 编写 `get_mysql_tables` 命令测试
  - 编写 `execute_mysql_query` 命令测试
  - 编写 `get_table_data` 命令测试
  - _Requirements: 3.1, 4.3_

- [ ] 19. 实现 MySQL 命令
  - 创建 `src-tauri/src/commands/mysql.rs`
  - 实现所有 MySQL 相关 Tauri 命令
  - 注册命令到 invoke_handler
  - 运行测试确认通过
  - _Requirements: 3.1_

- [ ] 20. 创建 Redis 命令测试并实现
  - 编写 `get_redis_keys` 命令测试
  - 编写 `get_redis_value` 命令测试
  - 编写 `set_redis_value` 命令测试
  - 创建 `src-tauri/src/commands/redis.rs`
  - 实现所有 Redis 相关 Tauri 命令
  - _Requirements: 3.2, 4.3_

## Phase 8: 错误处理（TDD）

- [ ] 21. 实现统一错误处理
  - 创建 `src-tauri/src/utils/error.rs`
  - 实现 AppError 枚举和 thiserror 派生
  - 实现 Serialize trait 用于 IPC 传输
  - 实现 user_message() 用户友好提示
  - 编写错误转换测试
  - _Requirements: 5.3_

- [ ] 22. 前端错误处理适配
  - 创建 `src/utils/error-handler.ts`
  - 实现 Tauri 错误格式解析
  - 集成 Naive UI 消息提示
  - 编写错误处理测试
  - _Requirements: 5.3_

## Phase 9: 前端集成测试

- [ ] 23. 测试 MySQL Store 集成
  - 编写 store 与 Tauri 适配层集成测试
  - 测试数据库列表加载
  - 测试表列表加载
  - 测试 SQL 查询执行
  - _Requirements: 2.1, 2.4, 4.2_

- [ ] 24. 测试 Redis Store 集成
  - 编写 Redis store 与 Tauri 适配层集成测试
  - 测试键列表扫描
  - 测试值读取（多种类型）
  - 测试值写入
  - _Requirements: 2.1, 4.2_

- [ ] 25. 测试连接管理 Store 集成
  - 编写连接 store 与 Tauri 适配层集成测试
  - 测试连接创建流程
  - 测试连接测试流程
  - 测试连接切换
  - _Requirements: 2.1, 5.2, 5.5, 4.2_

## Phase 10: 应用打包与更新

- [ ] 26. 配置应用图标和元数据
  - 创建各平台图标（icns, ico, png）
  - 配置 tauri.conf.json 产品信息
  - 配置窗口默认尺寸和行为
  - 测试各平台启动效果
  - _Requirements: 1.1, 1.2, 1.3, 1.5_

- [ ] 27. 配置自动更新
  - 配置 Tauri updater 插件
  - 设置更新服务器端点
  - 实现更新检查 UI
  - 实现更新下载和安装提示
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 28. 配置多平台构建
  - 配置 GitHub Actions 多平台构建
  - 配置 macOS 签名（可选）
  - 配置 Windows 签名（可选）
  - 生成 DMG、MSI、DEB 安装包
  - 验证各平台安装包体积 < 20MB
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

## Phase 11: 端到端测试

- [ ] 29. 编写核心流程 E2E 测试
  - 创建连接 → 测试连接 → 保存连接流程
  - 选择连接 → 查看数据库 → 查看表 → 查询数据流程
  - 编辑数据 → 保存修改流程
  - 查询历史保存和加载流程
  - _Requirements: 4.4, 5.2_

- [ ] 30. 运行完整测试套件并确保覆盖率达标
  - 运行 Rust 测试并生成覆盖率报告
  - 运行前端测试并生成覆盖率报告
  - 确认 Rust 覆盖率 >= 80%
  - 确认前端覆盖率 >= 80%
  - 修复任何覆盖率不足的模块
  - _Requirements: 4.2, 4.3, 4.5_

