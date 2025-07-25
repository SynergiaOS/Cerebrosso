#!/bin/bash

# 🧪 Simple Devnet Test - Cerberus Phoenix v2.0
# Basic testing without Docker dependencies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Simple Devnet Test - Cerberus Phoenix v2.0${NC}"
echo "=================================================="

# Test wallet setup
WALLET_ADDRESS="9cBo5UJhAGcE9YVLbxUPihSX24DVhPkmqaRGKCJvHM7s"
echo -e "${GREEN}✅ Test wallet: $WALLET_ADDRESS${NC}"

# Check Solana devnet connectivity
echo -e "${YELLOW}Testing Solana Devnet connectivity...${NC}"
if curl -s -X POST https://api.devnet.solana.com \
   -H "Content-Type: application/json" \
   -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' | grep -q "ok"; then
    echo -e "${GREEN}✅ Solana Devnet is accessible${NC}"
else
    echo -e "${RED}❌ Solana Devnet connection failed${NC}"
    exit 1
fi

# Check if HFT-Ninja is running locally
echo -e "${YELLOW}Checking for local HFT-Ninja...${NC}"
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ HFT-Ninja is running on port 8080${NC}"
    HFT_NINJA_URL="http://localhost:8080"
elif curl -s http://localhost:8090/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ HFT-Ninja is running on port 8090${NC}"
    HFT_NINJA_URL="http://localhost:8090"
else
    echo -e "${YELLOW}⚠️  HFT-Ninja not running, will test when available${NC}"
    HFT_NINJA_URL=""
fi

# Test webhook payload creation
echo -e "${YELLOW}Creating test webhook payloads...${NC}"

# Create devnet test payload
cat > /tmp/devnet_test_payload.json << EOF
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

echo -e "${GREEN}✅ Test payload created${NC}"

# Test webhook if HFT-Ninja is available
if [ ! -z "$HFT_NINJA_URL" ]; then
    echo -e "${YELLOW}Testing webhook endpoint...${NC}"
    
    response=$(curl -s -w "%{http_code}" \
        -X POST "$HFT_NINJA_URL/webhooks/helius" \
        -H "Authorization: Bearer devnet_test_token" \
        -H "Content-Type: application/json" \
        -d @/tmp/devnet_test_payload.json \
        -o /tmp/webhook_response.json)
    
    http_code="${response: -3}"
    
    if [ "$http_code" = "200" ]; then
        echo -e "${GREEN}✅ Webhook test successful: HTTP $http_code${NC}"
        echo -e "${BLUE}Response:${NC}"
        cat /tmp/webhook_response.json | jq '.' 2>/dev/null || cat /tmp/webhook_response.json
    else
        echo -e "${YELLOW}⚠️  Webhook test: HTTP $http_code${NC}"
        echo -e "${BLUE}Response:${NC}"
        cat /tmp/webhook_response.json
    fi
    
    # Test metrics endpoint
    echo -e "${YELLOW}Testing metrics endpoint...${NC}"
    if curl -s "$HFT_NINJA_URL/webhooks/metrics" | jq '.' > /tmp/metrics.json 2>/dev/null; then
        echo -e "${GREEN}✅ Metrics endpoint working${NC}"
        echo -e "${BLUE}Metrics:${NC}"
        cat /tmp/metrics.json
    else
        echo -e "${YELLOW}⚠️  Metrics endpoint not available${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping webhook tests - HFT-Ninja not running${NC}"
    echo -e "${BLUE}To test webhooks, start HFT-Ninja with:${NC}"
    echo "cd services/hft-ninja && RUST_LOG=debug SOLANA_NETWORK=devnet cargo run"
fi

# Test Solana devnet operations
echo -e "${YELLOW}Testing Solana devnet operations...${NC}"

# Get account info
echo -e "${BLUE}Getting account info for test wallet...${NC}"
account_info=$(curl -s -X POST https://api.devnet.solana.com \
   -H "Content-Type: application/json" \
   -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"$WALLET_ADDRESS\"]}")

if echo "$account_info" | jq -e '.result' > /dev/null 2>&1; then
    balance=$(echo "$account_info" | jq -r '.result.value.lamports // 0')
    sol_balance=$(echo "scale=9; $balance / 1000000000" | bc -l 2>/dev/null || echo "0")
    echo -e "${GREEN}✅ Account balance: $sol_balance SOL${NC}"
else
    echo -e "${YELLOW}⚠️  Account not found or empty${NC}"
fi

# Get recent blockhash
echo -e "${BLUE}Getting recent blockhash...${NC}"
blockhash_info=$(curl -s -X POST https://api.devnet.solana.com \
   -H "Content-Type: application/json" \
   -d '{"jsonrpc":"2.0","id":1,"method":"getLatestBlockhash"}')

if echo "$blockhash_info" | jq -e '.result.value.blockhash' > /dev/null 2>&1; then
    blockhash=$(echo "$blockhash_info" | jq -r '.result.value.blockhash')
    echo -e "${GREEN}✅ Recent blockhash: ${blockhash:0:20}...${NC}"
else
    echo -e "${RED}❌ Failed to get recent blockhash${NC}"
fi

# Test pump.fun program detection
echo -e "${YELLOW}Testing pump.fun program detection...${NC}"
pump_fun_program="6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"

program_info=$(curl -s -X POST https://api.devnet.solana.com \
   -H "Content-Type: application/json" \
   -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"$pump_fun_program\"]}")

if echo "$program_info" | jq -e '.result.value' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Pump.fun program detected on devnet${NC}"
else
    echo -e "${YELLOW}⚠️  Pump.fun program not found on devnet (expected)${NC}"
fi

# Performance test
echo -e "${YELLOW}Running performance test...${NC}"
start_time=$(date +%s%N)

for i in {1..10}; do
    curl -s -X POST https://api.devnet.solana.com \
       -H "Content-Type: application/json" \
       -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' > /dev/null
done

end_time=$(date +%s%N)
duration_ms=$(( (end_time - start_time) / 1000000 / 10 ))

echo -e "${GREEN}✅ Average RPC latency: ${duration_ms}ms${NC}"

if [ $duration_ms -lt 100 ]; then
    echo -e "${GREEN}✅ Excellent latency (<100ms)${NC}"
elif [ $duration_ms -lt 500 ]; then
    echo -e "${YELLOW}⚠️  Acceptable latency (<500ms)${NC}"
else
    echo -e "${RED}❌ High latency (>500ms)${NC}"
fi

# Cleanup
rm -f /tmp/devnet_test_payload.json /tmp/webhook_response.json /tmp/metrics.json

echo -e "\n${GREEN}🎉 Simple Devnet Test Completed!${NC}"
echo "=============================================="
echo -e "${BLUE}Summary:${NC}"
echo "• Solana Devnet: Accessible"
echo "• Test Wallet: $WALLET_ADDRESS"
echo "• RPC Latency: ${duration_ms}ms"
if [ ! -z "$HFT_NINJA_URL" ]; then
    echo "• HFT-Ninja: Running on $HFT_NINJA_URL"
else
    echo "• HFT-Ninja: Not running (start manually for webhook tests)"
fi

echo -e "\n${YELLOW}Next steps:${NC}"
echo "1. Start HFT-Ninja: cd services/hft-ninja && RUST_LOG=debug SOLANA_NETWORK=devnet cargo run"
echo "2. Test webhooks: ./scripts/test-helius-webhook.sh"
echo "3. Monitor with: curl http://localhost:8080/webhooks/metrics"

echo -e "\n${BLUE}🧪 Devnet testing environment ready!${NC}"
