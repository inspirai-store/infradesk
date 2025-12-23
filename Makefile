# ============================================================
# Zeni-X Makefile - 多环境构建部署管理
# ============================================================

.PHONY: help dev test uat build clean

# 变量
REGISTRY ?= registry.local
VERSION ?= latest
DEV_DIR ?= .dev
DEV_BACKEND_PID := $(DEV_DIR)/backend.pid
DEV_FRONTEND_PID := $(DEV_DIR)/frontend.pid
DEV_BACKEND_LOG := $(DEV_DIR)/backend.log
DEV_FRONTEND_LOG := $(DEV_DIR)/frontend.log

# 默认目标
help:
	@echo "Zeni-X Build System"
	@echo "==================="
	@echo "  make dev        - 本地开发环境（热重载）"
	@echo "  make dev-start  - 后台启动本地开发（写入 .dev/*.pid & .dev/*.log）"
	@echo "  make dev-stop   - 停止 dev-start 启动的服务"
	@echo "  make dev-status - 查看后台服务状态"
	@echo "  make build      - 构建生产版本"
	@echo "  make test       - 部署到测试环境 K8s"
	@echo "  make uat        - 部署到 UAT 环境 K8s"
	@echo "  make clean      - 清理构建产物"
	@echo ""
	@echo "开发命令:"
	@echo "  make dev-frontend  - 仅启动前端开发服务器"
	@echo "  make dev-backend   - 仅启动后端开发服务器"
	@echo "  make dev-check     - 检查开发环境依赖"

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
	cd frontend && pnpm dev

dev-backend:
	@echo "⚙️ Starting backend dev server..."
	cd backend && go run cmd/server/main.go -config configs/dev.yaml

# 后台启动（避免占用当前终端）
dev-start: dev-check
	@mkdir -p $(DEV_DIR)
	@echo "🚀 Starting dev services in background..."
	@# Ensure frontend deps exist (vite is a devDependency)
	@if [ ! -d "frontend/node_modules" ]; then \
		echo "📦 Installing frontend dependencies (pnpm install)..."; \
		( cd frontend && pnpm install ); \
	fi
	@# Backend
	@if [ -f "$(DEV_BACKEND_PID)" ] && kill -0 "$$(cat $(DEV_BACKEND_PID))" 2>/dev/null; then \
		echo "Backend already running (pid=$$(cat $(DEV_BACKEND_PID)))"; \
	else \
		echo "Starting backend..."; \
		( cd backend; nohup go run cmd/server/main.go -config configs/dev.yaml > ../$(DEV_BACKEND_LOG) 2>&1 & echo $$! > ../$(DEV_BACKEND_PID) ); \
		echo "Backend started (pid=$$(cat $(DEV_BACKEND_PID)))"; \
	fi
	@# Frontend
	@if [ -f "$(DEV_FRONTEND_PID)" ] && kill -0 "$$(cat $(DEV_FRONTEND_PID))" 2>/dev/null; then \
		echo "Frontend already running (pid=$$(cat $(DEV_FRONTEND_PID)))"; \
	else \
		echo "Starting frontend..."; \
		( cd frontend; nohup pnpm dev > ../$(DEV_FRONTEND_LOG) 2>&1 & echo $$! > ../$(DEV_FRONTEND_PID) ); \
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
	cd frontend && pnpm install
	cd backend && go mod download
	@echo "✅ Dependencies installed!"

# ------------------------------------------------------------
# 构建
# ------------------------------------------------------------
build: build-frontend build-backend
	@echo "✅ Build complete!"

build-frontend:
	@echo "📦 Building frontend..."
	cd frontend && pnpm install && pnpm build
	@mkdir -p dist
	@cp -r frontend/dist dist/frontend

build-backend:
	@echo "📦 Building backend..."
	@mkdir -p dist
	cd backend && CGO_ENABLED=1 go build -o ../dist/zeni-x cmd/server/main.go

build-docker:
	@echo "🐳 Building Docker images..."
	docker build -t zeni-x-frontend:$(VERSION) frontend/
	docker build -t zeni-x-backend:$(VERSION) backend/

# ------------------------------------------------------------
# 测试环境 (K8s)
# ------------------------------------------------------------
test: build-docker test-push test-deploy test-verify
	@echo "✅ Deployed to TEST environment!"

test-push:
	@echo "📤 Pushing images to registry (test)..."
	docker tag zeni-x-frontend:$(VERSION) $(REGISTRY)/zeni-x-frontend:test
	docker tag zeni-x-backend:$(VERSION) $(REGISTRY)/zeni-x-backend:test
	docker push $(REGISTRY)/zeni-x-frontend:test
	docker push $(REGISTRY)/zeni-x-backend:test

test-deploy:
	@echo "🚀 Deploying to test environment..."
	kubectl apply -k deploy/k8s/overlays/test

test-verify:
	@echo "⏳ Waiting for deployment..."
	kubectl rollout status deployment/zeni-x -n zeni-x-test --timeout=120s
	@echo "🔍 Running health check..."
	@kubectl exec -n zeni-x-test deploy/zeni-x -c backend -- wget -q -O- http://localhost:8080/health || echo "Health check pending..."

test-logs:
	kubectl logs -f deployment/zeni-x -n zeni-x-test --all-containers=true

# ------------------------------------------------------------
# UAT 环境 (K8s)
# ------------------------------------------------------------
uat: build-docker uat-push uat-deploy uat-verify
	@echo "✅ Deployed to UAT environment!"

uat-push:
	@echo "📤 Pushing images to registry (uat)..."
	docker tag zeni-x-frontend:$(VERSION) $(REGISTRY)/zeni-x-frontend:uat
	docker tag zeni-x-backend:$(VERSION) $(REGISTRY)/zeni-x-backend:uat
	docker push $(REGISTRY)/zeni-x-frontend:uat
	docker push $(REGISTRY)/zeni-x-backend:uat

uat-deploy:
	@echo "🚀 Deploying to UAT environment..."
	kubectl apply -k deploy/k8s/overlays/uat

uat-verify:
	@echo "⏳ Waiting for deployment..."
	kubectl rollout status deployment/zeni-x -n zeni-x-uat --timeout=120s
	@echo "🔍 Running health check..."
	@kubectl exec -n zeni-x-uat deploy/zeni-x -c backend -- wget -q -O- http://localhost:8080/health || echo "Health check pending..."

uat-logs:
	kubectl logs -f deployment/zeni-x -n zeni-x-uat --all-containers=true

# ------------------------------------------------------------
# 清理
# ------------------------------------------------------------
clean:
	@echo "🧹 Cleaning build artifacts..."
	rm -rf dist/
	rm -rf frontend/dist/
	rm -rf frontend/node_modules/
	cd backend && go clean
	@echo "✅ Clean complete!"

# 清理 K8s 资源
clean-k8s-test:
	kubectl delete -k deploy/k8s/overlays/test --ignore-not-found

clean-k8s-uat:
	kubectl delete -k deploy/k8s/overlays/uat --ignore-not-found

