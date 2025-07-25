#!/bin/bash

# 🎣 Helius Webhook Integration Test Script
# Tests the complete webhook flow from HFT-Ninja to Cerebro-BFF

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
HFT_NINJA_URL="http://localhost:8090"
CEREBRO_BFF_URL="http://localhost:8081"
TEST_AUTH_TOKEN="test_webhook_token_12345"

echo -e "${BLUE}🎣 Testing Helius Webhook Integration${NC}"
echo "=================================================="

# Function to check service health
check_service() {
    local service_name=$1
    local url=$2
    
    echo -e "${YELLOW}Checking $service_name health...${NC}"
    
    if curl -s -f "$url/health" > /dev/null; then
        echo -e "${GREEN}✅ $service_name is healthy${NC}"
        return 0
    else
        echo -e "${RED}❌ $service_name is not responding${NC}"
        return 1
    fi
}

# Function to test webhook endpoint
test_webhook() {
    local test_name=$1
    local payload_file=$2
    
    echo -e "${YELLOW}Testing: $test_name${NC}"
    
    response=$(curl -s -w "%{http_code}" \
        -X POST "$HFT_NINJA_URL/webhooks/helius" \
        -H "Authorization: Bearer $TEST_AUTH_TOKEN" \
        -H "Content-Type: application/json" \
        -d @"$payload_file" \
        -o /tmp/webhook_response.json)
    
    http_code="${response: -3}"
    
    if [ "$http_code" = "200" ]; then
        echo -e "${GREEN}✅ $test_name: HTTP $http_code${NC}"
        return 0
    else
        echo -e "${RED}❌ $test_name: HTTP $http_code${NC}"
        cat /tmp/webhook_response.json
        return 1
    fi
}

# Function to get webhook metrics
get_webhook_metrics() {
    echo -e "${YELLOW}Fetching webhook metrics...${NC}"
    
    curl -s "$HFT_NINJA_URL/webhooks/metrics" | jq '.' || echo "Failed to get metrics"
}

# Create test payloads
create_test_payloads() {
    echo -e "${YELLOW}Creating test payloads...${NC}"
    
    # Large volume transfer test
    cat > /tmp/large_volume_test.json << 'EOF'
{
  "account_addresses": ["TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj"],
  "transaction_types": ["token_transfer"],
  "events": [
    {
      "transaction": {
        "signature": "5LargeVolumeTransfer123456789abcdef",
        "timestamp": 1678886400,
        "slot": 123456789,
        "fee": 5000,
        "fee_payer": "11111111111111111111111111111111",
        "recent_blockhash": "BlockHash123456789"
      },
      "token_transfers": [
        {
          "from_user_account": "FromAccount111111111111111111111111",
          "to_user_account": "ToAccount222222222222222222222222",
          "token_amount": 50000.0,
          "mint": "LargeVolumeToken111111111111111111111",
          "token_standard": "Fungible"
        }
      ],
      "account_data": [
        {
          "account": "FromAccount111111111111111111111111",
          "native_balance_change": -5000000000,
          "token_balance_changes": [
            {
              "mint": "LargeVolumeToken111111111111111111111",
              "raw_token_amount": {
                "token_amount": "50000000000000",
                "decimals": 9
              },
              "token_account": "TokenAccount111111111111111111111",
              "user_account": "FromAccount111111111111111111111111"
            }
          ]
        }
      ]
    }
  ],
  "webhook_type": "enhanced",
  "timestamp": 1678886400
}
EOF

    # Pump.fun new token test
    cat > /tmp/pump_fun_test.json << 'EOF'
{
  "account_addresses": ["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"],
  "transaction_types": ["token_mint"],
  "events": [
    {
      "transaction": {
        "signature": "5PumpFunNewToken123456789abcdef",
        "timestamp": 1678886500,
        "slot": 123456790,
        "fee": 10000,
        "fee_payer": "PumpFunCreator111111111111111111111",
        "recent_blockhash": "BlockHash123456790"
      },
      "token_transfers": [
        {
          "from_user_account": "11111111111111111111111111111111",
          "to_user_account": "PumpFunCreator111111111111111111111",
          "token_amount": 1000000000.0,
          "mint": "NewPumpFunToken111111111111111111111",
          "token_standard": "Fungible"
        }
      ],
      "instructions": [
        {
          "accounts": ["PumpFunCreator111111111111111111111", "NewPumpFunToken111111111111111111111"],
          "data": "CreateTokenInstruction",
          "program_id": "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"
        }
      ],
      "account_data": [
        {
          "account": "PumpFunCreator111111111111111111111",
          "native_balance_change": -10000000,
          "token_balance_changes": [
            {
              "mint": "NewPumpFunToken111111111111111111111",
              "raw_token_amount": {
                "token_amount": "1000000000000000000",
                "decimals": 9
              },
              "token_account": "CreatorTokenAccount111111111111111",
              "user_account": "PumpFunCreator111111111111111111111"
            }
          ]
        }
      ]
    }
  ],
  "webhook_type": "enhanced",
  "timestamp": 1678886500
}
EOF

    # Suspicious wash trading test
    cat > /tmp/wash_trading_test.json << 'EOF'
{
  "account_addresses": ["SuspiciousAccount111111111111111111"],
  "transaction_types": ["token_transfer"],
  "events": [
    {
      "transaction": {
        "signature": "5WashTradingPattern123456789abcdef",
        "timestamp": 1678886600,
        "slot": 123456791,
        "fee": 15000,
        "fee_payer": "SuspiciousAccount111111111111111111",
        "recent_blockhash": "BlockHash123456791"
      },
      "token_transfers": [
        {
          "from_user_account": "SuspiciousAccount111111111111111111",
          "to_user_account": "SuspiciousAccount111111111111111111",
          "token_amount": 10000.0,
          "mint": "SuspiciousToken111111111111111111111",
          "token_standard": "Fungible"
        }
      ],
      "account_data": [
        {
          "account": "SuspiciousAccount111111111111111111",
          "native_balance_change": -15000000,
          "token_balance_changes": [
            {
              "mint": "SuspiciousToken111111111111111111111",
              "raw_token_amount": {
                "token_amount": "10000000000000",
                "decimals": 9
              },
              "token_account": "SuspiciousTokenAccount111111111111",
              "user_account": "SuspiciousAccount111111111111111111"
            }
          ]
        }
      ]
    }
  ],
  "webhook_type": "enhanced",
  "timestamp": 1678886600
}
EOF

    echo -e "${GREEN}✅ Test payloads created${NC}"
}

# Main test execution
main() {
    echo -e "${BLUE}Starting Helius Webhook Integration Tests${NC}"
    echo "=============================================="
    
    # Check if jq is installed
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}❌ jq is required but not installed${NC}"
        exit 1
    fi
    
    # Check service health
    echo -e "\n${BLUE}1. Health Checks${NC}"
    check_service "HFT-Ninja" "$HFT_NINJA_URL" || exit 1
    check_service "Cerebro-BFF" "$CEREBRO_BFF_URL" || exit 1
    
    # Create test payloads
    echo -e "\n${BLUE}2. Creating Test Payloads${NC}"
    create_test_payloads
    
    # Test webhook endpoints
    echo -e "\n${BLUE}3. Testing Webhook Endpoints${NC}"
    
    test_webhook "Large Volume Transfer" "/tmp/large_volume_test.json"
    sleep 1
    
    test_webhook "Pump.fun New Token" "/tmp/pump_fun_test.json"
    sleep 1
    
    test_webhook "Wash Trading Detection" "/tmp/wash_trading_test.json"
    sleep 1
    
    # Get metrics
    echo -e "\n${BLUE}4. Webhook Metrics${NC}"
    get_webhook_metrics
    
    # Test authentication failure
    echo -e "\n${BLUE}5. Testing Authentication${NC}"
    echo -e "${YELLOW}Testing invalid auth token...${NC}"
    
    response=$(curl -s -w "%{http_code}" \
        -X POST "$HFT_NINJA_URL/webhooks/helius" \
        -H "Authorization: Bearer invalid_token" \
        -H "Content-Type: application/json" \
        -d '{"test": "data"}' \
        -o /dev/null)
    
    http_code="${response: -3}"
    
    if [ "$http_code" = "401" ]; then
        echo -e "${GREEN}✅ Authentication properly rejected invalid token${NC}"
    else
        echo -e "${RED}❌ Authentication test failed: HTTP $http_code${NC}"
    fi
    
    # Test rate limiting
    echo -e "\n${BLUE}6. Testing Rate Limiting${NC}"
    echo -e "${YELLOW}Sending rapid requests to test rate limiting...${NC}"
    
    rate_limit_failures=0
    for i in {1..5}; do
        response=$(curl -s -w "%{http_code}" \
            -X POST "$HFT_NINJA_URL/webhooks/helius" \
            -H "Authorization: Bearer $TEST_AUTH_TOKEN" \
            -H "Content-Type: application/json" \
            -d '{"test": "rate_limit"}' \
            -o /dev/null)
        
        http_code="${response: -3}"
        if [ "$http_code" = "429" ]; then
            rate_limit_failures=$((rate_limit_failures + 1))
        fi
    done
    
    if [ $rate_limit_failures -gt 0 ]; then
        echo -e "${GREEN}✅ Rate limiting is working ($rate_limit_failures/5 requests limited)${NC}"
    else
        echo -e "${YELLOW}⚠️  Rate limiting not triggered (may need higher load)${NC}"
    fi
    
    # Cleanup
    echo -e "\n${BLUE}7. Cleanup${NC}"
    rm -f /tmp/large_volume_test.json /tmp/pump_fun_test.json /tmp/wash_trading_test.json /tmp/webhook_response.json
    echo -e "${GREEN}✅ Test files cleaned up${NC}"
    
    echo -e "\n${GREEN}🎉 Helius Webhook Integration Tests Completed!${NC}"
    echo "=============================================="
    
    # Final metrics
    echo -e "\n${BLUE}Final Webhook Metrics:${NC}"
    get_webhook_metrics
}

# Run tests
main "$@"
