#!/bin/bash

# 强制覆盖功能 - 快速部署脚本

set -e

echo "🚀 开始部署强制覆盖功能..."

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 检查是否在 zeni-x 目录
if [ ! -d "frontend" ] || [ ! -d "backend" ]; then
    echo -e "${RED}❌ 错误：请在 zeni-x 目录下运行此脚本${NC}"
    exit 1
fi

# 1. 构建前端
echo -e "\n${YELLOW}📦 步骤 1/5: 构建前端...${NC}"
cd frontend
npm run build
cd ..
echo -e "${GREEN}✓ 前端构建完成${NC}"

# 2. 构建后端
echo -e "\n${YELLOW}🔨 步骤 2/5: 编译后端...${NC}"
cd backend
go build -o zeni-x-server cmd/server/main.go
cd ..
echo -e "${GREEN}✓ 后端编译完成${NC}"

# 3. 构建 Docker 镜像
echo -e "\n${YELLOW}🐳 步骤 3/5: 构建 Docker 镜像...${NC}"
docker build -t zeni-x-frontend:latest -f frontend/Dockerfile frontend/
docker build -t zeni-x-backend:latest -f backend/Dockerfile backend/
echo -e "${GREEN}✓ Docker 镜像构建完成${NC}"

# 4. 推送到镜像仓库（可选）
read -p "是否推送到远程镜像仓库？(y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    read -p "请输入镜像仓库地址 (例如: registry.example.com): " REGISTRY
    
    echo -e "\n${YELLOW}📤 步骤 4/5: 推送镜像到 ${REGISTRY}...${NC}"
    docker tag zeni-x-frontend:latest ${REGISTRY}/zeni-x-frontend:latest
    docker tag zeni-x-backend:latest ${REGISTRY}/zeni-x-backend:latest
    docker push ${REGISTRY}/zeni-x-frontend:latest
    docker push ${REGISTRY}/zeni-x-backend:latest
    echo -e "${GREEN}✓ 镜像推送完成${NC}"
else
    echo -e "${YELLOW}⏭️  跳过镜像推送${NC}"
fi

# 5. 部署到 Kubernetes
read -p "是否部署到 Kubernetes？(y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    read -p "选择环境 (test/uat/prod): " ENV
    
    if [ "$ENV" != "test" ] && [ "$ENV" != "uat" ] && [ "$ENV" != "prod" ]; then
        echo -e "${RED}❌ 错误：环境必须是 test, uat 或 prod${NC}"
        exit 1
    fi
    
    echo -e "\n${YELLOW}☸️  步骤 5/5: 部署到 ${ENV} 环境...${NC}"
    
    # 应用配置
    kubectl apply -k deploy/k8s/overlays/${ENV}
    
    # 重启 Deployment
    echo -e "\n${YELLOW}🔄 重启 Pods...${NC}"
    kubectl rollout restart deployment/zeni-x-frontend -n zeni-x
    kubectl rollout restart deployment/zeni-x-backend -n zeni-x
    
    # 等待部署完成
    echo -e "\n${YELLOW}⏳ 等待部署完成...${NC}"
    kubectl rollout status deployment/zeni-x-frontend -n zeni-x
    kubectl rollout status deployment/zeni-x-backend -n zeni-x
    
    echo -e "${GREEN}✓ 部署完成${NC}"
    
    # 显示 Pod 状态
    echo -e "\n${YELLOW}📊 Pod 状态：${NC}"
    kubectl get pods -n zeni-x -l app=zeni-x
    
else
    echo -e "${YELLOW}⏭️  跳过 Kubernetes 部署${NC}"
fi

echo -e "\n${GREEN}🎉 部署完成！${NC}"
echo -e "\n${YELLOW}📝 测试步骤：${NC}"
echo "1. 打开连接管理页面"
echo "2. 点击 '自动发现'"
echo "3. 选择已存在的服务并导入"
echo "4. 应该弹出确认对话框询问是否强制覆盖"
echo ""
echo -e "${YELLOW}📖 详细测试指南请查看：${NC}"
echo "   K8S_IMPORT_TESTING_GUIDE.md"

