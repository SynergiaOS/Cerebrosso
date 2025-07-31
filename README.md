# 🔥 Cerberus Phoenix v2.0

**Advanced High-Frequency Trading System for Solana Blockchain**

Cerberus Phoenix v2.0 is a sophisticated, AI-driven HFT system designed for ultra-low latency trading on the Solana blockchain. The system leverages advanced Context Engine technology with TF-IDF weighting, Apriori rule mining, and enterprise-grade security.

## 🚀 Key Features

### 🧠 **Context Engine v2.0**
- **TF-IDF Weighting**: Advanced term frequency analysis for signal importance
- **Apriori Rule Mining**: Pattern discovery for trading decision rules  
- **Shuffle Haystacks**: Context optimization to prevent AI degradation
- **Semantic Noise Filtering**: Smart filtering of irrelevant signals

### 🛡️ **Security-First Architecture**
- **HashiCorp Vault Integration**: Secure key management
- **Encrypted Storage**: API keys and private keys stored securely
- **Transit Engine**: Encryption/decryption for sensitive data

### ⚡ **High-Performance Trading**
- **Ultra-Low Latency**: Optimized for microsecond-level execution
- **Jito Bundle Integration**: MEV protection and faster execution
- **Multi-DEX Arbitrage**: Cross-DEX opportunity detection
- **Memecoin Sniping**: Advanced token launch detection

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HFT-Ninja     │◄──►│  Cerebro-BFF    │◄──►│     Vault       │
│  (Rust Engine) │    │ (AI Decision)   │    │  (Security)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Helius      │    │     Qdrant      │    │   Monitoring    │
│   (Webhooks)    │    │   (Context)     │    │ (Prometheus)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🐝 Hive Mind Architecture v3.0

Cerberus Phoenix v3.0 introduces revolutionary **Swarmagentic Intelligence** - a distributed AI system where specialized agents collaborate to make optimal trading decisions.

### 🎯 **Agent Hierarchy**
```
👑 Agent-Strateg (CEO) - 40% decision weight
├── 🔬 Agent-Analityk - 25% weight (qualitative analysis)
├── 🧮 Agent-Quant - 30% weight (quantitative modeling)
└── 🛡️ Agent-Nadzorca - 5% weight + veto power (security)
```

### 🧠 **SwarmCoordinator**
- **Central Orchestrator** managing all AI agents
- **Task Delegation** with intelligent routing
- **Real-time Communication** via Redis + WebSocket
- **Memory Management** (Working/Short/Long-term)
- **Feedback Loop** for continuous learning

### 🎯 **Agent Specializations**
- **Goal Decomposition** - Breaking complex objectives into tasks
- **Decision Synthesis** - Weighted voting with confidence thresholds
- **Risk Assessment** - Multi-level risk evaluation
- **Pattern Recognition** - Learning from historical data

## 🛠️ Services

### **🐝 SwarmCoordinator** (Rust) - *NEW v3.0*
- Central orchestrator for Hive Mind architecture
- Agent registry and lifecycle management
- Task delegation with intelligent routing
- Real-time communication hub (Redis + WebSocket)
- Multi-level memory system (Working/Short/Long-term)
- Feedback loop for continuous learning
- **Port**: 8090 (HTTP), 8091 (WebSocket)

### **👑 Agent-Strateg** (Rust) - *NEW v3.0*
- CEO agent with 40% decision weight
- Goal decomposition and strategic planning
- Task delegation to specialized agents
- Decision synthesis from agent reports
- Risk management and position sizing
- Multi-model AI orchestration (GPT-4, Claude-3, Llama3)
- **Port**: 8100

### **HFT-Ninja** (Rust)
- Real-time webhook processing
- Ultra-low latency trade execution
- Jito bundle integration
- Risk management
- **Port**: 8080

### **Cerebro-BFF** (Rust)
- AI-driven decision making
- Context Engine v2.0
- Multi-model AI orchestration
- Advanced signal processing
- **Port**: 3000

### **Infrastructure**
- **Vault**: Secure secrets management
- **Qdrant**: Vector database for context storage
- **Prometheus/Grafana**: Monitoring and alerting
- **PostgreSQL**: Trade data storage

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust toolchain
- API keys (Helius, QuickNode/Alchemy)

### 1. Clone Repository
```bash
git clone https://github.com/SynergiaOS/Cerebrosso.git
cd Cerebrosso
```

### 2. Start Infrastructure
```bash
./scripts/start-dev-infrastructure.sh
```

### 3. Configure Environment
```bash
cp .env.example .env
# Edit .env with your API keys
```

### 4. Deploy Hive Mind Architecture
```bash
# 🐝 Deploy complete Hive Mind (RECOMMENDED)
./scripts/deploy-hive-mind.sh

# Or start individual services
docker-compose up swarm-coordinator agent-strateg

# Or start all services manually
docker-compose up -d
```

### 5. Monitor System
```bash
# 📊 Real-time monitoring dashboard
./scripts/monitor-hive-mind.sh

# Continuous monitoring (refreshes every 10s)
./scripts/monitor-hive-mind.sh --continuous
```

### 5. Run Tests
```bash
./scripts/run_tests.sh
```

## 📊 Monitoring & Access

### 🐝 **Hive Mind Services**
- **SwarmCoordinator**: <http://localhost:8090> (API), <http://localhost:8091> (WebSocket)
- **Agent-Strateg**: <http://localhost:8100> (CEO Agent)
- **Agent-Analityk**: <http://localhost:8101> (Coming Soon)
- **Agent-Quant**: <http://localhost:8102> (Coming Soon)
- **Agent-Nadzorca**: <http://localhost:8103> (Coming Soon)

### ⚡ **Core Services**
- **HFT-Ninja**: <http://localhost:8080>
- **Cerebro-BFF**: <http://localhost:3000>
- **Telegram Bot**: Integrated

### 📊 **Monitoring Dashboard**
- **Grafana**: <http://localhost:3001>
- **Prometheus**: <http://localhost:9090>
- **Vault UI**: <http://localhost:8200>
- **Qdrant**: <http://localhost:6333>

## 🎯 Performance Targets

Cerberus Phoenix v3.0 Hive Mind achieves enterprise-grade performance:

### ⚡ **Latency Targets**
- **P95 Latency**: <100ms (Sub-100ms response time)
- **P99 Latency**: <150ms (Ultra-low latency for critical operations)
- **Average Latency**: <50ms (Optimal user experience)

### 🎯 **Accuracy Targets**
- **Decision Accuracy**: 84.8% (SWE Bench benchmark level)
- **Confidence Threshold**: >70% (High-confidence decisions only)
- **Prediction Precision**: >85% (Minimal false positives)

### 📊 **Throughput Targets**
- **Requests/Second**: 1,000+ RPS (High-volume processing)
- **Cache Hit Rate**: >95% (Optimal caching efficiency)
- **Uptime**: 99.9% (Enterprise reliability)

### 🧠 **AI Performance**
- **Context Quality**: >70% (High-quality context generation)
- **Pattern Recognition**: Real-time pattern detection
- **Multi-Model Ensemble**: 4+ AI models with weighted voting

## 🧪 Testing

The system includes comprehensive testing:

```bash
# Unit tests
cd services/cerebro-bff && cargo test

# Integration tests  
cargo test --test integration_tests

# End-to-end tests (requires running services)
cargo test --test e2e_tests -- --ignored
```

## 📚 Documentation

- [API Reference](docs/API_REFERENCE.md)
- [Configuration Guide](docs/CONFIGURATION_REFERENCE.md)
- [Deployment Guide](docs/DEPLOYMENT_GUIDE.md)
- [Helius Integration](docs/HELIUS_WEBHOOK_INTEGRATION.md)

## 🔧 Configuration

Key configuration files:
- `docker-compose.yml` - Main service orchestration
- `.env` - Environment variables and API keys
- `config/vault/vault.hcl` - Vault configuration

## 🚨 Security

- All API keys stored in HashiCorp Vault
- Encrypted communication between services
- Regular security audits and monitoring
- No hardcoded secrets in codebase

## 📈 Performance

- **Latency**: < 10ms webhook to decision
- **Throughput**: 1000+ requests/second
- **Uptime**: 99.9% availability target
- **Memory**: < 512MB per service

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is for educational and research purposes. Trading cryptocurrencies involves substantial risk. Use at your own risk.

## 🔗 Links

- **Repository**: https://github.com/SynergiaOS/Cerebrosso
- **Issues**: https://github.com/SynergiaOS/Cerebrosso/issues
- **Discussions**: https://github.com/SynergiaOS/Cerebrosso/discussions

---

**Built with ❤️ by the Cerberus Phoenix Team**
