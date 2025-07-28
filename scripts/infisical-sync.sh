#!/bin/bash
# 🔄 Infisical Sync Script for Cerberus Phoenix v2.0
# Synchronizacja secrets między Infisical a lokalnym środowiskiem

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
echo "🔄 INFISICAL SYNC - CERBERUS PHOENIX v2.0"
echo "========================================"
echo -e "${NC}"

# Configuration
INFISICAL_TOKEN="st.8c1ee774-233b-4187-b12e-cdd58d0898e1.ba805ff4a6f04b5c89b47a7952d35a5e.f87af14f5d44445bbf6c5acb1958a71b"
INFISICAL_PROJECT_ID="1232ea01-7ff9-4eac-be5a-c66a6cb34c88"
INFISICAL_ENV="dev"

# Export secrets to .env file
export_to_env() {
    echo -e "${BLUE}📤 Exporting secrets to .env file...${NC}"

    # Set token
    export INFISICAL_TOKEN="$INFISICAL_TOKEN"

    # Create backup of existing .env
    if [ -f .env ]; then
        cp .env .env.backup.$(date +%Y%m%d_%H%M%S)
        echo -e "${YELLOW}📋 Backup created: .env.backup.$(date +%Y%m%d_%H%M%S)${NC}"
    fi

    # Export secrets from Infisical
    echo -e "${CYAN}Fetching secrets from Infisical...${NC}"
    infisical export --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" --format=dotenv > .env.infisical.tmp

    # Merge with existing configuration
    cat > .env << EOF
# 🔐 Cerberus Phoenix v2.0 - Environment Variables
# Secrets from Infisical + Local Configuration
# Project ID: $INFISICAL_PROJECT_ID
# Environment: $INFISICAL_ENV
# Exported at: $(date -u +%Y-%m-%dT%H:%M:%SZ)

# 🔐 SECRETS FROM INFISICAL
EOF

    # Add Infisical secrets
    cat .env.infisical.tmp >> .env

    # Add local configuration
    cat >> .env << EOF

# 🌐 Solana Configuration - DEVNET
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_COMMITMENT=confirmed
SOLANA_WS_URL=wss://api.devnet.solana.com
SOLANA_NETWORK=devnet

# 🚀 Jito Configuration
JITO_BLOCK_ENGINE_URL=https://mainnet.block-engine.jito.wtf
JITO_TIP_AMOUNT=10000

# 🧠 AI Configuration
FINLLAMA_API_URL=http://localhost:11434
DEEPSEEK_API_URL=http://localhost:11435

# 🗄️ Qdrant Configuration
QDRANT_URL=http://qdrant:6333
QDRANT_COLLECTION=cerberus_context

# 🔄 Multi-RPC Configuration (FREE Providers)
HELIUS_BASE_URL=https://api.helius.xyz
HELIUS_MONTHLY_LIMIT=100000
ALCHEMY_MAINNET_URL=https://solana-mainnet.g.alchemy.com/v2/Wu2Kqfk_50kW_Zs4ifjuf3c7afxLOs7R
ALCHEMY_DEVNET_URL=https://solana-devnet.g.alchemy.com/v2/Wu2Kqfk_50kW_Zs4ifjuf3c7afxLOs7R
ALCHEMY_MONTHLY_LIMIT=100000
PUBLIC_MAINNET_RPC=https://api.mainnet-beta.solana.com
PUBLIC_DEVNET_RPC=https://api.devnet.solana.com

# 🦈 Piranha Strategy Configuration
MAX_POSITION_SIZE_SOL=0.1
ENABLE_JITO=true

# 🌐 Frontend Configuration
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_NINJA_URL=http://localhost:8081

# 📊 Monitoring
PROMETHEUS_RETENTION_TIME=30d
GRAFANA_ADMIN_PASSWORD=admin

# 🔧 Development Settings
DEV_MODE=true
RUST_LOG=debug
NODE_ENV=development

# 🎣 Helius Webhook Integration
HELIUS_WEBHOOK_URL=https://your-domain.com/webhooks/helius
KESTRA_TRIGGER_URL=http://kestra:8080/api/v1/executions/trigger/helius-webhook
CEREBRO_BFF_URL=http://cerebro-bff:3000
WEBHOOK_RATE_LIMIT=100
WEBHOOK_TIMEOUT_MS=5000
WEBHOOK_BATCH_SIZE=10
WEBHOOK_PARALLEL_PROCESSING=true
EOF

    # Clean up
    rm .env.infisical.tmp

    echo -e "${GREEN}✅ Secrets exported and merged with local config${NC}"
}

# Sync secrets to Vault
sync_to_vault() {
    echo -e "${BLUE}🔐 Syncing secrets to Vault...${NC}"
    
    # Check if Vault is running
    if ! docker ps | grep -q "cerberus-vault"; then
        echo -e "${YELLOW}⚠️ Vault container not running. Starting...${NC}"
        cd infrastructure && docker-compose up -d vault
        sleep 10
    fi
    
    # Check if Vault is unsealed
    VAULT_STATUS=$(docker exec cerberus-vault vault status -format=json 2>/dev/null || echo '{"sealed":true}')
    SEALED=$(echo $VAULT_STATUS | jq -r '.sealed // true')
    
    if [ "$SEALED" = "true" ]; then
        echo -e "${YELLOW}⚠️ Vault is sealed. Please unseal it first:${NC}"
        echo "./secure-key-manager.sh unseal"
        return 1
    fi
    
    # Get secrets from Infisical
    export INFISICAL_TOKEN="$INFISICAL_TOKEN"
    SECRETS_JSON=$(infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" --format=json)
    
    # Store each secret in Vault
    echo "$SECRETS_JSON" | jq -r '.[] | @base64' | while read -r secret; do
        SECRET_DATA=$(echo "$secret" | base64 -d)
        SECRET_KEY=$(echo "$SECRET_DATA" | jq -r '.key')
        SECRET_VALUE=$(echo "$SECRET_DATA" | jq -r '.value')
        
        # Store in Vault under infisical/ path
        docker exec cerberus-vault vault kv put "infisical/$SECRET_KEY" \
            value="$SECRET_VALUE" \
            source="infisical" \
            project_id="$INFISICAL_PROJECT_ID" \
            environment="$INFISICAL_ENV" \
            synced_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)" > /dev/null
        
        echo -e "${CYAN}  ✓ Synced: $SECRET_KEY${NC}"
    done
    
    echo -e "${GREEN}✅ Secrets synced to Vault${NC}"
}

# Update Docker Compose environment
update_docker_env() {
    echo -e "${BLUE}🐳 Updating Docker Compose environment...${NC}"
    
    # Create environment file for Docker Compose
    export INFISICAL_TOKEN="$INFISICAL_TOKEN"
    infisical export --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" --format=dotenv > infrastructure/.env.infisical
    
    echo -e "${GREEN}✅ Docker environment updated${NC}"
}

# Validate secrets
validate_secrets() {
    echo -e "${BLUE}🔍 Validating secrets...${NC}"
    
    # Required secrets for Cerberus Phoenix v2.0
    REQUIRED_SECRETS=(
        "HELIUS_API_KEY"
        "QUICKNODE_API_KEY"
        "QUICKNODE_RPC_URL"
        "SOLANA_RPC_URL"
        "DATABASE_URL"
        "REDIS_URL"
    )
    
    export INFISICAL_TOKEN="$INFISICAL_TOKEN"
    
    for secret in "${REQUIRED_SECRETS[@]}"; do
        if infisical secrets get "$secret" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" > /dev/null 2>&1; then
            echo -e "${GREEN}  ✓ $secret${NC}"
        else
            echo -e "${RED}  ✗ $secret (missing)${NC}"
        fi
    done
    
    echo -e "${GREEN}✅ Validation complete${NC}"
}

# Show secrets summary
show_summary() {
    echo -e "${BLUE}📊 Secrets summary...${NC}"
    
    export INFISICAL_TOKEN="$INFISICAL_TOKEN"
    
    echo -e "${CYAN}Project: $INFISICAL_PROJECT_ID${NC}"
    echo -e "${CYAN}Environment: $INFISICAL_ENV${NC}"
    echo ""
    
    # Count secrets
    SECRET_COUNT=$(infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" --format=json | jq length)
    echo -e "${CYAN}Total secrets: $SECRET_COUNT${NC}"
    
    # Show secret names (without values)
    echo -e "${CYAN}Available secrets:${NC}"
    infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" --format=json | jq -r '.[].key' | sort | while read -r key; do
        echo -e "${BLUE}  • $key${NC}"
    done
}

# Test API connections with synced secrets
test_api_connections() {
    echo -e "${BLUE}🧪 Testing API connections with synced secrets...${NC}"
    
    # Source the .env file
    if [ -f .env ]; then
        source .env
    else
        echo -e "${RED}❌ .env file not found. Run export first.${NC}"
        return 1
    fi
    
    # Test Helius API
    if [ ! -z "$HELIUS_API_KEY" ]; then
        echo -e "${CYAN}Testing Helius API...${NC}"
        HELIUS_RESPONSE=$(curl -s "https://api.helius.xyz/v0/addresses/So11111111111111111111111111111111111111112/balances?api-key=$HELIUS_API_KEY" | head -c 100)
        if [[ "$HELIUS_RESPONSE" == *"tokens"* ]] || [[ "$HELIUS_RESPONSE" == *"balance"* ]]; then
            echo -e "${GREEN}  ✓ Helius API connection successful${NC}"
        else
            echo -e "${RED}  ✗ Helius API connection failed${NC}"
        fi
    fi
    
    # Test QuickNode API
    if [ ! -z "$QUICKNODE_RPC_URL" ]; then
        echo -e "${CYAN}Testing QuickNode API...${NC}"
        QUICKNODE_RESPONSE=$(curl -s -X POST "$QUICKNODE_RPC_URL" \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc": "2.0", "id": 1, "method": "getHealth"}' | head -c 100)
        if [[ "$QUICKNODE_RESPONSE" == *"result"* ]] || [[ "$QUICKNODE_RESPONSE" == *"ok"* ]]; then
            echo -e "${GREEN}  ✓ QuickNode API connection successful${NC}"
        else
            echo -e "${RED}  ✗ QuickNode API connection failed${NC}"
        fi
    fi
    
    echo -e "${GREEN}✅ API connection tests complete${NC}"
}

# Main execution
main() {
    local action="${1:-all}"
    
    case "$action" in
        "export")
            export_to_env
            ;;
        "vault")
            sync_to_vault
            ;;
        "docker")
            update_docker_env
            ;;
        "validate")
            validate_secrets
            ;;
        "summary")
            show_summary
            ;;
        "test")
            test_api_connections
            ;;
        "all"|*)
            echo -e "${YELLOW}🚀 Starting full sync...${NC}"
            export_to_env
            sync_to_vault
            update_docker_env
            validate_secrets
            test_api_connections
            show_summary
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}🎉 INFISICAL SYNC COMPLETE!${NC}"
    echo -e "${CYAN}============================${NC}"
    echo ""
    echo -e "${YELLOW}📋 Available commands:${NC}"
    echo "  export   - Export secrets to .env file"
    echo "  vault    - Sync secrets to Vault"
    echo "  docker   - Update Docker environment"
    echo "  validate - Validate required secrets"
    echo "  summary  - Show secrets summary"
    echo "  test     - Test API connections"
    echo "  all      - Run all sync operations (default)"
    echo ""
    echo -e "${PURPLE}🔐 Your secrets are synchronized and secure! 🔐${NC}"
}

# Run main function
main "$@"
