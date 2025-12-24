# 集群管理和连接来源功能

## 概述

本文档描述了集群管理和连接来源（Source）功能的实现。该功能允许用户：
1. 管理多个 K8s 集群信息
2. 将连接关联到特定集群
3. 通过来源字段区分本地连接和 K8s 连接
4. 按集群分组查看连接

## 数据库结构

### Clusters 表

新增 `clusters` 表用于存储集群信息：

```sql
CREATE TABLE clusters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,          -- 集群名称
    description TEXT,                    -- 集群描述
    environment TEXT,                    -- 环境: dev, test, uat, prod
    context TEXT,                        -- K8s context
    api_server TEXT,                     -- K8s API Server 地址
    is_active BOOLEAN DEFAULT 1,         -- 是否激活
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Connections 表更新

在 `connections` 表中添加了两个新字段：

```sql
ALTER TABLE connections ADD COLUMN cluster_id INTEGER;
ALTER TABLE connections ADD COLUMN source TEXT DEFAULT 'local';
```

- **cluster_id**: 外键，关联到 clusters 表（可为 NULL）
- **source**: 连接来源，可选值：
  - `local` - 本地手动创建的连接
  - `k8s` - 从 K8s 集群导入的连接

## API 接口

### 集群管理 API

#### 1. 获取所有集群
```
GET /api/clusters
```

响应示例：
```json
[
  {
    "id": 1,
    "name": "test-cluster",
    "description": "测试环境集群",
    "environment": "test",
    "context": "test-context",
    "api_server": "https://test-k8s.example.com",
    "is_active": true,
    "created_at": "2025-12-24T12:49:50Z",
    "updated_at": "2025-12-24T12:49:50Z"
  }
]
```

#### 2. 获取单个集群
```
GET /api/clusters/:id
```

#### 3. 创建集群
```
POST /api/clusters
Content-Type: application/json

{
  "name": "prod-cluster",
  "description": "生产环境集群",
  "environment": "prod",
  "context": "prod-context",
  "api_server": "https://prod-k8s.example.com",
  "is_active": true
}
```

#### 4. 更新集群
```
PUT /api/clusters/:id
Content-Type: application/json

{
  "name": "prod-cluster",
  "description": "生产环境集群（更新）",
  "environment": "prod",
  "is_active": true
}
```

#### 5. 删除集群
```
DELETE /api/clusters/:id
```

注意：如果集群下有关联的连接，删除会失败。需要先删除或解除连接关联。

#### 6. 获取集群下的所有连接
```
GET /api/clusters/:id/connections
```

响应示例：
```json
[
  {
    "id": 1,
    "name": "backup/game-mysql",
    "type": "mysql",
    "host": "localhost",
    "port": 33061,
    "cluster_id": 1,
    "source": "k8s",
    "k8s_namespace": "backup",
    "k8s_service_name": "game-mysql",
    "created_at": "2025-12-24T12:50:03Z"
  }
]
```

### 连接 API 更新

#### Connection 对象新增字段

```json
{
  "id": 1,
  "name": "my-connection",
  "type": "mysql",
  "host": "localhost",
  "port": 3306,
  "cluster_id": 1,        // 新增：关联的集群 ID
  "source": "local",      // 新增：连接来源
  ...
}
```

## K8s 导入集成

### 导入时自动创建/关联集群

当从 K8s 导入连接时，系统会：

1. **检查集群是否存在**：根据 `cluster_name` 查找
2. **自动创建集群**：如果不存在，创建新的集群记录
3. **关联连接**：将导入的连接关联到该集群
4. **标记来源**：自动将 `source` 设置为 `k8s`

#### 导入请求示例

```json
POST /api/k8s/import
Content-Type: application/json

{
  "cluster_name": "test-cluster",
  "context": "test-context",
  "force_override": true,
  "services": [
    {
      "name": "game-mysql",
      "type": "mysql",
      "namespace": "backup",
      "host": "game-mysql.backup.svc.cluster.local",
      "port": 3306,
      "service_name": "game-mysql",
      "username": "root",
      "password": "password123"
    }
  ]
}
```

### 导入流程

```
1. 接收导入请求（包含 cluster_name）
   ↓
2. 查找或创建集群记录
   ↓
3. 遍历服务列表
   ↓
4. 为每个服务创建连接
   - cluster_id = 集群 ID
   - source = "k8s"
   - k8s_namespace, k8s_service_name 等
   ↓
5. 返回导入结果
```

## 使用场景

### 场景 1: 多集群环境管理

组织有多个 K8s 集群：

```
开发集群 (dev-cluster)
  ├── dev/mysql-1
  ├── dev/redis-1
  └── dev/mongodb-1

测试集群 (test-cluster)
  ├── test/mysql-1
  └── test/redis-1

生产集群 (prod-cluster)
  ├── prod/mysql-main
  ├── prod/mysql-read
  └── prod/redis-cache
```

### 场景 2: 混合连接管理

同时管理本地和集群连接：

```
本地连接 (source=local, cluster_id=null)
  ├── 本地 MySQL
  └── 本地 Redis

测试集群连接 (source=k8s, cluster_id=1)
  ├── test/game-mysql
  └── test/game-redis
```

### 场景 3: 环境隔离

按环境区分集群：

```sql
-- 查看测试环境的所有集群
SELECT * FROM clusters WHERE environment = 'test';

-- 查看生产环境集群的所有连接
SELECT c.* FROM connections c
JOIN clusters cl ON c.cluster_id = cl.id
WHERE cl.environment = 'prod';
```

## Connection 结构完整示例

### 本地连接
```json
{
  "id": 1,
  "name": "local-mysql",
  "type": "mysql",
  "host": "127.0.0.1",
  "port": 3306,
  "username": "root",
  "source": "local",
  "cluster_id": null,
  "k8s_namespace": "",
  "k8s_service_name": ""
}
```

### K8s 连接
```json
{
  "id": 2,
  "name": "backup/game-mysql",
  "type": "mysql",
  "host": "localhost",
  "port": 33061,
  "username": "root",
  "source": "k8s",
  "cluster_id": 1,
  "k8s_namespace": "backup",
  "k8s_service_name": "game-mysql",
  "k8s_service_port": 3306,
  "forward_status": "active",
  "forward_id": "pf-123"
}
```

## 前端集成建议

### 1. 连接列表分组显示

```typescript
// 按集群和来源分组显示连接
interface ConnectionGroup {
  cluster?: Cluster
  source: 'local' | 'k8s'
  connections: Connection[]
}

// 示例显示结构
本地连接
  ├── local-mysql
  └── local-redis

测试集群 (test-cluster)
  ├── test/mysql-1
  └── test/redis-1

生产集群 (prod-cluster)
  ├── prod/mysql-main
  └── prod/redis-cache
```

### 2. 集群选择器

```vue
<template>
  <select v-model="selectedCluster">
    <option :value="null">所有集群</option>
    <option :value="0">本地连接</option>
    <option v-for="cluster in clusters" :key="cluster.id" :value="cluster.id">
      {{ cluster.name }} ({{ cluster.environment }})
    </option>
  </select>
</template>
```

### 3. 连接标签

```vue
<template>
  <div class="connection-card">
    <div class="connection-name">{{ connection.name }}</div>
    <div class="connection-tags">
      <span class="tag" :class="`source-${connection.source}`">
        {{ connection.source }}
      </span>
      <span v-if="connection.cluster_id" class="tag cluster">
        {{ getClusterName(connection.cluster_id) }}
      </span>
    </div>
  </div>
</template>
```

## 数据迁移

### 为现有连接设置默认值

```sql
-- 将所有现有连接标记为本地连接
UPDATE connections SET source = 'local' WHERE source IS NULL;

-- 清理没有集群信息的 K8s 连接
UPDATE connections 
SET source = 'local', 
    cluster_id = NULL 
WHERE k8s_namespace = '' AND k8s_service_name = '';
```

## API 测试示例

### 完整测试流程

```bash
# 1. 创建集群
curl -X POST http://localhost:15080/api/clusters \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-cluster",
    "description": "测试集群",
    "environment": "test",
    "is_active": true
  }'

# 2. 获取所有集群
curl http://localhost:15080/api/clusters

# 3. 创建关联到集群的连接
curl -X POST http://localhost:15080/api/connections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-mysql",
    "type": "mysql",
    "host": "localhost",
    "port": 3306,
    "cluster_id": 1,
    "source": "local"
  }'

# 4. 获取集群下的所有连接
curl http://localhost:15080/api/clusters/1/connections

# 5. K8s 导入（自动创建集群）
curl -X POST http://localhost:15080/api/k8s/import \
  -H "Content-Type: application/json" \
  -d '{
    "cluster_name": "prod-cluster",
    "context": "prod-context",
    "services": [...]
  }'

# 6. 删除集群（需要先删除关联的连接）
curl -X DELETE http://localhost:15080/api/clusters/1
```

## 注意事项

1. **外键约束**：集群删除时会自动将关联连接的 `cluster_id` 设置为 NULL
2. **唯一性**：集群名称必须唯一
3. **默认值**：
   - 新建本地连接时，`source` 默认为 `local`
   - K8s 导入时，`source` 自动设为 `k8s`
4. **查询性能**：已为 `cluster_id` 和 `source` 字段创建索引

## 相关文件

- `backend/internal/store/sqlite.go` - 数据库结构和 CRUD 操作
- `backend/internal/api/cluster.go` - 集群管理 API
- `backend/internal/api/k8s.go` - K8s 导入逻辑更新
- `backend/internal/api/router.go` - 路由配置

## 后续优化建议

1. **环境管理**：添加环境配置表，标准化环境名称
2. **集群监控**：添加集群健康状态检查
3. **批量操作**：支持批量导入多个集群
4. **权限控制**：按集群配置访问权限
5. **统计信息**：集群连接数、使用频率等统计

