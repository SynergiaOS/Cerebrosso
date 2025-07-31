#!/bin/bash
# 🚀 Cerberus Phoenix v2.0 - Complete System Startup
# Uruchamia cały system z Vault i wszystkimi serwisami

set -euo pipefail

# 🎨 Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# 🔐 Setup Vault with all API keys
setup_vault() {
    log "🔐 Setting up Vault with API keys..."
    
    # Start Vault in dev mode (for development)
    if ! pgrep -f "vault server" > /dev/null; then
        log "Starting Vault server in dev mode..."
        # Generate random token for security
        local vault_token=$(openssl rand -hex 16)
        vault server -dev -dev-root-token-id="$vault_token" &
        sleep 3
        export VAULT_ADDR='http://127.0.0.1:8200'
        export VAULT_TOKEN="$vault_token"
        log "Vault token: $vault_token (save this for manual access)"
    fi
    
    # Enable KV secrets engine
    vault secrets enable -path=cerberus kv-v2 2>/dev/null || true
    
    # Store API keys placeholders - REPLACE WITH REAL VALUES
    log "Storing API key placeholders in Vault..."
    vault kv put cerberus/api-keys \
        helius_api_key="REPLACE_WITH_REAL_HELIUS_KEY" \
        alchemy_api_key="REPLACE_WITH_REAL_ALCHEMY_KEY" \
        quicknode_api_key="REPLACE_WITH_REAL_QUICKNODE_KEY" \
        birdeye_api_key="REPLACE_WITH_REAL_BIRDEYE_KEY"

    # Store wallet placeholders - NEVER USE REAL KEYS IN SCRIPTS
    vault kv put cerberus/wallets \
        mainnet_wallet_private_key="REPLACE_WITH_REAL_WALLET_KEY" \
        test_wallet_seed="REPLACE_WITH_REAL_SEED"
    
    # Store database credentials
    vault kv put cerberus/database \
        postgres_password="cerberus_secure_password_2024" \
        postgres_user="cerberus" \
        postgres_db="cerberus_phoenix"
    
    # Store monitoring credentials
    vault kv put cerberus/monitoring \
        grafana_password="admin" \
        prometheus_retention="30d"
    
    # Store trading configuration
    vault kv put cerberus/trading \
        max_position_size="0.1" \
        enable_jito="true" \
        jito_tip_amount="10000" \
        risk_limit="0.02"
    
    success "Vault configured with all secrets"
}

# 🐳 Create Docker secrets from Vault
create_docker_secrets() {
    log "🐳 Creating Docker secrets from Vault..."
    
    # Initialize Docker Swarm if needed
    if ! docker info | grep -q "Swarm: active"; then
        docker swarm init --advertise-addr 127.0.0.1 || true
    fi
    
    # Remove existing secrets
    docker secret rm postgres_password grafana_password helius_api_key quicknode_api_key birdeye_api_key 2>/dev/null || true
    
    # Create new secrets from Vault
    vault kv get -field=postgres_password cerberus/database | docker secret create postgres_password -
    vault kv get -field=grafana_password cerberus/monitoring | docker secret create grafana_password -
    vault kv get -field=helius_api_key cerberus/api-keys | docker secret create helius_api_key -
    vault kv get -field=quicknode_api_key cerberus/api-keys | docker secret create quicknode_api_key -
    vault kv get -field=birdeye_api_key cerberus/api-keys | docker secret create birdeye_api_key -
    
    success "Docker secrets created"
}

# 🏗️ Start infrastructure services
start_infrastructure() {
    log "🏗️ Starting infrastructure services..."
    
    cd infrastructure
    
    # Start core infrastructure
    docker-compose up -d postgres qdrant traefik
    
    # Wait for services to be ready
    log "Waiting for infrastructure to be ready..."
    sleep 15
    
    # Check PostgreSQL
    if docker exec cerberus-postgres pg_isready -U cerberus; then
        success "PostgreSQL is ready"
    else
        warning "PostgreSQL not ready yet"
    fi
    
    # Check Qdrant
    if curl -sf http://localhost:6333/health > /dev/null; then
        success "Qdrant is ready"
    else
        warning "Qdrant not ready yet"
    fi
    
    cd ..
}

# 📊 Start monitoring services
start_monitoring() {
    log "📊 Starting monitoring services..."
    
    cd infrastructure
    docker-compose up -d grafana prometheus
    cd ..
    
    sleep 10
    
    if curl -sf http://localhost:3001/api/health > /dev/null; then
        success "Grafana is ready at http://localhost:3001"
    else
        warning "Grafana starting up..."
    fi
}

# 🥷 Start HFT-Ninja
start_hft_ninja() {
    log "🥷 Starting HFT-Ninja..."
    
    # Export environment variables from Vault
    export HELIUS_API_KEY=$(vault kv get -field=helius_api_key cerberus/api-keys)
    export QUICKNODE_API_KEY=$(vault kv get -field=quicknode_api_key cerberus/api-keys)
    export ALCHEMY_API_KEY=$(vault kv get -field=alchemy_api_key cerberus/api-keys)
    
    cd services/hft-ninja
    
    # Start in background
    RUST_LOG=info cargo run &
    HFT_NINJA_PID=$!
    
    cd ../..
    
    # Wait for startup
    sleep 10
    
    if curl -sf http://localhost:8090/health > /dev/null; then
        success "HFT-Ninja is ready at http://localhost:8090"
    else
        warning "HFT-Ninja starting up..."
    fi
}

# 🧠 Start Cerebro-BFF
start_cerebro_bff() {
    log "🧠 Starting Cerebro-BFF..."
    
    # Export environment variables from Vault
    export HELIUS_API_KEY=$(vault kv get -field=helius_api_key cerberus/api-keys)
    export BIRDEYE_API_KEY=$(vault kv get -field=birdeye_api_key cerberus/api-keys)
    export DATABASE_URL="postgresql://cerberus:$(vault kv get -field=postgres_password cerberus/database)@localhost:5432/cerberus_phoenix"
    export QDRANT_URL="http://localhost:6333"
    
    cd services/cerebro-bff
    
    # Start in background
    RUST_LOG=info cargo run &
    CEREBRO_BFF_PID=$!
    
    cd ../..
    
    # Wait for startup
    sleep 10
    
    if curl -sf http://localhost:3000/health > /dev/null; then
        success "Cerebro-BFF is ready at http://localhost:3000"
    else
        warning "Cerebro-BFF starting up..."
    fi
}

# 📊 Display system status
show_status() {
    log "🎉 Cerberus Phoenix v2.0 System Status"
    echo ""
    echo -e "${PURPLE}🔗 Service URLs:${NC}"
    echo "  🥷 HFT-Ninja (Trading):     http://localhost:8090"
    echo "  🧠 Cerebro-BFF (AI):        http://localhost:3000"
    echo "  📊 Grafana (Monitoring):    http://localhost:3001 (admin/admin)"
    echo "  🗄️  Qdrant (Vector DB):      http://localhost:6333"
    echo "  🔐 Vault (Secrets):         http://localhost:8200"
    echo ""
    echo -e "${PURPLE}🔐 Vault Secrets:${NC}"
    echo "  📋 API Keys:     vault kv get cerberus/api-keys"
    echo "  💰 Wallets:      vault kv get cerberus/wallets"
    echo "  🗄️  Database:     vault kv get cerberus/database"
    echo "  📊 Monitoring:   vault kv get cerberus/monitoring"
    echo "  🎯 Trading:      vault kv get cerberus/trading"
    echo ""
    echo -e "${PURPLE}🛠️  Management Commands:${NC}"
    echo "  📊 View logs:    docker-compose -f infrastructure/docker-compose.yml logs -f"
    echo "  🔍 Check status: docker ps"
    echo "  🛑 Stop system:  pkill -f 'cargo run' && docker-compose -f infrastructure/docker-compose.yml down"
    echo ""
    echo -e "${GREEN}🚀 System ready for trading on Solana devnet!${NC}"
}

# 🚀 Main execution
main() {
    log "🚀 Starting Cerberus Phoenix v2.0 Complete System..."
    
    # Check prerequisites
    if ! command -v vault &> /dev/null; then
        error "Vault CLI not found. Please install HashiCorp Vault."
    fi
    
    if ! command -v docker &> /dev/null; then
        error "Docker not found. Please install Docker."
    fi
    
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo not found. Please install Rust."
    fi
    
    # Setup and start services
    setup_vault
    create_docker_secrets
    start_infrastructure
    start_monitoring
    start_hft_ninja
    start_cerebro_bff
    show_status
    
    success "🎉 Cerberus Phoenix v2.0 fully operational!"
    
    # Keep script running
    log "Press Ctrl+C to stop all services..."
    trap 'log "Stopping services..."; pkill -f "cargo run"; docker-compose -f infrastructure/docker-compose.yml down; exit 0' INT
    
    while true; do
        sleep 60
        log "System running... (Ctrl+C to stop)"
    done
}

# Execute main function
main "$@"
