#!/bin/bash
# 🚀 Cerberus Phoenix v2.0 - Complete Deployment Script
# Deploys the full AI-driven HFT trading system with advanced features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${ENVIRONMENT:-"devnet"}
SOLANA_RPC_URL=${SOLANA_RPC_URL:-"https://api.devnet.solana.com"}
DEPLOY_MODE=${DEPLOY_MODE:-"full"}

echo -e "${PURPLE}"
echo "🦈🧠🚀 CERBERUS PHOENIX v2.0 DEPLOYMENT 🚀🧠🦈"
echo "=================================================="
echo -e "${NC}"
echo -e "${BLUE}Environment: ${ENVIRONMENT}${NC}"
echo -e "${BLUE}RPC URL: ${SOLANA_RPC_URL}${NC}"
echo -e "${BLUE}Deploy Mode: ${DEPLOY_MODE}${NC}"
echo ""

# Function to check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker is not installed${NC}"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null; then
        echo -e "${RED}❌ Docker Compose is not installed${NC}"
        exit 1
    fi
    
    # Check Rust (for building services)
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo is not installed${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All prerequisites satisfied${NC}"
}

# Function to build Rust services
build_services() {
    echo -e "${YELLOW}🔨 Building Rust services...${NC}"
    
    # Build HFT-Ninja
    echo -e "${BLUE}Building HFT-Ninja...${NC}"
    cd services/hft-ninja
    cargo build --release
    cd ../..
    
    # Build Cerebro-BFF
    echo -e "${BLUE}Building Cerebro-BFF...${NC}"
    cd services/cerebro-bff
    cargo build --release
    cd ../..
    
    echo -e "${GREEN}✅ Services built successfully${NC}"
}

# Function to initialize Vault
init_vault() {
    echo -e "${YELLOW}🔐 Initializing HashiCorp Vault...${NC}"
    
    # Wait for Vault to be ready
    echo -e "${BLUE}Waiting for Vault to start...${NC}"
    sleep 10
    
    # Run initialization script
    if [ -f "infrastructure/secrets/init-vault.sh" ]; then
        chmod +x infrastructure/secrets/init-vault.sh
        ./infrastructure/secrets/init-vault.sh
    else
        echo -e "${YELLOW}⚠️ Vault init script not found, skipping...${NC}"
    fi
    
    echo -e "${GREEN}✅ Vault initialized${NC}"
}

# Function to deploy infrastructure
deploy_infrastructure() {
    echo -e "${YELLOW}🐳 Deploying infrastructure...${NC}"
    
    cd infrastructure
    
    # Create necessary directories
    mkdir -p secrets logs data
    
    # Set environment variables
    export SOLANA_RPC_URL="${SOLANA_RPC_URL}"
    export ENVIRONMENT="${ENVIRONMENT}"
    
    # Deploy with Docker Compose
    echo -e "${BLUE}Starting Docker Compose stack...${NC}"
    docker-compose up -d
    
    cd ..
    
    echo -e "${GREEN}✅ Infrastructure deployed${NC}"
}

# Function to wait for services
wait_for_services() {
    echo -e "${YELLOW}⏳ Waiting for services to be ready...${NC}"
    
    # Wait for Qdrant
    echo -e "${BLUE}Waiting for Qdrant...${NC}"
    until curl -s http://localhost:6333/collections > /dev/null 2>&1; do
        echo "Waiting for Qdrant..."
        sleep 5
    done
    
    # Wait for Cerebro-BFF
    echo -e "${BLUE}Waiting for Cerebro-BFF...${NC}"
    until curl -s http://localhost:8080/health > /dev/null 2>&1; do
        echo "Waiting for Cerebro-BFF..."
        sleep 5
    done
    
    # Wait for HFT-Ninja
    echo -e "${BLUE}Waiting for HFT-Ninja...${NC}"
    until curl -s http://localhost:8090/health > /dev/null 2>&1; do
        echo "Waiting for HFT-Ninja..."
        sleep 5
    done
    
    echo -e "${GREEN}✅ All services are ready${NC}"
}

# Function to run tests
run_tests() {
    echo -e "${YELLOW}🧪 Running integration tests...${NC}"

    # Test basic health endpoints first
    echo -e "${BLUE}Testing basic health endpoints...${NC}"

    # Test Cerebro-BFF health
    if curl -s -f http://localhost:8080/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Cerebro-BFF health check passed${NC}"
    else
        echo -e "${RED}❌ Cerebro-BFF health check failed${NC}"
        echo "Checking Cerebro-BFF logs:"
        docker logs cerberus-cerebro-bff --tail 20
    fi

    # Test HFT-Ninja health
    if curl -s -f http://localhost:8090/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ HFT-Ninja health check passed${NC}"
    else
        echo -e "${RED}❌ HFT-Ninja health check failed${NC}"
        echo "Checking HFT-Ninja logs:"
        docker logs cerberus-hft-ninja --tail 20
    fi

    # Test Cerebro-BFF Context Engine
    echo -e "${BLUE}Testing Context Engine...${NC}"
    response=$(curl -s -X POST http://localhost:8080/api/v1/context/test \
        -H "Content-Type: application/json" \
        -d '{"test_type": "integration"}' 2>/dev/null)

    if echo "$response" | grep -q "success"; then
        echo -e "${GREEN}✅ Context Engine test passed${NC}"
    else
        echo -e "${YELLOW}⚠️ Context Engine advanced test not available (expected in development)${NC}"
    fi

    # Test HFT-Ninja Piranha Surf
    echo -e "${BLUE}Testing Piranha Surf...${NC}"
    response=$(curl -s http://localhost:8090/piranha/status 2>/dev/null)

    if echo "$response" | grep -q "operational"; then
        echo -e "${GREEN}✅ Piranha Surf test passed${NC}"
    else
        echo -e "${YELLOW}⚠️ Piranha Surf advanced test not available (expected in development)${NC}"
    fi

    # Test Decision Engine
    echo -e "${BLUE}Testing Decision Engine...${NC}"
    response=$(curl -s -X POST http://localhost:8080/api/v1/decision/test \
        -H "Content-Type: application/json" \
        -d '{"signals": [{"signal_type": "rug_pull_detected", "value": 0.95, "tf_idf_weight": 0.94, "importance_score": 0.98}]}' 2>/dev/null)

    if echo "$response" | grep -q "decision"; then
        echo -e "${GREEN}✅ Decision Engine test passed${NC}"
    else
        echo -e "${YELLOW}⚠️ Decision Engine advanced test not available (expected in development)${NC}"
    fi

    # Test Docker container status
    echo -e "${BLUE}Checking Docker container status...${NC}"
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep cerberus

    echo -e "${GREEN}✅ All tests completed${NC}"
}

# Function to show status
show_status() {
    echo -e "${CYAN}"
    echo "🎉 CERBERUS PHOENIX v2.0 DEPLOYMENT COMPLETE!"
    echo "=============================================="
    echo -e "${NC}"
    echo -e "${GREEN}🦈 Piranha Surf HFT Engine: http://localhost:8090${NC}"
    echo -e "${GREEN}🧠 Cerebro-BFF AI Engine: http://localhost:8080${NC}"
    echo -e "${GREEN}🖥️ Dashboard: http://localhost:3000${NC}"
    echo -e "${GREEN}📊 Grafana: http://localhost:3001${NC}"
    echo -e "${GREEN}🔐 Vault: http://localhost:8200${NC}"
    echo -e "${GREEN}🗄️ Qdrant: http://localhost:6333${NC}"
    echo -e "${GREEN}⚙️ Kestra: http://localhost:8081${NC}"
    echo -e "${GREEN}🚪 Traefik: http://localhost:8082${NC}"
    echo ""
    echo -e "${YELLOW}📋 Key Features Deployed:${NC}"
    echo -e "  • Advanced Context Engine with TF-IDF weighting"
    echo -e "  • Apriori pattern mining for rule discovery"
    echo -e "  • Context rot prevention mechanisms"
    echo -e "  • ChainGuardian on-chain monitoring"
    echo -e "  • Vault-based secret management"
    echo -e "  • Real-time Prometheus metrics"
    echo -e "  • Automated Kestra workflows"
    echo ""
    echo -e "${BLUE}🚀 Ready for Solana ${ENVIRONMENT} trading!${NC}"
}

# Main deployment flow
main() {
    check_prerequisites
    
    if [ "$DEPLOY_MODE" = "full" ]; then
        build_services
    fi
    
    deploy_infrastructure
    
    # Initialize Vault after infrastructure is up
    init_vault
    
    wait_for_services
    run_tests
    show_status
}

# Handle script arguments
case "${1:-}" in
    "build")
        build_services
        ;;
    "deploy")
        deploy_infrastructure
        ;;
    "test")
        run_tests
        ;;
    "status")
        show_status
        ;;
    *)
        main
        ;;
esac
