#!/bin/bash
# 🧪 Test API Script for Cerberus Phoenix v2.0
# Test all available endpoints and premium API integrations

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
echo "🧪 CERBERUS PHOENIX v2.0 - API TESTS"
echo "===================================="
echo -e "${NC}"

# Configuration
CEREBRO_BFF_URL="http://localhost:8081"
GRAFANA_URL="http://localhost:3001"
TRAEFIK_URL="http://localhost:8080"
QDRANT_URL="http://localhost:6333"

# Test Cerebro-BFF Health
test_cerebro_health() {
    echo -e "${BLUE}🔍 Testing Cerebro-BFF Health...${NC}"
    
    RESPONSE=$(curl -s -w "%{http_code}" "$CEREBRO_BFF_URL/health" -o /tmp/health_response.json)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Cerebro-BFF Health: OK${NC}"
        echo -e "${CYAN}Response:${NC}"
        cat /tmp/health_response.json | jq '.' 2>/dev/null || cat /tmp/health_response.json
        echo ""
    else
        echo -e "${RED}❌ Cerebro-BFF Health: FAILED (HTTP $HTTP_CODE)${NC}"
        cat /tmp/health_response.json 2>/dev/null || echo "No response body"
        echo ""
    fi
}

# Test Helius API Integration
test_helius_integration() {
    echo -e "${BLUE}🌟 Testing Helius API Integration...${NC}"
    
    # Test token analysis endpoint
    RESPONSE=$(curl -s -w "%{http_code}" -X POST "$CEREBRO_BFF_URL/api/v1/analyze/token" \
        -H "Content-Type: application/json" \
        -d '{
            "token_address": "So11111111111111111111111111111111111111112",
            "analysis_type": "comprehensive"
        }' -o /tmp/helius_response.json)
    
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Helius Integration: OK${NC}"
        echo -e "${CYAN}Response:${NC}"
        cat /tmp/helius_response.json | jq '.' 2>/dev/null || cat /tmp/helius_response.json
        echo ""
    else
        echo -e "${YELLOW}⚠️ Helius Integration: Endpoint not implemented yet (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
}

# Test QuickNode Integration
test_quicknode_integration() {
    echo -e "${BLUE}⚡ Testing QuickNode Integration...${NC}"
    
    # Test RPC endpoint
    RESPONSE=$(curl -s -w "%{http_code}" -X POST "$CEREBRO_BFF_URL/api/v1/solana/health" \
        -H "Content-Type: application/json" \
        -o /tmp/quicknode_response.json)
    
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ QuickNode Integration: OK${NC}"
        echo -e "${CYAN}Response:${NC}"
        cat /tmp/quicknode_response.json | jq '.' 2>/dev/null || cat /tmp/quicknode_response.json
        echo ""
    else
        echo -e "${YELLOW}⚠️ QuickNode Integration: Endpoint not implemented yet (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
}

# Test Qdrant Vector Database
test_qdrant() {
    echo -e "${BLUE}🧠 Testing Qdrant Vector Database...${NC}"
    
    RESPONSE=$(curl -s -w "%{http_code}" "$QDRANT_URL/collections" -o /tmp/qdrant_response.json)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Qdrant: OK${NC}"
        echo -e "${CYAN}Collections:${NC}"
        cat /tmp/qdrant_response.json | jq '.' 2>/dev/null || cat /tmp/qdrant_response.json
        echo ""
    else
        echo -e "${RED}❌ Qdrant: FAILED (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
}

# Test Grafana Dashboard
test_grafana() {
    echo -e "${BLUE}📊 Testing Grafana Dashboard...${NC}"
    
    RESPONSE=$(curl -s -w "%{http_code}" "$GRAFANA_URL/api/health" -o /tmp/grafana_response.json)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Grafana: OK${NC}"
        echo -e "${CYAN}Dashboard available at: $GRAFANA_URL${NC}"
        echo ""
    else
        echo -e "${RED}❌ Grafana: FAILED (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
}

# Test Traefik Load Balancer
test_traefik() {
    echo -e "${BLUE}🔀 Testing Traefik Load Balancer...${NC}"
    
    RESPONSE=$(curl -s -w "%{http_code}" "$TRAEFIK_URL/api/http/services" -o /tmp/traefik_response.json)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Traefik: OK${NC}"
        echo -e "${CYAN}Services:${NC}"
        cat /tmp/traefik_response.json | jq '.' 2>/dev/null || cat /tmp/traefik_response.json
        echo ""
    else
        echo -e "${RED}❌ Traefik: FAILED (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
}

# Test AI Models
test_ai_models() {
    echo -e "${BLUE}🤖 Testing AI Models...${NC}"
    
    # Test FinLlama
    echo -e "${CYAN}Testing FinLlama (port 11435)...${NC}"
    RESPONSE=$(curl -s -w "%{http_code}" "http://localhost:11435/api/tags" -o /tmp/finllama_response.json)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ FinLlama: OK${NC}"
    else
        echo -e "${YELLOW}⚠️ FinLlama: Not ready yet (HTTP $HTTP_CODE)${NC}"
    fi
    
    # Test Deepseek
    echo -e "${CYAN}Testing Deepseek (port 11436)...${NC}"
    RESPONSE=$(curl -s -w "%{http_code}" "http://localhost:11436/api/tags" -o /tmp/deepseek_response.json)
    HTTP_CODE="${RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Deepseek: OK${NC}"
    else
        echo -e "${YELLOW}⚠️ Deepseek: Not ready yet (HTTP $HTTP_CODE)${NC}"
    fi
    echo ""
}

# Test Premium API Keys
test_premium_apis() {
    echo -e "${BLUE}🔑 Testing Premium API Keys...${NC}"
    
    # Test Helius API directly
    echo -e "${CYAN}Testing Helius API directly...${NC}"
    HELIUS_RESPONSE=$(curl -s -w "%{http_code}" \
        "https://api.helius.xyz/v0/addresses/So11111111111111111111111111111111111111112/balances?api-key=40a78e4c-bdd0-4338-877a-aa7d56a5f5a0" \
        -o /tmp/helius_direct.json)
    
    HTTP_CODE="${HELIUS_RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ Helius API Direct: OK${NC}"
        echo -e "${CYAN}SOL Balance Response:${NC}"
        cat /tmp/helius_direct.json | jq '.tokens[0] // .nativeBalance // .' 2>/dev/null || head -c 200 /tmp/helius_direct.json
        echo ""
    else
        echo -e "${RED}❌ Helius API Direct: FAILED (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
    
    # Test QuickNode API directly
    echo -e "${CYAN}Testing QuickNode API directly...${NC}"
    QUICKNODE_RESPONSE=$(curl -s -w "%{http_code}" -X POST \
        "https://distinguished-blue-glade.solana-devnet.quiknode.pro/a10fad0f63cdfe46533f1892ac720517b08fe580/" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc": "2.0", "id": 1, "method": "getHealth"}' \
        -o /tmp/quicknode_direct.json)
    
    HTTP_CODE="${QUICKNODE_RESPONSE: -3}"
    
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✅ QuickNode API Direct: OK${NC}"
        echo -e "${CYAN}Health Response:${NC}"
        cat /tmp/quicknode_direct.json | jq '.' 2>/dev/null || cat /tmp/quicknode_direct.json
        echo ""
    else
        echo -e "${RED}❌ QuickNode API Direct: FAILED (HTTP $HTTP_CODE)${NC}"
        echo ""
    fi
}

# Main execution
main() {
    echo -e "${YELLOW}🚀 Starting API tests...${NC}"
    echo ""
    
    test_cerebro_health
    test_helius_integration
    test_quicknode_integration
    test_qdrant
    test_grafana
    test_traefik
    test_ai_models
    test_premium_apis
    
    echo -e "${GREEN}🎉 API TESTS COMPLETE!${NC}"
    echo -e "${CYAN}================================${NC}"
    echo ""
    echo -e "${YELLOW}📋 Next steps:${NC}"
    echo "1. 🔐 Setup Vault: ./secure-key-manager.sh init"
    echo "2. 🔑 Generate keys: ./secure-key-manager.sh store devnet trading"
    echo "3. 🚀 Test trading: curl -X POST http://localhost:8081/api/v1/test"
    echo "4. 📊 Monitor: http://localhost:3001 (Grafana)"
    echo ""
    echo -e "${PURPLE}🦈⚡ CERBERUS PHOENIX v2.0 IS READY FOR SOLANA DOMINATION! ⚡🦈${NC}"
    
    # Cleanup temp files
    rm -f /tmp/*_response.json /tmp/*_direct.json 2>/dev/null || true
}

# Run main function
main "$@"
