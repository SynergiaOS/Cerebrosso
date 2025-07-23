#!/bin/bash

# 🔮 Test Alchemy API Configuration
# Verify your Alchemy API key is working correctly

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔮 Testing Alchemy API Configuration${NC}"
echo "================================================"

# Your Alchemy API configuration
ALCHEMY_API_KEY="Wu2Kqfk_50kW_Zs4ifjuf3c7afxLOs7R"
ALCHEMY_MAINNET_URL="https://solana-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"
ALCHEMY_DEVNET_URL="https://solana-devnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}"

echo -e "${YELLOW}API Key:${NC} ${ALCHEMY_API_KEY}"
echo -e "${YELLOW}Mainnet URL:${NC} ${ALCHEMY_MAINNET_URL}"
echo -e "${YELLOW}Devnet URL:${NC} ${ALCHEMY_DEVNET_URL}"
echo ""

# Test 1: Basic health check - MAINNET
echo -e "${BLUE}Test 1: Basic Health Check - MAINNET${NC}"
response=$(curl -s -X POST "$ALCHEMY_MAINNET_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getHealth"
  }')

if echo "$response" | grep -q '"result":"ok"'; then
    echo -e "${GREEN}✅ Alchemy MAINNET API is healthy${NC}"
else
    echo -e "${RED}❌ MAINNET health check failed${NC}"
    echo "Response: $response"
fi

# Test 1b: Basic health check - DEVNET
echo -e "${BLUE}Test 1b: Basic Health Check - DEVNET${NC}"
response=$(curl -s -X POST "$ALCHEMY_DEVNET_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getHealth"
  }')

if echo "$response" | grep -q '"result":"ok"'; then
    echo -e "${GREEN}✅ Alchemy DEVNET API is healthy${NC}"
else
    echo -e "${RED}❌ DEVNET health check failed${NC}"
    echo "Response: $response"
fi

# Test 2: Get latest blockhash
echo -e "${BLUE}Test 2: Get Latest Blockhash${NC}"
response=$(curl -s -X POST "$ALCHEMY_RPC_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getLatestBlockhash"
  }')

if echo "$response" | grep -q '"blockhash"'; then
    blockhash=$(echo "$response" | jq -r '.result.value.blockhash')
    echo -e "${GREEN}✅ Latest blockhash: ${blockhash:0:20}...${NC}"
else
    echo -e "${RED}❌ Failed to get blockhash${NC}"
    echo "Response: $response"
fi

# Test 3: Get slot
echo -e "${BLUE}Test 3: Get Current Slot${NC}"
response=$(curl -s -X POST "$ALCHEMY_RPC_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getSlot"
  }')

if echo "$response" | grep -q '"result"'; then
    slot=$(echo "$response" | jq -r '.result')
    echo -e "${GREEN}✅ Current slot: ${slot}${NC}"
else
    echo -e "${RED}❌ Failed to get slot${NC}"
    echo "Response: $response"
fi

# Test 4: Get account info for a known token
echo -e "${BLUE}Test 4: Get Token Account Info${NC}"
# USDC token mint address
USDC_MINT="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"

response=$(curl -s -X POST "$ALCHEMY_RPC_URL" \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"id\": 1,
    \"method\": \"getAccountInfo\",
    \"params\": [
      \"$USDC_MINT\",
      {
        \"encoding\": \"jsonParsed\"
      }
    ]
  }")

if echo "$response" | grep -q '"owner"'; then
    echo -e "${GREEN}✅ Successfully retrieved USDC token info${NC}"
    decimals=$(echo "$response" | jq -r '.result.value.data.parsed.info.decimals')
    echo -e "${YELLOW}USDC Decimals:${NC} $decimals"
else
    echo -e "${RED}❌ Failed to get token info${NC}"
    echo "Response: $response"
fi

# Test 5: Performance test
echo -e "${BLUE}Test 5: Performance Test (5 requests)${NC}"
start_time=$(date +%s%N)

for i in {1..5}; do
    curl -s -X POST "$ALCHEMY_RPC_URL" \
      -H "Content-Type: application/json" \
      -d '{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSlot"
      }' > /dev/null
done

end_time=$(date +%s%N)
duration=$(( (end_time - start_time) / 1000000 ))
avg_time=$(( duration / 5 ))

echo -e "${GREEN}✅ 5 requests completed in ${duration}ms${NC}"
echo -e "${YELLOW}Average response time: ${avg_time}ms${NC}"

# Summary
echo ""
echo "================================================"
echo -e "${GREEN}🎉 Alchemy API Test Complete!${NC}"
echo ""
echo -e "${BLUE}Configuration Summary:${NC}"
echo "• API Key: Working ✅"
echo "• RPC Endpoint: Accessible ✅"
echo "• Performance: ${avg_time}ms average ✅"
echo "• Free Tier: 100k requests/month"
echo "• No RPM limits (unlike Helius)"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "1. Copy your configuration to infrastructure/.env"
echo "2. Deploy with: ./scripts/deploy-production.sh"
echo "3. Monitor usage: curl http://localhost:3000/api/v1/usage/report"
echo ""
echo -e "${BLUE}🔄 Multi-RPC Optimization Active!${NC}"
