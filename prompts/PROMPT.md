# Task: Convert Zeni-X Web App to Cross-Platform Desktop Application with Tauri

Transform the Zeni-X database management platform from a Web application + K8s deployment architecture to a cross-platform desktop application using Tauri 2.0 framework (Rust + Vue 3), following TDD development practices throughout.

## Requirements

### Phase 1: Project Initialization & Infrastructure
- [ ] Initialize Tauri 2.0 project structure with `pnpm create tauri-app --rc`
- [ ] Configure `src-tauri/tauri.conf.json` to point to existing frontend directory
- [ ] Configure Cargo.toml with dependencies (sqlx, redis, keyring, serde, thiserror)
- [ ] Verify `pnpm tauri dev` launches blank application successfully
- [ ] Install `@tauri-apps/api` and `@tauri-apps/plugin-*` frontend dependencies
- [ ] Modify `vite.config.ts` to add Tauri dev server configuration
- [ ] Create `src/utils/platform.ts` for runtime environment detection (Tauri/Web)

### Phase 2: Tauri API Adapter Layer (TDD)
- [ ] Write tests for `isTauri()` environment detection function
- [ ] Write tests for `resolveCommand()` route mapping function
- [ ] Write tests for `tauriRequest()` request wrapper function
- [ ] Create `src/api/tauri-adapter.ts` implementing API route to Tauri command mapping
- [ ] Modify `src/api/index.ts` to add Tauri interceptor for axios
- [ ] Ensure Web environment maintains original HTTP behavior

### Phase 3: Rust Data Models (TDD)
- [ ] Write Connection model serialization/deserialization tests
- [ ] Write QueryResult model tests
- [ ] Write DbType enum tests
- [ ] Create `src-tauri/src/models/connection.rs` with Connection, DbType
- [ ] Create `src-tauri/src/models/query.rs` with QueryResult, ColumnInfo, TableInfo
- [ ] Create `src-tauri/src/models/result.rs` with RedisKey, RedisValue

### Phase 4: Local Storage Service (TDD)
- [ ] Write SQLite connection initialization tests
- [ ] Write connections table CRUD tests
- [ ] Write query_history table operation tests
- [ ] Create `src-tauri/src/services/storage.rs` with SQLite initialization and migrations
- [ ] Implement Connection, QueryHistory, SavedQuery CRUD operations
- [ ] Write keychain access tests (using mocks)
- [ ] Create `src-tauri/src/utils/keychain.rs` for cross-platform keychain access

### Phase 5: MySQL Service Layer (TDD)
- [ ] Write `apply_limit()` function tests
- [ ] Write connection pool management tests
- [ ] Write `get_databases()` tests with mocks
- [ ] Write `execute_query()` tests with mocks
- [ ] Create `src-tauri/src/services/mysql.rs` with connection pool management
- [ ] Implement `get_databases()`, `get_tables()`, `execute_query()` functions
- [ ] Implement table data CRUD operations

### Phase 6: Redis Service Layer (TDD)
- [ ] Write connection management tests
- [ ] Write `scan_keys()` tests
- [ ] Write `get_value()` tests for multiple data types
- [ ] Write `set_value()` and `delete_key()` tests
- [ ] Create `src-tauri/src/services/redis.rs` with all Redis operations

### Phase 7: Tauri Command Layer (TDD)
- [ ] Write connection management command tests (get/create/test/delete)
- [ ] Create `src-tauri/src/commands/connection.rs` with Tauri commands
- [ ] Write MySQL command tests (databases/tables/query)
- [ ] Create `src-tauri/src/commands/mysql.rs` with Tauri commands
- [ ] Write Redis command tests (keys/value operations)
- [ ] Create `src-tauri/src/commands/redis.rs` with Tauri commands
- [ ] Register all commands in `main.rs` invoke_handler

### Phase 8: Error Handling (TDD)
- [ ] Create `src-tauri/src/utils/error.rs` with AppError enum using thiserror
- [ ] Implement Serialize trait for IPC transport
- [ ] Implement `user_message()` for user-friendly error messages
- [ ] Create `src/utils/error-handler.ts` for frontend error handling
- [ ] Integrate with Naive UI message notifications

### Phase 9: Frontend Integration Testing
- [ ] Write MySQL Store integration tests with Tauri adapter
- [ ] Write Redis Store integration tests with Tauri adapter
- [ ] Write Connection Store integration tests with Tauri adapter
- [ ] Test Monaco SQL editor functionality in desktop environment
- [ ] Verify all Naive UI components render correctly

### Phase 10: Application Packaging & Updates
- [ ] Create platform-specific icons (icns, ico, png)
- [ ] Configure tauri.conf.json product metadata
- [ ] Configure window default size and behavior
- [ ] Configure Tauri updater plugin with update server endpoints
- [ ] Implement update check UI and download/install prompts
- [ ] Configure GitHub Actions for multi-platform builds
- [ ] Generate DMG, MSI, DEB installation packages

### Phase 11: End-to-End Testing
- [ ] Write E2E test: Create connection -> Test connection -> Save connection
- [ ] Write E2E test: Select connection -> View databases -> View tables -> Query data
- [ ] Write E2E test: Edit data -> Save modifications
- [ ] Write E2E test: Query history save and load
- [ ] Ensure Rust test coverage >= 80%
- [ ] Ensure frontend test coverage >= 80%

## Technical Specifications

- **Desktop Framework**: Tauri 2.0 (Rust backend + WebView frontend)
- **Frontend**: Vue 3 + TypeScript + Vite (reuse existing codebase)
- **UI Components**: Naive UI (preserve existing components)
- **SQL Editor**: Monaco Editor (preserve existing functionality)
- **State Management**: Pinia (reuse existing stores)
- **Rust Dependencies**: sqlx (MySQL), redis, keyring, rusqlite, serde, thiserror, tokio
- **Local Storage**: SQLite for connections/history, system keychain for passwords
- **Testing**: Vitest + Vue Test Utils (frontend), Rust built-in test framework (backend)
- **Build Targets**: macOS (DMG), Windows (MSI), Linux (DEB)
- **Package Size Target**: < 20MB per platform

### Architecture Overview

```
Frontend (WebView) -> Tauri IPC (invoke) -> Rust Backend
     Vue 3                                  Commands -> Services -> DB Drivers
     Naive UI                               mysql.rs    mysql.rs   sqlx
     Monaco                                 redis.rs    redis.rs   redis
     Pinia                                  storage.rs  storage.rs rusqlite + keyring
```

### Directory Structure

```
zeni-x/
├── src-tauri/                    # Tauri Rust backend
│   ├── src/
│   │   ├── commands/             # Tauri commands (mysql, redis, connection, storage)
│   │   ├── services/             # Business logic layer
│   │   ├── models/               # Data models
│   │   └── utils/                # Utilities (error, crypto, keychain)
│   └── icons/                    # Application icons
├── services/zeni-x/frontend/     # Frontend (mostly reused)
│   ├── src/
│   │   ├── api/
│   │   │   └── tauri-adapter.ts  # NEW: Tauri IPC adapter
│   │   └── utils/
│   │       └── platform.ts       # NEW: Platform detection
└── tests/                        # Test directory
    ├── rust/                     # Rust unit tests
    ├── frontend/                 # Frontend unit tests
    └── e2e/                      # End-to-end tests
```

## Success Criteria

- Application launches natively on macOS, Windows, and Linux with platform-appropriate window decorations
- Application startup completes within 3 seconds and shows main interface
- Packaged application size is less than 20MB per platform
- 90%+ of existing Vue 3 frontend code is reused without modification
- MySQL connection, database listing, table listing, and SQL query execution work correctly
- Redis connection, key scanning, value read/write work correctly for all data types
- Connection passwords are stored securely using system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Query history and saved queries persist across application restarts
- Automatic update checking and installation works on all platforms
- Rust backend test coverage >= 80%
- Frontend test coverage >= 80%
- All CI tests pass before code can be merged to main branch
- SQL query performance is within 1.2x of original Go backend
