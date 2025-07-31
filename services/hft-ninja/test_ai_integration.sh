#!/bin/bash

# 🧠 Test AI Integration with Cerebro-BFF
# Test nowego endpointu analizy tokenów z AI

echo "🧠 Testing HFT-Ninja AI Integration"
echo "===================================="

# Test 1: Health Check
echo "📋 Test 1: Health Check"
curl -s http://localhost:8090/health | jq '.' || echo "❌ Health check failed"
echo ""

# Test 2: AI Token Analysis - Mock Token
echo "🧠 Test 2: AI Token Analysis - Mock Token"
curl -s -X POST http://localhost:8090/api/analyze-token \
  -H "Content-Type: application/json" \
  -d '{
    "mint_address": "So11111111111111111111111111111111111111112",
    "symbol": "SOL",
    "name": "Solana",
    "market_cap": 50000000000,
    "liquidity_usd": 10000000,
    "volume_24h": 500000000,
    "price_change_24h": 5.2,
    "holder_count": 1000000,
    "dev_allocation_percentage": 0.0,
    "freeze_authority": false,
    "mint_authority": false,
    "team_doxxed": true,
    "contract_verified": true,
    "risk_signals": [],
    "opportunity_signals": ["high_volume", "positive_momentum"],
    "timestamp": "2025-07-30T14:00:00Z"
  }' | jq '.' || echo "❌ AI analysis failed"
echo ""

# Test 3: AI Token Analysis - High Risk Token
echo "⚠️ Test 3: AI Token Analysis - High Risk Token"
curl -s -X POST http://localhost:8090/api/analyze-token \
  -H "Content-Type: application/json" \
  -d '{
    "mint_address": "RiskToken123456789012345678901234567890",
    "symbol": "RISK",
    "name": "Risky Memecoin",
    "market_cap": 100000,
    "liquidity_usd": 5000,
    "volume_24h": 50000,
    "price_change_24h": -80.5,
    "holder_count": 50,
    "dev_allocation_percentage": 95.0,
    "freeze_authority": true,
    "mint_authority": true,
    "team_doxxed": false,
    "contract_verified": false,
    "risk_signals": ["rug_pull_potential", "dev_allocation_high", "freeze_authority", "low_liquidity"],
    "opportunity_signals": [],
    "timestamp": "2025-07-30T14:00:00Z"
  }' | jq '.' || echo "❌ AI analysis failed"
echo ""

# Test 4: Fee Optimization (existing endpoint)
echo "💰 Test 4: Fee Optimization"
curl -s -X POST http://localhost:8090/api/optimize-fee \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "PiranhaSurf",
    "amount_sol": 2.0,
    "urgency_level": 9
  }' | jq '.' || echo "❌ Fee optimization failed"
echo ""

echo "✅ AI Integration tests completed!"
