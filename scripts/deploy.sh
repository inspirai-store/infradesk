#!/bin/bash
set -e

# Default environment
ENV=${1:-test}

echo "🚀 Deploying to $ENV environment..."

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Validate environment
if [[ "$ENV" != "test" && "$ENV" != "uat" ]]; then
    echo -e "${RED}Invalid environment: $ENV${NC}"
    echo "Usage: $0 [test|uat]"
    exit 1
fi

# Build Docker images
echo -e "${YELLOW}🐳 Building Docker images...${NC}"
docker build -t zeni-x-frontend:$ENV frontend/
docker build -t zeni-x-backend:$ENV backend/

# Tag and push
REGISTRY=${REGISTRY:-registry.local}
echo -e "${YELLOW}📤 Pushing to registry...${NC}"
docker tag zeni-x-frontend:$ENV $REGISTRY/zeni-x-frontend:$ENV
docker tag zeni-x-backend:$ENV $REGISTRY/zeni-x-backend:$ENV
docker push $REGISTRY/zeni-x-frontend:$ENV
docker push $REGISTRY/zeni-x-backend:$ENV

# Deploy to K8s
echo -e "${YELLOW}🚢 Deploying to Kubernetes...${NC}"
kubectl apply -k deploy/k8s/overlays/$ENV

# Wait for rollout
echo -e "${YELLOW}⏳ Waiting for deployment...${NC}"
kubectl rollout status deployment/zeni-x -n zeni-x-$ENV --timeout=120s

echo -e "${GREEN}✅ Deployed to $ENV environment!${NC}"

# Show access info
if [[ "$ENV" == "test" ]]; then
    echo "Access: http://<node-ip>:30180"
elif [[ "$ENV" == "uat" ]]; then
    echo "Access: http://<node-ip>:30280"
fi

