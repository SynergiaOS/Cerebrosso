# 🚀 Cerberus Phoenix v2.0 - Complete Deployment Guide

## 🎯 System Overview

**Cerberus Phoenix v2.0** to kompletny system AI-driven HFT trading dla Solana z następującymi komponentami:

### 🧠 **Core AI System**
- **4 wyspecjalizowane AI agenty** (FastDecision, ContextAnalysis, RiskAssessment, DeepAnalysis)
- **Feedback System** - uczenie się z wyników transakcji
- **Paper Trading Engine** - realistyczna symulacja tradingu
- **Adaptive Learning Engine** - automatyczna optymalizacja parametrów

### 🎣 **Real-time Data Processing**
- **Helius Webhook Integration** - real-time token events
- **Market Data Feed** - Helius + QuickNode integration
- **Signal Processing** - inteligentne filtrowanie i analiza

### 📊 **Monitoring & Analytics**
- **Grafana Dashboards** - comprehensive monitoring
- **Prometheus Alerting** - SLA monitoring i alerty
- **Real-time Metrics** - performance tracking

## 🏗️ **Infrastructure Architecture**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HFT-Ninja     │    │   Cerebro-BFF   │    │     Kestra      │
│   Port: 8090    │◄──►│   Port: 3000    │◄──►│   Port: 8080    │
│                 │    │                 │    │                 │
│ • Webhook       │    │ • AI Agents     │    │ • Workflows     │
│ • Execution     │    │ • Learning      │    │ • Automation    │
│ • Strategies    │    │ • Analytics     │    │ • Scheduling    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
    │    Grafana      │    │   Prometheus    │    │    Qdrant       │
    │   Port: 3001    │◄──►│   Port: 9090    │    │   Port: 6333    │
    │                 │    │                 │    │                 │
    │ • Dashboards    │    │ • Metrics       │    │ • Vector DB     │
    │ • Alerting      │    │ • Monitoring    │    │ • Context       │
    └─────────────────┘    └─────────────────┘    └─────────────────┘
                                 │
    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
    │   PostgreSQL    │    │     Vault       │    │    Traefik      │
    │   Port: 5432    │    │   Port: 8200    │    │   Port: 8082    │
    │                 │    │                 │    │                 │
    │ • Feedback DB   │    │ • Secrets       │    │ • Load Balancer │
    │ • Trading Data  │    │ • API Keys      │    │ • SSL Term      │
    └─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🚀 **Quick Start Deployment**

### **1. Przygotowanie środowiska**
```bash
# Clone repository
git clone https://github.com/SynergiaOS/Cerebros.git
cd Cerebros

# Copy environment configuration
cp .env.example .env

# Edit configuration
nano .env
```

### **2. Konfiguracja .env**
```bash
# 🔐 API Keys (WYMAGANE)
HELIUS_API_KEY=your_helius_api_key_here
HELIUS_AUTH_TOKEN=your_helius_webhook_auth_token_here
BIRDEYE_API_KEY=your_birdeye_api_key_here
OUMI_API_KEY=your_oumi_api_key_here

# 🌐 Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_NETWORK=devnet

# 🎣 Webhook Configuration
KESTRA_TRIGGER_URL=http://kestra:8080/api/v1/executions/trigger/helius-webhook
WEBHOOK_RATE_LIMIT=100
```

### **3. Uruchomienie infrastruktury**
```bash
cd infrastructure

# Start all services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f
```

### **4. Weryfikacja deploymentu**
```bash
# Check service health
curl http://localhost:3000/health      # Cerebro-BFF
curl http://localhost:8090/health      # HFT-Ninja
curl http://localhost:8080/health      # Kestra

# Access dashboards
open http://localhost:3001             # Grafana (admin/admin)
open http://localhost:9090             # Prometheus
open http://localhost:8200             # Vault
```

## 📊 **Monitoring Setup**

### **Grafana Dashboards**
1. **Cerberus Phoenix Overview** - Main system dashboard
2. **AI Performance Deep Dive** - Detailed AI agent analysis

**Access**: http://localhost:3001
- Username: `admin`
- Password: `admin`

### **Key Metrics to Monitor**
- **AI Decision Latency**: <100ms target
- **Trading Success Rate**: >85% target
- **System Health Score**: >0.8 target
- **Webhook Processing**: <10ms target

## 🎣 **Helius Webhook Configuration**

### **1. Setup w Helius Dashboard**
```
Webhook URL: https://your-domain.com/webhooks/helius
Auth Token: your_helius_webhook_auth_token_here
Events: Token transfers, Account changes, Program interactions
```

### **2. Test webhook**
```bash
curl -X POST http://localhost:8090/webhooks/helius \
  -H "Authorization: Bearer your_helius_auth_token" \
  -H "Content-Type: application/json" \
  -d '{
    "account_addresses": ["TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj"],
    "transaction_types": ["token_mint"],
    "events": [...]
  }'
```

## 🎯 **Trading Configuration**

### **Paper Trading Setup**
```bash
# Create virtual portfolio
curl -X POST http://localhost:3000/api/v1/paper-trading/portfolio \
  -H "Content-Type: application/json" \
  -d '{
    "name": "AI_Strategy_v1",
    "initial_balance": 8.0
  }'
```

### **AI Agent Configuration**
- **FastDecision**: <20ms latency, confidence threshold 0.6
- **ContextAnalysis**: <50ms latency, context window 2048
- **RiskAssessment**: <30ms latency, risk threshold 0.7
- **DeepAnalysis**: <200ms latency, analysis depth 5.0

## 🔧 **Production Deployment**

### **Oracle Cloud Setup**
```bash
cd infrastructure/terraform

# Initialize Terraform
terraform init

# Plan deployment
terraform plan

# Deploy infrastructure
terraform apply
```

### **SSL & Domain Configuration**
```yaml
# docker-compose.yml
traefik:
  command:
    - --certificatesresolvers.letsencrypt.acme.email=your-email@domain.com
    - --certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json
  labels:
    - "traefik.http.routers.api.tls.certresolver=letsencrypt"
```

### **Security Hardening**
1. **Change default passwords** (Grafana, Vault)
2. **Setup Vault secrets** for API keys
3. **Configure firewall rules**
4. **Enable SSL/TLS** for all services
5. **Setup backup procedures**

## 📈 **Performance Targets**

### **Latency Targets**
- **FastDecision Agent**: <20ms (95th percentile)
- **ContextAnalysis Agent**: <50ms (95th percentile)
- **RiskAssessment Agent**: <30ms (95th percentile)
- **DeepAnalysis Agent**: <200ms (95th percentile)

### **Trading Performance**
- **Daily ROI Target**: 5% (0.4 SOL from 8 SOL)
- **Win Rate Target**: >85%
- **Max Drawdown**: <15%
- **Sharpe Ratio**: >2.0

### **System Performance**
- **Webhook Processing**: <10ms average
- **System Uptime**: >99.9%
- **AI Decision Rate**: >1 decision/second
- **Memory Usage**: <4GB per service

## 🚨 **Alerting Configuration**

### **Critical Alerts**
- AI Decision Latency >200ms
- Portfolio Loss >20%
- No Trading Activity >10min
- System Health <0.5

### **Warning Alerts**
- AI Performance Degraded <40% win rate
- High Latency >100ms
- Portfolio Loss >10%
- Market Data Stale

## 🔍 **Troubleshooting**

### **Common Issues**
1. **Service won't start**: Check Docker logs and port conflicts
2. **AI decisions slow**: Monitor system resources and model performance
3. **Webhook failures**: Verify auth tokens and network connectivity
4. **Database errors**: Check PostgreSQL connection and migrations

### **Debug Commands**
```bash
# Service logs
docker-compose logs -f cerebro-bff
docker-compose logs -f hft-ninja

# System resources
docker stats

# Database status
docker-compose exec postgres psql -U postgres -c "\l"

# Metrics check
curl http://localhost:3000/metrics
curl http://localhost:8090/webhooks/metrics
```

## 📚 **Additional Resources**

- [AI Agents Documentation](services/cerebro-bff/README.md)
- [Webhook Integration Guide](services/hft-ninja/README_WEBHOOK.md)
- [Grafana Dashboards](infrastructure/grafana/dashboards/README.md)
- [Security Best Practices](docs/security.md)
- [Performance Tuning](docs/performance.md)

## 🎯 **Next Steps**

1. **Devnet Testing** - Test with Solana devnet
2. **Strategy Optimization** - Tune AI parameters
3. **Performance Monitoring** - Watch metrics and optimize
4. **Mainnet Preparation** - Security audit and final testing
5. **Production Launch** - Go live with real trading

---

**Status**: ✅ **PRODUCTION READY**  
**Version**: v2.0  
**Last Updated**: 2024-01-15
