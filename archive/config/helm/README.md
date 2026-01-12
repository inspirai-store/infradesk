# Helm 配置说明

此目录用于存放各环境 Helm 部署的用户配置文件。这些文件不会被提交到 git。

## 快速开始

### 1. 创建环境配置文件

从 helm/zeni-x/ 目录复制对应的 values 文件：

```bash
# Test 环境
cp helm/zeni-x/values-test.yaml config/helm/values-test.yaml

# UAT 环境
cp helm/zeni-x/values-uat.yaml config/helm/values-uat.yaml

# Prod 环境
cp helm/zeni-x/values-prod.yaml config/helm/values-prod.yaml
```

### 2. 编辑 values 文件（可选）

编辑 `values-test.yaml` 覆盖默认配置：

```yaml
# 镜像配置（可选）
image:
  frontend:
    tag: test
  backend:
    tag: test

# 副本数（可选）
replicaCount: 1
```

### 3. 部署

```bash
# 部署到 Test 环境
make helm-test

# 部署到 UAT 环境
make helm-uat

# 部署到 Prod 环境
make helm-prod
```

### 4. 生成部署清单（Dry-run）

```bash
# 生成 test 环境部署清单
make helm-test-dryrun

# 生成 uat 环境部署清单
make helm-uat-dryrun

# 生成 prod 环境部署清单
make helm-prod-dryrun
```

生成的清单文件位于 `debug/{env}/manifests.yaml`。

## 文件说明

| 文件 | 说明 | 是否提交 |
|------|------|----------|
| `values-test.yaml` | Test 环境用户配置 | ❌ 否 |
| `values-uat.yaml` | UAT 环境用户配置 | ❌ 否 |
| `values-prod.yaml` | Prod 环境用户配置 | ❌ 否 |
| `.gitignore` | Git 忽略规则 | ✅ 是 |

## 配置优先级

Helm 部署时会按以下优先级加载配置文件：

1. `config/helm/values-{ENV}.yaml` (用户自定义，优先级最高)
2. `helm/zeni-x/values-{ENV}.yaml` (默认配置)

如果需要在特定环境覆盖默认配置，在 `config/helm/` 目录创建对应的 values 文件即可。

## 目录结构

```
config/helm/
├── .gitignore              # 忽略用户配置文件
├── values-test.yaml        # 用户创建：test 环境配置（不提交）
├── values-uat.yaml         # 用户创建：uat 环境配置（不提交）
├── values-prod.yaml        # 用户创建：prod 环境配置（不提交）
└── README.md              # 本文件
```
