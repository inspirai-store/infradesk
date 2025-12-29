# Design Document

## Overview

本设计文档描述了如何将 Zeni-X 项目从 Kustomize 迁移到 Helm Chart，以实现跨集群（Dev/Test 集群和 UAT/Prod 集群）的配置管理。Helm Chart 将提供统一的模板，同时通过 values 文件支持不同环境的差异化配置。

### 设计目标

1. **单一 Chart，多环境支持**：通过一套 Helm Chart 支持 dev、test、uat、prod 四个环境
2. **跨集群部署**：支持部署到两个不同的 Kubernetes 集群（开发集群和生产集群）
3. **向后兼容**：保留现有 Kustomize 配置作为备份和对比参考
4. **可扩展性**：易于添加新环境和新配置项

### 集群环境映射

| 环境 | 集群 | Ingress | 域名模式 | 镜像仓库 |
|------|------|---------|----------|----------|
| dev | 开发集群 | Traefik | `*.inner.inspirai.store` | `registry.local/*:dev` |
| test | 开发集群 | Traefik | `*.inner.inspirai.store` | `alexxiong/*:test` |
| uat | 生产集群 | AWS ALB | `*.uat.yunizeni.com` | `registry.local/*:uat` |
| prod | 生产集群 | AWS ALB | `*.yunizeni.com` | 生产仓库 |

## Architecture

### 整体架构

```
zeni-x/
├── helm/zeni-x/                    # Helm Chart 根目录
│   ├── Chart.yaml                  # Chart 元数据
│   ├── values.yaml                 # 默认配置
│   ├── values-dev.yaml             # Dev 环境配置
│   ├── values-test.yaml            # Test 环境配置
│   ├── values-uat.yaml             # UAT 环境配置
│   ├── values-prod.yaml            # Prod 环境配置
│   ├── values.schema.json          # Values 校验模式
│   ├── templates/                  # 模板目录
│   │   ├── NOTES.txt               # 安装后说明
│   │   ├── _helpers.tpl            # 模板辅助函数
│   │   ├── deployment.yaml         # Deployment 模板
│   │   ├── service.yaml            # Service 模板
│   │   ├── ingress.yaml            # Ingress 模板
│   │   ├── configmap.yaml          # ConfigMap 模板
│   │   ├── secret.yaml             # Secret 模板（示例）
│   │   ├── pvc.yaml                # PVC 模板
│   │   ├── rbac.yaml               # RBAC 模板
│   │   ├── namespace.yaml          # Namespace 模板
│   │   └── tests/                  # 测试模板
│   │       ├── test-connection.yaml
│   │       └── test-resources.yaml
│   └── README.md                   # Chart 使用说明
├── k8s/                            # 保留现有 Kustomize 配置
└── Makefile                        # 集成 Helm 命令
```

### 部署架构

```
                    +-------------------------+
                    |   Helm Chart            |
                    |   (helm/zeni-x/)        |
                    +-------------------------+
                               |
                +--------------+--------------+
                |                             |
        +-------v-------+             +-------v-------+
        |  Dev Cluster  |             |  Prod Cluster |
        | (dev, test)   |             |  (uat, prod)  |
        +---------------+             +---------------+
        | Traefik Ingress|            |  AWS ALB      |
        | 内网域名        |            |  公网域名      |
        +---------------+             +---------------+
```

## Components and Interfaces

### 1. Chart 元数据 (Chart.yaml)

定义 Chart 的基本信息、依赖关系和版本。

```yaml
apiVersion: v2
name: zeni-x
description: A Helm chart for Zeni-X database management platform
type: application
version: 1.0.0
appVersion: "1.0.0"
keywords:
  - database
  - management
  - mysql
  - redis
home: https://github.com/yourorg/zeni-x
maintainers:
  - name: Zeni-X Team
    email: public@yunizeni.com
```

### 2. Values 结构设计

#### 默认 Values (values.yaml)

提供所有配置项的默认值，作为其他环境文件的基础。

```yaml
# 全局配置
global:
  environment: dev
  clusterType: dev  # dev | prod

# 镜像配置
image:
  frontend:
    repository: registry.local/zeni-x-frontend
    tag: latest
    pullPolicy: IfNotPresent
  backend:
    repository: registry.local/zeni-x-backend
    tag: latest
    pullPolicy: IfNotPresent

# Namespace 配置
namespace:
  name: zeni-x
  create: true
  labels:
    app: zeni-x
    environment: dev

# 副本数
replicaCount: 1

# Pod 配置
pod:
  annotations: {}
  labels: {}
  securityContext: {}

# 容器配置
containers:
  frontend:
    resources:
      requests:
        memory: "64Mi"
        cpu: "50m"
      limits:
        memory: "128Mi"
        cpu: "100m"
    livenessProbe:
      httpGet:
        path: /
        port: http
      initialDelaySeconds: 5
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /
        port: http
      initialDelaySeconds: 3
      periodSeconds: 5

  backend:
    resources:
      requests:
        memory: "128Mi"
        cpu: "100m"
      limits:
        memory: "256Mi"
        cpu: "200m"
    livenessProbe:
      httpGet:
        path: /health
        port: api
      initialDelaySeconds: 10
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: api
      initialDelaySeconds: 5
      periodSeconds: 5

    env: []
    envFrom: []

# Service 配置
service:
  type: ClusterIP
  ports:
    http:
      port: 80
      targetPort: 80
      protocol: TCP
      name: http
    api:
      port: 8080
      targetPort: 8080
      protocol: TCP
      name: api

# Ingress 配置
ingress:
  enabled: true
  className: ""  # 根据集群类型动态设置
  annotations: {}
  hosts:
    - host: zeni-x.local
      paths:
        - path: /
          pathType: Prefix
          service: http
        - path: /api
          pathType: Prefix
          service: api
  tls: []

# ConfigMap 配置
configmap:
  enabled: true
  data:
    SERVER_PORT: "8080"
    SERVER_MODE: "release"
    SQLITE_PATH: "/data/zeni-x.db"
    MYSQL_HOST: "mysql.dev-services.svc.cluster.local"
    MYSQL_PORT: "3306"
    MYSQL_USER: "root"
    REDIS_HOST: "redis.dev-services.svc.cluster.local"
    REDIS_PORT: "6379"
    REDIS_DB: "0"

# Secret 配置
secret:
  enabled: true
  create: true
  name: zeni-x-secrets
  # 敏感数据建议使用外部 Secret 管理系统
  data: {}

# PVC 配置
persistence:
  enabled: true
  existingClaim: ""
  storageClass: ""
  accessMode: ReadWriteOnce
  size: 1Gi
  mountPath: /data

# RBAC 配置
rbac:
  create: true
  serviceAccount:
    create: true
    name: zeni-x-reader
  role:
    rules: []
```

#### 集群特定 Values

**values-dev.yaml / values-test.yaml** (开发集群):

```yaml
global:
  clusterType: dev

namespace:
  name: zeni-x-dev

image:
  frontend:
    repository: alexxiong/zeni-x-frontend  # 或 registry.local/zeni-x-frontend
    tag: dev
  backend:
    repository: alexxiong/zeni-x-backend
    tag: dev

ingress:
  className: traefik
  annotations:
    kubernetes.io/ingress.class: traefik
  hosts:
    - host: zeni-x-dev.inner.inspirai.store
      paths:
        - path: /
          pathType: Prefix
          service: http
        - path: /api
          pathType: Prefix
          service: api

# NodePort for dev cluster
service:
  type: NodePort
  ports:
    http:
      nodePort: 30173
    api:
      nodePort: 30180
```

**values-uat.yaml / values-prod.yaml** (生产集群):

```yaml
global:
  clusterType: prod

namespace:
  name: zeni-x-uat  # 或 zeni-x-prod

image:
  frontend:
    repository: yunizeni-registry.cn-shenzhen.cr.aliyuncs.com/yunizeni/zeni-x-frontend
    tag: uat  # 或 prod
  backend:
    repository: yunizeni-registry.cn-shenzhen.cr.aliyuncs.com/yunizeni/zeni-x-backend
    tag: uat

# 更高的资源配置
containers:
  backend:
    resources:
      requests:
        memory: "256Mi"
        cpu: "200m"
      limits:
        memory: "512Mi"
        cpu: "400m"

ingress:
  className: alb
  annotations:
    kubernetes.io/ingress.class: alb
    alb.ingress.kubernetes.io/scheme: internet-facing
    alb.ingress.kubernetes.io/target-type: ip
    alb.ingress.kubernetes.io/listen-ports: '[{"HTTP": 80}]'
  hosts:
    - host: zeni-x.uat.yunizeni.com  # 或 zeni-x.yunizeni.com
      paths:
        - path: /
          pathType: Prefix
          service: http
        - path: /api
          pathType: Prefix
          service: api

# NodePort for prod cluster
service:
  type: NodePort
  ports:
    http:
      nodePort: 30273
    api:
      nodePort: 30280
```

### 3. 模板设计

#### 3.1 模板辅助函数 (_helpers.tpl)

定义可重用的模板函数：

```yaml
{{/* vim: set filetype=mustache: */}}
{{/*
Expand the name of the chart.
*/}}
{{- define "zeni-x.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "zeni-x.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "zeni-x.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "zeni-x.labels" -}}
helm.sh/chart: {{ include "zeni-x.chart" . }}
{{ include "zeni-x.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "zeni-x.selectorLabels" -}}
app.kubernetes.io/name: {{ include "zeni-x.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Ingress Class based on cluster type
*/}}
{{- define "zeni-x.ingress.className" -}}
{{- if .Values.ingress.className }}
{{- .Values.ingress.className }}
{{- else if eq .Values.global.clusterType "dev" }}
traefik
{{- else if eq .Values.global.clusterType "prod" }}
alb
{{- else }}
nginx
{{- end }}
{{- end }}
```

#### 3.2 Deployment 模板

包含 frontend 和 backend 两个容器的 Deployment：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "zeni-x.fullname" . }}
  namespace: {{ .Values.namespace.name }}
  labels:
    {{- include "zeni-x.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "zeni-x.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      annotations:
        checksum/config: {{ include (print $.Template.BasePath "/configmap.yaml") . | sha256sum }}
        {{- with .Values.pod.annotations }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
      labels:
        {{- include "zeni-x.selectorLabels" . | nindent 8 }}
        {{- with .Values.pod.labels }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
    spec:
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ include "zeni-x.serviceAccountName" . }}
      securityContext:
        {{- toYaml .Values.pod.securityContext | nindent 8 }}
      containers:
      # Frontend Container
      - name: frontend
        securityContext:
          {{- toYaml .Values.containers.frontend.securityContext | nindent 10 }}
        image: "{{ .Values.image.frontend.repository }}:{{ .Values.image.frontier.tag | default .Chart.AppVersion }}"
        imagePullPolicy: {{ .Values.image.frontend.pullPolicy }}
        ports:
        - name: http
          containerPort: {{ .Values.service.ports.http.targetPort }}
          protocol: TCP
        {{- with .Values.containers.frontend.livenessProbe }}
        livenessProbe:
          {{- toYaml . | nindent 10 }}
        {{- end }}
        {{- with .Values.containers.frontend.readinessProbe }}
        readinessProbe:
          {{- toYaml . | nindent 10 }}
        {{- end }}
        resources:
          {{- toYaml .Values.containers.frontend.resources | nindent 10 }}
      # Backend Container
      - name: backend
        securityContext:
          {{- toYaml .Values.containers.backend.securityContext | nindent 10 }}
        image: "{{ .Values.image.backend.repository }}:{{ .Values.image.backend.tag | default .Chart.AppVersion }}"
        imagePullPolicy: {{ .Values.image.backend.pullPolicy }}
        ports:
        - name: api
          containerPort: {{ .Values.service.ports.api.targetPort }}
          protocol: TCP
        env:
        - name: SERVER_PORT
          value: {{ .Values.configmap.data.SERVER_PORT | quote }}
        {{- with .Values.containers.backend.env }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
        envFrom:
        {{- if .Values.secret.enabled }}
        - secretRef:
            name: {{ include "zeni-x.fullname" . }}-secrets
        {{- end }}
        - configMapRef:
            name: {{ include "zeni-x.fullname" . }}-config
        {{- with .Values.containers.backend.envFrom }}
        {{- toYaml . | nindent 8 }}
        {{- end }}
        volumeMounts:
        - name: sqlite-data
          mountPath: {{ .Values.persistence.mountPath }}
        {{- with .Values.containers.backend.livenessProbe }}
        livenessProbe:
          {{- toYaml . | nindent 10 }}
        {{- end }}
        {{- with .Values.containers.backend.readinessProbe }}
        readinessProbe:
          {{- toYaml . | nindent 10 }}
        {{- end }}
        resources:
          {{- toYaml .Values.containers.backend.resources | nindent 10 }}
      volumes:
      - name: sqlite-data
        {{- if .Values.persistence.enabled }}
        persistentVolumeClaim:
          claimName: {{ .Values.persistence.existingClaim | default (include "zeni-x.fullname" .) }}
        {{- else }}
        emptyDir: {}
        {{- end }}
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
```

#### 3.3 Ingress 模板

支持动态 Ingress Class 和 annotations：

```yaml
{{- if .Values.ingress.enabled -}}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ include "zeni-x.fullname" . }}-ingress
  namespace: {{ .Values.namespace.name }}
  labels:
    {{- include "zeni-x.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  ingressClassName: {{ include "zeni-x.ingress.className" . }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            pathType: {{ .pathType }}
            backend:
              service:
                name: {{ include "zeni-x.fullname" $ }}
                port:
                  number: {{ $.Values.service.ports[.service].port }}
          {{- end }}
    {{- end }}
{{- end }}
```

### 4. Makefile 集成

在 Makefile 中添加 Helm 相关命令：

```makefile
# Helm 配置
HELM_CHART_DIR ?= helm/zeni-x
HELM_RELEASE_NAME ?= zeni-x
HELM_NAMESPACE ?= zeni-x
HELM_VALUES_FILE ?= values-dev.yaml
HELM_KUBECONTEXT ?=

# Helm 命令
.PHONY: helm-lint
helm-lint:
	@echo "Linting Helm chart..."
	helm lint $(HELM_CHART_DIR)

.PHONY: helm-template
helm-template:
	@echo "Template Helm chart..."
	helm template $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/$(HELM_VALUES_FILE) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kubecontext $(HELM_KUBECONTEXT))

.PHONY: helm-diff
helm-diff:
	@echo "Diff Helm release..."
	helm diff upgrade $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/$(HELM_VALUES_FILE) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kubecontext $(HELM_KUBECONTEXT)) \
		--install --allow-unreleased

.PHONY: helm-install
helm-install: helm-diff
	@echo "Installing Helm release..."
	helm upgrade $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/$(HELM_VALUES_FILE) \
		--namespace $(HELM_NAMESPACE) \
		--create-namespace \
		--install \
		--wait \
		--timeout 5m \
		$(if $(HELM_KUBECONTEXT),--kubecontext $(HELM_KUBECONTEXT))

.PHONY: helm-uninstall
helm-uninstall:
	@echo "Uninstalling Helm release..."
	helm uninstall $(HELM_RELEASE_NAME) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kubecontext $(HELM_KUBECONTEXT))

.PHONY: helm-status
helm-status:
	@echo "Helm release status..."
	helm status $(HELM_RELEASE_NAME) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kubecontext $(HELM_KUBECONTEXT))

# 环境快捷命令
.PHONY: helm-dev
helm-dev:
	$(MAKE) helm-install HELM_VALUES_FILE=values-dev.yaml HELM_NAMESPACE=zeni-x-dev

.PHONY: helm-test
helm-test:
	$(MAKE) helm-install HELM_VALUES_FILE=values-test.yaml HELM_NAMESPACE=zeni-x-test

.PHONY: helm-uat
helm-uat:
	$(MAKE) helm-install HELM_VALUES_FILE=values-uat.yaml HELM_NAMESPACE=zeni-x-uat

.PHONY: helm-prod
helm-prod:
	$(MAKE) helm-install HELM_VALUES_FILE=values-prod.yaml HELM_NAMESPACE=zeni-x-prod
```

## Data Models

### Values Schema (values.schema.json)

使用 JSON Schema 验证 values 文件：

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Zeni-X Helm Chart Values",
  "type": "object",
  "required": ["global"],
  "properties": {
    "global": {
      "type": "object",
      "properties": {
        "environment": {
          "type": "string",
          "enum": ["dev", "test", "uat", "prod"]
        },
        "clusterType": {
          "type": "string",
          "enum": ["dev", "prod"]
        }
      }
    },
    "image": {
      "type": "object",
      "properties": {
        "frontend": {
          "type": "object",
          "properties": {
            "repository": {"type": "string"},
            "tag": {"type": "string"},
            "pullPolicy": {"type": "string", "enum": ["Always", "IfNotPresent", "Never"]}
          }
        },
        "backend": {
          "type": "object",
          "properties": {
            "repository": {"type": "string"},
            "tag": {"type": "string"},
            "pullPolicy": {"type": "string", "enum": ["Always", "IfNotPresent", "Never"]}
          }
        }
      }
    },
    "ingress": {
      "type": "object",
      "properties": {
        "enabled": {"type": "boolean"},
        "className": {"type": "string"},
        "annotations": {"type": "object"},
        "hosts": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "host": {"type": "string"},
              "paths": {"type": "array"}
            }
          }
        }
      }
    }
  }
}
```

## Error Handling

### 模板验证

1. **Helm Lint**: 在部署前运行 `helm lint` 检查 Chart 语法
2. **Values Schema**: 使用 `values.schema.json` 验证输入参数
3. **Dry Run**: 使用 `--dry-run` 参数预览渲染结果

### 常见错误处理

| 错误场景 | 处理策略 |
|----------|----------|
| Values 文件缺失必需参数 | Values Schema 验证失败，提示用户 |
| 镜像拉取失败 | 设置 `imagePullSecrets`，配置私有仓库认证 |
| 资源不足 | 通过 values 文件调整 resources 限制 |
| Ingress 配置错误 | 提供集群特定的默认配置，支持覆盖 |

### 部署回滚

Helm 自动保留发布历史，支持快速回滚：

```bash
# 查看历史
helm history zeni-x -n zeni-x-dev

# 回滚到上一版本
helm rollback zeni-x -n zeni-x-dev

# 回滚到指定版本
helm rollback zeni-x 2 -n zeni-x-dev
```

## Testing Strategy

### 1. 单元测试

使用 `helm template` 测试模板渲染：

```bash
# 渲染模板并验证 YAML 语法
helm template zeni-x helm/zeni-x \
  --values helm/zeni-x/values-dev.yaml \
  --namespace zeni-x-dev | kubectl apply --dry-run=client -f -
```

### 2. 集成测试

在测试环境中部署并验证：

```bash
# 部署到测试环境
make helm-test

# 运行测试 Pod
helm test zeni-x -n zeni-x-test
```

### 3. 测试模板 (tests/)

创建测试模板验证部署：

```yaml
# tests/test-connection.yaml
apiVersion: v1
kind: Pod
metadata:
  name: "{{ include "zeni-x.fullname" . }}-test-connection"
  labels:
    {{- include "zeni-x.labels" . | nindent 4 }}
  annotations:
    "helm.sh/hook": test-success
spec:
  containers:
    - name: wget
      image: busybox
      command: ['wget']
      args:
        - '--spider'
        - 'http://{{ include "zeni-x.fullname" . }}:80'
  restartPolicy: Never
```

### 4. 多环境验证

确保所有环境的 values 文件都能正确渲染：

```bash
for env in dev test uat prod; do
  echo "Testing $env environment..."
  helm template zeni-x helm/zeni-x \
    --values helm/zeni-x/values-$env.yaml \
    --namespace zeni-x-$env | kubectl apply --dry-run=server -f -
done
```

### 5. CI/CD 集成

在 CI/CD 流程中添加验证步骤：

```yaml
# .github/workflows/helm-test.yml
name: Helm Chart Test
on: [push, pull_request]
jobs:
  lint-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: azure/setup-helm@v3
      - name: Lint Helm Chart
        run: helm lint helm/zeni-x
      - name: Template Helm Chart
        run: |
          for env in dev test uat prod; do
            helm template zeni-x helm/zeni-x --values helm/zeni-x/values-$env.yaml
          done
```

## Design Decisions

### 决策 1: 集群类型通过 global.clusterType 区分

**理由**：通过一个简单的标识符区分集群类型，可以在模板中使用条件逻辑来设置集群特定的配置。

**权衡**：需要在 values 文件中明确设置 clusterType，但简化了模板逻辑。

### 决策 2: 保留现有 Kustomize 配置

**理由**：作为迁移的备份和对比参考，降低迁移风险。

**权衡**：增加了维护成本，但提供了回退方案。

### 决策 3: 使用独立的 values 文件而非 overlays

**理由**：Helm 的标准实践，易于理解和维护。

**权衡**：可能在多个文件间有重复配置，但可以通过合理的默认值来最小化重复。

### 决策 4: Ingress Class 通过 values 配置

**理由**：不同集群使用不同的 Ingress Controller，需要灵活配置。

**权衡**：增加了配置复杂度，但提供了必要的灵活性。

## References

研究资料来源：
- [Mastering Helm: Best Practices for Multi-Environment Kubernetes Deployments](https://medium.com/@DynamoDevOps/mastering-helm-best-practices-for-multi-environment-kubernetes-deployments-00a89356cde6)
- [Helm Best Practices 2025](https://user-cube.medium.com/helm-best-practices-2025-what-changed-with-helm-4-and-what-you-should-know-b6d3065fb6e1)
- [Official Helm Chart Best Practices](https://helm.sh/docs/chart_best_practices/)
- [Deploying apps on multiple Kubernetes clusters with Helm](https://medium.com/dailymotion/deploying-apps-on-multiple-kubernetes-clusters-with-helm-19ee2b06179e)
- [Working with Helm Values: Common Operations & Best Practices](https://komodor.com/learn/working-with-helm-values-common-operations-and-best-practices/)
