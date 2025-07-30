#!/bin/bash
# 🚀 Cerberus Phoenix v2.0 - Quick Start (No Vault)
# Szybkie uruchomienie systemu z .env

set -euo pipefail

# 🎨 Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# 📝 Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
    exit 1
}

# 🔧 Setup environment
setup_environment() {
    log "🔧 Setting up environment..."
    
    # Use .env.example as template - USER MUST SET REAL KEYS
    if [[ -f .env.example ]]; then
        cp .env.example .env
        warning "Using .env.example template - YOU MUST SET REAL API KEYS!"
        echo "Edit .env file and replace placeholder values with real API keys"
    else
        error "No .env.example found!"
    fi
    
    # Source environment variables
    source .env
    
    success "Environment configured"
}

# 🐳 Start infrastructure
start_infrastructure() {
    log "🐳 Starting infrastructure services..."
    
    cd infrastructure
    
    # Start core services
    docker-compose up -d postgres qdrant traefik grafana prometheus
    
    cd ..
    
    # Wait for services
    log "Waiting for services to start..."
    sleep 20
    
    # Health checks
    if docker exec cerberus-postgres pg_isready -U cerberus 2>/dev/null; then
        success "PostgreSQL ready"
    else
        warning "PostgreSQL starting..."
    fi
    
    if curl -sf http://localhost:6333/health > /dev/null 2>&1; then
        success "Qdrant ready"
    else
        warning "Qdrant starting..."
    fi
    
    if curl -sf http://localhost:3001/api/health > /dev/null 2>&1; then
        success "Grafana ready"
    else
        warning "Grafana starting..."
    fi
}

# 🥷 Start HFT-Ninja
start_hft_ninja() {
    log "🥷 Starting HFT-Ninja..."
    
    cd services/hft-ninja
    
    # Start in background
    RUST_LOG=info cargo run > ../../logs/hft-ninja.log 2>&1 &
    HFT_NINJA_PID=$!
    echo $HFT_NINJA_PID > ../../logs/hft-ninja.pid
    
    cd ../..
    
    # Wait and check
    sleep 15
    
    if curl -sf http://localhost:8090/health > /dev/null 2>&1; then
        success "HFT-Ninja ready at http://localhost:8090"
    else
        warning "HFT-Ninja starting... (check logs/hft-ninja.log)"
    fi
}

# 🧠 Start Cerebro-BFF
start_cerebro_bff() {
    log "🧠 Starting Cerebro-BFF..."
    
    cd services/cerebro-bff
    
    # Start in background
    RUST_LOG=info cargo run > ../../logs/cerebro-bff.log 2>&1 &
    CEREBRO_BFF_PID=$!
    echo $CEREBRO_BFF_PID > ../../logs/cerebro-bff.pid
    
    cd ../..
    
    # Wait and check
    sleep 15
    
    if curl -sf http://localhost:3000/health > /dev/null 2>&1; then
        success "Cerebro-BFF ready at http://localhost:3000"
    else
        warning "Cerebro-BFF starting... (check logs/cerebro-bff.log)"
    fi
}

# 📊 Show system status
show_status() {
    log "🎉 Cerberus Phoenix v2.0 Quick Start Complete!"
    echo ""
    echo -e "${PURPLE}🔗 Service URLs:${NC}"
    echo "  🥷 HFT-Ninja (Trading):     http://localhost:8090"
    echo "  🧠 Cerebro-BFF (AI):        http://localhost:3000"
    echo "  📊 Grafana (Monitoring):    http://localhost:3001 (admin/admin)"
    echo "  🗄️  Qdrant (Vector DB):      http://localhost:6333"
    echo "  🔄 Traefik (Proxy):         http://localhost:8080"
    echo ""
    echo -e "${PURPLE}🔑 API Keys status:${NC}"
    echo "  🌐 Helius:    ${HELIUS_API_KEY:0:10}... (${#HELIUS_API_KEY} chars)"
    echo "  ⚡ Alchemy:   ${ALCHEMY_API_KEY:0:10}... (${#ALCHEMY_API_KEY} chars)"
    echo "  🚀 QuickNode: ${QUICKNODE_API_KEY:0:10}... (${#QUICKNODE_API_KEY} chars)"
    echo "  🐦 Birdeye:   ${BIRDEYE_API_KEY:0:10}... (${#BIRDEYE_API_KEY} chars)"
    echo ""
    echo -e "${PURPLE}📋 Log Files:${NC}"
    echo "  🥷 HFT-Ninja:  tail -f logs/hft-ninja.log"
    echo "  🧠 Cerebro-BFF: tail -f logs/cerebro-bff.log"
    echo "  🐳 Docker:     docker-compose -f infrastructure/docker-compose.yml logs -f"
    echo ""
    echo -e "${PURPLE}🛠️  Management:${NC}"
    echo "  🛑 Stop all:   ./scripts/stop-cerberus.sh"
    echo "  📊 Status:     docker ps"
    echo "  🔍 Health:     curl http://localhost:8090/health"
    echo ""
    echo -e "${GREEN}🚀 System ready for Solana devnet trading!${NC}"
}

# 🚀 Main execution
main() {
    log "🚀 Cerberus Phoenix v2.0 Quick Start..."
    
    # Create logs directory
    mkdir -p logs
    
    # Check prerequisites
    if ! command -v docker &> /dev/null; then
        error "Docker not found. Please install Docker."
    fi
    
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Please install Rust."
    fi
    
    # Start services
    setup_environment
    start_infrastructure
    start_hft_ninja
    start_cerebro_bff
    show_status
    
    success "🎉 Quick start completed!"
    
    # Keep script running
    log "Press Ctrl+C to stop all services..."
    trap 'log "Stopping services..."; ./scripts/stop-cerberus.sh 2>/dev/null || true; exit 0' INT
    
    while true; do
        sleep 60
        log "System running... (Ctrl+C to stop)"
    done
}

# Execute main function
main "$@"
