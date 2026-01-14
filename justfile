# Zeni-X Development Justfile
# ============================
#
# Usage:
#   just dev     - Start Tauri desktop app in development mode (IPC)
#   just web     - Start Web debug mode (HTTP API + Browser UI)
#   just build   - Build production release

# Directories
frontend_dir := "services/zeni-x/frontend"

# Show available commands
default:
    @just --list

# Start Tauri desktop app in development mode (IPC)
dev: _check-deps
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting Tauri dev mode..."
    cargo tauri dev

# Start Web debug mode (HTTP API + Browser UI)
# Starts backend first, waits for it to be ready, then starts frontend
# Uses port 15074 for frontend (different from dev mode's 15073) to allow both modes to run simultaneously
web: _check-deps
    #!/usr/bin/env bash
    set -euo pipefail

    # Cleanup function
    cleanup() {
        echo ""
        echo "Shutting down..."
        # Kill background processes
        if [ -n "${BACKEND_PID:-}" ]; then
            kill $BACKEND_PID 2>/dev/null || true
        fi
        if [ -n "${FRONTEND_PID:-}" ]; then
            kill $FRONTEND_PID 2>/dev/null || true
        fi
        exit 0
    }
    trap cleanup SIGINT SIGTERM

    # Kill any existing processes on our ports
    echo "Checking for existing processes..."

    # Kill processes on HTTP API port (12420)
    if lsof -ti:12420 >/dev/null 2>&1; then
        echo "  Stopping process on port 12420..."
        lsof -ti:12420 | xargs kill -9 2>/dev/null || true
    fi

    # Kill processes on Vite dev server port (15074 for web mode)
    if lsof -ti:15074 >/dev/null 2>&1; then
        echo "  Stopping process on port 15074..."
        lsof -ti:15074 | xargs kill -9 2>/dev/null || true
    fi

    # Small delay to ensure ports are released
    sleep 0.5

    echo ""
    echo "===== Starting Web Debug Mode ====="
    echo ""

    # Step 1: Start backend HTTP server
    echo "[1/3] Starting backend HTTP server..."
    cargo run --bin web-server --manifest-path src-tauri/Cargo.toml &
    BACKEND_PID=$!

    # Step 2: Wait for backend to be ready
    echo "[2/3] Waiting for backend to be ready..."
    max_attempts=30
    attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://127.0.0.1:12420/api/health > /dev/null 2>&1; then
            echo "       Backend is ready!"
            break
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    if [ $attempt -eq $max_attempts ]; then
        echo "ERROR: Backend failed to start within 30 seconds"
        cleanup
        exit 1
    fi

    # Step 3: Start frontend Vite dev server with log piping
    echo "[3/3] Starting frontend Vite dev server..."
    export VITE_PORT=15074
    pnpm --dir services/zeni-x/frontend dev 2>&1 | ./scripts/vite-log-pipe.sh &
    FRONTEND_PID=$!

    # Wait for Vite to be ready
    sleep 2

    echo ""
    echo "===== Web Debug Mode Started ====="
    echo ""
    echo "  HTTP API:   http://127.0.0.1:12420"
    echo "  Browser UI: http://localhost:15074"
    echo "  Log Viewer: http://localhost:15074/log-viewer.html"
    echo ""
    echo "Press Ctrl+C to stop all services."
    echo ""

    # Open Log Viewer in browser
    if command -v open >/dev/null 2>&1; then
        open "http://localhost:15074/log-viewer.html"
    elif command -v xdg-open >/dev/null 2>&1; then
        xdg-open "http://localhost:15074/log-viewer.html"
    fi

    # Wait for any background process to exit
    wait

# Build Tauri desktop app for production
build TAG="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{TAG}}" ]; then
        echo "Setting version to {{TAG}}..."
        sed -i.bak 's/"version": "[^"]*"/"version": "{{TAG}}"/' src-tauri/tauri.conf.json && rm -f src-tauri/tauri.conf.json.bak
        sed -i.bak 's/^version = "[^"]*"/version = "{{TAG}}"/' src-tauri/Cargo.toml && rm -f src-tauri/Cargo.toml.bak
    fi
    echo "Building Tauri desktop app..."
    cargo tauri build
    echo "Build complete! Output: src-tauri/target/release/bundle/"

# Install dependencies
install:
    echo "Installing dependencies..."
    cd {{frontend_dir}} && pnpm install
    echo "Dependencies installed!"

# Run frontend tests
test:
    cd {{frontend_dir}} && pnpm test:run

# Lint frontend code
lint:
    cd {{frontend_dir}} && pnpm lint

# Clean build artifacts
clean:
    echo "Cleaning build artifacts..."
    rm -rf {{frontend_dir}}/dist
    rm -rf {{frontend_dir}}/node_modules
    cargo clean --manifest-path src-tauri/Cargo.toml
    echo "Clean complete!"

# Check dependencies
_check-deps:
    #!/usr/bin/env bash
    echo "Checking dependencies..."
    command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo not found"; exit 1; }
    command -v pnpm >/dev/null 2>&1 || { echo "pnpm not found"; exit 1; }
    command -v node >/dev/null 2>&1 || { echo "Node.js not found"; exit 1; }
    echo "All dependencies found!"
