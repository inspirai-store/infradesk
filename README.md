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
| 部署 | K8s + Kustomize |

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
├── k8s/                  # K8s 配置（kustomize）
│   ├── base/
│   └── overlays/
├── config/               # 配置文件
│   ├── env/
│   └── backend/
└── scripts/              # 辅助脚本
```

## 配置

环境变量通过 K8s Secrets 注入：

- `MYSQL_ROOT_PASSWORD`: MySQL root 密码
- `REDIS_PASSWORD`: Redis 密码

## License

MIT

