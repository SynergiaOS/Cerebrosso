#!/bin/bash
# 🔐 Infisical Setup Script for Cerberus Phoenix v2.0
# Integracja z Infisical dla enterprise-grade secret management

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
echo "🔐 INFISICAL SETUP - CERBERUS PHOENIX v2.0"
echo "=========================================="
echo -e "${NC}"

# Configuration
INFISICAL_TOKEN="st.8c1ee774-233b-4187-b12e-cdd58d0898e1.ba805ff4a6f04b5c89b47a7952d35a5e.f87af14f5d44445bbf6c5acb1958a71b"
INFISICAL_PROJECT_ID="1232ea01-7ff9-4eac-be5a-c66a6cb34c88"
INFISICAL_ENV="dev"

# Check if Infisical CLI is installed
check_infisical_cli() {
    echo -e "${BLUE}🔍 Checking Infisical CLI...${NC}"
    
    if ! command -v infisical &> /dev/null; then
        echo -e "${YELLOW}⚠️ Infisical CLI not found. Installing...${NC}"
        
        # Install Infisical CLI
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl -1sLf 'https://dl.cloudsmith.io/public/infisical/infisical-cli/setup.deb.sh' | sudo -E bash
            sudo apt-get update && sudo apt-get install -y infisical
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install infisical/get-cli/infisical
        else
            echo -e "${RED}❌ Unsupported OS. Please install Infisical CLI manually.${NC}"
            echo "Visit: https://infisical.com/docs/cli/overview"
            exit 1
        fi
    fi
    
    echo -e "${GREEN}✅ Infisical CLI ready${NC}"
}

# Login to Infisical
login_infisical() {
    echo -e "${BLUE}🔑 Logging into Infisical...${NC}"
    
    # Set token
    export INFISICAL_TOKEN="$INFISICAL_TOKEN"
    
    # Verify login
    if infisical whoami > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Successfully logged into Infisical${NC}"
    else
        echo -e "${RED}❌ Failed to login to Infisical${NC}"
        echo "Please check your token: $INFISICAL_TOKEN"
        exit 1
    fi
}

# Create .env file with Infisical configuration
create_env_file() {
    echo -e "${BLUE}📝 Creating .env file...${NC}"
    
    cat > .env << EOF
# 🔐 Infisical Configuration
INFISICAL_TOKEN=$INFISICAL_TOKEN
INFISICAL_PROJECT_ID=$INFISICAL_PROJECT_ID
INFISICAL_ENV=$INFISICAL_ENV

# 🌟 Helius API Pro
HELIUS_API_KEY=40a78e4c-bdd0-4338-877a-aa7d56a5f5a0

# ⚡ QuickNode Premium
QUICKNODE_API_KEY=QN_5ca5bc11920f47e6892ed21e8c306a07
QUICKNODE_RPC_URL=https://distinguished-blue-glade.solana-devnet.quiknode.pro/a10fad0f63cdfe46533f1892ac720517b08fe580/

# 🔗 Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet

# 🔐 Vault Configuration
VAULT_URL=http://localhost:8200
VAULT_TOKEN=

# 📊 Database Configuration
DATABASE_URL=postgresql://cerberus:cerberus_password@localhost:5432/cerberus
REDIS_URL=redis://localhost:6379

# 🧠 AI Configuration
OPENAI_API_KEY=
ANTHROPIC_API_KEY=

# 📈 Monitoring
GRAFANA_ADMIN_PASSWORD=admin
PROMETHEUS_RETENTION=15d

# 🚀 Application Configuration
RUST_LOG=info
ENVIRONMENT=development
EOF

    echo -e "${GREEN}✅ .env file created${NC}"
}

# Setup Infisical secrets
setup_secrets() {
    echo -e "${BLUE}🔐 Setting up secrets in Infisical...${NC}"
    
    # Trading secrets
    echo -e "${CYAN}Setting up trading secrets...${NC}"
    infisical secrets set HELIUS_API_KEY "40a78e4c-233b-4187-b12e-cdd58d0898e1" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    infisical secrets set QUICKNODE_API_KEY "QN_5ca5bc11920f47e6892ed21e8c306a07" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    infisical secrets set QUICKNODE_RPC_URL "https://distinguished-blue-glade.solana-devnet.quiknode.pro/a10fad0f63cdfe46533f1892ac720517b08fe580/" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    
    # Solana configuration
    echo -e "${CYAN}Setting up Solana configuration...${NC}"
    infisical secrets set SOLANA_RPC_URL "https://api.devnet.solana.com" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    infisical secrets set SOLANA_NETWORK "devnet" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    
    # Database configuration
    echo -e "${CYAN}Setting up database configuration...${NC}"
    infisical secrets set DATABASE_URL "postgresql://cerberus:cerberus_password@localhost:5432/cerberus" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    infisical secrets set REDIS_URL "redis://localhost:6379" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    
    # Vault configuration
    echo -e "${CYAN}Setting up Vault configuration...${NC}"
    infisical secrets set VAULT_URL "http://localhost:8200" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    
    # Monitoring
    echo -e "${CYAN}Setting up monitoring configuration...${NC}"
    infisical secrets set GRAFANA_ADMIN_PASSWORD "admin" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    infisical secrets set PROMETHEUS_RETENTION "15d" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    
    # Application configuration
    echo -e "${CYAN}Setting up application configuration...${NC}"
    infisical secrets set RUST_LOG "info" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    infisical secrets set ENVIRONMENT "development" --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" || true
    
    echo -e "${GREEN}✅ Secrets configured in Infisical${NC}"
}

# Create Infisical configuration file
create_infisical_config() {
    echo -e "${BLUE}📋 Creating Infisical configuration...${NC}"
    
    cat > .infisical.json << EOF
{
  "workspaceId": "$INFISICAL_PROJECT_ID",
  "defaultEnvironment": "$INFISICAL_ENV",
  "gitBranchToEnvironmentMapping": {
    "main": "prod",
    "staging": "staging", 
    "dev": "dev"
  }
}
EOF

    echo -e "${GREEN}✅ Infisical configuration created${NC}"
}

# Create Docker Compose override for Infisical
create_docker_override() {
    echo -e "${BLUE}🐳 Creating Docker Compose override for Infisical...${NC}"
    
    cat > infrastructure/docker-compose.infisical.yml << EOF
version: '3.8'

services:
  # 🧠 Cerebro-BFF with Infisical
  cerebro-bff:
    environment:
      - INFISICAL_TOKEN=$INFISICAL_TOKEN
      - INFISICAL_PROJECT_ID=$INFISICAL_PROJECT_ID
      - INFISICAL_ENV=$INFISICAL_ENV
    volumes:
      - ../.infisical.json:/app/.infisical.json:ro

  # 🥷 HFT-Ninja with Infisical  
  hft-ninja:
    environment:
      - INFISICAL_TOKEN=$INFISICAL_TOKEN
      - INFISICAL_PROJECT_ID=$INFISICAL_PROJECT_ID
      - INFISICAL_ENV=$INFISICAL_ENV
    volumes:
      - ../.infisical.json:/app/.infisical.json:ro

  # 🔐 Infisical Agent (optional)
  infisical-agent:
    image: infisical/cli:latest
    container_name: cerberus-infisical-agent
    restart: unless-stopped
    environment:
      - INFISICAL_TOKEN=$INFISICAL_TOKEN
      - INFISICAL_PROJECT_ID=$INFISICAL_PROJECT_ID
      - INFISICAL_ENV=$INFISICAL_ENV
    command: ["infisical", "agent", "--interval", "30s"]
    networks:
      - cerberus-net
    labels:
      - "traefik.enable=false"
EOF

    echo -e "${GREEN}✅ Docker Compose override created${NC}"
}

# Test Infisical connection
test_connection() {
    echo -e "${BLUE}🧪 Testing Infisical connection...${NC}"
    
    # Test listing secrets
    if infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Infisical connection successful${NC}"
        
        # Show some secrets (without values)
        echo -e "${CYAN}Available secrets:${NC}"
        infisical secrets --projectId="$INFISICAL_PROJECT_ID" --env="$INFISICAL_ENV" --format=table | head -10
    else
        echo -e "${RED}❌ Failed to connect to Infisical${NC}"
        exit 1
    fi
}

# Main execution
main() {
    echo -e "${YELLOW}🚀 Starting Infisical setup...${NC}"
    
    check_infisical_cli
    login_infisical
    create_env_file
    setup_secrets
    create_infisical_config
    create_docker_override
    test_connection
    
    echo ""
    echo -e "${GREEN}🎉 INFISICAL SETUP COMPLETE!${NC}"
    echo -e "${CYAN}================================${NC}"
    echo ""
    echo -e "${YELLOW}📋 Next steps:${NC}"
    echo "1. 🔄 Sync secrets: make infisical-sync"
    echo "2. 🚀 Deploy with Infisical: docker-compose -f docker-compose.yml -f docker-compose.infisical.yml up -d"
    echo "3. 🔐 Manage secrets: infisical secrets --projectId=$INFISICAL_PROJECT_ID --env=$INFISICAL_ENV"
    echo ""
    echo -e "${PURPLE}🔐 Your secrets are now enterprise-grade secure with Infisical! 🔐${NC}"
}

# Run main function
main "$@"
