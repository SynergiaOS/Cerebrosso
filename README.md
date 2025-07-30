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

## 🛠️ Services

### **HFT-Ninja** (Rust)
- Real-time webhook processing
- Ultra-low latency trade execution
- Jito bundle integration
- Risk management

### **Cerebro-BFF** (Rust)
- AI-driven decision making
- Context Engine v2.0
- Multi-model AI orchestration
- Advanced signal processing

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

### 4. Start Services
```bash
./scripts/start-cerberus.sh
```

### 5. Run Tests
```bash
./scripts/run_tests.sh
```

## 📊 Monitoring

Access the monitoring dashboard at:
- **Grafana**: http://localhost:3001
- **Prometheus**: http://localhost:9090
- **Vault UI**: http://localhost:8200

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
