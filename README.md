# Zeni-X Database Manager

一个现代化的数据库管理平台，用于管理 K8s 集群中的 MySQL、Redis、MongoDB 和 MinIO 服务。

## 功能特性

- 🗄️ **MySQL 管理**: 数据库/表/数据 CRUD、表结构管理、SQL 查询器
- 🔴 **Redis 管理**: Key 浏览器、多数据类型支持、TTL 管理
- 📊 **数据导入导出**: 支持 CSV/JSON/SQL 格式
- 🎨 **现代化 UI**: 暗色主题、赛博朋克风格

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Vue.js 3 + TypeScript + Naive UI + Vite |
| 后端 | Go + Gin + SQLite |
| 部署 | K8s + Helm |

## 快速开始

### 环境要求

- Go 1.21+
- Node.js 18+
- pnpm

### 安装依赖

```bash
make install
```

### 本地开发

```bash
make dev
```

前端访问: http://localhost:15073
后端 API: http://localhost:15080

## 构建部署

### 构建生产版本

```bash
make build
```

### 部署到测试环境

```bash
make test
```

### 部署到 UAT 环境

```bash
make uat
```

## 目录结构

```
zeni-x/
├── Makefile              # 构建入口
├── services/
│   └── zeni-x/
│       ├── frontend/     # Vue.js 前端
│       └── backend/      # Go 后端
├── helm/                 # Helm Charts
│   └── zeni-x/
│       ├── Chart.yaml    # Chart 元数据
│       ├── values.yaml   # 默认配置
│       ├── values-*.yaml # 环境配置
│       └── templates/    # K8s 资源模板
├── config/               # 配置文件
│   ├── env/
│   └── backend/
└── scripts/              # 辅助脚本
```

## 配置

### 敏感信息配置

首次部署前，需要创建各环境的敏感配置文件：

```bash
# 创建测试环境敏感配置
cp helm/zeni-x/values-test.secret.example helm/zeni-x/values-test.secret.yaml
vim helm/zeni-x/values-test.secret.yaml

# 创建 UAT 环境敏感配置
cp helm/zeni-x/values-uat.secret.example helm/zeni-x/values-uat.secret.yaml
vim helm/zeni-x/values-uat.secret.yaml
```

### 环境变量

应用配置通过 Helm values 文件管理，敏感信息通过 K8s Secrets 注入：

- `DATABASE_PASSWORD`: 数据库密码
- `API_KEY`: API 密钥
- `REDIS_PASSWORD`: Redis 密码（可选）

## License

MIT

