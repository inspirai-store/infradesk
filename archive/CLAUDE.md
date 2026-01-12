# Archive - Reference Only

This directory contains archived code and configurations from Zeni-X project.

**These files are for reference only and are no longer maintained.**

## Contents

- `go-backend/` - Go Web backend server (Gin + SQLite)
- `rust-http/` - Rust HTTP server (Axum, for web debug mode)
- `helm/` - Kubernetes Helm Charts
- `config/` - Archived configuration files
- `scripts/` - Deployment scripts

## Important Notes

1. This code may not be compatible with the current Tauri version
2. For historical reference and architectural learning only
3. If you need to restore web mode, re-adaptation is required

## Original Architecture

The archived code supported a dual-mode deployment:
- **Tauri Desktop Mode**: IPC communication via Tauri
- **Web Mode**: HTTP REST API via Go/Rust servers

The project has been simplified to focus solely on the Tauri desktop application.
