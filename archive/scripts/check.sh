#!/bin/bash

# Check development environment dependencies
echo "🔍 Checking development environment..."

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

ERRORS=0

# Check Go
if command -v go &> /dev/null; then
    GO_VERSION=$(go version | awk '{print $3}')
    echo -e "${GREEN}✅ Go installed: $GO_VERSION${NC}"
else
    echo -e "${RED}❌ Go not found. Please install Go 1.21+${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check Node.js
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo -e "${GREEN}✅ Node.js installed: $NODE_VERSION${NC}"
else
    echo -e "${RED}❌ Node.js not found. Please install Node.js 18+${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check pnpm
if command -v pnpm &> /dev/null; then
    PNPM_VERSION=$(pnpm --version)
    echo -e "${GREEN}✅ pnpm installed: $PNPM_VERSION${NC}"
else
    echo -e "${RED}❌ pnpm not found. Please install pnpm${NC}"
    echo "  Run: npm install -g pnpm"
    ERRORS=$((ERRORS + 1))
fi

# Check Docker (optional)
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version | awk '{print $3}')
    echo -e "${GREEN}✅ Docker installed: $DOCKER_VERSION${NC}"
else
    echo -e "${YELLOW}⚠️ Docker not found (optional for local dev)${NC}"
fi

# Check kubectl (optional)
if command -v kubectl &> /dev/null; then
    KUBECTL_VERSION=$(kubectl version --client -o json 2>/dev/null | grep gitVersion | head -1 | awk -F'"' '{print $4}')
    echo -e "${GREEN}✅ kubectl installed: $KUBECTL_VERSION${NC}"
else
    echo -e "${YELLOW}⚠️ kubectl not found (needed for K8s deployment)${NC}"
fi

echo ""
if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✅ All required dependencies are installed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Missing $ERRORS required dependencies${NC}"
    exit 1
fi

