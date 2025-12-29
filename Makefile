# ============================================================
# Zeni-X Makefile - 多环境构建部署管理
# ============================================================

.PHONY: help dev test uat build clean

# 变量
REGISTRY ?= yunizeni-registry.cn-shenzhen.cr.aliyuncs.com/yunizeni
VERSION ?= latest
DEV_DIR ?= $(CURDIR)/.dev
SERVICE_DIR ?= $(CURDIR)/services/zeni-x
FRONTEND_DIR ?= $(SERVICE_DIR)/frontend
BACKEND_DIR ?= $(SERVICE_DIR)/backend

# K8s 上下文
TEST_CONTEXT := inner
UAT_CONTEXT := aliyun
DEV_BACKEND_PID := $(DEV_DIR)/backend.pid
DEV_FRONTEND_PID := $(DEV_DIR)/frontend.pid
DEV_BACKEND_LOG := $(DEV_DIR)/backend.log
DEV_FRONTEND_LOG := $(DEV_DIR)/frontend.log

# 默认目标
help:
	@echo "Zeni-X Build System"
	@echo "==================="
	@echo ""
	@echo "[基础]"
	@echo "  make help         - 显示帮助"
	@echo ""
	@echo "[本地开发]"
	@echo "  make dev          - 本地开发环境（热重载）"
	@echo "  make dev-start    - 后台启动本地开发（写入 .dev/*.pid & .dev/*.log）"
	@echo "  make dev-stop     - 停止 dev-start 启动的服务"
	@echo "  make dev-status   - 查看后台服务状态"
	@echo "  make dev-frontend - 仅启动前端开发服务器"
	@echo "  make dev-backend  - 仅启动后端开发服务器"
	@echo "  make dev-check    - 检查开发环境依赖"
	@echo ""
	@echo "[依赖]"
	@echo "  make install      - 安装前后端依赖（pnpm + go mod）"
	@echo ""
	@echo "[构建]"
	@echo "  make build        - 构建生产版本（frontend + backend）"
	@echo "  make build-docker - 构建 Docker 镜像（frontend + backend）"
	@echo ""
	@echo "[K8s 部署 (Helm)]"
	@echo "  make test              - 部署到测试环境 K8s（Helm）"
	@echo "  make test-logs         - 追踪测试环境日志"
	@echo "  make uat               - 部署到 UAT 环境 K8s（Helm）"
	@echo "  make uat-logs          - 追踪 UAT 环境日志"
	@echo "  make prod              - 部署到生产环境 K8s（Helm）"
	@echo ""
	@echo "[Helm 工具]"
	@echo "  make helm-validate     - 验证 Helm Chart（helm lint）"
	@echo "  make helm-test-dryrun  - 生成 test 环境部署清单到 debug/test/"
	@echo "  make helm-uat-dryrun   - 生成 uat 环境部署清单到 debug/uat/"
	@echo "  make helm-prod-dryrun  - 生成 prod 环境部署清单到 debug/prod/"
	@echo ""
	@echo "[清理]"
	@echo "  make clean        - 清理构建产物（dist + frontend/node_modules）"
	@echo "  make clean-k8s-test - 删除测试环境 K8s 资源（context: $(TEST_CONTEXT)）"
	@echo "  make clean-k8s-uat  - 删除 UAT 环境 K8s 资源（context: $(UAT_CONTEXT)）"

# ------------------------------------------------------------
# 本地开发环境
# ------------------------------------------------------------
dev: dev-check
	@echo "🚀 Starting local development..."
	@trap 'kill 0' EXIT; \
	$(MAKE) dev-backend & \
	$(MAKE) dev-frontend & \
	wait

dev-frontend:
	@echo "🎨 Starting frontend dev server..."
	cd $(FRONTEND_DIR) && pnpm dev

dev-backend:
	@echo "⚙️ Starting backend dev server..."
	cd $(BACKEND_DIR) && \
		export SERVER_PORT=15080 && \
		export SERVER_MODE=debug && \
		export SQLITE_PATH=./data/zeni-x.db && \
		go run cmd/server/main.go

# 后台启动（避免占用当前终端）
dev-start: dev-check
	@mkdir -p $(DEV_DIR)
	@echo "🚀 Starting dev services in background..."
	@# Ensure frontend deps exist (vite is a devDependency)
	@if [ ! -d "$(FRONTEND_DIR)/node_modules" ]; then \
		echo "📦 Installing frontend dependencies (pnpm install)..."; \
		( cd $(FRONTEND_DIR) && pnpm install ); \
	fi
	@# Backend
	@if [ -f "$(DEV_BACKEND_PID)" ] && kill -0 "$$(cat $(DEV_BACKEND_PID))" 2>/dev/null; then \
		echo "Backend already running (pid=$$(cat $(DEV_BACKEND_PID)))"; \
	else \
		echo "Starting backend..."; \
		cd $(BACKEND_DIR) && \
			export SERVER_PORT=15080 && \
			export SERVER_MODE=debug && \
			export SQLITE_PATH=./data/zeni-x.db && \
			nohup go run cmd/server/main.go > $(DEV_BACKEND_LOG) 2>&1 & \
			echo $$! > $(DEV_BACKEND_PID); \
		echo "Backend started (pid=$$(cat $(DEV_BACKEND_PID)))"; \
	fi
	@# Frontend
	@if [ -f "$(DEV_FRONTEND_PID)" ] && kill -0 "$$(cat $(DEV_FRONTEND_PID))" 2>/dev/null; then \
		echo "Frontend already running (pid=$$(cat $(DEV_FRONTEND_PID)))"; \
	else \
		echo "Starting frontend..."; \
		cd $(FRONTEND_DIR) && \
			nohup pnpm dev > $(DEV_FRONTEND_LOG) 2>&1 & \
			echo $$! > $(DEV_FRONTEND_PID); \
		echo "Frontend started (pid=$$(cat $(DEV_FRONTEND_PID)))"; \
	fi
	@echo ""
	@echo "Frontend: http://localhost:15073"
	@echo "Backend:  http://localhost:15080"
	@echo "Logs:     $(DEV_FRONTEND_LOG), $(DEV_BACKEND_LOG)"

dev-stop:
	@echo "🛑 Stopping dev services..."
	@# Stop frontend
	@if [ -f "$(DEV_FRONTEND_PID)" ]; then \
		PID="$$(cat $(DEV_FRONTEND_PID))"; \
		if kill -0 "$$PID" 2>/dev/null; then \
			echo "Stopping frontend (pid=$$PID)..."; \
			kill "$$PID" 2>/dev/null || true; \
		else \
			echo "Frontend pid file exists but process is not running (pid=$$PID)"; \
		fi; \
		rm -f "$(DEV_FRONTEND_PID)"; \
	else \
		echo "Frontend not running (no pid file). Trying to stop by port 15073..."; \
		if command -v lsof >/dev/null 2>&1; then \
			PID="$$(lsof -ti tcp:15073 2>/dev/null | head -n 1)"; \
			if [ -n "$$PID" ]; then \
				echo "Stopping frontend by port (pid=$$PID)..."; \
				kill "$$PID" 2>/dev/null || true; \
			fi; \
		fi; \
	fi
	@# Stop backend
	@if [ -f "$(DEV_BACKEND_PID)" ]; then \
		PID="$$(cat $(DEV_BACKEND_PID))"; \
		if kill -0 "$$PID" 2>/dev/null; then \
			echo "Stopping backend (pid=$$PID)..."; \
			kill "$$PID" 2>/dev/null || true; \
		else \
			echo "Backend pid file exists but process is not running (pid=$$PID)"; \
		fi; \
		rm -f "$(DEV_BACKEND_PID)"; \
	else \
		echo "Backend not running (no pid file). Trying to stop by port 15080..."; \
		if command -v lsof >/dev/null 2>&1; then \
			PID="$$(lsof -ti tcp:15080 2>/dev/null | head -n 1)"; \
			if [ -n "$$PID" ]; then \
				echo "Stopping backend by port (pid=$$PID)..."; \
				kill "$$PID" 2>/dev/null || true; \
			fi; \
		fi; \
	fi
	@echo "✅ Done."

dev-status:
	@mkdir -p $(DEV_DIR)
	@echo "🔎 Dev service status:"
	@if [ -f "$(DEV_BACKEND_PID)" ] && kill -0 "$$(cat $(DEV_BACKEND_PID))" 2>/dev/null; then \
		echo "  Backend : running (pid=$$(cat $(DEV_BACKEND_PID)))"; \
	else \
		echo "  Backend : stopped"; \
	fi
	@if [ -f "$(DEV_FRONTEND_PID)" ] && kill -0 "$$(cat $(DEV_FRONTEND_PID))" 2>/dev/null; then \
		echo "  Frontend: running (pid=$$(cat $(DEV_FRONTEND_PID)))"; \
	else \
		echo "  Frontend: stopped"; \
	fi

dev-check:
	@echo "✅ Checking dependencies..."
	@command -v go >/dev/null 2>&1 || { echo "❌ Go not found. Please install Go 1.21+"; exit 1; }
	@command -v pnpm >/dev/null 2>&1 || { echo "❌ pnpm not found. Please install pnpm"; exit 1; }
	@command -v node >/dev/null 2>&1 || { echo "❌ Node.js not found. Please install Node.js 18+"; exit 1; }
	@echo "✅ All dependencies found!"

# 安装依赖
install:
	@echo "📦 Installing dependencies..."
	cd $(FRONTEND_DIR) && pnpm install
	cd $(BACKEND_DIR) && go mod download
	@echo "✅ Dependencies installed!"

# ------------------------------------------------------------
# 构建
# ------------------------------------------------------------
build: build-frontend build-backend
	@echo "✅ Build complete!"

build-frontend:
	@echo "📦 Building frontend..."
	cd $(FRONTEND_DIR) && pnpm install && pnpm build
	@mkdir -p dist
	@cp -r $(FRONTEND_DIR)/dist dist/frontend

build-backend:
	@echo "📦 Building backend..."
	@mkdir -p dist
	cd $(BACKEND_DIR) && CGO_ENABLED=1 go build -o ../../../dist/zeni-x cmd/server/main.go

build-docker:
	@echo "🐳 Building Docker images..."
	docker build -t zeni-x-frontend:$(VERSION) $(FRONTEND_DIR)/
	@# Copy configs to backend directory for docker build context
	@mkdir -p $(BACKEND_DIR)/configs
	@cp -r config/backend/* $(BACKEND_DIR)/configs/
	docker build -t zeni-x-backend:$(VERSION) $(BACKEND_DIR)/
	@rm -rf $(BACKEND_DIR)/configs

# ------------------------------------------------------------
# 测试环境 (K8s - Helm)
# ------------------------------------------------------------
test: build-docker test-push test-deploy test-verify
	@echo "✅ Deployed to TEST environment!"

test-push:
	@echo "📤 Pushing images to registry (test)..."
	docker tag zeni-x-frontend:$(VERSION) alexxiong/zeni-x-frontend:test
	docker tag zeni-x-backend:$(VERSION) alexxiong/zeni-x-backend:test
	docker push alexxiong/zeni-x-frontend:test
	docker push alexxiong/zeni-x-backend:test

test-deploy:
	@echo "🚀 Deploying to test environment using Helm (context: $(TEST_CONTEXT))..."
	@if [ ! -f helm/zeni-x/values-test.secret.yaml ]; then \
		echo "⚠️  helm/zeni-x/values-test.secret.yaml not found, creating from example..."; \
		cp helm/zeni-x/values-test.secret.example helm/zeni-x/values-test.secret.yaml; \
		echo "✅ Created helm/zeni-x/values-test.secret.yaml from example"; \
		echo "💡 Tip: Update this file with actual secrets for production use"; \
	fi
	@# Check if namespace exists to avoid create-namespace conflict
	@NS_EXISTS=$$(kubectl --context=$(TEST_CONTEXT) get namespace zeni-x-test -o name 2>/dev/null || echo ""); \
	if [ -z "$$NS_EXISTS" ]; then \
		helm upgrade --install zeni-x-test helm/zeni-x \
			--namespace zeni-x-test \
			--create-namespace \
			--values helm/zeni-x/values-test.yaml \
			--values helm/zeni-x/values-test.secret.yaml \
			--kube-context $(TEST_CONTEXT) \
			--wait \
			--timeout 5m; \
	else \
		helm upgrade --install zeni-x-test helm/zeni-x \
			--namespace zeni-x-test \
			--values helm/zeni-x/values-test.yaml \
			--values helm/zeni-x/values-test.secret.yaml \
			--kube-context $(TEST_CONTEXT) \
			--wait \
			--timeout 5m; \
	fi

test-verify:
	@echo "⏳ Waiting for deployment (context: $(TEST_CONTEXT))..."
	kubectl --context=$(TEST_CONTEXT) rollout status deployment/zeni-x -n zeni-x-test --timeout=120s
	@echo "🔍 Running health check..."
	@kubectl --context=$(TEST_CONTEXT) exec -n zeni-x-test deploy/zeni-x -c backend -- wget -q -O- http://localhost:8080/health || echo "Health check pending..."

test-logs:
	kubectl --context=$(TEST_CONTEXT) logs -f deployment/zeni-x -n zeni-x-test --all-containers=true

# ------------------------------------------------------------
# UAT 环境 (K8s - Helm)
# ------------------------------------------------------------
uat: build-docker uat-push uat-deploy uat-verify
	@echo "✅ Deployed to UAT environment!"

uat-push:
	@echo "📤 Pushing images to registry (uat)..."
	docker tag zeni-x-frontend:$(VERSION) registry.cn-hangzhou.aliyuncs.com/zeni-x/zeni-x-frontend:uat
	docker tag zeni-x-backend:$(VERSION) registry.cn-hangzhou.aliyuncs.com/zeni-x/zeni-x-backend:uat
	docker push registry.cn-hangzhou.aliyuncs.com/zeni-x/zeni-x-frontend:uat
	docker push registry.cn-hangzhou.aliyuncs.com/zeni-x/zeni-x-backend:uat

uat-deploy:
	@echo "🚀 Deploying to UAT environment using Helm (context: $(UAT_CONTEXT))..."
	@if [ ! -f helm/zeni-x/values-uat.secret.yaml ]; then \
		echo "⚠️  helm/zeni-x/values-uat.secret.yaml not found, creating from example..."; \
		cp helm/zeni-x/values-uat.secret.example helm/zeni-x/values-uat.secret.yaml; \
		echo "✅ Created helm/zeni-x/values-uat.secret.yaml from example"; \
		echo "💡 Tip: Update this file with actual secrets for production use"; \
	fi
	@# Check if namespace exists to avoid create-namespace conflict
	@NS_EXISTS=$$(kubectl --context=$(UAT_CONTEXT) get namespace zeni-x-uat -o name 2>/dev/null || echo ""); \
	if [ -z "$$NS_EXISTS" ]; then \
		helm upgrade --install zeni-x-uat helm/zeni-x \
			--namespace zeni-x-uat \
			--create-namespace \
			--values helm/zeni-x/values-uat.yaml \
			--values helm/zeni-x/values-uat.secret.yaml \
			--kube-context $(UAT_CONTEXT) \
			--wait \
			--timeout 5m; \
	else \
		helm upgrade --install zeni-x-uat helm/zeni-x \
			--namespace zeni-x-uat \
			--values helm/zeni-x/values-uat.yaml \
			--values helm/zeni-x/values-uat.secret.yaml \
			--kube-context $(UAT_CONTEXT) \
			--wait \
			--timeout 5m; \
	fi

uat-verify:
	@echo "⏳ Waiting for deployment (context: $(UAT_CONTEXT))..."
	kubectl --context=$(UAT_CONTEXT) rollout status deployment/zeni-x -n zeni-x-uat --timeout=120s
	@echo "🔍 Running health check..."
	@kubectl --context=$(UAT_CONTEXT) exec -n zeni-x-uat deploy/zeni-x -c backend -- wget -q -O- http://localhost:8080/health || echo "Health check pending..."

uat-logs:
	kubectl --context=$(UAT_CONTEXT) logs -f deployment/zeni-x -n zeni-x-uat --all-containers=true

# ------------------------------------------------------------
# 清理
# ------------------------------------------------------------
clean:
	@echo "🧹 Cleaning build artifacts..."
	rm -rf dist/
	rm -rf $(FRONTEND_DIR)/dist/
	rm -rf $(FRONTEND_DIR)/node_modules/
	cd $(BACKEND_DIR) && go clean
	@echo "✅ Clean complete!"

# 清理 K8s 资源 (使用 Helm uninstall)
clean-k8s-test:
	@echo "🗑️  Cleaning test environment resources..."
	helm uninstall zeni-x-test --namespace zeni-x-test --kube-context $(TEST_CONTEXT) || echo "No release to uninstall"

clean-k8s-uat:
	@echo "🗑️  Cleaning UAT environment resources..."
	helm uninstall zeni-x-uat --namespace zeni-x-uat --kube-context $(UAT_CONTEXT) || echo "No release to uninstall"

# ------------------------------------------------------------
# Helm 部署
# ------------------------------------------------------------
# Helm 配置
HELM_CHART_DIR ?= helm/zeni-x
HELM_RELEASE_NAME ?= zeni-x
HELM_NAMESPACE ?= zeni-x
HELM_VALUES_FILE ?= values-dev.yaml
HELM_KUBECONTEXT ?=

.PHONY: helm-lint helm-template helm-diff helm-install helm-uninstall helm-status
.PHONY: helm-test helm-uat helm-prod
.PHONY: helm-test-dryrun helm-uat-dryrun helm-prod-dryrun
.PHONY: helm-validate

# Lint Helm Chart
helm-lint:
	@echo "🔍 Linting Helm chart..."
	helm lint $(HELM_CHART_DIR)

# Template Helm Chart
helm-template:
	@echo "📄 Templating Helm chart..."
	helm template $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/$(HELM_VALUES_FILE) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT))

# Diff Helm release (requires helm-diff plugin)
helm-diff:
	@echo "🔄 Diffing Helm release..."
	@if ! helm plugin list | grep -q "diff"; then \
		echo "⚠️  helm-diff plugin not found. Installing..."; \
		helm plugin install https://github.com/databus23/helm-diff; \
	fi
	@if [ -f "config/helm/values-$(HELM_ENV).yaml" ]; then \
		echo "  Using config/helm/values-$(HELM_ENV).yaml"; \
		helm diff upgrade $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
			--values config/helm/values-$(HELM_ENV).yaml \
			--namespace $(HELM_NAMESPACE) \
			$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT)) \
			--install --allow-unreleased; \
	else \
		echo "  Using helm/zeni-x/values-$(HELM_ENV).yaml"; \
		helm diff upgrade $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
			--values $(HELM_CHART_DIR)/values-$(HELM_ENV).yaml \
			--namespace $(HELM_NAMESPACE) \
			$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT)) \
			--install --allow-unreleased; \
	fi

# Install/Upgrade Helm release
helm-install: helm-diff
	@echo "🚀 Installing Helm release..."
	@if [ -f "config/helm/values-$(HELM_ENV).yaml" ]; then \
		echo "  Using config/helm/values-$(HELM_ENV).yaml"; \
		helm upgrade $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
			--values config/helm/values-$(HELM_ENV).yaml \
			--namespace $(HELM_NAMESPACE) \
			--create-namespace \
			--install \
			--wait \
			--timeout 5m \
			$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT)); \
	else \
		echo "  Using helm/zeni-x/values-$(HELM_ENV).yaml"; \
		helm upgrade $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
			--values $(HELM_CHART_DIR)/values-$(HELM_ENV).yaml \
			--namespace $(HELM_NAMESPACE) \
			--create-namespace \
			--install \
			--wait \
			--timeout 5m \
			$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT)); \
	fi
	@echo "✅ Helm release installed successfully!"

# Uninstall Helm release
helm-uninstall:
	@echo "🗑️  Uninstalling Helm release..."
	helm uninstall $(HELM_RELEASE_NAME) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT))
	@echo "✅ Helm release uninstalled!"

# Show Helm release status
helm-status:
	@echo "📊 Helm release status..."
	helm status $(HELM_RELEASE_NAME) \
		--namespace $(HELM_NAMESPACE) \
		$(if $(HELM_KUBECONTEXT),--kube-context $(HELM_KUBECONTEXT))

# 验证 Helm Chart (所有环境)
helm-validate:
	@echo "✅ Validating Helm chart..."
	helm lint $(HELM_CHART_DIR)
	@echo "✅ Helm chart validation passed!"

# 环境快捷命令 - Test
helm-test: build-docker test-push
	$(MAKE) helm-install HELM_ENV=test HELM_NAMESPACE=zeni-x-test $(if $(TEST_CONTEXT),HELM_KUBECONTEXT=$(TEST_CONTEXT))

# 环境快捷命令 - UAT
helm-uat: build-docker uat-push
	$(MAKE) helm-install HELM_ENV=uat HELM_NAMESPACE=zeni-x-uat $(if $(UAT_CONTEXT),HELM_KUBECONTEXT=$(UAT_CONTEXT))

# 环境快捷命令 - Prod
helm-prod:
	$(MAKE) helm-install HELM_ENV=prod HELM_NAMESPACE=zeni-x-prod

# Dry-run 生成部署内容到 debug/ 目录
helm-test-dryrun:
	@echo "📄 Generating test environment manifests to debug/test/..."
	@mkdir -p debug/test
	helm template $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/values-test.yaml \
		--namespace zeni-x-test \
		> debug/test/manifests.yaml
	@echo "✅ Generated: debug/test/manifests.yaml"
	@echo "📁 Total size: $$(du -sh debug/test | cut -f1)"

helm-uat-dryrun:
	@echo "📄 Generating uat environment manifests to debug/uat/..."
	@mkdir -p debug/uat
	helm template $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/values-uat.yaml \
		--namespace zeni-x-uat \
		> debug/uat/manifests.yaml
	@echo "✅ Generated: debug/uat/manifests.yaml"
	@echo "📁 Total size: $$(du -sh debug/uat | cut -f1)"

helm-prod-dryrun:
	@echo "📄 Generating prod environment manifests to debug/prod/..."
	@mkdir -p debug/prod
	helm template $(HELM_RELEASE_NAME) $(HELM_CHART_DIR) \
		--values $(HELM_CHART_DIR)/values-prod.yaml \
		--namespace zeni-x-prod \
		> debug/prod/manifests.yaml
	@echo "✅ Generated: debug/prod/manifests.yaml"
	@echo "📁 Total size: $$(du -sh debug/prod | cut -f1)"

