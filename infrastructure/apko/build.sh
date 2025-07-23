#!/bin/bash
# 🐺 Cerberus Phoenix v2.0 - Apko Build Script
# Builds all ultra-minimal container images with SBOM

set -e

echo "🐺 Building Cerberus Phoenix v2.0 Container Images with Apko..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REGISTRY=${REGISTRY:-"ghcr.io/synergiaos"}
TAG=${TAG:-"v2.0.0"}
PLATFORM=${PLATFORM:-"linux/amd64,linux/arm64"}

# Function to build image with Apko
build_image() {
    local service=$1
    local manifest=$2
    
    echo -e "${BLUE}🔨 Building ${service}...${NC}"
    
    # Build with Apko
    apko build ${manifest} ${REGISTRY}/cerberus-${service}:${TAG} cerberus-${service}.tar
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Successfully built ${service}${NC}"
        
        # Generate SBOM
        echo -e "${YELLOW}📋 Generating SBOM for ${service}...${NC}"
        apko sbom ${manifest} > cerberus-${service}-sbom.json
        
        # Load and push image (if registry is available)
        if command -v docker &> /dev/null; then
            echo -e "${YELLOW}📦 Loading image to Docker...${NC}"
            docker load < cerberus-${service}.tar
            
            # Tag for different architectures
            docker tag ${REGISTRY}/cerberus-${service}:${TAG} ${REGISTRY}/cerberus-${service}:latest
            
            echo -e "${GREEN}✅ Image loaded: ${REGISTRY}/cerberus-${service}:${TAG}${NC}"
        fi
    else
        echo -e "${RED}❌ Failed to build ${service}${NC}"
        exit 1
    fi
}

# Check if apko is installed
if ! command -v apko &> /dev/null; then
    echo -e "${RED}❌ Apko is not installed. Please install it first:${NC}"
    echo "go install chainguard.dev/apko@latest"
    exit 1
fi

echo -e "${BLUE}🚀 Starting build process...${NC}"
echo -e "${YELLOW}Registry: ${REGISTRY}${NC}"
echo -e "${YELLOW}Tag: ${TAG}${NC}"
echo -e "${YELLOW}Platform: ${PLATFORM}${NC}"
echo ""

# Build all services
build_image "hft-ninja" "hft-ninja.yaml"
build_image "cerebro-bff" "cerebro-bff.yaml"
build_image "dashboard" "dashboard.yaml"

echo ""
echo -e "${GREEN}🎉 All images built successfully!${NC}"
echo ""
echo -e "${BLUE}📊 Image Summary:${NC}"
echo -e "  • ${REGISTRY}/cerberus-hft-ninja:${TAG}"
echo -e "  • ${REGISTRY}/cerberus-cerebro-bff:${TAG}"
echo -e "  • ${REGISTRY}/cerberus-dashboard:${TAG}"
echo ""
echo -e "${YELLOW}📋 SBOM files generated:${NC}"
echo -e "  • cerberus-hft-ninja-sbom.json"
echo -e "  • cerberus-cerebro-bff-sbom.json"
echo -e "  • cerberus-dashboard-sbom.json"
echo ""
echo -e "${GREEN}✅ Build complete! Images are ready for deployment.${NC}"
