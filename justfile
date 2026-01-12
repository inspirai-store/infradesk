# Zeni-X Development Justfile
# ============================
#
# Usage:
#   just dev     - Start Tauri desktop app in development mode
#   just build   - Build production release

# Directories
frontend_dir := "services/zeni-x/frontend"

# Show available commands
default:
    @just --list

# Start Tauri desktop app in development mode
dev: _check-deps
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting Tauri dev mode..."
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
