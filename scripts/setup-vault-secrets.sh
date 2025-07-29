#!/bin/bash
# 🔐 Cerberus Phoenix v2.0 - Vault Secrets Management
# Secure setup for production secrets

set -euo pipefail

# 🎨 Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 📝 Logging function
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

# 🔍 Check if Vault is available
check_vault() {
    log "Checking Vault availability..."
    
    if ! command -v vault &> /dev/null; then
        error "Vault CLI not found. Please install HashiCorp Vault."
    fi
    
    if ! vault status &> /dev/null; then
        error "Vault server not accessible. Please start Vault server."
    fi
    
    success "Vault is available"
}

# 🔑 Initialize Vault secrets
setup_secrets() {
    log "Setting up Vault secrets for Cerberus Phoenix..."
    
    # Enable KV secrets engine if not already enabled
    vault secrets enable -path=cerberus kv-v2 2>/dev/null || true
    
    # Create secrets with placeholder values
    log "Creating secret placeholders..."
    
    # API Keys
    vault kv put cerberus/api-keys \
        helius_api_key="REPLACE_WITH_REAL_HELIUS_KEY" \
        quicknode_api_key="REPLACE_WITH_REAL_QUICKNODE_KEY" \
        alchemy_api_key="REPLACE_WITH_REAL_ALCHEMY_KEY" \
        birdeye_api_key="REPLACE_WITH_REAL_BIRDEYE_KEY"
    
    # Database credentials
    vault kv put cerberus/database \
        postgres_password="$(openssl rand -base64 32)" \
        postgres_user="cerberus" \
        postgres_db="cerberus_phoenix"
    
    # Monitoring credentials
    vault kv put cerberus/monitoring \
        grafana_password="$(openssl rand -base64 16)" \
        prometheus_password="$(openssl rand -base64 16)"
    
    # Trading configuration
    vault kv put cerberus/trading \
        max_position_size="1000" \
        risk_limit="0.02" \
        stop_loss_percentage="0.05" \
        take_profit_percentage="0.15"
    
    success "Vault secrets initialized"
}

# 🐳 Create Docker secrets
create_docker_secrets() {
    log "Creating Docker secrets from Vault..."
    
    # Get secrets from Vault and create Docker secrets
    vault kv get -field=postgres_password cerberus/database | docker secret create postgres_password - 2>/dev/null || true
    vault kv get -field=grafana_password cerberus/monitoring | docker secret create grafana_password - 2>/dev/null || true
    vault kv get -field=helius_api_key cerberus/api-keys | docker secret create helius_api_key - 2>/dev/null || true
    vault kv get -field=quicknode_api_key cerberus/api-keys | docker secret create quicknode_api_key - 2>/dev/null || true
    vault kv get -field=birdeye_api_key cerberus/api-keys | docker secret create birdeye_api_key - 2>/dev/null || true
    
    success "Docker secrets created"
}

# 📋 Display setup instructions
show_instructions() {
    log "🔐 Vault Setup Complete!"
    echo ""
    echo "📝 Next steps:"
    echo "1. Update API keys in Vault:"
    echo "   vault kv patch cerberus/api-keys helius_api_key=YOUR_REAL_KEY"
    echo ""
    echo "2. Start secure deployment:"
    echo "   docker-compose -f infrastructure/docker-compose.chainguard-secure.yml up -d"
    echo ""
    echo "3. Access services:"
    echo "   - Grafana: http://localhost:3001 (admin/$(vault kv get -field=grafana_password cerberus/monitoring))"
    echo "   - HFT-Ninja: http://localhost:8090"
    echo "   - Cerebro-BFF: http://localhost:3000"
    echo ""
    warning "Remember to update placeholder API keys with real values!"
}

# 🚀 Main execution
main() {
    log "🛡️ Starting Cerberus Phoenix Vault Setup..."
    
    check_vault
    setup_secrets
    create_docker_secrets
    show_instructions
    
    success "Vault setup completed successfully!"
}

# Execute main function
main "$@"
