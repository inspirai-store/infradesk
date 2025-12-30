# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

This is a monorepo with:
- `frontend/` - Vue 3 + TypeScript + Vite frontend (runs on port 15073)
- `backend/` - Go + Gin backend API (runs on port 15080)

## Development Commands

### Frontend (in `frontend/`)
```bash
npm run dev        # Start dev server on port 15073
npm run build      # Production build (vue-tsc + vite build)
npm run test       # Run tests in watch mode (vitest)
npm run test:run   # Run tests once
npm run lint       # ESLint with auto-fix
```

### Backend (in `backend/`)
```bash
go run main.go              # Start server on port 15080
go build -o zeni-x ./...    # Build binary
go test ./...               # Run tests
```

## Architecture Overview

### Frontend Architecture

**Connection Management Pattern:**
- All API requests to MySQL/Redis use `X-Connection-ID` header injection via axios interceptor
- Active connection IDs are managed by `src/stores/connections.ts`
- Call `setActiveConnectionId(type, id)` to set active connection for a type
- `getActiveConnectionId(type)` retrieves the active ID

**State Management (Pinia):**
- `connections.ts` - Manages connection configurations and active connections per type
- `mysql.ts` - MySQL-specific state (databases, tables, schema, users) + query limit
- `redis.ts` - Redis-specific state (keys, values)
- `history.ts` - Query history tracking
- `llm.ts` - LLM configuration for AI assistant

**Routing:**
- Nested routes for MySQL (`/mysql/:database/:table`)
- Query editor at `/mysql/query`
- Redis key browser at `/redis` with detail view at `/redis/key/:key`

**Key Components:**
- `MonacoSQLEditor.vue` - Monaco-based SQL editor with auto-completion
- `TableDataEditor.vue` - Editable data table with cell modification tracking
- `QueryHistoryPanel.vue` - Query history and saved queries management
- `AIAssistantPanel.vue` - AI-powered SQL generation and optimization

### Backend Architecture

**Handler Pattern:**
- `internal/api/mysql.go` - MySQLHandler with sqlite DB for history/saved queries
- `internal/api/redis.go` - RedisHandler
- `internal/api/k8s.go` - K8sHandler for service discovery
- `internal/api/portforward.go` - PortForwardHandler for K8s port forwarding
- `internal/api/cluster.go` - ClusterHandler for K8s cluster management

**Service Layer:**
- `internal/service/mysql.go` - MySQLService (direct DB connections, uses connection credentials from sqlite)
- `internal/service/redis.go` - RedisService
- `internal/service/discovery.go` - K8s service discovery
- `internal/service/forward_monitor.go` - Background monitor for idle port forwards

**K8s Integration:**
- `internal/k8s/client.go` - K8s client initialization
- `internal/k8s/portforward.go` - PortForwardManager manages active forwards with idle timeout

**Storage:**
- `internal/store/sqlite.go` - SQLite for connections, clusters, query history, saved queries, port forwards

**API Routing:**
- All routes under `/api/` prefix
- Type-specific routes: `/api/mysql/*`, `/api/redis/*`, `/api/k8s/*`
- Connection management: `/api/connections/*`
- Port forwarding: `/api/port-forward/*`

## Important Patterns

### MySQL Query Limit
The MySQL store auto-applies LIMIT clauses to SELECT queries:
- `store.applyLimit(sql)` adds LIMIT (default 100) to SELECT queries without LIMIT
- `store.extractBaseQuery(sql)` removes LIMIT/OFFSET for pagination
- `store.queryLimit` is configurable (10-1000) via `store.saveQueryLimit(limit)`
- Stored in localStorage at key `mysql-query-limit`

### Port Forwarding Flow
1. User discovers K8s services via `/api/k8s/discover`
2. Services are imported as connections with k8s metadata
3. When using a K8s connection, `PortForwardManager` creates local forward
4. ForwardMonitor tracks last-used time and stops idle forwards
5. Connection credentials use `forward_local_port` when available

### MySQL Direct Table Editing
- `TableDataEditor.vue` tracks cell modifications in a Map
- `getTablePrimaryKey(database, table)` API gets primary key column
- `updateRecord(database, table, primaryKey, primaryValue, updates)` API updates single row
- Modifications are grouped by row before saving

## Type Safety

### Frontend API Types
All API request/response types are defined in `src/api/index.ts`:
- `Connection`, `Cluster`, `DiscoveredService`, `ForwardInfo`
- `CreateDatabaseRequest`, `AlterTableRequest`, `SetKeyRequest`
- `QueryHistory`, `SavedQuery`, etc.

### Go Struct Tags
- JSON tags use `snake_case` for API responses
- SQLite struct tags use `snake_case` for DB columns

## Configuration

- Frontend proxy: `/api` → `http://localhost:15080` (vite.config.ts)
- Backend config: `internal/config/config.go` (supports environment variables and viper)
- Database: SQLite file at `data/zeni-x.db` (backend)
