# Zeni-X - Database Management Desktop App

## Overview

Zeni-X is a cross-platform desktop database management application built with Tauri 2.0.

## Tech Stack

- **Backend**: Rust + Tauri 2.0
- **Frontend**: Vue 3 + TypeScript + Naive UI
- **Database**: SQLite (app config) + MySQL/Redis (target databases)
- **Kubernetes**: Cluster resource visualization and port forwarding

## Development Commands

```bash
just dev      # Start Tauri development mode
just build    # Build production release
just install  # Install dependencies
just test     # Run frontend tests
just lint     # Code linting
just clean    # Clean build artifacts
```

## Project Structure

```
zeni-x/
├── src-tauri/                  # Tauri Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri commands (IPC interface)
│   │   ├── services/           # Business logic
│   │   └── db/                 # SQLite data layer
│   └── Cargo.toml
├── services/zeni-x/frontend/   # Vue frontend
├── config/env/                 # Environment configuration
├── data/                       # Runtime data
└── archive/                    # Archived content (reference only)
```

## Architecture

```
Vue Frontend (IPC) → Tauri Commands → Rust Services → Database/K8s
```

## Archived Content

The `archive/` directory contains legacy web mode code:
- Go backend server
- Rust HTTP server
- Kubernetes Helm charts
- Deployment scripts

**These are for reference only and are no longer maintained.**

## Key Features

- MySQL database management (browse, query, CRUD)
- Redis key-value management
- Kubernetes cluster resource visualization
- Automatic port forwarding for K8s services
- Query history and saved queries
- Dark theme UI

## 开发指南 - 双模式 API

本项目支持两种 API 调用模式，开发新接口时**必须同时注册两个入口**：

### 架构

```
IPC 调用 (just dev)           HTTP 调用 (just web)
       ↓                              ↓
Tauri Commands                  Axum Routes
(src-tauri/src/commands/)      (src-tauri/src/http/mod.rs)
       ↓                              ↓
       └──────────┬───────────────────┘
                  ↓
          共用服务层 (Services)
          src-tauri/src/services/
```

### 新增接口检查清单

开发新功能时，需要更新以下文件：

| 层级 | 文件 | 说明 |
|------|------|------|
| 服务层 | `src-tauri/src/services/*.rs` | 业务逻辑实现 |
| IPC 入口 | `src-tauri/src/commands/*.rs` | Tauri command 定义 |
| IPC 注册 | `src-tauri/src/lib.rs` | `generate_handler![]` 中注册 |
| HTTP 入口 | `src-tauri/src/http/mod.rs` | Axum route handler |
| 前端类型 | `frontend/src/api/types.ts` | TypeScript 接口定义 |
| IPC 适配器 | `frontend/src/api/adapters/ipc/index.ts` | IPC 调用实现 |
| HTTP 适配器 | `frontend/src/api/adapters/http/index.ts` | HTTP 调用实现 |

### 示例：添加新的查询接口

1. **服务层** (`services/mysql.rs`):
   ```rust
   pub async fn my_new_query(&self) -> AppResult<MyResult> { ... }
   ```

2. **IPC 命令** (`commands/mysql.rs`):
   ```rust
   #[tauri::command]
   pub async fn mysql_my_new_query(...) -> Result<MyResult, String> { ... }
   ```

3. **注册 IPC** (`lib.rs`):
   ```rust
   .invoke_handler(tauri::generate_handler![
       commands::mysql_my_new_query,  // 添加这行
   ])
   ```

4. **HTTP 路由** (`http/mod.rs`):
   ```rust
   .route("/api/mysql/my-new-query", get(mysql_my_new_query))

   async fn mysql_my_new_query(...) -> Result<Json<MyResult>, AppError> { ... }
   ```

5. **前端适配器** (IPC 和 HTTP 两个文件都要更新)
