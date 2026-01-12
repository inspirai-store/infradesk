# Zeni-X - Database Management Desktop App

## Overview

Zeni-X is a cross-platform desktop database management application built with Tauri 2.0.

## Tech Stack

- **Backend**: Rust + Tauri 2.0
- **Frontend**: Vue 3 + TypeScript + Naive UI
- **Database**: SQLite (app config) + MySQL/Redis (target databases)
- **Kubernetes**: Cluster resource visualization and port forwarding

## Development Commands

```bash
just dev      # Start Tauri development mode
just build    # Build production release
just install  # Install dependencies
just test     # Run frontend tests
just lint     # Code linting
just clean    # Clean build artifacts
```

## Project Structure

```
zeni-x/
├── src-tauri/                  # Tauri Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri commands (IPC interface)
│   │   ├── services/           # Business logic
│   │   └── db/                 # SQLite data layer
│   └── Cargo.toml
├── services/zeni-x/frontend/   # Vue frontend
├── config/env/                 # Environment configuration
├── data/                       # Runtime data
└── archive/                    # Archived content (reference only)
```

## Architecture

```
Vue Frontend (IPC) → Tauri Commands → Rust Services → Database/K8s
```

## Archived Content

The `archive/` directory contains legacy web mode code:
- Go backend server
- Rust HTTP server
- Kubernetes Helm charts
- Deployment scripts

**These are for reference only and are no longer maintained.**

## Key Features

- MySQL database management (browse, query, CRUD)
- Redis key-value management
- Kubernetes cluster resource visualization
- Automatic port forwarding for K8s services
- Query history and saved queries
- Dark theme UI
