#!/bin/bash
# 🏗️ Start minimal infrastructure for development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🏗️ Starting Development Infrastructure${NC}"

# Change to project root
cd "$(dirname "$0")/.."

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker is not running${NC}"
    echo -e "${YELLOW}💡 Please start Docker and try again${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker is running${NC}"

# Stop any existing containers
echo -e "${YELLOW}🛑 Stopping existing containers...${NC}"
docker-compose -f infrastructure/docker-compose.yml down 2>/dev/null || true

# Start only essential services for development
echo -e "${BLUE}🚀 Starting essential services...${NC}"

# Start Postgres
echo -e "${YELLOW}📊 Starting Postgres...${NC}"
docker-compose -f infrastructure/docker-compose.yml up -d postgres

# Wait for Postgres to be ready
echo -e "${YELLOW}⏳ Waiting for Postgres to be ready...${NC}"
timeout=30
counter=0
while ! docker exec cerberus-postgres pg_isready -U cerberus &> /dev/null; do
    if [ $counter -ge $timeout ]; then
        echo -e "${RED}❌ Postgres failed to start within ${timeout} seconds${NC}"
        exit 1
    fi
    sleep 1
    counter=$((counter + 1))
    echo -n "."
done
echo ""
echo -e "${GREEN}✅ Postgres is ready${NC}"

# Start Qdrant
echo -e "${YELLOW}🔍 Starting Qdrant...${NC}"
docker-compose -f infrastructure/docker-compose.yml up -d qdrant

# Wait for Qdrant to be ready
echo -e "${YELLOW}⏳ Waiting for Qdrant to be ready...${NC}"
timeout=30
counter=0
while ! curl -s http://localhost:6333/ &> /dev/null; do
    if [ $counter -ge $timeout ]; then
        echo -e "${RED}❌ Qdrant failed to start within ${timeout} seconds${NC}"
        exit 1
    fi
    sleep 1
    counter=$((counter + 1))
    echo -n "."
done
echo ""
echo -e "${GREEN}✅ Qdrant is ready${NC}"

# Optional: Start monitoring (Prometheus + Grafana) if requested
if [ "$1" = "--with-monitoring" ]; then
    echo -e "${YELLOW}📈 Starting monitoring services...${NC}"
    docker-compose -f infrastructure/docker-compose.yml up -d prometheus grafana
    
    echo -e "${YELLOW}⏳ Waiting for monitoring services...${NC}"
    sleep 5
    
    if curl -s http://localhost:9090/-/healthy &> /dev/null; then
        echo -e "${GREEN}✅ Prometheus is ready at http://localhost:9090${NC}"
    fi
    
    if curl -s http://localhost:3001/api/health &> /dev/null; then
        echo -e "${GREEN}✅ Grafana is ready at http://localhost:3001${NC}"
    fi
fi

echo ""
echo -e "${GREEN}🎉 Development infrastructure is ready!${NC}"
echo ""
echo -e "${BLUE}📊 Services running:${NC}"
echo -e "  • Postgres: ${GREEN}localhost:5432${NC} (user: cerberus, db: cerberus_phoenix)"
echo -e "  • Qdrant: ${GREEN}localhost:6334${NC} (vector database)"

if [ "$1" = "--with-monitoring" ]; then
    echo -e "  • Prometheus: ${GREEN}localhost:9090${NC} (metrics)"
    echo -e "  • Grafana: ${GREEN}localhost:3001${NC} (dashboards)"
fi

echo ""
echo -e "${YELLOW}🚀 Ready to start development!${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Run: ${GREEN}./scripts/dev-cerebro-bff.sh${NC} (for Cerebro-BFF development)"
echo -e "  2. Or run: ${GREEN}./scripts/dev-hft-ninja.sh${NC} (for HFT-Ninja development)"
echo ""
echo -e "${YELLOW}💡 To stop all services: ${GREEN}docker-compose -f infrastructure/docker-compose.yml down${NC}"

# Show container status
echo ""
echo -e "${BLUE}📋 Container Status:${NC}"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep cerberus || echo "No containers running"
