#!/bin/bash

# 🧪 Test Fee Optimizer API
# Prosty test endpointu optymalizacji opłat

echo "🧪 Testing HFT-Ninja Fee Optimizer API"
echo "======================================="

# Test 1: Health Check
echo "📋 Test 1: Health Check"
curl -s http://localhost:8090/health | jq '.' || echo "❌ Health check failed"
echo ""

# Test 2: Fee Optimization - Piranha Surf
echo "💰 Test 2: Fee Optimization - Piranha Surf"
curl -s -X POST http://localhost:8090/api/optimize-fee \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "PiranhaSurf",
    "amount_sol": 1.0,
    "urgency_level": 8
  }' | jq '.' || echo "❌ Fee optimization failed"
echo ""

# Test 3: Fee Optimization - Cross DEX Arbitrage
echo "🔄 Test 3: Fee Optimization - Cross DEX Arbitrage"
curl -s -X POST http://localhost:8090/api/optimize-fee \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "CrossDexArbitrage",
    "amount_sol": 5.0,
    "urgency_level": 5
  }' | jq '.' || echo "❌ Fee optimization failed"
echo ""

# Test 4: Metrics
echo "📊 Test 4: Metrics"
curl -s http://localhost:8090/api/metrics | jq '.' || echo "❌ Metrics failed"
echo ""

echo "✅ Tests completed!"
