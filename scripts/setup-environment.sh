#!/bin/bash

# 🔧 Cerberus Phoenix v2.0 - Environment Setup Script
# Automated setup for development and production environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${PURPLE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              🔧 CERBERUS ENVIRONMENT SETUP                  ║"
    echo "║                  Phoenix v2.0 Edition                       ║"
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

# Check if running on supported OS
check_os() {
    print_step "Checking operating system..."
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        print_success "Linux detected"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        print_success "macOS detected"
    else
        print_error "Unsupported operating system: $OSTYPE"
        exit 1
    fi
}

# Install Docker if not present
install_docker() {
    if command -v docker &> /dev/null; then
        print_success "Docker already installed: $(docker --version)"
        return
    fi
    
    print_step "Installing Docker..."
    
    if [[ "$OS" == "linux" ]]; then
        # Install Docker on Linux
        curl -fsSL https://get.docker.com -o get-docker.sh
        sudo sh get-docker.sh
        sudo usermod -aG docker $USER
        rm get-docker.sh
        print_warning "Please log out and back in for Docker group changes to take effect"
    elif [[ "$OS" == "macos" ]]; then
        # Install Docker on macOS
        if command -v brew &> /dev/null; then
            brew install --cask docker
        else
            print_error "Please install Docker Desktop manually from https://docker.com/products/docker-desktop"
            exit 1
        fi
    fi
    
    print_success "Docker installed successfully"
}

# Install Docker Compose if not present
install_docker_compose() {
    if docker compose version &> /dev/null || command -v docker-compose &> /dev/null; then
        print_success "Docker Compose already available"
        return
    fi
    
    print_step "Installing Docker Compose..."
    
    if [[ "$OS" == "linux" ]]; then
        sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        sudo chmod +x /usr/local/bin/docker-compose
    fi
    
    print_success "Docker Compose installed successfully"
}

# Install Rust if not present
install_rust() {
    if command -v cargo &> /dev/null; then
        print_success "Rust already installed: $(rustc --version)"
        return
    fi
    
    print_step "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed successfully"
}

# Install additional tools
install_tools() {
    print_step "Installing additional tools..."
    
    # Install jq for JSON processing
    if ! command -v jq &> /dev/null; then
        if [[ "$OS" == "linux" ]]; then
            sudo apt-get update && sudo apt-get install -y jq curl netcat-openbsd
        elif [[ "$OS" == "macos" ]]; then
            if command -v brew &> /dev/null; then
                brew install jq curl netcat
            fi
        fi
        print_success "Additional tools installed"
    else
        print_success "Additional tools already available"
    fi
}

# Setup project directories
setup_directories() {
    print_step "Setting up project directories..."
    
    # Create necessary directories
    mkdir -p logs
    mkdir -p data/postgres
    mkdir -p data/qdrant
    mkdir -p data/grafana
    mkdir -p data/prometheus
    mkdir -p data/vault
    mkdir -p config/secrets
    
    # Set proper permissions
    chmod 755 logs data
    chmod -R 755 data/
    
    print_success "Project directories created"
}

# Generate environment file
generate_env_file() {
    if [ -f ".env" ]; then
        print_warning ".env file already exists, skipping generation"
        return
    fi
    
    print_step "Generating .env file..."
    
    cat > .env << EOF
# 🚀 Cerberus Phoenix v2.0 - Environment Configuration
# Generated on $(date)

# 🗄️ Database Configuration
POSTGRES_PASSWORD=cerberus_$(openssl rand -hex 12)
POSTGRES_DB=cerberus
POSTGRES_USER=cerberus

# 🔐 Vault Configuration
VAULT_ROOT_TOKEN=cerberus_vault_$(openssl rand -hex 16)
VAULT_DEV_ROOT_TOKEN_ID=cerberus_vault_$(openssl rand -hex 16)

# 📊 Monitoring Configuration
GRAFANA_PASSWORD=admin_$(openssl rand -hex 8)
PROMETHEUS_RETENTION_TIME=30d

# 🔑 Infisical Configuration (Production Secrets Management)
INFISICAL_API_URL=https://app.infisical.com/api
INFISICAL_PROJECT_ID=your_project_id_here
INFISICAL_ENVIRONMENT=development
INFISICAL_CLIENT_ID=your_client_id_here
INFISICAL_CLIENT_SECRET=your_client_secret_here

# 🌐 Solana RPC Configuration (Get from providers)
HELIUS_API_KEY=your_helius_api_key_here
QUICKNODE_API_KEY=your_quicknode_api_key_here
JITO_API_KEY=your_jito_api_key_here

# 💰 Trading Configuration
MAX_POSITION_SIZE_SOL=1.0
DAILY_LOSS_LIMIT_SOL=10.0
EMERGENCY_STOP_PERCENTAGE=25.0
TRADING_ENABLED=false

# 🤖 Discord Bot Configuration (Optional)
DISCORD_TOKEN=your_discord_bot_token_here
DISCORD_CHANNEL_ID=your_discord_channel_id_here

# 📱 Telegram Bot Configuration (Optional)
TELEGRAM_TOKEN=your_telegram_bot_token_here
TELEGRAM_CHAT_ID=your_telegram_chat_id_here

# 🌍 Environment Settings
ENVIRONMENT=development
RUST_LOG=info
LOG_LEVEL=info

# 🔧 Service URLs (for Docker networking)
HFT_NINJA_URL=http://hft-ninja:8090
CEREBRO_BFF_URL=http://cerebro-bff:3000
PROMETHEUS_URL=http://prometheus:9090
GRAFANA_URL=http://grafana:3000

# 🌐 External URLs (for local development)
HFT_NINJA_EXTERNAL_URL=http://localhost:8090
CEREBRO_BFF_EXTERNAL_URL=http://localhost:3000
GRAFANA_EXTERNAL_URL=http://localhost:3001
PROMETHEUS_EXTERNAL_URL=http://localhost:9090
EOF

    print_success ".env file generated with secure random passwords"
    print_warning "Please edit .env file and configure your API keys!"
}

# Setup Git hooks
setup_git_hooks() {
    if [ ! -d ".git" ]; then
        print_warning "Not a Git repository, skipping Git hooks setup"
        return
    fi
    
    print_step "Setting up Git hooks..."
    
    # Pre-commit hook for security checks
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook for Cerberus Phoenix v2.0

echo "🔍 Running pre-commit security checks..."

# Check for secrets in code
if grep -r "sk-" --include="*.rs" --include="*.toml" --include="*.yml" .; then
    echo "❌ Potential API key found in code!"
    exit 1
fi

# Check for hardcoded passwords
if grep -r "password.*=" --include="*.rs" --include="*.toml" .; then
    echo "❌ Potential hardcoded password found!"
    exit 1
fi

echo "✅ Security checks passed"
EOF

    chmod +x .git/hooks/pre-commit
    print_success "Git hooks configured"
}

# Create quick start aliases
create_aliases() {
    print_step "Creating convenient aliases..."
    
    cat > cerberus-aliases.sh << 'EOF'
#!/bin/bash
# Cerberus Phoenix v2.0 - Convenient Aliases

# Deployment aliases
alias cerberus-deploy='./scripts/deploy-cerberus.sh'
alias cerberus-dev='./scripts/deploy-cerberus.sh -e development'
alias cerberus-prod='./scripts/deploy-cerberus.sh -e production -d -g'
alias cerberus-monitor='./scripts/deploy-cerberus.sh -m'

# Service management
alias cerberus-logs='docker-compose -f docker-compose.production.yml logs -f'
alias cerberus-status='docker-compose -f docker-compose.production.yml ps'
alias cerberus-stop='docker-compose -f docker-compose.production.yml down'
alias cerberus-restart='docker-compose -f docker-compose.production.yml restart'

# Quick access to services
alias cerberus-hft='curl -s http://localhost:8090/health | jq'
alias cerberus-cerebro='curl -s http://localhost:3000/health | jq'
alias cerberus-grafana='open http://localhost:3001'
alias cerberus-prometheus='open http://localhost:9090'

# Trading commands
alias cerberus-pnl='curl -s http://localhost:8090/api/dashboard/summary | jq'
alias cerberus-trades='curl -s http://localhost:8090/api/dashboard/trades | jq'

echo "🚀 Cerberus Phoenix v2.0 aliases loaded!"
echo "Use 'cerberus-deploy' to start deployment"
EOF

    chmod +x cerberus-aliases.sh
    
    # Add to bashrc/zshrc if user wants
    echo ""
    echo -e "${YELLOW}To load aliases automatically, add this to your ~/.bashrc or ~/.zshrc:${NC}"
    echo "source $(pwd)/cerberus-aliases.sh"
    
    print_success "Aliases created in cerberus-aliases.sh"
}

# Main setup function
main() {
    print_banner
    
    print_step "Starting Cerberus Phoenix v2.0 environment setup..."
    
    check_os
    install_docker
    install_docker_compose
    install_rust
    install_tools
    setup_directories
    generate_env_file
    setup_git_hooks
    create_aliases
    
    print_success "🎉 Environment setup completed successfully!"
    
    echo ""
    echo -e "${CYAN}📋 Next Steps:${NC}"
    echo "1. Edit .env file with your API keys"
    echo "2. Run: source cerberus-aliases.sh"
    echo "3. Deploy: cerberus-deploy"
    echo "4. Monitor: cerberus-grafana"
    echo ""
    echo -e "${GREEN}🚀 Cerberus Phoenix v2.0 is ready for deployment!${NC}"
}

# Run main function
main
