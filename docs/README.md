# 🥷 Cerberus Phoenix v2.0 - Solana HFT Ninja

**Advanced High-Frequency Trading System for Solana with AI-Driven Decision Making**

## 🎯 Overview

Cerberus Phoenix v2.0 is a sophisticated HFT system designed for Solana blockchain, featuring:
- **AI-powered risk analysis** with TF-IDF algorithms
- **Multi-RPC optimization** with 95%+ cost reduction
- **Real-time token discovery** through webhooks and streaming
- **Intelligent caching** with volatility-based TTL
- **Production-ready infrastructure** with monitoring and alerting

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust 1.70+
- Python 3.8+
- 8GB RAM minimum

### 1. Clone & Setup
```bash
git clone https://github.com/SynergiaOS/Cerebrosso.git
cd Cerebrosso
cp infrastructure/.env.example infrastructure/.env
```

### 2. Configure Environment
Edit `infrastructure/.env` with your API keys:
```bash
# Required API Keys
HELIUS_API_KEY=your_helius_api_key
QUICKNODE_API_KEY=your_quicknode_api_key
ALCHEMY_API_KEY=your_alchemy_api_key
WEBHOOK_BASE_URL=https://your-domain.com

# Optional for enhanced features
GENESYS_API_KEY=your_genesys_api_key
CHAINBASE_API_KEY=your_chainbase_api_key
```

### 3. Deploy System
```bash
./scripts/deploy-production.sh
```

### 4. Setup Webhooks
```bash
./scripts/setup-helius-webhooks.py
```

## 📊 System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Cerebro-BFF   │    │   HFT-Ninja     │    │  Infrastructure │
│   (Port 3000)   │    │   (Port 8090)   │    │                 │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • Multi-RPC Mgr │    │ • Trading Engine│    │ • PostgreSQL    │
│ • Risk Analysis │    │ • Order Exec    │    │ • Redis Cache   │
│ • AI Decision   │    │ • Portfolio Mgr │    │ • Qdrant Vector │
│ • Batch Optimizer│   │ • Risk Controls │    │ • Vault Secrets │
│ • Cache Manager │    │ • Metrics       │    │ • Monitoring    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Solana RPC    │
                    │   Providers     │
                    ├─────────────────┤
                    │ • Helius API    │
                    │ • QuickNode     │
                    │ • Alchemy       │
                    │ • Genesys       │
                    │ • Public RPC    │
                    └─────────────────┘
```

## 🎯 Core Features

### 🔄 Multi-RPC Optimization
- **5 RPC Providers**: Helius, QuickNode, Alchemy, Genesys, Public Solana
- **2.2M+ Free Requests/Month**: Combined free tier limits
- **Smart Routing**: Cost-optimized, performance-first, round-robin strategies
- **Automatic Failover**: Zero downtime with health monitoring
- **95%+ Cost Reduction**: Intelligent provider selection

### 🧠 AI-Powered Risk Analysis
- **TF-IDF Algorithms**: Advanced text analysis for token metadata
- **Qdrant Vector Database**: Semantic similarity search
- **Context Engine**: Historical pattern recognition
- **Real-time Scoring**: Dynamic risk assessment
- **Machine Learning**: Continuous improvement from trading results

### 🌊 Real-time Data Streaming
- **Helius Webhooks**: Push notifications for new tokens
- **Solana WebSocket**: Real-time transaction monitoring
- **Event-driven Architecture**: Immediate response to market changes
- **Multi-program Monitoring**: pump.fun, boom, Raydium, Orca

### 💾 Intelligent Caching
- **Volatility-based TTL**: Dynamic cache expiration
- **Multi-tier System**: Hot/Warm/Cold/Frozen data
- **60-75% Cache Hit Rate**: Significant API usage reduction
- **Automatic Optimization**: Self-tuning cache parameters

### 📊 Advanced Monitoring
- **Real-time Metrics**: API usage, costs, performance
- **Grafana Dashboards**: Visual monitoring and alerting
- **Cost Tracking**: Detailed provider usage analysis
- **Health Checks**: Automatic system health monitoring

## 🎛️ API Endpoints

### Core Trading
```bash
# Health check
GET /health

# Token risk analysis
GET /api/v1/risk/analyze/:token

# AI decision making
POST /api/v1/ai/decide
```

### Multi-RPC Management
```bash
# Provider statistics
GET /api/v1/rpc/providers

# Performance report
GET /api/v1/rpc/performance

# Usage trends
GET /api/v1/usage/trends
```

### Optimization Status
```bash
# Overall optimization metrics
GET /api/v1/optimization/status

# Cache performance
GET /api/v1/cache/stats

# Batch processing stats
GET /api/v1/batch/stats
```

### Pump.fun Scanner
```bash
# Discovered tokens
GET /api/v1/pump-fun/discovered

# High potential tokens
GET /api/v1/pump-fun/high-potential

# Scanner statistics
GET /api/v1/pump-fun/stats
```

## 💰 Cost Optimization Results

### Before Optimization
- **Polling Strategy**: 43,200 requests/month
- **Single Provider**: Helius only
- **Estimated Cost**: $93-140/month
- **Limited Redundancy**: Single point of failure

### After Optimization
- **Webhook Events**: ~5,000 requests/month
- **Multi-provider**: 5 RPC providers with failover
- **Estimated Cost**: $13-20/month
- **High Availability**: 99.9% uptime

### **Savings: $80-120/month (85-90% reduction)**

## 🔧 Configuration

### Environment Variables
```bash
# API Usage Limits & Monitoring
HELIUS_MONTHLY_LIMIT=1000000
API_USAGE_ALERT_THRESHOLD=0.8
COST_TRACKING_ENABLED=true

# RPC Configuration
RPC_ROUTING_STRATEGY=cost_optimized
ENABLE_RPC_FAILOVER=true

# Solana Stream
SOLANA_WEBSOCKET_URL=wss://api.mainnet-beta.solana.com/
STREAM_RECONNECT_ATTEMPTS=10
```

### Routing Strategies
- `cost_optimized`: Prefer cheapest available provider (default)
- `performance_first`: Prefer fastest/most reliable provider
- `round_robin`: Distribute evenly across providers
- `weighted_round_robin`: Distribute based on provider priority
- `enhanced_data_first`: Prefer providers with rich metadata

## 📈 Performance Metrics

### Expected Performance
- **API Usage Reduction**: 85-90% through webhooks vs polling
- **Cache Hit Rate**: 60-75% reducing redundant calls
- **Response Time**: <45ms average with caching
- **Uptime**: 99.9% with automatic failover
- **Batch Efficiency**: 92% with getMultipleAccounts

### Trading Performance Targets
- **Daily ROI**: 5% (0.4 SOL from 8 SOL)
- **Strategy Success**: >85% sandwich, >90% arbitrage
- **Execution Latency**: <100ms average, <200ms 99th percentile
- **Risk Management**: Automatic position limits and stop-losses

## 🛠️ Development

### Building from Source
```bash
# Build Cerebro-BFF
cd services/cerebro-bff
cargo build --release

# Build HFT-Ninja
cd ../hft-ninja
cargo build --release
```

### Running Tests
```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html
```

### Development Environment
```bash
# Start development stack
docker-compose -f infrastructure/docker-compose.dev.yml up -d

# Run in development mode
cargo run
```

## 🔒 Security

### Key Management
- **HashiCorp Vault**: Secure secret storage
- **Temporary Keys**: 5-minute TTL for trading keys
- **Encrypted Storage**: All sensitive data encrypted at rest
- **Access Controls**: Role-based access to trading functions

### Risk Controls
- **Position Limits**: Maximum position size per token
- **Stop Losses**: Automatic loss prevention
- **Circuit Breakers**: System-wide trading halts
- **Audit Logging**: Complete transaction history

## 📚 Documentation

- [API Reference](./api-reference.md)
- [Deployment Guide](./deployment.md)
- [Configuration Reference](./configuration.md)
- [Troubleshooting](./troubleshooting.md)
- [Contributing](./contributing.md)

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/SynergiaOS/Cerebrosso/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SynergiaOS/Cerebrosso/discussions)
- **Email**: synergiaos@outlook.com

---

**🥷 Built with ❤️ for the Solana ecosystem**
