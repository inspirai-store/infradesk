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
# Tauri window shows log viewer, browser shows full UI
web: _check-deps
    #!/usr/bin/env bash
    set -euo pipefail

    # Kill any existing processes on our ports
    echo "Checking for existing processes..."

    # Kill processes on HTTP API port (12420)
    if lsof -ti:12420 >/dev/null 2>&1; then
        echo "  Stopping process on port 12420..."
        lsof -ti:12420 | xargs kill -9 2>/dev/null || true
    fi

    # Kill processes on Vite dev server port (15073)
    if lsof -ti:15073 >/dev/null 2>&1; then
        echo "  Stopping process on port 15073..."
        lsof -ti:15073 | xargs kill -9 2>/dev/null || true
    fi

    # Small delay to ensure ports are released
    sleep 0.5

    echo "Starting Web debug mode..."
    echo ""
    echo "  HTTP API:   http://127.0.0.1:12420"
    echo "  Browser UI: http://localhost:15073"
    echo "  Tauri:      Log viewer window"
    echo ""
    echo "Open http://localhost:15073 in your browser to use the UI."
    echo "The Tauri window will show backend logs."
    echo ""
    export ZENI_WEB_MODE=1
    cargo tauri dev

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
