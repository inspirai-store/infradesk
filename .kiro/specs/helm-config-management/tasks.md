# Implementation Plan

本实施计划将 Helm Chart 分解为一系列可执行的编码任务。每个任务都聚焦于具体的代码实现，按照合理的依赖顺序排列。

- [ ] 1. 创建 Helm Chart 基础结构和元数据文件
  - 创建 `helm/zeni-x/` 目录结构（templates/、charts/ 等）
  - 编写 `Chart.yaml` 文件，包含 Chart 名称、版本、依赖等元数据
  - 创建 `README.md` 说明 Chart 的用途和基本使用方法
  - _Requirements: 1.1, 1.2, 1.3_

- [ ] 2. 创建默认 values.yaml 配置文件
  - 编写 `values.yaml` 包含所有配置项的默认值
  - 定义 global 配置（environment、clusterType）
  - 定义 image 配置（frontend 和 backend 的 repository、tag、pullPolicy）
  - 定义 namespace、replicaCount、pod 配置
  - 定义 containers 配置（resources、livenessProbe、readinessProbe）
  - 定义 service、ingress、configmap、secret、persistence、rbac 配置
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 3. 创建环境特定 values 文件
  - [ ] 3.1 创建开发集群环境 values 文件（dev 和 test）
    - 编写 `values-dev.yaml`，设置开发集群特定配置（Traefik Ingress、内网域名、本地镜像仓库）
    - 编写 `values-test.yaml`，设置测试环境特定配置（Traefik Ingress、NodePort 30180/30188）
    - _Requirements: 2.1, 2.1.1, 2.1.2, 2.1.3, 2.1.6, 2.1.8_

  - [ ] 3.2 创建生产集群环境 values 文件（uat 和 prod）
    - 编写 `values-uat.yaml`，设置 UAT 环境配置（ALB Ingress、公网域名、更高资源限制、NodePort 30280/30288）
    - 编写 `values-prod.yaml`，设置生产环境配置（ALB Ingress、生产镜像仓库）
    - _Requirements: 2.1, 2.1.1, 2.1.4, 2.1.5, 2.1.7, 2.1.8_

- [ ] 4. 创建 values.schema.json 验证文件
  - 编写 JSON Schema 文件验证 values 结构
  - 定义必需字段和可选字段
  - 添加字段类型和枚举值约束
  - _Requirements: 2.4_

- [ ] 5. 创建模板辅助函数文件 _helpers.tpl
  - 编写模板辅助函数（name、fullname、chart、labels、selectorLabels）
  - 创建 Ingress Class 辅助函数，根据 clusterType 动态返回 ingress class
  - 创建 serviceAccountName 辅助函数
  - _Requirements: 1.2, 2.1.1, 7.2_

- [ ] 6. 创建 Namespace 模板
  - 编写 `namespace.yaml` 模板文件
  - 支持通过 values 配置是否创建 namespace
  - 支持自定义 namespace 名称和 labels
  - _Requirements: 1.2, 2.1_

- [ ] 7. 创建 ConfigMap 模板
  - 编写 `configmap.yaml` 模板文件
  - 将现有 k8s/base/configmap.yaml 的配置转换为 Helm 模板
  - 支持通过 values 覆盖配置项（SERVER_PORT、MYSQL_HOST、REDIS_HOST 等）
  - _Requirements: 3.1, 3.4, 3.5_

- [ ] 8. 创建 Secret 模板
  - 编写 `secret.yaml` 模板文件（作为示例，实际生产环境使用外部 Secrets）
  - 支持通过 values 配置敏感数据
  - 在模板注释中说明生产环境应使用外部 Secret 管理方案
  - _Requirements: 3.1, 8.1, 8.2, 8.3_

- [ ] 9. 创建 Deployment 模板
  - 编写 `deployment.yaml` 模板文件
  - 将现有 k8s/base/deployment.yaml 转换为 Helm 模板
  - 包含 frontend 和 backend 两个容器
  - 支持副本数、镜像、资源限制、健康检查配置
  - 添加 PVC volume 挂载
  - 添加 ConfigMap 和 Secret 的 envFrom 引用
  - 添加 config checksum annotation 用于触发滚动更新
  - _Requirements: 3.1, 3.2, 3.3, 4.1, 4.2, 5.1, 5.2, 5.3, 6.1, 6.2, 6.3_

- [ ] 10. 创建 Service 模板
  - 编写 `service.yaml` 模板文件
  - 将现有 k8s/base/service.yaml 转换为 Helm 模板
  - 支持 ClusterIP 和 NodePort 类型
  - 支持自定义端口和 NodePort 配置
  - _Requirements: 3.1, 7.2, 7.3, 2.1.4_

- [ ] 11. 创建 Ingress 模板
  - 编写 `ingress.yaml` 模板文件
  - 将现有 k8s/base/ingress.yaml 转换为 Helm 模板
  - 支持通过 values 配置启用/禁用 Ingress
  - 使用辅助函数动态设置 ingressClassName（Traefik/ALB）
  - 支持动态 annotations 配置
  - 支持 TLS 配置
  - 支持多 host 和多 path 配置
  - _Requirements: 3.1, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 2.1.1, 2.1.2, 2.1.5_

- [ ] 12. 创建 PVC 模板
  - 编写 `pvc.yaml` 模板文件
  - 将现有 k8s/base/pvc.yaml 转换为 Helm 模板
  - 支持通过 values 配置存储类、访问模式、存储大小
  - 支持使用现有 PVC
  - _Requirements: 3.1, 6.1, 6.2, 6.3_

- [ ] 13. 创建 RBAC 模板
  - 编写 `rbac.yaml` 模板文件
  - 将现有 k8s/base/rbac.yaml 转换为 Helm 模板
  - 包含 ServiceAccount、Role、RoleBinding
  - 支持通过 values 配置启用/禁用 RBAC
  - 支持通过 values 配置 Role 权限规则
  - _Requirements: 3.1, 9.1, 9.2, 9.3_

- [ ] 14. 创建测试模板
  - [ ] 14.1 创建连接测试模板
    - 编写 `tests/test-connection.yaml` 测试 Pod
    - 验证 frontend HTTP 服务连接
    - _Requirements: 12.1, 12.2_

  - [ ] 14.2 创建资源验证测试模板
    - 编写 `tests/test-resources.yaml` 验证资源创建
    - 验证 PVC 可正确挂载
    - _Requirements: 12.1, 12.2_

- [ ] 15. 创建 NOTES.txt 文件
  - 编写 `templates/NOTES.txt` 文件
  - 包含安装后的访问信息说明
  - 包含获取 Pod 状态的命令
  - 包含卸载命令说明
  - _Requirements: 12.2, 12.3_

- [ ] 16. 更新 Makefile 集成 Helm 命令
  - 在 Makefile 中添加 Helm 相关变量（HELM_CHART_DIR、HELM_RELEASE_NAME 等）
  - 添加 helm-lint、helm-template、helm-diff、helm-install、helm-uninstall、helm-status 命令
  - 添加环境快捷命令（helm-dev、helm-test、helm-uat、helm-prod）
  - 确保命令支持跨集群部署（通过 KUBECONTEXT 参数）
  - _Requirements: 10.1, 10.2, 10.3_

- [ ] 17. 创建 Helm Chart 使用文档
  - 在 Chart README.md 中添加使用说明
  - 列出所有可配置的 values 参数
  - 提供各环境部署命令示例
  - 说明如何从 Kustomize 切换到 Helm
  - 添加故障排查指南
  - _Requirements: 12.1, 12.2, 12.3, 11.1, 11.2, 11.3_

- [ ] 18. 创建验证脚本
  - 编写脚本验证所有环境的 values 文件能正确渲染
  - 使用 helm template 和 kubectl dry-run 验证生成的 YAML 语法
  - 添加到 Makefile 或 scripts 目录中
  - _Requirements: 10.1, 12.1_
