#!/bin/bash
echo "🚀 Deploying Complete Devnet Infrastructure..."

# Start full stack
docker-compose -f infrastructure/docker-compose.devnet.yml up -d

# Wait for services
sleep 30

# Test integration
curl http://localhost:8081/health  # Cerebro-BFF
curl http://localhost:8082/health  # Kestra
curl http://localhost:3001         # Grafana

echo "✅ Full infrastructure deployed!"