#!/bin/bash
set -e

echo "🔨 Building Zeni-X..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build frontend
echo -e "${YELLOW}📦 Building frontend...${NC}"
cd frontend
pnpm install
pnpm build
cd ..

# Build backend
echo -e "${YELLOW}📦 Building backend...${NC}"
cd backend
CGO_ENABLED=1 go build -o ../dist/zeni-x cmd/server/main.go
cd ..

echo -e "${GREEN}✅ Build complete!${NC}"
echo "Frontend: frontend/dist/"
echo "Backend: dist/zeni-x"

