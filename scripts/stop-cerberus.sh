#!/bin/bash
# 🛑 Cerberus Phoenix v2.0 - System Stop
# Zatrzymuje wszystkie serwisy

set -euo pipefail

# 🎨 Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 📝 Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# 🛑 Stop Rust services
stop_rust_services() {
    log "🛑 Stopping Rust services..."
    
    # Stop HFT-Ninja
    if [[ -f logs/hft-ninja.pid ]]; then
        PID=$(cat logs/hft-ninja.pid)
        if kill -0 $PID 2>/dev/null; then
            kill $PID
            success "HFT-Ninja stopped"
        fi
        rm -f logs/hft-ninja.pid
    fi
    
    # Stop Cerebro-BFF
    if [[ -f logs/cerebro-bff.pid ]]; then
        PID=$(cat logs/cerebro-bff.pid)
        if kill -0 $PID 2>/dev/null; then
            kill $PID
            success "Cerebro-BFF stopped"
        fi
        rm -f logs/cerebro-bff.pid
    fi
    
    # Kill any remaining cargo processes
    pkill -f "cargo run" 2>/dev/null || true
    
    success "Rust services stopped"
}

# 🐳 Stop Docker services
stop_docker_services() {
    log "🐳 Stopping Docker services..."
    
    cd infrastructure
    docker-compose down
    cd ..
    
    success "Docker services stopped"
}

# 🔐 Stop Vault
stop_vault() {
    log "🔐 Stopping Vault..."
    
    pkill -f "vault server" 2>/dev/null || true
    
    success "Vault stopped"
}

# 🧹 Cleanup
cleanup() {
    log "🧹 Cleaning up..."
    
    # Remove PID files
    rm -f logs/*.pid
    
    # Clean up any orphaned processes
    pkill -f "hft-ninja" 2>/dev/null || true
    pkill -f "cerebro-bff" 2>/dev/null || true
    
    success "Cleanup completed"
}

# 🚀 Main execution
main() {
    log "🛑 Stopping Cerberus Phoenix v2.0..."
    
    stop_rust_services
    stop_docker_services
    stop_vault
    cleanup
    
    success "🎉 All services stopped successfully!"
}

# Execute main function
main "$@"
