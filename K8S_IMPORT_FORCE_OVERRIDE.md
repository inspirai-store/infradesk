# K8s 服务导入 - 强制覆盖功能

## 功能说明

当从 K8s 集群导入服务时，如果发现连接已存在（相同的 host 和 port），系统会：

1. **首次导入**：自动检测重复连接并跳过
2. **二次确认**：弹出确认对话框，询问是否强制覆盖
3. **强制覆盖**：用户确认后，更新已存在的连接

## 用户体验流程

### 场景一：没有重复连接

```
用户点击"导入选中的服务"
  ↓
系统检查连接（无重复）
  ↓
全部导入成功 ✓
  ↓
显示：成功导入 X 个连接
```

### 场景二：有重复连接（新增功能）

```
用户点击"导入选中的服务"
  ↓
系统检查连接（发现重复）
  ↓
弹出确认对话框：
┌────────────────────────────────┐
│ ⚠️  发现重复连接                │
│                                │
│ 以下 14 个连接已存在：         │
│                                │
│ game-mysql, redis, mysql...   │
│                                │
│ 是否要强制覆盖这些已有连接？   │
│                                │
│   [取消]      [强制覆盖]       │
└────────────────────────────────┘
  ↓
用户选择：
```

**选项一：点击"取消"**
```
→ 只导入新连接
→ 跳过重复连接
→ 显示：成功导入 X 个新连接，跳过 Y 个重复连接
```

**选项二：点击"强制覆盖"**
```
→ 重新导入所有服务
→ 更新已存在的连接
→ 显示：成功导入连接：新建 X 个，覆盖 Y 个
```

## API 变更

### 请求参数

```typescript
// POST /api/k8s/import
{
  "services": [...],
  "force_override": false  // 新增：是否强制覆盖
}
```

### 响应格式

```typescript
{
  "success": 5,     // 成功导入的数量（新建 + 更新）
  "failed": 0,      // 失败的数量
  "updated": 3,     // 新增：覆盖更新的数量
  "skipped": 2,     // 新增：跳过的数量
  "results": [
    {
      "name": "mysql",
      "success": true,
      "updated": true,   // 新增：是否是更新操作
      "id": 123
    },
    {
      "name": "redis",
      "success": false,
      "skipped": true,   // 新增：是否被跳过
      "error": "connection already exists"
    }
  ]
}
```

## 代码变更

### 后端 (`internal/api/k8s.go`)

1. **新增请求字段**：
   ```go
   type ImportConnectionsRequest struct {
       Services      []ImportServiceItem
       ForceOverride bool  // 新增
   }
   ```

2. **新增响应字段**：
   ```go
   type ImportConnectionsResponse struct {
       Success int
       Failed  int
       Updated int  // 新增
       Skipped int  // 新增
       Results []ImportConnectionResult
   }
   
   type ImportConnectionResult struct {
       Name    string
       Success bool
       Updated bool  // 新增
       Skipped bool  // 新增
       Error   string
       ID      int64
   }
   ```

3. **导入逻辑**：
   ```go
   if existingConn != nil {
       if req.ForceOverride {
           // 强制覆盖：更新现有连接
           conn.ID = existingConn.ID
           h.db.UpdateConnection(&conn)
           result.Updated = true
       } else {
           // 不覆盖：跳过
           result.Skipped = true
       }
   }
   ```

### 前端 (`components/ServiceDiscovery.vue`)

1. **API 调用更新**：
   ```typescript
   // 第一次尝试（不强制覆盖）
   await k8sApi.importConnections(toImport, false)
   
   // 用户确认后（强制覆盖）
   await k8sApi.importConnections(toImport, true)
   ```

2. **二次确认对话框**：
   ```typescript
   dialog.warning({
     title: '发现重复连接',
     content: `以下 ${count} 个连接已存在...\n\n是否要强制覆盖？`,
     positiveText: '强制覆盖',
     negativeText: '取消',
     onPositiveClick: async () => {
       // 强制覆盖逻辑
     },
     onNegativeClick: () => {
       // 取消逻辑
     }
   })
   ```

## 使用示例

### 示例一：首次导入（无重复）

```bash
# 扫描集群
发现 5 个中间件服务

# 导入
成功导入 5 个连接 ✓
```

### 示例二：部分重复

```bash
# 扫描集群
发现 10 个中间件服务

# 导入
[弹出对话框]
发现重复连接：
- game-mysql
- redis
是否要强制覆盖？

→ 选择"取消"
  成功导入 8 个新连接，跳过 2 个重复连接

→ 选择"强制覆盖"
  成功导入连接：新建 8 个，覆盖 2 个
```

### 示例三：全部重复（用户场景）

```bash
# 用户需要更新所有连接配置（例如密码变更）

# 扫描集群
发现 14 个中间件服务

# 导入
[弹出对话框]
发现重复连接：
- game-mysql
- redis
- mysql
... (共 14 个)
是否要强制覆盖？

→ 选择"强制覆盖"
  成功导入连接：覆盖 14 个 ✓
```

## 技术细节

### 重复检测逻辑

连接被认为是重复的，当且仅当：
- `type` 相同
- `host` 相同
- `port` 相同

### 覆盖保留字段

强制覆盖时，以下字段会被保留：
- `id` - 连接 ID
- `is_default` - 默认连接标记

其他字段会被更新：
- `name` - 连接名称
- `username` - 用户名
- `password` - 密码
- `database_name` - 数据库名

### 日志记录

所有操作都会记录日志：

```go
// 创建新连接
log.Printf("Created new connection: %s (ID: %d)", connName, conn.ID)

// 更新现有连接
log.Printf("Updated existing connection: %s (ID: %d)", connName, conn.ID)

// 跳过重复连接
log.Printf("Skipped existing connection: %s", connName)
```

## 测试场景

### 测试 1：无重复连接
1. 清空所有连接
2. 扫描并导入 5 个服务
3. ✓ 全部导入成功，无对话框

### 测试 2：部分重复
1. 手动创建 2 个连接
2. 扫描并导入 5 个服务（包含这 2 个）
3. ✓ 弹出对话框显示 2 个重复
4. 点击"取消"
5. ✓ 导入 3 个新连接，跳过 2 个

### 测试 3：强制覆盖
1. 手动创建 2 个连接
2. 扫描并导入 5 个服务（包含这 2 个）
3. ✓ 弹出对话框显示 2 个重复
4. 点击"强制覆盖"
5. ✓ 导入 3 个新连接，覆盖 2 个
6. ✓ 验证连接信息已更新

### 测试 4：全部重复
1. 已有 14 个连接
2. 扫描并导入相同的 14 个服务
3. ✓ 弹出对话框显示 14 个重复
4. 点击"取消"
5. ✓ 显示"跳过 14 个重复连接"

## 用户反馈处理

原始问题：
```
14 个连接导入失败: game-mysql: connection already exists, ...
```

解决方案：
1. ✅ 自动检测重复连接
2. ✅ 提供二次确认对话框
3. ✅ 支持强制覆盖选项
4. ✅ 清晰的结果反馈

现在用户体验：
```
发现重复连接
[明确的对话框提示]
→ 用户可选择：取消 或 强制覆盖
→ 清晰的结果反馈（新建 X 个，覆盖 Y 个）
```

## 部署说明

1. **后端更新**：
   - 修改了 `internal/api/k8s.go`
   - 需要重新编译和部署

2. **前端更新**：
   - 修改了 `components/ServiceDiscovery.vue`
   - 修改了 `api/index.ts`
   - 需要重新构建和部署

3. **数据库**：
   - 无需变更

4. **兼容性**：
   - 向后兼容（`force_override` 默认为 `false`）
   - 老版本前端仍可正常使用（只是没有覆盖功能）

## 未来改进

1. **批量操作优化**
   - 支持选择性覆盖（只覆盖部分连接）
   - 显示连接差异对比

2. **冲突解决策略**
   - 提供更多选项：保留新的/保留旧的/合并
   - 支持按字段选择保留策略

3. **导入历史**
   - 记录导入历史
   - 支持回滚操作

## 总结

✅ **问题已解决**：用户可以强制覆盖已存在的连接
✅ **用户体验友好**：二次确认防止误操作
✅ **结果清晰**：明确显示新建、覆盖、跳过的数量
✅ **代码质量**：完整的错误处理和日志记录

