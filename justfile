# Zeni-X Development Justfile
# ============================
#
# Usage:
#   just dev           - Start Tauri client dev mode with IPC (default)
#   just web           - Pure web mode: Vite + Rust HTTP server (no Tauri window)
#   just rust-server   - Start only Rust HTTP server

# Directories
frontend_dir := "services/zeni-x/frontend"
backend_dir := "services/zeni-x/backend"

# Show available commands
default:
    @just --list

# Start Tauri client in development mode (IPC mode)
dev: _check-deps
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🚀 Starting Tauri dev mode (IPC)..."
    echo "🔌 Using IPC mode (Tauri native)"
    export VITE_API_MODE=ipc
    cargo tauri dev

# Pure web mode: Vite + Rust HTTP server (no Tauri window)
# Use this for frontend-only debugging in browser
web: _check-deps
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🌐 Starting pure web mode..."
    echo "   Rust HTTP server: http://localhost:15080"
    echo "   Frontend:         http://localhost:15073"
    echo ""

    # Start Rust HTTP server in background
    echo "⚙️  Starting Rust HTTP server..."
    export SERVER_PORT=15080
    export SQLITE_PATH=./data/zeni-x.db
    cargo run --manifest-path src-tauri/Cargo.toml --bin zeni-x-server &
    SERVER_PID=$!

    # Wait for server to be ready
    sleep 2

    # Start Vite frontend
    echo "🎨 Starting Vite frontend..."
    export VITE_API_MODE=web
    cd {{frontend_dir}} && pnpm dev &
    VITE_PID=$!

    # Cleanup on exit
    trap "kill $SERVER_PID $VITE_PID 2>/dev/null" EXIT

    echo ""
    echo "✅ Web mode started!"
    echo "   Open http://localhost:15073 in your browser"
    echo "   Press Ctrl+C to stop both servers"

    # Wait for either process to exit
    wait

# Start only Rust HTTP server (for debugging or when frontend is running separately)
rust-server:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "⚙️  Starting Rust HTTP server..."
    export SERVER_PORT=15080
    export SQLITE_PATH=./data/zeni-x.db
    cargo run --manifest-path src-tauri/Cargo.toml --bin zeni-x-server

# Start only the backend (Go server)
backend:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "⚙️ Starting backend server..."
    cd {{backend_dir}} && \
        export SERVER_PORT=15080 && \
        export SERVER_MODE=debug && \
        export SQLITE_PATH=./data/zeni-x.db && \
        go run cmd/server/main.go

# Start only the frontend (Vite dev server)
frontend ENV="ipc":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "🎨 Starting frontend dev server (API_MODE={{ENV}})..."
    export VITE_API_MODE={{ENV}}
    cd {{frontend_dir}} && pnpm dev

# Build Tauri client for production
build TAG="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{TAG}}" ]; then
        echo "🏷️ Setting version to {{TAG}}..."
        sed -i.bak 's/"version": "[^"]*"/"version": "{{TAG}}"/' src-tauri/tauri.conf.json && rm -f src-tauri/tauri.conf.json.bak
        sed -i.bak 's/^version = "[^"]*"/version = "{{TAG}}"/' src-tauri/Cargo.toml && rm -f src-tauri/Cargo.toml.bak
    fi
    echo "📦 Building Tauri client..."
    cargo tauri build
    echo "✅ Build complete! Output: src-tauri/target/release/bundle/"

# Install dependencies
install:
    echo "📦 Installing dependencies..."
    cd {{frontend_dir}} && pnpm install
    cd {{backend_dir}} && go mod download
    echo "✅ Dependencies installed!"

# Run frontend tests
test:
    cd {{frontend_dir}} && pnpm test:run

# Lint frontend code
lint:
    cd {{frontend_dir}} && pnpm lint

# Clean build artifacts
clean:
    echo "🧹 Cleaning build artifacts..."
    rm -rf {{frontend_dir}}/dist
    rm -rf {{frontend_dir}}/node_modules
    cd {{backend_dir}} && go clean
    cargo clean --manifest-path src-tauri/Cargo.toml
    echo "✅ Clean complete!"

# Check dependencies
_check-deps:
    #!/usr/bin/env bash
    echo "✅ Checking dependencies..."
    command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo not found"; exit 1; }
    command -v pnpm >/dev/null 2>&1 || { echo "❌ pnpm not found"; exit 1; }
    command -v node >/dev/null 2>&1 || { echo "❌ Node.js not found"; exit 1; }
    echo "✅ All dependencies found!"
