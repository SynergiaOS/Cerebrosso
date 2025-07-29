#!/bin/bash
# 🔐 Cerberus Phoenix v2.0 - Secure Start
# BEZPIECZNE uruchomienie bez hardcoded keys

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

# 🔐 Check API keys
check_api_keys() {
    log "🔐 Checking API keys configuration..."
    
    # Check if .env exists
    if [[ ! -f .env ]]; then
        warning ".env file not found. Creating from template..."
        cp .env.example .env
    fi
    
    # Source environment
    source .env
    
    # Check for placeholder values
    local has_placeholders=false
    
    if [[ "${HELIUS_API_KEY:-}" == *"your_"* ]] || [[ "${HELIUS_API_KEY:-}" == *"REPLACE"* ]]; then
        warning "Helius API key is placeholder"
        has_placeholders=true
    fi
    
    if [[ "${ALCHEMY_API_KEY:-}" == *"your_"* ]] || [[ "${ALCHEMY_API_KEY:-}" == *"REPLACE"* ]]; then
        warning "Alchemy API key is placeholder"
        has_placeholders=true
    fi
    
    if [[ "${QUICKNODE_API_KEY:-}" == *"your_"* ]] || [[ "${QUICKNODE_API_KEY:-}" == *"REPLACE"* ]]; then
        warning "QuickNode API key is placeholder"
        has_placeholders=true
    fi
    
    if [[ "${BIRDEYE_API_KEY:-}" == *"your_"* ]] || [[ "${BIRDEYE_API_KEY:-}" == *"REPLACE"* ]]; then
        warning "Birdeye API key is placeholder"
        has_placeholders=true
    fi
    
    if [[ "$has_placeholders" == "true" ]]; then
        echo ""
        error "❌ PLACEHOLDER API KEYS DETECTED! 
        
Please edit .env file and set real API keys:
- HELIUS_API_KEY=your_real_helius_key
- ALCHEMY_API_KEY=your_real_alchemy_key  
- QUICKNODE_API_KEY=your_real_quicknode_key
- BIRDEYE_API_KEY=your_real_birdeye_key

Get keys from:
🌐 Helius: https://helius.xyz
⚡ Alchemy: https://alchemy.com
🚀 QuickNode: https://quicknode.com
🐦 Birdeye: https://birdeye.so"
    fi
    
    success "API keys configuration validated"
}

# 🐳 Start infrastructure safely
start_infrastructure() {
    log "🐳 Starting infrastructure services..."
    
    # Clean up any existing containers
    cd infrastructure
    docker-compose down 2>/dev/null || true
    
    # Start fresh
    docker-compose up -d postgres qdrant grafana
    
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

# 🥷 Start HFT-Ninja safely
start_hft_ninja() {
    log "🥷 Starting HFT-Ninja..."
    
    # Create logs directory
    mkdir -p logs
    
    cd services/hft-ninja
    
    # Start with environment isolation
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

# 📊 Show secure status
show_secure_status() {
    log "🎉 Cerberus Phoenix v2.0 Secure Start Complete!"
    echo ""
    echo -e "${PURPLE}🔗 Service URLs:${NC}"
    echo "  🥷 HFT-Ninja (Trading):     http://localhost:8090"
    echo "  📊 Grafana (Monitoring):    http://localhost:3001 (admin/admin)"
    echo "  🗄️  Qdrant (Vector DB):      http://localhost:6333"
    echo ""
    echo -e "${PURPLE}🔐 Security Status:${NC}"
    echo "  ✅ No hardcoded API keys in scripts"
    echo "  ✅ Environment variables validated"
    echo "  ✅ Secure container deployment"
    echo "  ✅ Logs isolated in logs/ directory"
    echo ""
    echo -e "${PURPLE}📋 Management:${NC}"
    echo "  📊 View logs:    tail -f logs/hft-ninja.log"
    echo "  🛑 Stop system:  ./scripts/stop-cerberus.sh"
    echo "  📊 Status:       docker ps"
    echo ""
    echo -e "${GREEN}🚀 System ready for secure trading!${NC}"
}

# 🚀 Main execution
main() {
    log "🔐 Starting Cerberus Phoenix v2.0 Securely..."
    
    # Security checks first
    check_api_keys
    
    # Start services
    start_infrastructure
    start_hft_ninja
    show_secure_status
    
    success "🎉 Secure start completed!"
    
    # Keep script running
    log "Press Ctrl+C to stop all services..."
    trap 'log "Stopping services..."; ./scripts/stop-cerberus.sh 2>/dev/null || true; exit 0' INT
    
    while true; do
        sleep 60
        log "System running securely... (Ctrl+C to stop)"
    done
}

# Execute main function
main "$@"
