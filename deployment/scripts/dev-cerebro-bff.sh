#!/bin/bash
# 🚀 Development script for Cerebro-BFF with hot reload and mock data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting Cerebro-BFF Development Environment${NC}"

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo -e "${YELLOW}📦 Installing cargo-watch...${NC}"
    cargo install cargo-watch
fi

# Set development environment variables
export ENVIRONMENT=development
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Mock API credentials for development
export HELIUS_API_KEY=mock_helius_key
export HELIUS_BASE_URL=https://api.helius.xyz
export QUICKNODE_RPC_URL=https://api.devnet.solana.com
export QUICKNODE_API_KEY=mock_quicknode_key

# Database configuration
export DATABASE_URL=postgresql://cerberus:phoenix@localhost:5432/cerberus_phoenix
export FEEDBACK_DATABASE_URL=postgresql://cerberus:phoenix@localhost:5432/cerberus_phoenix

# Qdrant configuration
export QDRANT_URL=http://localhost:6334
export QDRANT_API_KEY=""

# Server configuration
export SERVER_HOST=0.0.0.0
export SERVER_PORT=8080

echo -e "${GREEN}✅ Environment configured for development${NC}"
echo -e "${YELLOW}📊 Using mock data for external APIs${NC}"

# Check if required services are running
echo -e "${BLUE}🔍 Checking required services...${NC}"

# Check Postgres
if ! pg_isready -h localhost -p 5432 -U cerberus &> /dev/null; then
    echo -e "${RED}❌ Postgres not running on localhost:5432${NC}"
    echo -e "${YELLOW}💡 Start with: docker-compose -f infrastructure/docker-compose.yml up -d postgres${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Postgres is running${NC}"

# Check Qdrant
if ! curl -s http://localhost:6334/health &> /dev/null; then
    echo -e "${RED}❌ Qdrant not running on localhost:6334${NC}"
    echo -e "${YELLOW}💡 Start with: docker-compose -f infrastructure/docker-compose.yml up -d qdrant${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Qdrant is running${NC}"

# Change to cerebro-bff directory
cd "$(dirname "$0")/../services/cerebro-bff"

echo -e "${BLUE}🔥 Starting hot reload development server...${NC}"
echo -e "${YELLOW}📝 Logs will show debug information${NC}"
echo -e "${YELLOW}🎭 Using mock clients for external APIs${NC}"
echo -e "${YELLOW}🔄 Files will auto-reload on changes${NC}"
echo ""
echo -e "${GREEN}🌐 Server will be available at: http://localhost:8080${NC}"
echo -e "${GREEN}📊 Health check: http://localhost:8080/health${NC}"
echo -e "${GREEN}📈 Metrics: http://localhost:8080/metrics${NC}"
echo ""
echo -e "${BLUE}Press Ctrl+C to stop${NC}"
echo ""

# Start cargo watch with clear screen and detailed output
cargo watch \
    --clear \
    --watch src \
    --watch Cargo.toml \
    --exec "run" \
    --shell "echo '🔄 Recompiling...' && cargo run"
