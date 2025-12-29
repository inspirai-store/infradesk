# Requirements Document

## Introduction

本功能旨在为 Zeni-X 项目引入 Helm Chart 来管理不同环境（dev、test、uat、prod）的 Kubernetes 配置。目前项目使用 Kustomize 管理配置，但 Helm 提供了更强大的版本管理、依赖管理和环境差异化的能力。通过 Helm Charts，我们可以更方便地进行应用的打包、分发和部署。

## Requirements

### Requirement 1: Helm Chart 结构

**User Story:** 作为开发者，我希望有一个标准的 Helm Chart 结构，以便能够清晰地管理模板和配置。

#### Acceptance Criteria

1. WHEN 创建 Helm Chart 时 THEN 系统 SHALL 创建符合 Helm 最佳实践的目录结构（templates/、values/、Chart.yaml 等）
2. WHEN 组织模板文件时 THEN 系统 SHALL 按照资源类型分类（deployment、service、ingress、configmap、rbac 等）
3. WHEN 命名 Chart 时 THEN 系统 SHALL 使用 "zeni-x" 作为 Chart 名称

### Requirement 2: 多环境配置管理

**User Story:** 作为运维人员，我希望能够通过不同的 values 文件管理不同环境的配置，以便在同一套 Helm Chart 中支持 dev、test、uat、prod 环境。

#### Acceptance Criteria

1. WHEN 部署到不同环境时 THEN 系统 SHALL 通过独立的 values 文件（values-dev.yaml、values-test.yaml、values-uat.yaml、values-prod.yaml）管理环境特定配置
2. WHEN 使用 values 文件时 THEN 系统 SHALL 支持覆盖镜像仓库、镜像标签、副本数、资源限制等参数
3. WHEN 管理环境配置时 THEN 系统 SHALL 保持 values.yaml 作为默认配置基准
4. WHEN 环境配置差异较大时 THEN 系统 SHALL 支持通过 values 文件的嵌套结构组织配置

### Requirement 2.1: 跨集群配置管理

**User Story:** 作为运维人员，我希望能够管理跨多个 Kubernetes 集群的部署，以便 Dev/Test 环境部署到开发集群，UAT/Prod 环境部署到生产集群。

#### Acceptance Criteria

1. WHEN 部署到不同集群时 THEN 系统 SHALL 支持通过 values 文件配置集群特定的 Ingress Controller（Traefik/ALB/NGINX）
2. WHEN 部署到不同集群时 THEN 系统 SHALL 支持不同的域名配置（内网域名 vs 外网域名）
3. WHEN 部署到不同集群时 THEN 系统 SHALL 支持不同的镜像仓库配置（本地仓库 vs 生产仓库）
4. WHEN 部署到不同集群时 THEN 系统 SHALL 支持不同的 NodePort 范围配置
5. WHEN 配置 Ingress 时 THEN 系统 SHALL 支持集群特定的 annotations（如 ALB 的 scheme、target-type 等配置）
6. WHEN 部署到 Dev/Test 集群时 THEN 系统 SHALL 使用 Traefik Ingress Controller 和内网域名
7. WHEN 部署到 UAT/Prod 集群时 THEN 系统 SHALL 使用 AWS ALB 和公网域名
8. WHEN 配置资源时 THEN 系统 SHALL 为不同集群设置不同的资源限制（开发集群较低，生产集群较高）

### Requirement 3: K8s 资源模板化

**User Story:** 作为开发者，我希望将现有的 Kustomize 资源转换为 Helm 模板，以便能够利用 Helm 的模板引擎能力。

#### Acceptance Criteria

1. WHEN 转换资源时 THEN 系统 SHALL 包含以下 K8s 资源模板：Namespace、Deployment、Service、Ingress、ConfigMap、Secret、PVC、RBAC（ServiceAccount、Role、RoleBinding）
2. WHEN 创建模板时 THEN 系统 SHALL 使用 Helm 内置对象和函数（.Values、.Release、tpl 函数等）
3. WHEN 定义模板时 THEN 系统 SHALL 为所有可配置参数提供合理的默认值
4. WHEN 模板需要引用环境变量时 THEN 系统 SHALL 支持 ConfigMap 和 Secret 的挂载

### Requirement 4: 镜像和版本管理

**User Story:** 作为运维人员，我希望能够通过 values 文件灵活配置容器镜像和版本，以便能够方便地升级应用。

#### Acceptance Criteria

1. WHEN 配置镜像时 THEN 系统 SHALL 支持在 values 文件中定义镜像仓库、镜像名称和标签
2. WHEN 升级镜像时 THEN 系统 SHALL 支持为 frontend 和 backend 容器独立配置镜像
3. WHEN 部署不同环境时 THEN 系统 SHALL 支持通过 values 文件覆盖镜像配置

### Requirement 5: 资源配置和健康检查

**User Story:** 作为运维人员，我希望能够通过 values 文件配置资源限制和健康检查，以便确保应用的稳定性和资源可控。

#### Acceptance Criteria

1. WHEN 配置资源时 THEN 系统 SHALL 支持为 frontend 和 backend 容器独立配置 CPU/内存的 requests 和 limits
2. WHEN 配置健康检查时 THEN 系统 SHALL 支持自定义 livenessProbe 和 readinessProbe 的参数
3. WHEN 设置默认值时 THEN 系统 SHALL 使用现有 Kustomize 配置中的资源值作为默认配置

### Requirement 6: 持久化存储配置

**User Story:** 作为运维人员，我希望能够通过 Helm 管理 PVC 配置，以便应用能够持久化 SQLite 数据。

#### Acceptance Criteria

1. WHEN 配置存储时 THEN 系统 SHALL 支持 PVC 的存储类和存储大小配置
2. WHEN 部署到不同环境时 THEN 系统 SHALL 支持通过 values 文件覆盖存储配置
3. WHEN 创建 PVC 时 THEN 系统 SHALL 确保 Pod 正确挂载 PVC 到 /data 路径

### Requirement 7: Ingress 和网络配置

**User Story:** 作为运维人员，我希望能够通过 values 文件配置 Ingress，以便不同环境可以有不同的域名、Ingress Controller 和路由规则。

#### Acceptance Criteria

1. WHEN 配置 Ingress 时 THEN 系统 SHALL 支持启用/禁用 Ingress
2. WHEN 配置 Ingress 时 THEN 系统 SHALL 支持自定义 host、path、ingress class 和 annotations
3. WHEN 配置 Service 时 THEN 系统 SHALL 支持 NodePort 和 ClusterPort 配置
4. WHEN 配置 Traefik Ingress 时 THEN 系统 SHALL 支持 `kubernetes.io/ingress.class: traefik` annotation
5. WHEN 配置 AWS ALB Ingress 时 THEN 系统 SHALL 支持 `kubernetes.io/ingress.class: alb` 和 ALB 特定 annotations（scheme、target-type、listen-ports 等）
6. WHEN 配置 Ingress annotations 时 THEN 系统 SHALL 支持通过 values 文件定义动态 annotations 列表

### Requirement 8: Secrets 管理

**User Story:** 作为安全运维人员，我希望能够安全地管理敏感信息，以便数据库密码等敏感数据不会泄露。

#### Acceptance Criteria

1. WHEN 管理 Secrets 时 THEN 系统 SHALL 支持通过 Helm Secrets 或外部 Secrets 工具管理敏感数据
2. WHEN 创建 Secret 模板时 THEN 系统 SHALL 提供示例 values 文件展示如何配置敏感数据
3. WHEN 部署时 THEN 系统 SHALL 支持从外部 Secrets 管理系统（如 Kubernetes Secrets、Vault）引用密钥

### Requirement 9: RBAC 配置

**User Story:** 作为安全运维人员，我希望能够通过 Helm 管理 RBAC 配置，以便应用可以安全地访问 K8s API。

#### Acceptance Criteria

1. WHEN 配置 RBAC 时 THEN 系统 SHALL 支持通过 values 文件启用/禁用 RBAC
2. WHEN 创建 RBAC 时 THEN 系统 SHALL 包含 ServiceAccount、Role 和 RoleBinding 模板
3. WHEN 定义 Role 时 THEN 系统 SHALL 支持通过 values 文件配置权限规则

### Requirement 10: 部署脚本集成

**User Story:** 作为开发者，我希望能够通过 Makefile 集成 Helm 命令，以便能够方便地执行部署操作。

#### Acceptance Criteria

1. WHEN 使用 Makefile 时 THEN 系统 SHALL 提供 helm-install、helm-upgrade、helm-uninstall、helm-diff 等 target
2. WHEN 部署到环境时 THEN 系统 SHALL 支持通过 make 命令指定环境（如 make helm-test ENV=test）
3. WHEN 执行 helm 操作时 THEN 系统 SHALL 在部署前显示 diff 预览变更

### Requirement 11: 与现有 Kustomize 配置兼容

**User Story:** 作为项目负责人，我希望保留现有的 Kustomize 配置作为备份，以便在 Helm 迁移出现问题时可以回滚。

#### Acceptance Criteria

1. WHEN 创建 Helm Chart 时 THEN 系统 SHALL 不删除现有的 k8s/ 目录
2. WHEN 完成迁移后 THEN 系统 SHALL 在文档中说明如何从 Kustomize 切换到 Helm
3. WHEN 新旧方案共存时 THEN 系统 SHALL 确保两者的部署结果等价

### Requirement 12: 文档和说明

**User Story:** 作为新加入的团队成员，我希望有清晰的文档说明如何使用 Helm 部署，以便快速上手。

#### Acceptance Criteria

1. WHEN 创建 Chart 时 THEN 系统 SHALL 包含 README.md 说明 Chart 的使用方法
2. WHEN 编写文档时 THEN 系统 SHALL 包含各环境部署命令示例
3. WHEN 编写文档时 THEN 系统 SHALL 列出所有可配置的 values 参数说明
