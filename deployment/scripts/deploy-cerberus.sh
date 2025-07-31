#!/bin/bash

# 🚀 Cerberus Phoenix v2.0 - Master Deployment Script
# Complete deployment automation for HFT trading bot

set -e

# Make script executable
chmod +x "$0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$PROJECT_ROOT/.env"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.production.yml"

# Default values
ENVIRONMENT="development"
SKIP_BUILD=false
SKIP_TESTS=false
MONITORING_ONLY=false
DISCORD_BOT=false
TELEGRAM_BOT=false

# Functions
print_banner() {
    echo -e "${PURPLE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                 🚀 CERBERUS PHOENIX v2.0                    ║"
    echo "║              AI-Driven HFT Trading Bot                      ║"
    echo "║                  Deployment Manager                         ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_step() {
    echo -e "${CYAN}[$(date +'%H:%M:%S')] $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -e, --environment ENV    Deployment environment (development|staging|production)"
    echo "  -s, --skip-build        Skip Docker image building"
    echo "  -t, --skip-tests        Skip running tests"
    echo "  -m, --monitoring-only   Deploy only monitoring stack"
    echo "  -d, --discord           Enable Discord bot"
    echo "  -g, --telegram          Enable Telegram bot"
    echo "  -h, --help              Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Development deployment"
    echo "  $0 -e production -d -g               # Production with bots"
    echo "  $0 -m                                # Monitoring stack only"
    echo "  $0 -s -t                             # Skip build and tests"
}

check_prerequisites() {
    print_step "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_error "Docker Compose is not installed"
        exit 1
    fi
    
    # Check .env file
    if [ ! -f "$ENV_FILE" ]; then
        print_warning ".env file not found. Creating template..."
        create_env_template
    fi
    
    print_success "Prerequisites check passed"
}

create_env_template() {
    cat > "$ENV_FILE" << EOF
# 🚀 Cerberus Phoenix v2.0 - Environment Configuration

# 🗄️ Database
POSTGRES_PASSWORD=cerberus_secure_password_$(openssl rand -hex 8)

# 🔐 Vault
VAULT_ROOT_TOKEN=cerberus_vault_token_$(openssl rand -hex 16)

# 📊 Monitoring
GRAFANA_PASSWORD=admin_$(openssl rand -hex 8)

# 🔑 Infisical (Production Secrets Management)
INFISICAL_API_URL=https://app.infisical.com/api
INFISICAL_PROJECT_ID=your_project_id_here
INFISICAL_ENVIRONMENT=development
INFISICAL_CLIENT_ID=your_client_id_here
INFISICAL_CLIENT_SECRET=your_client_secret_here

# 🌐 Solana RPC (Get from providers)
HELIUS_API_KEY=your_helius_api_key_here
QUICKNODE_API_KEY=your_quicknode_api_key_here
JITO_API_KEY=your_jito_api_key_here

# 💰 Trading Limits
MAX_POSITION_SIZE_SOL=1.0
DAILY_LOSS_LIMIT_SOL=10.0
EMERGENCY_STOP_PERCENTAGE=25.0

# 🤖 Discord Bot (Optional)
DISCORD_TOKEN=your_discord_bot_token_here
DISCORD_CHANNEL_ID=your_discord_channel_id_here

# 📱 Telegram Bot (Optional)
TELEGRAM_TOKEN=your_telegram_bot_token_here
TELEGRAM_CHAT_ID=your_telegram_chat_id_here

# 🌍 Environment
ENVIRONMENT=$ENVIRONMENT
EOF
    
    print_warning "Created .env template. Please configure your API keys!"
    print_warning "Edit $ENV_FILE with your actual credentials"
}

run_tests() {
    if [ "$SKIP_TESTS" = true ]; then
        print_warning "Skipping tests"
        return
    fi
    
    print_step "Running tests..."
    
    # Test HFT-Ninja
    cd "$PROJECT_ROOT/services/hft-ninja"
    if cargo test --release; then
        print_success "HFT-Ninja tests passed"
    else
        print_error "HFT-Ninja tests failed"
        exit 1
    fi
    
    # Test Cerebro-BFF
    cd "$PROJECT_ROOT/services/cerebro-bff"
    if cargo test --release; then
        print_success "Cerebro-BFF tests passed"
    else
        print_error "Cerebro-BFF tests failed"
        exit 1
    fi
    
    cd "$PROJECT_ROOT"
}

build_images() {
    if [ "$SKIP_BUILD" = true ]; then
        print_warning "Skipping Docker image building"
        return
    fi
    
    print_step "Building Docker images..."
    
    # Build core services
    docker-compose -f "$COMPOSE_FILE" build cerebro-bff hft-ninja
    
    # Build bots if enabled
    if [ "$DISCORD_BOT" = true ]; then
        docker-compose -f "$COMPOSE_FILE" build discord-bot
    fi
    
    if [ "$TELEGRAM_BOT" = true ]; then
        docker-compose -f "$COMPOSE_FILE" build telegram-bot
    fi
    
    print_success "Docker images built successfully"
}

deploy_infrastructure() {
    print_step "Deploying infrastructure..."
    
    # Start core infrastructure
    docker-compose -f "$COMPOSE_FILE" up -d \
        postgres \
        qdrant \
        vault \
        prometheus \
        grafana \
        traefik \
        alertmanager \
        kestra
    
    print_step "Waiting for infrastructure to be ready..."
    sleep 30
    
    # Health checks
    check_service_health "postgres" "5432"
    check_service_health "qdrant" "6333"
    check_service_health "vault" "8200"
    check_service_health "prometheus" "9090"
    check_service_health "grafana" "3001"
    
    print_success "Infrastructure deployed successfully"
}

deploy_trading_services() {
    if [ "$MONITORING_ONLY" = true ]; then
        print_warning "Skipping trading services (monitoring-only mode)"
        return
    fi
    
    print_step "Deploying trading services..."
    
    # Start Cerebro-BFF first
    docker-compose -f "$COMPOSE_FILE" up -d cerebro-bff
    sleep 20
    check_service_health "cerebro-bff" "3000"
    
    # Start HFT-Ninja
    docker-compose -f "$COMPOSE_FILE" up -d hft-ninja
    sleep 15
    check_service_health "hft-ninja" "8090"
    
    print_success "Trading services deployed successfully"
}

deploy_bots() {
    if [ "$DISCORD_BOT" = true ]; then
        print_step "Deploying Discord bot..."
        docker-compose -f "$COMPOSE_FILE" up -d discord-bot
        print_success "Discord bot deployed"
    fi
    
    if [ "$TELEGRAM_BOT" = true ]; then
        print_step "Deploying Telegram bot..."
        docker-compose -f "$COMPOSE_FILE" up -d telegram-bot
        print_success "Telegram bot deployed"
    fi
}

check_service_health() {
    local service=$1
    local port=$2
    local max_attempts=30
    local attempt=1
    
    print_step "Checking $service health..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f -s "http://localhost:$port/health" > /dev/null 2>&1 || \
           nc -z localhost "$port" > /dev/null 2>&1; then
            print_success "$service is healthy"
            return 0
        fi
        
        echo -n "."
        sleep 2
        ((attempt++))
    done
    
    print_error "$service health check failed"
    return 1
}

show_status() {
    print_step "Deployment Status:"
    echo ""
    
    # Show running containers
    docker-compose -f "$COMPOSE_FILE" ps
    
    echo ""
    print_step "Service URLs:"
    echo "🧠 Cerebro-BFF:     http://localhost:3000"
    echo "⚡ HFT-Ninja:       http://localhost:8090"
    echo "📊 Grafana:         http://localhost:3001 (admin/password from .env)"
    echo "📈 Prometheus:      http://localhost:9090"
    echo "🔐 Vault:           http://localhost:8200"
    echo "🌐 Traefik:         http://localhost:8082"
    echo "🔔 AlertManager:    http://localhost:9093"
    echo "🔄 Kestra:          http://localhost:8080"
    
    if [ "$MONITORING_ONLY" != true ]; then
        echo ""
        print_step "Trading Status:"
        curl -s http://localhost:8090/api/dashboard/summary | jq '.' 2>/dev/null || echo "Trading services not ready"
    fi
}

cleanup() {
    print_step "Cleaning up..."
    docker-compose -f "$COMPOSE_FILE" down
    print_success "Cleanup completed"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -s|--skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -t|--skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        -m|--monitoring-only)
            MONITORING_ONLY=true
            shift
            ;;
        -d|--discord)
            DISCORD_BOT=true
            shift
            ;;
        -g|--telegram)
            TELEGRAM_BOT=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        --cleanup)
            cleanup
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main deployment flow
main() {
    print_banner
    
    print_step "Starting Cerberus Phoenix v2.0 deployment..."
    print_step "Environment: $ENVIRONMENT"
    
    check_prerequisites
    run_tests
    build_images
    deploy_infrastructure
    deploy_trading_services
    deploy_bots
    
    print_success "🎉 Cerberus Phoenix v2.0 deployed successfully!"
    show_status
    
    echo ""
    print_step "Next steps:"
    echo "1. Configure your API keys in .env file"
    echo "2. Set up Infisical for production secrets"
    echo "3. Configure Discord/Telegram bots if needed"
    echo "4. Monitor the system via Grafana dashboards"
    echo "5. Check logs: docker-compose -f $COMPOSE_FILE logs -f"
}

# Trap cleanup on exit
trap cleanup EXIT

# Run main function
main
