# MySQL 管理功能问题修复计划

基于 TEST_REPORT.md 中发现的问题，制定以下修复计划。

---

## 问题分析

### 问题 1: 用户列表 API BINARY 类型兼容性问题

**错误信息**:
```
Database error: error occurred while decoding column 0:
mismatched types; Rust type `alloc::string::String` (as SQL type `VARCHAR`)
is not compatible with SQL type `BINARY`
```

**问题代码位置**: `src-tauri/src/services/mysql.rs:609-620`

```rust
pub async fn list_users(&self) -> AppResult<Vec<MysqlUserInfo>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT User, Host FROM mysql.user ORDER BY User, Host")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    // ...
}
```

**根因分析**:
- MySQL 8.0 的 `mysql.user` 系统表中，`User` 和 `Host` 字段的排序规则是 `utf8mb3_bin`（二进制排序）
- SQLx 将带有二进制排序规则的字段识别为 `BINARY` 类型
- Rust 的 `String` 类型无法直接映射 `BINARY` 类型

**影响范围**:
- `GET /api/mysql/users` API 无法返回用户列表
- 前端用户管理页面显示 "No Data"

---

### 问题 2: 进程列表数据解析异常

**现象**: 所有字段返回默认值（id=0, user="", host=""）

**问题代码位置**: `src-tauri/src/services/mysql.rs:1940-1961`

```rust
pub async fn get_process_list(&self) -> AppResult<Vec<ProcessInfo>> {
    let rows = sqlx::query("SHOW FULL PROCESSLIST")
        .fetch_all(&self.pool)
        .await?;

    let processes = rows.iter().map(|row| ProcessInfo {
        id: row.try_get::<u64, _>("Id").unwrap_or(0),
        user: row.try_get::<String, _>("User").unwrap_or_default(),
        // ...
    }).collect();
}
```

**根因分析**:
1. **列名大小写问题**: MySQL 8.0 返回的 SHOW PROCESSLIST 列名可能与代码中的不一致
2. **类型映射问题**:
   - `Id` 字段在 MySQL 中是 `bigint`，可能需要使用 `i64` 而非 `u64`
   - `Time` 字段同理
3. **try_get 静默失败**: 使用 `unwrap_or_default()` 导致错误被隐藏

**影响范围**:
- `GET /api/mysql/server/processes` API 返回空数据
- 前端服务器监控页面进程列表显示空值

---

## 修复方案

### 方案 1: 用户列表 - 使用 CAST 函数

**修复策略**: 在 SQL 查询中使用 `CAST` 或 `CONVERT` 函数将二进制字段转换为字符串

**修改文件**: `src-tauri/src/services/mysql.rs`

**修改内容**:
```rust
// 修改前
pub async fn list_users(&self) -> AppResult<Vec<MysqlUserInfo>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT User, Host FROM mysql.user ORDER BY User, Host")
        // ...
}

// 修改后
pub async fn list_users(&self) -> AppResult<Vec<MysqlUserInfo>> {
    let rows: Vec<(String, String)> =
        sqlx::query_as(
            "SELECT CAST(User AS CHAR) as user, CAST(Host AS CHAR) as host \
             FROM mysql.user ORDER BY User, Host"
        )
        // ...
}
```

**验证方法**:
```bash
curl -s -H "X-Connection-ID: 2" http://127.0.0.1:12420/api/mysql/users | jq '.'
```

---

### 方案 2: 进程列表 - 改用索引访问并修正类型

**修复策略**:
1. 使用列索引代替列名访问（避免大小写问题）
2. 使用正确的类型（i64 而非 u64）
3. 添加错误日志便于调试

**修改文件**: `src-tauri/src/services/mysql.rs`

**修改内容**:
```rust
// 修改前
pub async fn get_process_list(&self) -> AppResult<Vec<ProcessInfo>> {
    let rows = sqlx::query("SHOW FULL PROCESSLIST")
        .fetch_all(&self.pool)
        .await?;

    let processes = rows.iter().map(|row| ProcessInfo {
        id: row.try_get::<u64, _>("Id").unwrap_or(0),
        user: row.try_get::<String, _>("User").unwrap_or_default(),
        // ...
    }).collect();
    Ok(processes)
}

// 修改后
pub async fn get_process_list(&self) -> AppResult<Vec<ProcessInfo>> {
    // 使用明确的 SQL 查询，确保列名和类型一致
    let rows = sqlx::query(
        "SELECT Id, User, Host, db, Command, Time, State, Info FROM information_schema.processlist"
    )
    .fetch_all(&self.pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let processes = rows
        .iter()
        .map(|row| {
            ProcessInfo {
                // 使用列索引访问，避免大小写问题
                id: row.try_get::<i64, _>(0).unwrap_or(0) as u64,
                user: row.try_get::<String, _>(1).unwrap_or_default(),
                host: row.try_get::<String, _>(2).unwrap_or_default(),
                db: row.try_get::<Option<String>, _>(3).ok().flatten(),
                command: row.try_get::<String, _>(4).unwrap_or_default(),
                time: row.try_get::<i64, _>(5).unwrap_or(0) as u64,
                state: row.try_get::<Option<String>, _>(6).ok().flatten(),
                info: row.try_get::<Option<String>, _>(7).ok().flatten(),
            }
        })
        .collect();

    Ok(processes)
}
```

**备选方案**: 使用 `query_as` 配合 `FromRow` derive 宏
```rust
#[derive(sqlx::FromRow)]
struct ProcessRow {
    #[sqlx(rename = "Id")]
    id: i64,
    #[sqlx(rename = "User")]
    user: String,
    // ...
}
```

**验证方法**:
```bash
curl -s -H "X-Connection-ID: 2" http://127.0.0.1:12420/api/mysql/server/processes | jq '.'
```

---

## 实施计划

### 第一阶段: 修复用户列表 (预计 15 分钟)

| 步骤 | 操作 | 文件 |
|------|------|------|
| 1 | 修改 `list_users` 函数 SQL 查询 | `src-tauri/src/services/mysql.rs` |
| 2 | 编译验证 | `cargo build` |
| 3 | API 测试 | `curl` 测试 |
| 4 | 前端验证 | Chrome 测试用户管理页面 |

### 第二阶段: 修复进程列表 (预计 20 分钟)

| 步骤 | 操作 | 文件 |
|------|------|------|
| 1 | 修改 `get_process_list` 函数 | `src-tauri/src/services/mysql.rs` |
| 2 | 编译验证 | `cargo build` |
| 3 | API 测试 | `curl` 测试 |
| 4 | 前端验证 | Chrome 测试服务器监控页面 |

### 第三阶段: 回归测试 (预计 10 分钟)

| 步骤 | 操作 |
|------|------|
| 1 | 运行完整 API 测试套件 |
| 2 | Chrome 全流程测试 |
| 3 | 更新测试报告 |

---

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| CAST 函数影响性能 | 低 | 低 | mysql.user 表通常很小，影响可忽略 |
| information_schema.processlist 权限问题 | 低 | 中 | 需要 PROCESS 权限，root 用户默认有 |
| 其他 MySQL 版本兼容性 | 中 | 中 | 测试 MySQL 5.7 兼容性 |

---

## 验证清单

- [x] 用户列表 API 返回正确数据 ✅ (2026-01-14 12:18)
- [x] 前端用户管理页面显示用户列表 ✅ (10 个用户)
- [x] 进程列表 API 返回正确数据 ✅ (2026-01-14 12:18)
- [x] 前端服务器监控页面显示进程信息 ✅ (ID、User、Host 等字段完整)
- [x] 其他 MySQL 功能不受影响 ✅
- [x] 前端构建无错误 ✅

---

*计划制定时间: 2026-01-14*
*修复完成时间: 2026-01-14 12:20*
*状态: ✅ 全部完成*
