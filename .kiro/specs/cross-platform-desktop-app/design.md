# Design Document

## Overview

本设计文档描述如何将 Zeni-X 数据库管理平台从 Web 应用改造为跨平台桌面应用。采用 Tauri 2.0 框架，复用现有 Vue 3 前端，将 Go 后端功能迁移至 Rust。

### 技术选型

| 层级 | 当前技术 | 目标技术 |
|------|----------|----------|
| 前端框架 | Vue 3 + TypeScript | Vue 3 + TypeScript（复用） |
| UI 组件库 | Naive UI | Naive UI（复用） |
| 构建工具 | Vite | Vite + Tauri CLI |
| 后端运行时 | Go + Gin | Rust + Tauri |
| 数据存储 | SQLite | SQLite（复用） |
| 部署方式 | K8s + Helm | 桌面安装包 |

### 选择 Tauri 的原因

1. **轻量级打包**: 相比 Electron 的 150MB+，Tauri 打包体积约 10-20MB
2. **原生性能**: Rust 后端提供接近原生的性能
3. **前端复用**: 完全兼容现有 Vue 3 + Vite 技术栈
4. **安全性**: Rust 内存安全 + Tauri 权限系统
5. **跨平台**: 原生支持 macOS、Windows、Linux

## Architecture

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     Zeni-X Desktop App                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐   │
│  │                  Frontend (WebView)                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │   │
│  │  │   Vue 3     │  │  Naive UI   │  │   Monaco    │  │   │
│  │  │ Components  │  │  Components │  │   Editor    │  │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  │   │
│  │         │                │                │         │   │
│  │  ┌──────┴────────────────┴────────────────┴──────┐  │   │
│  │  │              Pinia State Management            │  │   │
│  │  └──────────────────────┬────────────────────────┘  │   │
│  │                         │                           │   │
│  │  ┌──────────────────────┴────────────────────────┐  │   │
│  │  │           Tauri API Adapter Layer             │  │   │
│  │  │  (axios interceptor → @tauri-apps/api/core)   │  │   │
│  │  └──────────────────────┬────────────────────────┘  │   │
│  └─────────────────────────┼───────────────────────────┘   │
│                            │ IPC (invoke)                   │
│  ┌─────────────────────────┼───────────────────────────┐   │
│  │                  Backend (Rust)                      │   │
│  │  ┌──────────────────────┴────────────────────────┐  │   │
│  │  │              Tauri Commands                    │  │   │
│  │  └──────┬─────────────┬─────────────┬────────────┘  │   │
│  │         │             │             │               │   │
│  │  ┌──────┴──────┐ ┌────┴────┐ ┌──────┴──────┐       │   │
│  │  │   MySQL     │ │  Redis  │ │   Storage   │       │   │
│  │  │   Service   │ │ Service │ │   Service   │       │   │
│  │  └──────┬──────┘ └────┬────┘ └──────┬──────┘       │   │
│  │         │             │             │               │   │
│  │  ┌──────┴──────┐ ┌────┴────┐ ┌──────┴──────┐       │   │
│  │  │  sqlx       │ │  redis  │ │   rusqlite  │       │   │
│  │  │  (mysql)    │ │  crate  │ │   + keyring │       │   │
│  │  └─────────────┘ └─────────┘ └─────────────┘       │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 目录结构

```
zeni-x/
├── src-tauri/                    # Tauri Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri 配置
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 库入口
│   │   ├── commands/             # Tauri 命令（对应原 Go handlers）
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs          # MySQL 相关命令
│   │   │   ├── redis.rs          # Redis 相关命令
│   │   │   ├── connection.rs     # 连接管理命令
│   │   │   └── storage.rs        # 本地存储命令
│   │   ├── services/             # 业务逻辑层
│   │   │   ├── mod.rs
│   │   │   ├── mysql.rs
│   │   │   ├── redis.rs
│   │   │   └── storage.rs
│   │   ├── models/               # 数据模型
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs
│   │   │   ├── query.rs
│   │   │   └── result.rs
│   │   └── utils/                # 工具函数
│   │       ├── mod.rs
│   │       ├── crypto.rs         # 加密工具
│   │       └── keychain.rs       # 密钥链访问
│   └── icons/                    # 应用图标
├── services/zeni-x/frontend/     # 前端（基本复用）
│   ├── src/
│   │   ├── api/
│   │   │   ├── index.ts          # API 类型定义（复用）
│   │   │   └── tauri-adapter.ts  # 新增：Tauri IPC 适配器
│   │   ├── stores/               # Pinia stores（复用）
│   │   ├── components/           # Vue 组件（复用）
│   │   └── views/                # 页面视图（复用）
│   ├── vite.config.ts            # 需修改：添加 Tauri 插件
│   └── package.json              # 需修改：添加 Tauri 依赖
└── tests/                        # 测试目录
    ├── rust/                     # Rust 单元测试
    ├── frontend/                 # 前端单元测试
    └── e2e/                      # 端到端测试
```

## Components and Interfaces

### Frontend 适配层

#### Tauri API 适配器

创建适配层，使现有 axios 调用无缝切换到 Tauri IPC：

```typescript
// src/api/tauri-adapter.ts
import { invoke } from '@tauri-apps/api/core';

// 检测是否在 Tauri 环境
export const isTauri = () => '__TAURI__' in window;

// 通用请求适配器
export async function tauriRequest<T>(
  command: string,
  payload?: Record<string, unknown>
): Promise<T> {
  return invoke<T>(command, payload);
}

// API 路由映射
const API_COMMAND_MAP: Record<string, string> = {
  'GET /api/connections': 'get_connections',
  'POST /api/connections': 'create_connection',
  'DELETE /api/connections/:id': 'delete_connection',
  'POST /api/mysql/query': 'execute_mysql_query',
  'GET /api/mysql/databases': 'get_mysql_databases',
  'GET /api/mysql/tables': 'get_mysql_tables',
  'GET /api/redis/keys': 'get_redis_keys',
  'GET /api/redis/key/:key': 'get_redis_value',
  // ... 其他映射
};
```

#### Axios 拦截器改造

```typescript
// src/api/index.ts 修改
import axios from 'axios';
import { isTauri, tauriRequest, resolveCommand } from './tauri-adapter';

const api = axios.create({
  baseURL: '/api',
});

// Tauri 环境拦截器
api.interceptors.request.use(async (config) => {
  if (isTauri()) {
    const command = resolveCommand(config.method, config.url);
    const result = await tauriRequest(command, {
      ...config.params,
      ...config.data,
    });
    // 返回 mock response 格式
    return Promise.reject({
      __tauri_response__: true,
      data: result,
    });
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.__tauri_response__) {
      return { data: error.data };
    }
    throw error;
  }
);
```

### Rust Backend Commands

#### MySQL 命令模块

```rust
// src-tauri/src/commands/mysql.rs
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;
use tauri::State;

use crate::models::{Connection, QueryResult, TableInfo};
use crate::services::mysql::MySqlService;

#[derive(Serialize)]
pub struct DatabaseList {
    databases: Vec<String>,
}

#[tauri::command]
pub async fn get_mysql_databases(
    connection_id: String,
    service: State<'_, MySqlService>,
) -> Result<DatabaseList, String> {
    service
        .get_databases(&connection_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_mysql_query(
    connection_id: String,
    sql: String,
    limit: Option<u32>,
    service: State<'_, MySqlService>,
) -> Result<QueryResult, String> {
    service
        .execute_query(&connection_id, &sql, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mysql_tables(
    connection_id: String,
    database: String,
    service: State<'_, MySqlService>,
) -> Result<Vec<TableInfo>, String> {
    service
        .get_tables(&connection_id, &database)
        .await
        .map_err(|e| e.to_string())
}
```

#### Redis 命令模块

```rust
// src-tauri/src/commands/redis.rs
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::models::{RedisKey, RedisValue};
use crate::services::redis::RedisService;

#[tauri::command]
pub async fn get_redis_keys(
    connection_id: String,
    pattern: Option<String>,
    service: State<'_, RedisService>,
) -> Result<Vec<RedisKey>, String> {
    let pattern = pattern.unwrap_or_else(|| "*".to_string());
    service
        .scan_keys(&connection_id, &pattern)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_redis_value(
    connection_id: String,
    key: String,
    service: State<'_, RedisService>,
) -> Result<RedisValue, String> {
    service
        .get_value(&connection_id, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_redis_value(
    connection_id: String,
    key: String,
    value: String,
    ttl: Option<i64>,
    service: State<'_, RedisService>,
) -> Result<(), String> {
    service
        .set_value(&connection_id, &key, &value, ttl)
        .await
        .map_err(|e| e.to_string())
}
```

#### 连接管理命令

```rust
// src-tauri/src/commands/connection.rs
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::models::Connection;
use crate::services::storage::StorageService;

#[derive(Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub db_type: String,  // "mysql" | "redis"
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
}

#[tauri::command]
pub async fn get_connections(
    storage: State<'_, StorageService>,
) -> Result<Vec<Connection>, String> {
    storage.get_all_connections().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_connection(
    request: CreateConnectionRequest,
    storage: State<'_, StorageService>,
) -> Result<Connection, String> {
    storage
        .create_connection(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(
    connection_id: String,
    storage: State<'_, StorageService>,
) -> Result<bool, String> {
    storage
        .test_connection(&connection_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_connection(
    connection_id: String,
    storage: State<'_, StorageService>,
) -> Result<(), String> {
    storage
        .delete_connection(&connection_id)
        .await
        .map_err(|e| e.to_string())
}
```

### Service Layer

#### MySQL Service

```rust
// src-tauri/src/services/mysql.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::Row;

use crate::models::{Connection, QueryResult, TableInfo, ColumnInfo};

pub struct MySqlService {
    pools: Arc<RwLock<HashMap<String, MySqlPool>>>,
    storage: Arc<StorageService>,
}

impl MySqlService {
    pub fn new(storage: Arc<StorageService>) -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    async fn get_pool(&self, connection_id: &str) -> Result<MySqlPool, anyhow::Error> {
        // 检查缓存
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(connection_id) {
                return Ok(pool.clone());
            }
        }

        // 创建新连接
        let conn = self.storage.get_connection(connection_id).await?;
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            conn.username.unwrap_or_default(),
            conn.password.unwrap_or_default(),
            conn.host,
            conn.port,
            conn.database.unwrap_or_default()
        );

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        // 缓存连接池
        {
            let mut pools = self.pools.write().await;
            pools.insert(connection_id.to_string(), pool.clone());
        }

        Ok(pool)
    }

    pub async fn get_databases(&self, connection_id: &str) -> Result<Vec<String>, anyhow::Error> {
        let pool = self.get_pool(connection_id).await?;
        let rows = sqlx::query("SHOW DATABASES")
            .fetch_all(&pool)
            .await?;

        let databases: Vec<String> = rows
            .iter()
            .map(|row| row.get::<String, _>(0))
            .collect();

        Ok(databases)
    }

    pub async fn execute_query(
        &self,
        connection_id: &str,
        sql: &str,
        limit: Option<u32>,
    ) -> Result<QueryResult, anyhow::Error> {
        let pool = self.get_pool(connection_id).await?;

        // 应用 LIMIT（如果是 SELECT 且没有 LIMIT）
        let sql = self.apply_limit(sql, limit.unwrap_or(100));

        let rows = sqlx::query(&sql).fetch_all(&pool).await?;

        // 转换为 QueryResult 格式
        // ... 实现细节

        Ok(QueryResult { /* ... */ })
    }

    fn apply_limit(&self, sql: &str, limit: u32) -> String {
        let sql_upper = sql.to_uppercase();
        if sql_upper.starts_with("SELECT") && !sql_upper.contains("LIMIT") {
            format!("{} LIMIT {}", sql, limit)
        } else {
            sql.to_string()
        }
    }
}
```

## Data Models

### Rust 数据模型

```rust
// src-tauri/src/models/connection.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    #[serde(skip_serializing)]  // 不序列化密码到前端
    pub password: Option<String>,
    pub database: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbType {
    MySQL,
    Redis,
}

// src-tauri/src/models/query.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub affected_rows: u64,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub engine: String,
    pub row_count: u64,
    pub data_size: u64,
}

// src-tauri/src/models/result.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisKey {
    pub key: String,
    pub key_type: String,
    pub ttl: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisValue {
    pub key: String,
    pub value: serde_json::Value,
    pub key_type: String,
    pub ttl: i64,
}
```

### SQLite 本地存储 Schema

```sql
-- connections 表
CREATE TABLE IF NOT EXISTS connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    db_type TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT,
    database_name TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- 密码存储在系统密钥链中，不存储在 SQLite

-- query_history 表
CREATE TABLE IF NOT EXISTS query_history (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    sql TEXT NOT NULL,
    execution_time_ms INTEGER,
    row_count INTEGER,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (connection_id) REFERENCES connections(id)
);

-- saved_queries 表
CREATE TABLE IF NOT EXISTS saved_queries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sql TEXT NOT NULL,
    connection_id TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## Error Handling

### Rust 错误处理

```rust
// src-tauri/src/utils/error.rs
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Keychain error: {0}")]
    KeychainError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// 为 Tauri 命令实现 Serialize
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// 转换为用户友好的错误消息
impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            AppError::ConnectionError(_) => "无法连接到数据库，请检查连接配置".to_string(),
            AppError::QueryError(msg) => format!("查询执行失败: {}", msg),
            AppError::StorageError(_) => "本地存储错误，请重启应用".to_string(),
            AppError::KeychainError(_) => "无法访问系统密钥链".to_string(),
            AppError::NotFound(item) => format!("{} 不存在", item),
            AppError::InvalidInput(msg) => format!("输入无效: {}", msg),
        }
    }
}
```

### 前端错误处理

```typescript
// src/utils/error-handler.ts
import { message } from 'naive-ui';

export interface ApiError {
  code: string;
  message: string;
}

export function handleApiError(error: unknown): void {
  if (typeof error === 'string') {
    message.error(error);
    return;
  }

  if (error instanceof Error) {
    message.error(error.message);
    return;
  }

  message.error('发生未知错误');
}

// 在 store 中统一处理
export function createErrorHandler(context: string) {
  return (error: unknown) => {
    console.error(`[${context}]`, error);
    handleApiError(error);
  };
}
```

## Testing Strategy

### TDD 工作流程

```
1. 编写测试（Red）
   ↓
2. 运行测试，确认失败
   ↓
3. 编写最小实现代码（Green）
   ↓
4. 运行测试，确认通过
   ↓
5. 重构代码（Refactor）
   ↓
6. 运行测试，确认仍然通过
```

### Rust 单元测试

```rust
// src-tauri/src/services/mysql.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_limit_adds_limit_to_select() {
        let service = MySqlService::new(/* mock storage */);
        let sql = "SELECT * FROM users";
        let result = service.apply_limit(sql, 100);
        assert_eq!(result, "SELECT * FROM users LIMIT 100");
    }

    #[test]
    fn test_apply_limit_preserves_existing_limit() {
        let service = MySqlService::new(/* mock storage */);
        let sql = "SELECT * FROM users LIMIT 50";
        let result = service.apply_limit(sql, 100);
        assert_eq!(result, "SELECT * FROM users LIMIT 50");
    }

    #[test]
    fn test_apply_limit_ignores_non_select() {
        let service = MySqlService::new(/* mock storage */);
        let sql = "UPDATE users SET name = 'test'";
        let result = service.apply_limit(sql, 100);
        assert_eq!(result, "UPDATE users SET name = 'test'");
    }

    #[tokio::test]
    async fn test_get_databases_returns_list() {
        // 使用 testcontainers 或 mock
        let storage = create_mock_storage();
        let service = MySqlService::new(Arc::new(storage));

        // 设置 mock 连接
        // ...

        let result = service.get_databases("test-conn").await;
        assert!(result.is_ok());
    }
}
```

### 前端单元测试

```typescript
// src/api/__tests__/tauri-adapter.spec.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { isTauri, tauriRequest, resolveCommand } from '../tauri-adapter';

describe('Tauri Adapter', () => {
  describe('isTauri', () => {
    it('returns false when not in Tauri environment', () => {
      expect(isTauri()).toBe(false);
    });

    it('returns true when in Tauri environment', () => {
      (window as any).__TAURI__ = {};
      expect(isTauri()).toBe(true);
      delete (window as any).__TAURI__;
    });
  });

  describe('resolveCommand', () => {
    it('resolves GET /api/connections to get_connections', () => {
      expect(resolveCommand('GET', '/api/connections')).toBe('get_connections');
    });

    it('resolves POST /api/mysql/query to execute_mysql_query', () => {
      expect(resolveCommand('POST', '/api/mysql/query')).toBe('execute_mysql_query');
    });
  });
});

// src/stores/__tests__/mysql.spec.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useMySqlStore } from '../mysql';

describe('MySQL Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('applies limit to SELECT queries', () => {
    const store = useMySqlStore();
    store.queryLimit = 100;

    const sql = 'SELECT * FROM users';
    const result = store.applyLimit(sql);

    expect(result).toBe('SELECT * FROM users LIMIT 100');
  });

  it('extracts base query without LIMIT', () => {
    const store = useMySqlStore();

    const sql = 'SELECT * FROM users LIMIT 100 OFFSET 50';
    const result = store.extractBaseQuery(sql);

    expect(result).toBe('SELECT * FROM users');
  });
});
```

### 端到端测试

```rust
// tests/e2e/connection_flow.rs
use tauri::test::{mock_builder, MockRuntime};

#[tokio::test]
async fn test_create_and_use_connection() {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            create_connection,
            get_connections,
            test_connection,
        ])
        .build();

    // 创建连接
    let result: Connection = app
        .invoke("create_connection", json!({
            "name": "Test MySQL",
            "db_type": "mysql",
            "host": "localhost",
            "port": 3306,
            "username": "root",
            "password": "password"
        }))
        .await
        .unwrap();

    assert!(!result.id.is_empty());

    // 获取连接列表
    let connections: Vec<Connection> = app
        .invoke("get_connections", json!({}))
        .await
        .unwrap();

    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].name, "Test MySQL");
}
```

### 测试覆盖率要求

| 模块 | 最低覆盖率 | 测试类型 |
|------|------------|----------|
| Rust Commands | 80% | 单元测试 |
| Rust Services | 85% | 单元测试 + 集成测试 |
| Rust Models | 90% | 单元测试 |
| Frontend Stores | 80% | 单元测试 |
| Frontend Components | 75% | 组件测试 |
| API Adapter | 90% | 单元测试 |
| E2E Flows | 核心流程 | 端到端测试 |

### CI 配置

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Run Rust tests
        run: cargo test --workspace
      - name: Check coverage
        run: cargo tarpaulin --out Xml --fail-under 80

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - name: Install dependencies
        run: pnpm install
        working-directory: services/zeni-x/frontend
      - name: Run tests with coverage
        run: pnpm test:run --coverage
        working-directory: services/zeni-x/frontend
      - name: Check coverage threshold
        run: |
          coverage=$(cat coverage/coverage-summary.json | jq '.total.lines.pct')
          if (( $(echo "$coverage < 80" | bc -l) )); then
            echo "Coverage $coverage% is below 80%"
            exit 1
          fi
```

## Tauri 配置

```json
// src-tauri/tauri.conf.json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Zeni-X",
  "version": "1.0.0",
  "identifier": "com.zeni-x.app",
  "build": {
    "frontendDist": "../services/zeni-x/frontend/dist"
  },
  "app": {
    "windows": [
      {
        "title": "Zeni-X Database Manager",
        "width": 1280,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi", "deb"],
    "icon": ["icons/icon.icns", "icons/icon.ico", "icons/icon.png"]
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://releases.zeni-x.com/update/{{target}}/{{arch}}/{{current_version}}"],
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

