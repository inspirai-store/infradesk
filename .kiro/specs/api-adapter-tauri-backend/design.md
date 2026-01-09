# Design Document

## Overview

本设计采用适配器模式实现前端 API 层的平台抽象。核心思想是定义统一的 API 接口，然后为 Web（HTTP）和 Tauri（IPC）分别实现适配器。前端代码只与接口交互，运行时根据平台自动选择适配器实现。

```
┌─────────────────────────────────────────────────────────┐
│                     Frontend                             │
│  ┌─────────────────────────────────────────────────┐    │
│  │              Pinia Stores / Components           │    │
│  └─────────────────────┬───────────────────────────┘    │
│                        │                                 │
│  ┌─────────────────────▼───────────────────────────┐    │
│  │           Unified API Interface                  │    │
│  │  connectionApi, mysqlApi, redisApi, etc.        │    │
│  └─────────────────────┬───────────────────────────┘    │
│                        │                                 │
│           ┌────────────┴────────────┐                   │
│           ▼                         ▼                   │
│  ┌─────────────────┐      ┌─────────────────┐          │
│  │  HTTP Adapter   │      │   IPC Adapter   │          │
│  │  (axios)        │      │  (invoke)       │          │
│  └────────┬────────┘      └────────┬────────┘          │
└───────────┼─────────────────────────┼───────────────────┘
            │                         │
            ▼                         ▼
     ┌──────────────┐         ┌──────────────┐
     │  Go Backend  │         │ Rust Backend │
     │  (HTTP API)  │         │ (Tauri IPC)  │
     └──────────────┘         └──────────────┘
```

## Architecture

### 前端架构

采用分层设计，从上到下：

1. **API Interface Layer** (`src/api/types.ts`) - 定义所有 API 接口类型
2. **Adapter Factory** (`src/api/adapter.ts`) - 根据平台创建适配器
3. **HTTP Adapter** (`src/api/adapters/http.ts`) - 封装现有 axios 调用
4. **IPC Adapter** (`src/api/adapters/ipc.ts`) - 封装 Tauri invoke 调用
5. **API Exports** (`src/api/index.ts`) - 导出统一的 API 对象

### Tauri Rust 后端架构

```
src-tauri/src/
├── lib.rs              # Tauri 应用入口，注册 commands
├── main.rs             # main 函数
├── commands/           # Tauri commands (IPC handlers)
│   ├── mod.rs
│   ├── connection.rs   # 连接管理命令
│   ├── mysql.rs        # MySQL 操作命令
│   └── redis.rs        # Redis 操作命令
├── db/                 # 数据库操作
│   ├── mod.rs
│   ├── sqlite.rs       # 本地 SQLite 存储
│   └── models.rs       # 数据模型
├── services/           # 业务逻辑
│   ├── mod.rs
│   ├── connection.rs   # 连接服务
│   ├── mysql.rs        # MySQL 服务
│   ├── redis.rs        # Redis 服务
│   └── keyring.rs      # 密码安全存储
└── error.rs            # 统一错误类型
```

## Components and Interfaces

### 1. API 接口定义 (TypeScript)

```typescript
// src/api/types.ts

// 适配器接口 - 所有适配器必须实现
export interface IConnectionApi {
  getAll(): Promise<Connection[]>
  getById(id: number): Promise<Connection>
  create(data: Connection): Promise<Connection>
  update(id: number, data: Connection): Promise<Connection>
  delete(id: number): Promise<void>
  test(data: Connection): Promise<TestConnectionResult>
  getByType(type: string): Promise<Connection[]>
}

export interface IMysqlApi {
  getInfo(): Promise<MysqlInfo>
  listDatabases(): Promise<Database[]>
  listTables(database: string): Promise<Table[]>
  executeQuery(database: string, query: string): Promise<QueryResult>
  // ... 其他方法
}

export interface IRedisApi {
  getInfo(): Promise<RedisInfo>
  listKeys(pattern?: string, cursor?: number, count?: number): Promise<KeyListResult>
  getKey(key: string): Promise<KeyValue>
  setKey(data: SetKeyRequest): Promise<void>
  deleteKey(key: string): Promise<void>
  // ... 其他方法
}

// 统一 API 接口
export interface IApiAdapter {
  connection: IConnectionApi
  mysql: IMysqlApi
  redis: IRedisApi
  // 后续可扩展: history, savedQuery, cluster, k8s, portForward
}
```

### 2. 适配器工厂

```typescript
// src/api/adapter.ts
import { isTauri } from '@/utils/platform'
import { createHttpAdapter } from './adapters/http'
import { createIpcAdapter } from './adapters/ipc'
import type { IApiAdapter } from './types'

// 模块启用配置 - 控制哪些模块使用 IPC
const ipcEnabledModules = {
  connection: true,  // 已实现
  mysql: false,      // 待实现时改为 true
  redis: false,      // 待实现时改为 true
}

export function createApiAdapter(): IApiAdapter {
  const httpAdapter = createHttpAdapter()

  if (!isTauri()) {
    return httpAdapter
  }

  const ipcAdapter = createIpcAdapter()

  // 混合适配器：已实现的用 IPC，未实现的回退 HTTP
  return {
    connection: ipcEnabledModules.connection ? ipcAdapter.connection : httpAdapter.connection,
    mysql: ipcEnabledModules.mysql ? ipcAdapter.mysql : httpAdapter.mysql,
    redis: ipcEnabledModules.redis ? ipcAdapter.redis : httpAdapter.redis,
  }
}
```

### 3. HTTP 适配器实现

```typescript
// src/api/adapters/http.ts
import axios from 'axios'
import type { IApiAdapter, IConnectionApi } from '../types'

class HttpConnectionApi implements IConnectionApi {
  async getAll(): Promise<Connection[]> {
    const response = await axios.get<Connection[]>('/api/connections')
    return response.data
  }

  async create(data: Connection): Promise<Connection> {
    const response = await axios.post<Connection>('/api/connections', data)
    return response.data
  }
  // ... 其他方法
}

export function createHttpAdapter(): IApiAdapter {
  return {
    connection: new HttpConnectionApi(),
    mysql: new HttpMysqlApi(),
    redis: new HttpRedisApi(),
  }
}
```

### 4. IPC 适配器实现

```typescript
// src/api/adapters/ipc.ts
import { invoke } from '@tauri-apps/api/core'
import type { IApiAdapter, IConnectionApi } from '../types'

class IpcConnectionApi implements IConnectionApi {
  async getAll(): Promise<Connection[]> {
    return invoke<Connection[]>('get_all_connections')
  }

  async create(data: Connection): Promise<Connection> {
    return invoke<Connection>('create_connection', { data })
  }
  // ... 其他方法
}

export function createIpcAdapter(): IApiAdapter {
  return {
    connection: new IpcConnectionApi(),
    mysql: new IpcMysqlApi(),
    redis: new IpcRedisApi(),
  }
}
```

### 5. Rust Tauri Commands

```rust
// src-tauri/src/commands/connection.rs
use tauri::State;
use crate::db::sqlite::SqlitePool;
use crate::services::connection::ConnectionService;
use crate::db::models::Connection;
use crate::error::AppError;

#[tauri::command]
pub async fn get_all_connections(
    pool: State<'_, SqlitePool>
) -> Result<Vec<Connection>, AppError> {
    let service = ConnectionService::new(pool.inner());
    service.get_all().await
}

#[tauri::command]
pub async fn create_connection(
    pool: State<'_, SqlitePool>,
    data: Connection
) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner());
    service.create(data).await
}

#[tauri::command]
pub async fn test_connection(
    data: Connection
) -> Result<TestResult, AppError> {
    let service = ConnectionService::new_stateless();
    service.test(&data).await
}
```

## Data Models

### Rust 数据模型

```rust
// src-tauri/src/db/models.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Connection {
    pub id: Option<i64>,
    pub name: String,
    #[serde(rename = "type")]
    pub conn_type: String,  // mysql, redis, mongodb, minio
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    // password 不存储在数据库，使用 keyring
    #[sqlx(skip)]
    pub password: Option<String>,
    pub database_name: Option<String>,
    pub is_default: bool,
    pub source: Option<String>,  // local, k8s
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub error: Option<String>,
    pub message: Option<String>,
}
```

### SQLite Schema

```sql
-- 本地 SQLite 数据库 schema
CREATE TABLE IF NOT EXISTS connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT,
    database_name TEXT,
    is_default INTEGER DEFAULT 0,
    source TEXT DEFAULT 'local',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_connections_type ON connections(type);
```

## Error Handling

### Rust 统一错误类型

```rust
// src-tauri/src/error.rs
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Keyring error: {0}")]
    Keyring(String),
}

// 实现 Into<tauri::InvokeError> 以便在 command 中使用
impl From<AppError> for tauri::InvokeError {
    fn from(err: AppError) -> Self {
        tauri::InvokeError::from_error(err)
    }
}
```

### TypeScript 错误处理

```typescript
// src/api/error.ts
export class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: unknown
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

// 适配器中统一处理错误
function handleError(error: unknown): never {
  if (error instanceof Error) {
    throw new ApiError(error.message)
  }
  throw new ApiError('Unknown error occurred')
}
```

## Testing Strategy

### 前端测试

1. **接口类型测试** - 确保类型定义正确
2. **HTTP 适配器单元测试** - Mock axios
3. **IPC 适配器单元测试** - Mock invoke
4. **适配器工厂测试** - Mock 平台检测

```typescript
// src/api/adapters/__tests__/http.test.ts
import { describe, it, expect, vi } from 'vitest'
import axios from 'axios'
import { createHttpAdapter } from '../http'

vi.mock('axios')

describe('HttpConnectionApi', () => {
  it('should fetch all connections', async () => {
    const mockData = [{ id: 1, name: 'Test', type: 'mysql' }]
    vi.mocked(axios.get).mockResolvedValue({ data: mockData })

    const adapter = createHttpAdapter()
    const result = await adapter.connection.getAll()

    expect(result).toEqual(mockData)
    expect(axios.get).toHaveBeenCalledWith('/api/connections')
  })
})
```

### Rust 测试

```rust
// src-tauri/src/services/connection.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_connection() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = SqlitePool::new(&db_path).await.unwrap();

        let service = ConnectionService::new(&pool);
        let conn = Connection {
            id: None,
            name: "Test".to_string(),
            conn_type: "mysql".to_string(),
            host: "localhost".to_string(),
            port: 3306,
            ..Default::default()
        };

        let result = service.create(conn).await.unwrap();
        assert!(result.id.is_some());
        assert_eq!(result.name, "Test");
    }
}
```

## Implementation Phases

按 TDD 方式分阶段实现：

### Phase 1: 基础架构
- 定义 API 接口类型
- 实现适配器工厂
- 编写接口测试

### Phase 2: HTTP 适配器
- 重构现有 API 为 HTTP 适配器
- 编写 HTTP 适配器测试
- 确保现有功能不受影响

### Phase 3: Rust 基础设施
- 实现 SQLite 数据库层
- 实现错误处理
- 编写数据库测试

### Phase 4: 连接管理 IPC
- 实现 Rust connection commands
- 实现 keyring 密码存储
- 实现 IPC 连接适配器
- 编写端到端测试

### Phase 5: MySQL IPC (可选后续)
- 实现 Rust MySQL commands
- 实现 IPC MySQL 适配器

### Phase 6: Redis IPC (可选后续)
- 实现 Rust Redis commands
- 实现 IPC Redis 适配器

## Migration Strategy

```typescript
// vite.config.ts - 更新代理配置以支持回退
export default defineConfig({
  server: {
    proxy: {
      // 始终启用代理，支持 Tauri 模式下的 HTTP 回退
      '/api': {
        target: 'http://localhost:15080',
        changeOrigin: true,
      },
    },
  },
})
```

API 导出方式变更：

```typescript
// src/api/index.ts (before)
export const connectionApi = { ... }

// src/api/index.ts (after)
const adapter = createApiAdapter()
export const connectionApi = adapter.connection
export const mysqlApi = adapter.mysql
export const redisApi = adapter.redis
```

这种方式确保所有使用 `connectionApi` 等的代码无需修改。
