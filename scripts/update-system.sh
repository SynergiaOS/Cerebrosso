#!/bin/bash
# System update script

echo "🔄 Updating Cerberus Phoenix v2.0..."

# Pull latest changes
git pull origin main

# Update dependencies
cd services/cerebro-bff && cargo update
cd ../hft-ninja && cargo update
cd ../..

# Rebuild and restart
./scripts/deploy-production.sh restart

echo "✅ System updated successfully!"
