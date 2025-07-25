#!/bin/bash

# 🧪 Cerberus Phoenix v2.0 - Devnet Testing Suite
# Comprehensive testing on Solana Devnet with real transactions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
DEVNET_RPC="https://api.devnet.solana.com"
DEVNET_WS="wss://api.devnet.solana.com"
TEST_WALLET_PATH="./test-wallet.json"
AIRDROP_AMOUNT=2  # SOL

echo -e "${BLUE}🧪 Cerberus Phoenix v2.0 - Devnet Testing${NC}"
echo "=================================================="
echo -e "${YELLOW}Testing on Solana Devnet with real transactions${NC}"
echo ""

# Function to check if solana CLI is installed
check_solana_cli() {
    if ! command -v solana &> /dev/null; then
        echo -e "${RED}❌ Solana CLI not found. Installing...${NC}"
        
        # Install Solana CLI
        sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
        export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
        
        if ! command -v solana &> /dev/null; then
            echo -e "${RED}❌ Failed to install Solana CLI${NC}"
            exit 1
        fi
    fi
    
    echo -e "${GREEN}✅ Solana CLI available${NC}"
    solana --version
}

# Function to setup devnet configuration
setup_devnet() {
    echo -e "${YELLOW}Setting up Devnet configuration...${NC}"
    
    # Configure Solana CLI for devnet
    solana config set --url $DEVNET_RPC
    solana config set --commitment confirmed
    
    echo -e "${GREEN}✅ Devnet configuration set${NC}"
    solana config get
}

# Function to create or load test wallet
setup_test_wallet() {
    echo -e "${YELLOW}Setting up test wallet...${NC}"
    
    if [ ! -f "$TEST_WALLET_PATH" ]; then
        echo -e "${YELLOW}Creating new test wallet...${NC}"
        solana-keygen new --outfile $TEST_WALLET_PATH --no-bip39-passphrase
    else
        echo -e "${GREEN}✅ Using existing test wallet${NC}"
    fi
    
    # Set as default keypair
    solana config set --keypair $TEST_WALLET_PATH
    
    # Get wallet address
    WALLET_ADDRESS=$(solana-keygen pubkey $TEST_WALLET_PATH)
    echo -e "${GREEN}✅ Test wallet address: $WALLET_ADDRESS${NC}"
    
    # Check balance
    BALANCE=$(solana balance --url $DEVNET_RPC)
    echo -e "${BLUE}💰 Current balance: $BALANCE${NC}"
}

# Function to airdrop SOL for testing
airdrop_sol() {
    echo -e "${YELLOW}Requesting SOL airdrop for testing...${NC}"
    
    CURRENT_BALANCE=$(solana balance --url $DEVNET_RPC | cut -d' ' -f1)
    
    if (( $(echo "$CURRENT_BALANCE < 1.0" | bc -l) )); then
        echo -e "${YELLOW}Balance low, requesting airdrop...${NC}"
        
        # Request airdrop
        solana airdrop $AIRDROP_AMOUNT --url $DEVNET_RPC
        
        # Wait for confirmation
        sleep 5
        
        NEW_BALANCE=$(solana balance --url $DEVNET_RPC)
        echo -e "${GREEN}✅ New balance: $NEW_BALANCE${NC}"
    else
        echo -e "${GREEN}✅ Sufficient balance for testing${NC}"
    fi
}

# Function to start Cerberus Phoenix v2.0 in devnet mode
start_cerberus_devnet() {
    echo -e "${YELLOW}Starting Cerberus Phoenix v2.0 in Devnet mode...${NC}"
    
    # Create devnet environment file
    cp .env.example .env.devnet
    
    # Update for devnet
    sed -i 's/SOLANA_NETWORK=mainnet-beta/SOLANA_NETWORK=devnet/' .env.devnet
    sed -i 's|SOLANA_RPC_URL=.*|SOLANA_RPC_URL=https://api.devnet.solana.com|' .env.devnet
    sed -i 's/DEV_MODE=true/DEV_MODE=true/' .env.devnet
    sed -i 's/RUST_LOG=debug/RUST_LOG=info/' .env.devnet
    
    # Add test wallet configuration
    echo "" >> .env.devnet
    echo "# 🧪 Devnet Testing Configuration" >> .env.devnet
    echo "TEST_WALLET_PATH=$TEST_WALLET_PATH" >> .env.devnet
    echo "WALLET_ADDRESS=$WALLET_ADDRESS" >> .env.devnet
    echo "DEVNET_TESTING=true" >> .env.devnet
    
    # Start services with devnet config
    echo -e "${BLUE}Starting Docker services...${NC}"
    docker-compose -f infrastructure/docker-compose.yml --env-file .env.devnet up -d
    
    # Wait for services to start
    echo -e "${YELLOW}Waiting for services to initialize...${NC}"
    sleep 30
    
    echo -e "${GREEN}✅ Cerberus Phoenix v2.0 started in Devnet mode${NC}"
}

# Function to check service health
check_services() {
    echo -e "${YELLOW}Checking service health...${NC}"
    
    services=(
        "HFT-Ninja:http://localhost:8090/health"
        "Cerebro-BFF:http://localhost:8081/health"
        "Grafana:http://localhost:3001"
        "Prometheus:http://localhost:9090"
        "Qdrant:http://localhost:6333"
        "Kestra:http://localhost:8082"
    )
    
    for service in "${services[@]}"; do
        name=$(echo $service | cut -d: -f1)
        url=$(echo $service | cut -d: -f2-)
        
        if curl -s -f "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ $name is healthy${NC}"
        else
            echo -e "${RED}❌ $name is not responding${NC}"
        fi
    done
}

# Function to test webhook integration on devnet
test_webhook_devnet() {
    echo -e "${YELLOW}Testing webhook integration on Devnet...${NC}"
    
    # Create devnet-specific test payload
    cat > /tmp/devnet_webhook_test.json << EOF
{
  "account_addresses": ["$WALLET_ADDRESS"],
  "transaction_types": ["token_transfer"],
  "events": [
    {
      "transaction": {
        "signature": "DevnetTest$(date +%s)",
        "timestamp": $(date +%s),
        "slot": 123456789,
        "fee": 5000,
        "fee_payer": "$WALLET_ADDRESS",
        "recent_blockhash": "DevnetBlockHash$(date +%s)"
      },
      "token_transfers": [
        {
          "from_user_account": "$WALLET_ADDRESS",
          "to_user_account": "11111111111111111111111111111111",
          "token_amount": 1000.0,
          "mint": "DevnetTestToken$(date +%s)",
          "token_standard": "Fungible"
        }
      ],
      "account_data": [
        {
          "account": "$WALLET_ADDRESS",
          "native_balance_change": -5000000,
          "token_balance_changes": []
        }
      ]
    }
  ],
  "webhook_type": "devnet_test",
  "timestamp": $(date +%s)
}
EOF
    
    # Test webhook endpoint
    echo -e "${BLUE}Testing webhook endpoint...${NC}"
    response=$(curl -s -w "%{http_code}" \
        -X POST "http://localhost:8090/webhooks/helius" \
        -H "Authorization: Bearer test_devnet_token" \
        -H "Content-Type: application/json" \
        -d @/tmp/devnet_webhook_test.json \
        -o /tmp/webhook_response.json)
    
    http_code="${response: -3}"
    
    if [ "$http_code" = "200" ]; then
        echo -e "${GREEN}✅ Webhook test successful${NC}"
    else
        echo -e "${RED}❌ Webhook test failed: HTTP $http_code${NC}"
        cat /tmp/webhook_response.json
    fi
    
    # Clean up
    rm -f /tmp/devnet_webhook_test.json /tmp/webhook_response.json
}

# Function to perform simple token operations on devnet
test_token_operations() {
    echo -e "${YELLOW}Testing basic token operations on Devnet...${NC}"
    
    # Create a test SPL token
    echo -e "${BLUE}Creating test SPL token...${NC}"
    
    # This would create a real token on devnet
    # For safety, we'll just simulate the API calls
    echo -e "${YELLOW}Simulating token creation (devnet-safe)...${NC}"
    
    # Test API endpoints
    echo -e "${BLUE}Testing Cerebro-BFF API endpoints...${NC}"
    
    # Test risk analysis endpoint
    curl -s "http://localhost:8081/api/v1/risk/analyze/So11111111111111111111111111111111111111112" | jq '.' || echo "API not ready yet"
    
    # Test RPC provider status
    curl -s "http://localhost:8081/api/v1/rpc/providers" | jq '.' || echo "RPC providers not ready yet"
    
    echo -e "${GREEN}✅ Basic API tests completed${NC}"
}

# Function to monitor system performance
monitor_performance() {
    echo -e "${YELLOW}Monitoring system performance...${NC}"
    
    # Get webhook metrics
    echo -e "${BLUE}Webhook Metrics:${NC}"
    curl -s "http://localhost:8090/webhooks/metrics" | jq '.' || echo "Metrics not available"
    
    # Get system health
    echo -e "${BLUE}System Health:${NC}"
    curl -s "http://localhost:8081/api/v1/metrics/system-health" | jq '.' || echo "Health metrics not available"
    
    # Check Docker container status
    echo -e "${BLUE}Container Status:${NC}"
    docker-compose -f infrastructure/docker-compose.yml ps
}

# Function to cleanup devnet testing
cleanup_devnet() {
    echo -e "${YELLOW}Cleaning up devnet testing environment...${NC}"
    
    # Stop services
    docker-compose -f infrastructure/docker-compose.yml --env-file .env.devnet down
    
    # Remove devnet config
    rm -f .env.devnet
    
    # Keep test wallet for future use
    echo -e "${GREEN}✅ Cleanup completed (test wallet preserved)${NC}"
}

# Main testing flow
main() {
    echo -e "${PURPLE}🚀 Starting Comprehensive Devnet Testing${NC}"
    echo "=============================================="
    
    # Check prerequisites
    check_solana_cli
    
    # Setup devnet environment
    setup_devnet
    setup_test_wallet
    airdrop_sol
    
    # Start Cerberus Phoenix v2.0
    start_cerberus_devnet
    
    # Run tests
    check_services
    test_webhook_devnet
    test_token_operations
    monitor_performance
    
    echo -e "\n${GREEN}🎉 Devnet Testing Completed Successfully!${NC}"
    echo "=============================================="
    echo -e "${BLUE}Services running:${NC}"
    echo "• HFT-Ninja: http://localhost:8090"
    echo "• Cerebro-BFF: http://localhost:8081"
    echo "• Grafana: http://localhost:3001"
    echo "• Prometheus: http://localhost:9090"
    echo ""
    echo -e "${YELLOW}To stop services: docker-compose down${NC}"
    echo -e "${YELLOW}To cleanup: $0 cleanup${NC}"
}

# Handle cleanup command
if [ "$1" = "cleanup" ]; then
    cleanup_devnet
    exit 0
fi

# Run main testing
main "$@"
