#!/bin/bash
# 🐝 Cerberus Phoenix v3.0 - Hive Mind Deployment Script
# Complete deployment of the Hive Mind architecture

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

# Banner
echo -e "${PURPLE}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                🐝 CERBERUS PHOENIX v3.0                     ║
║                   HIVE MIND DEPLOYMENT                       ║
║                                                              ║
║  🧠 SwarmCoordinator  👑 Agent-Strateg  🔗 Synk            ║
║  🛡️ Chainguardia     ⚡ Performance    🧠 Context Engine   ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check prerequisites
log "🔍 Checking prerequisites..."

# Check Docker
if ! command -v docker &> /dev/null; then
    error "Docker is not installed. Please install Docker first."
fi

# Check Docker Compose
if ! command -v docker-compose &> /dev/null; then
    error "Docker Compose is not installed. Please install Docker Compose first."
fi

# Check if .env file exists
if [ ! -f .env ]; then
    warning ".env file not found. Creating from .env.example..."
    if [ -f .env.example ]; then
        cp .env.example .env
        warning "Please edit .env file with your configuration before continuing."
        read -p "Press Enter to continue after editing .env file..."
    else
        error ".env.example file not found. Please create .env file manually."
    fi
fi

success "Prerequisites check completed"

# Load environment variables
log "📋 Loading environment variables..."
set -a
source .env
set +a
success "Environment variables loaded"

# Create necessary directories
log "📁 Creating necessary directories..."
mkdir -p logs
mkdir -p data/postgres
mkdir -p data/redis
mkdir -p data/qdrant
mkdir -p data/vault
mkdir -p data/grafana
mkdir -p data/prometheus
success "Directories created"

# Build all services
log "🔨 Building all Hive Mind services..."

echo -e "${CYAN}Building SwarmCoordinator...${NC}"
docker-compose build swarm-coordinator

echo -e "${CYAN}Building Agent-Strateg...${NC}"
docker-compose build agent-strateg

echo -e "${CYAN}Building Context Engine...${NC}"
docker-compose build context-engine

echo -e "${CYAN}Building Synk...${NC}"
docker-compose build synk

echo -e "${CYAN}Building Chainguardia...${NC}"
docker-compose build chainguardia

echo -e "${CYAN}Building Performance Optimizer...${NC}"
docker-compose build performance-optimizer

echo -e "${CYAN}Building HFT-Ninja...${NC}"
docker-compose build hft-ninja

echo -e "${CYAN}Building Cerebro-BFF...${NC}"
docker-compose build cerebro-bff

echo -e "${CYAN}Building Telegram Bot...${NC}"
docker-compose build telegram-bot

success "All services built successfully"

# Start infrastructure services first
log "🏗️ Starting infrastructure services..."

echo -e "${CYAN}Starting PostgreSQL...${NC}"
docker-compose up -d postgres

echo -e "${CYAN}Starting Redis...${NC}"
docker-compose up -d redis

echo -e "${CYAN}Starting Qdrant...${NC}"
docker-compose up -d qdrant

echo -e "${CYAN}Starting Vault...${NC}"
docker-compose up -d vault

echo -e "${CYAN}Starting Prometheus...${NC}"
docker-compose up -d prometheus

echo -e "${CYAN}Starting Grafana...${NC}"
docker-compose up -d grafana

echo -e "${CYAN}Starting Traefik...${NC}"
docker-compose up -d traefik

# Wait for infrastructure to be ready
log "⏳ Waiting for infrastructure services to be ready..."
sleep 30

# Check infrastructure health
log "🏥 Checking infrastructure health..."

# Check PostgreSQL
if docker-compose exec -T postgres pg_isready -U postgres > /dev/null 2>&1; then
    success "PostgreSQL is ready"
else
    warning "PostgreSQL is not ready yet, continuing anyway..."
fi

# Check Redis
if docker-compose exec -T redis redis-cli ping > /dev/null 2>&1; then
    success "Redis is ready"
else
    warning "Redis is not ready yet, continuing anyway..."
fi

# Check Qdrant
if curl -f http://localhost:6333/health > /dev/null 2>&1; then
    success "Qdrant is ready"
else
    warning "Qdrant is not ready yet, continuing anyway..."
fi

# Start Hive Mind core services
log "🐝 Starting Hive Mind core services..."

echo -e "${CYAN}Starting SwarmCoordinator...${NC}"
docker-compose up -d swarm-coordinator

echo -e "${CYAN}Starting Agent-Strateg...${NC}"
docker-compose up -d agent-strateg

# Wait for core services
log "⏳ Waiting for core services to initialize..."
sleep 20

# Start supporting services
log "🔧 Starting supporting services..."

echo -e "${CYAN}Starting Context Engine...${NC}"
docker-compose up -d context-engine

echo -e "${CYAN}Starting Synk...${NC}"
docker-compose up -d synk

echo -e "${CYAN}Starting Chainguardia...${NC}"
docker-compose up -d chainguardia

echo -e "${CYAN}Starting Performance Optimizer...${NC}"
docker-compose up -d performance-optimizer

# Wait for supporting services
log "⏳ Waiting for supporting services to initialize..."
sleep 15

# Start application services
log "🚀 Starting application services..."

echo -e "${CYAN}Starting HFT-Ninja...${NC}"
docker-compose up -d hft-ninja

echo -e "${CYAN}Starting Cerebro-BFF...${NC}"
docker-compose up -d cerebro-bff

echo -e "${CYAN}Starting Telegram Bot...${NC}"
docker-compose up -d telegram-bot

# Final health check
log "🏥 Performing final health checks..."

# Wait for all services to be ready
sleep 30

# Check service health
services=(
    "swarm-coordinator:8090"
    "agent-strateg:8100"
    "context-engine:8200"
    "synk:8300"
    "chainguardia:8400"
    "performance-optimizer:8500"
    "hft-ninja:8080"
    "cerebro-bff:3000"
)

healthy_services=0
total_services=${#services[@]}

for service in "${services[@]}"; do
    service_name=$(echo $service | cut -d':' -f1)
    port=$(echo $service | cut -d':' -f2)
    
    if curl -f http://localhost:$port/health > /dev/null 2>&1; then
        success "$service_name is healthy"
        ((healthy_services++))
    else
        warning "$service_name is not responding on port $port"
    fi
done

# Display deployment summary
echo -e "\n${PURPLE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PURPLE}║                    DEPLOYMENT SUMMARY                        ║${NC}"
echo -e "${PURPLE}╚══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${CYAN}🐝 HIVE MIND SERVICES:${NC}"
echo -e "  👑 Agent-Strateg (CEO):     http://localhost:8100"
echo -e "  🐝 SwarmCoordinator:         http://localhost:8090"
echo -e "  🧠 Context Engine:           http://localhost:8200"

echo -e "\n${CYAN}🔧 SUPPORTING SERVICES:${NC}"
echo -e "  🔗 Synk:                     http://localhost:8300"
echo -e "  🛡️ Chainguardia:             http://localhost:8400"
echo -e "  ⚡ Performance Optimizer:    http://localhost:8500"

echo -e "\n${CYAN}🚀 APPLICATION SERVICES:${NC}"
echo -e "  ⚡ HFT-Ninja:                http://localhost:8080"
echo -e "  🧠 Cerebro-BFF:              http://localhost:3000"
echo -e "  📱 Telegram Bot:             Integrated"

echo -e "\n${CYAN}📊 MONITORING & INFRASTRUCTURE:${NC}"
echo -e "  📊 Grafana:                  http://localhost:3001"
echo -e "  📈 Prometheus:               http://localhost:9090"
echo -e "  🗄️ Qdrant:                   http://localhost:6333"
echo -e "  🔐 Vault:                    http://localhost:8200"
echo -e "  🌐 Traefik:                  http://localhost:8082"

echo -e "\n${CYAN}📈 HEALTH STATUS:${NC}"
echo -e "  Healthy Services: $healthy_services/$total_services"

if [ $healthy_services -eq $total_services ]; then
    echo -e "\n${GREEN}🎉 ALL SERVICES DEPLOYED SUCCESSFULLY!${NC}"
    echo -e "${GREEN}🐝 Hive Mind is fully operational and ready for trading!${NC}"
else
    echo -e "\n${YELLOW}⚠️ Some services may need additional time to start.${NC}"
    echo -e "${YELLOW}Check logs with: docker-compose logs [service-name]${NC}"
fi

echo -e "\n${CYAN}🔧 USEFUL COMMANDS:${NC}"
echo -e "  View logs:           docker-compose logs -f [service-name]"
echo -e "  Stop all services:   docker-compose down"
echo -e "  Restart service:     docker-compose restart [service-name]"
echo -e "  View status:         docker-compose ps"

echo -e "\n${PURPLE}🐝 Welcome to the Hive Mind! 🐝${NC}"
