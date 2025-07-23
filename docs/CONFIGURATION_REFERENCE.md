# ⚙️ Configuration Reference - Cerberus Phoenix v2.0

## 📁 Configuration Files

### Primary Configuration
- `infrastructure/.env` - Main environment variables
- `infrastructure/docker-compose.core.yml` - Docker services
- `infrastructure/grafana/dashboards/` - Monitoring dashboards
- `infrastructure/prometheus/prometheus.yml` - Metrics collection

---

## 🔧 Environment Variables

### 🔑 API Keys & Authentication
```bash
# Required API Keys
HELIUS_API_KEY=your_helius_api_key_here
QUICKNODE_API_KEY=your_quicknode_api_key_here
ALCHEMY_API_KEY=your_alchemy_api_key_here

# Optional Enhanced Providers
GENESYS_API_KEY=your_genesys_api_key_here
CHAINBASE_API_KEY=your_chainbase_api_key_here
BLOCKCHAIR_API_KEY=your_blockchair_api_key_here

# Webhook Configuration
WEBHOOK_BASE_URL=https://your-domain.com
HELIUS_WEBHOOK_SECRET=your_webhook_secret_here
```

### 🗄️ Database Configuration
```bash
# PostgreSQL
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=cerberus
POSTGRES_USER=postgres
POSTGRES_PASSWORD=secure_password_here

# Qdrant Vector Database
QDRANT_URL=http://localhost:6333
QDRANT_COLLECTION=cerberus_context
QDRANT_API_KEY=optional_api_key

# Redis Cache
REDIS_URL=redis://localhost:6380
REDIS_TTL=300
REDIS_MAX_CONNECTIONS=100
```

### 🔒 Security & Secrets
```bash
# HashiCorp Vault
VAULT_URL=http://localhost:8201
VAULT_TOKEN=vault_root_token_here
VAULT_MOUNT_PATH=secret

# JWT Configuration
JWT_SECRET=your_jwt_secret_here
JWT_EXPIRATION=3600

# API Rate Limiting
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60
```

### 🔄 Multi-RPC Configuration
```bash
# Routing Strategy Options:
# - cost_optimized (default)
# - performance_first
# - round_robin
# - weighted_round_robin
# - enhanced_data_first
RPC_ROUTING_STRATEGY=cost_optimized

# Provider Limits (for tracking)
HELIUS_MONTHLY_LIMIT=1000000
QUICKNODE_MONTHLY_LIMIT=100000
ALCHEMY_MONTHLY_LIMIT=100000
GENESYS_MONTHLY_LIMIT=1000000

# Failover Configuration
ENABLE_RPC_FAILOVER=true
RPC_HEALTH_CHECK_INTERVAL=300
RPC_TIMEOUT_SECONDS=30
```

### 📊 API Usage Monitoring
```bash
# Usage Limits & Alerts
API_USAGE_ALERT_THRESHOLD=0.8
COST_TRACKING_ENABLED=true
USAGE_RESET_DAY=1

# Monthly Cost Limits
MAX_MONTHLY_COST_USD=100
COST_ALERT_THRESHOLD=0.8
```

### 🌊 Solana Stream Configuration
```bash
# WebSocket Settings
SOLANA_WEBSOCKET_URL=wss://api.mainnet-beta.solana.com/
STREAM_RECONNECT_ATTEMPTS=10
STREAM_PING_INTERVAL=30000
STREAM_BUFFER_SIZE=1000

# Subscription Settings
ENABLE_TOKEN_MINT_STREAM=true
ENABLE_PUMP_FUN_STREAM=true
ENABLE_DEX_STREAM=true
```

### 💾 Cache Configuration
```bash
# Intelligent Cache Settings
CACHE_DEFAULT_TTL=300
CACHE_MAX_SIZE=10000
CACHE_CLEANUP_INTERVAL=3600

# Volatility-based TTL
CACHE_HOT_TTL=60
CACHE_WARM_TTL=300
CACHE_COLD_TTL=1800
CACHE_FROZEN_TTL=7200

# Cache Tiers
CACHE_HOT_THRESHOLD=0.8
CACHE_WARM_THRESHOLD=0.5
CACHE_COLD_THRESHOLD=0.2
```

### 🎯 Trading Configuration
```bash
# Risk Management
DEFAULT_RISK_TOLERANCE=0.7
MAX_POSITION_SIZE=0.1
STOP_LOSS_PERCENTAGE=0.05
TAKE_PROFIT_PERCENTAGE=0.15

# Trading Limits
MAX_DAILY_TRADES=50
MAX_CONCURRENT_POSITIONS=5
MIN_LIQUIDITY_USD=10000

# Execution Settings
SLIPPAGE_TOLERANCE=0.01
PRIORITY_FEE_LAMPORTS=5000
```

---

## 🐳 Docker Configuration

### Service Definitions
```yaml
# docker-compose.core.yml structure
services:
  cerebro-bff:
    image: cerberus/cerebro-bff:latest
    ports:
      - "3000:3000"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=${DATABASE_URL}
    
  hft-ninja:
    image: cerberus/hft-ninja:latest
    ports:
      - "8090:8090"
    
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    
  redis:
    image: redis:7-alpine
    command: redis-server --maxmemory 512mb
    
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
      - "6334:6334"
```

### Resource Limits
```yaml
# Resource constraints
services:
  cerebro-bff:
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'
        reservations:
          memory: 512M
          cpus: '0.5'
```

---

## 📊 Monitoring Configuration

### Prometheus Metrics
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  - "alerts.yml"

scrape_configs:
  - job_name: 'cerebro-bff'
    static_configs:
      - targets: ['cerebro-bff:3000']
    metrics_path: '/metrics'
    scrape_interval: 30s
  
  - job_name: 'hft-ninja'
    static_configs:
      - targets: ['hft-ninja:8090']
```

### Grafana Dashboards
```json
{
  "dashboard": {
    "title": "Cerberus Phoenix v2.0",
    "panels": [
      {
        "title": "API Usage",
        "type": "stat",
        "targets": [
          {
            "expr": "api_requests_total",
            "legendFormat": "Total Requests"
          }
        ]
      },
      {
        "title": "RPC Provider Health",
        "type": "table",
        "targets": [
          {
            "expr": "rpc_provider_health",
            "legendFormat": "{{provider}}"
          }
        ]
      }
    ]
  }
}
```

### Alert Rules
```yaml
# alerts.yml
groups:
  - name: cerberus_alerts
    rules:
      - alert: HighAPIUsage
        expr: api_usage_percentage > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "API usage above 80%"
          description: "Current usage: {{ $value }}%"
      
      - alert: RpcProviderDown
        expr: rpc_provider_health == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "RPC provider {{ $labels.provider }} is down"
      
      - alert: HighMemoryUsage
        expr: memory_usage_percent > 90
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
```

---

## 🔧 Application Configuration

### Rust Application Settings
```toml
# Cargo.toml features
[features]
default = ["production"]
development = ["debug-logs", "test-mode"]
production = ["optimized", "monitoring"]

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Logging Configuration
```bash
# Rust logging levels
RUST_LOG=info
RUST_BACKTRACE=1

# Component-specific logging
RUST_LOG=cerebro_bff=debug,hft_ninja=info,sqlx=warn

# Log format
LOG_FORMAT=json
LOG_LEVEL=info
```

### Performance Tuning
```bash
# Tokio runtime
TOKIO_WORKER_THREADS=4
TOKIO_BLOCKING_THREADS=16

# HTTP server
HTTP_TIMEOUT_SECONDS=30
HTTP_MAX_CONNECTIONS=1000
HTTP_KEEP_ALIVE_SECONDS=60

# Database connections
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_TIMEOUT_SECONDS=30
```

---

## 🎛️ Feature Flags

### Core Features
```bash
# Enable/disable major features
ENABLE_AI_DECISION_ENGINE=true
ENABLE_PUMP_FUN_SCANNER=true
ENABLE_RISK_ANALYSIS=true
ENABLE_BATCH_PROCESSING=true

# Experimental features
ENABLE_EXPERIMENTAL_FEATURES=false
ENABLE_BETA_ALGORITHMS=false
```

### Trading Features
```bash
# Trading modes
ENABLE_PAPER_TRADING=false
ENABLE_LIVE_TRADING=true
ENABLE_ARBITRAGE=true
ENABLE_SANDWICH_ATTACKS=false

# Safety features
ENABLE_CIRCUIT_BREAKERS=true
ENABLE_POSITION_LIMITS=true
ENABLE_LOSS_LIMITS=true
```

---

## 🔍 Development Configuration

### Development Environment
```bash
# Development mode
NODE_ENV=development
RUST_ENV=development
DEBUG=true

# Hot reloading
ENABLE_HOT_RELOAD=true
WATCH_FILES=true

# Test configuration
RUN_INTEGRATION_TESTS=true
MOCK_EXTERNAL_APIS=true
```

### Testing Configuration
```bash
# Test database
TEST_DATABASE_URL=postgresql://test:test@localhost:5433/test_cerberus

# Test API keys (use test/sandbox keys)
TEST_HELIUS_API_KEY=test_key_here
TEST_QUICKNODE_API_KEY=test_key_here

# Test settings
TEST_TIMEOUT_SECONDS=30
PARALLEL_TESTS=true
```

---

## 📋 Configuration Validation

### Required Variables Check
```bash
#!/bin/bash
# validate-config.sh

required_vars=(
    "HELIUS_API_KEY"
    "POSTGRES_PASSWORD"
    "VAULT_TOKEN"
    "WEBHOOK_BASE_URL"
)

for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "❌ Required variable $var is not set"
        exit 1
    fi
done

echo "✅ All required variables are set"
```

### Configuration Templates
```bash
# Generate configuration from template
envsubst < infrastructure/.env.template > infrastructure/.env

# Validate configuration
./scripts/validate-config.sh
```

---

## 🔄 Configuration Updates

### Hot Reloading
```bash
# Reload configuration without restart
curl -X POST http://localhost:3000/admin/reload-config

# Restart specific service
docker-compose restart cerebro-bff
```

### Configuration Backup
```bash
# Backup current configuration
cp infrastructure/.env backup/.env.$(date +%Y%m%d_%H%M%S)

# Restore configuration
cp backup/.env.20240115_103000 infrastructure/.env
```

---

## 📞 Configuration Support

### Common Configuration Issues
1. **Invalid API Keys**: Verify keys with provider APIs
2. **Database Connection**: Check credentials and network
3. **Port Conflicts**: Ensure ports are available
4. **Memory Limits**: Adjust based on available resources

### Configuration Validation Tools
```bash
# Test database connection
docker exec postgres pg_isready

# Test Redis connection
docker exec redis redis-cli ping

# Test API endpoints
curl http://localhost:3000/health
```
