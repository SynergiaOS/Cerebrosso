# 🚀 Deployment Guide - Cerberus Phoenix v2.0

## 📋 Prerequisites

### System Requirements
- **OS**: Linux (Ubuntu 20.04+ recommended)
- **RAM**: 8GB minimum, 16GB recommended
- **CPU**: 4 cores minimum, 8 cores recommended
- **Storage**: 50GB SSD minimum
- **Network**: Stable internet connection

### Software Dependencies
- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **Git**: 2.30+
- **Python**: 3.8+ (for webhook setup)
- **Rust**: 1.70+ (for building from source)

---

## 🔧 Quick Deployment

### 1. Clone Repository
```bash
git clone https://github.com/SynergiaOS/Cerebrosso.git
cd Cerebrosso
```

### 2. Environment Setup
```bash
# Copy environment template
cp infrastructure/.env.example infrastructure/.env

# Edit configuration
nano infrastructure/.env
```

### 3. Configure API Keys
Edit `infrastructure/.env` with your actual API keys:

```bash
# Required API Keys
HELIUS_API_KEY=your_helius_api_key_here
QUICKNODE_API_KEY=your_quicknode_api_key_here
ALCHEMY_API_KEY=your_alchemy_api_key_here

# Webhook Configuration
WEBHOOK_BASE_URL=https://your-domain.com

# Database & Security
POSTGRES_PASSWORD=secure_password_here
VAULT_TOKEN=vault_root_token_here

# Optional Enhanced Features
GENESYS_API_KEY=your_genesys_api_key_here
CHAINBASE_API_KEY=your_chainbase_api_key_here
```

### 4. Deploy System
```bash
# Make deployment script executable
chmod +x scripts/deploy-production.sh

# Run deployment
./scripts/deploy-production.sh
```

### 5. Setup Helius Webhook Integration
```bash
# Configure Helius webhooks for real-time token events
./scripts/setup-helius-webhooks.py

# Test webhook integration
./scripts/test-helius-webhook.sh
```

**Webhook Configuration:**
- **Endpoint**: `https://your-domain.com/webhooks/helius`
- **Authentication**: Bearer token from `HELIUS_AUTH_TOKEN`
- **Events**: Token transfers, account changes, program interactions
- **Rate Limit**: 100 requests/minute
- **Processing**: <10ms average latency

---

## 🏗️ Manual Deployment

### 1. Infrastructure Services
```bash
cd infrastructure

# Start core infrastructure
docker-compose -f docker-compose.core.yml up -d \
  postgres vault qdrant redis prometheus grafana traefik kestra
```

### 2. Build Applications
```bash
# Build Cerebro-BFF
cd services/cerebro-bff
cargo build --release

# Build HFT-Ninja
cd ../hft-ninja
cargo build --release
```

### 3. Start Applications
```bash
cd infrastructure

# Start applications
docker-compose -f docker-compose.core.yml up -d \
  cerebro-bff hft-ninja
```

---

## 🔍 Verification

### Health Checks
```bash
# Check all services
curl http://localhost:3000/health
curl http://localhost:8090/health

# Check infrastructure
curl http://localhost:3001  # Grafana
curl http://localhost:8082  # Traefik
curl http://localhost:9090  # Prometheus
```

### Service Status
```bash
# View running containers
docker ps

# Check logs
docker-compose -f infrastructure/docker-compose.core.yml logs -f
```

### API Testing
```bash
# Test Multi-RPC
curl http://localhost:3000/api/v1/rpc/providers

# Test optimization status
curl http://localhost:3000/api/v1/optimization/status

# Test usage monitoring
curl http://localhost:3000/api/v1/usage/report
```

---

## 🌐 Production Configuration

### SSL/HTTPS Setup
```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Generate SSL certificate
sudo certbot --nginx -d your-domain.com

# Update webhook URL
export WEBHOOK_BASE_URL=https://your-domain.com
```

### Firewall Configuration
```bash
# Allow required ports
sudo ufw allow 22    # SSH
sudo ufw allow 80    # HTTP
sudo ufw allow 443   # HTTPS
sudo ufw allow 3000  # Cerebro-BFF
sudo ufw allow 8090  # HFT-Ninja

# Enable firewall
sudo ufw enable
```

### Reverse Proxy (Nginx)
```nginx
# /etc/nginx/sites-available/cerberus-phoenix
server {
    listen 80;
    server_name your-domain.com;
    
    location /api/ {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
    
    location /webhooks/ {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 📊 Monitoring Setup

### Grafana Dashboards
1. Access Grafana: `http://localhost:3001`
2. Login: `admin/admin`
3. Import dashboards from `infrastructure/grafana/dashboards/`

### Prometheus Configuration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'cerebro-bff'
    static_configs:
      - targets: ['localhost:3000']
  
  - job_name: 'hft-ninja'
    static_configs:
      - targets: ['localhost:8090']
```

### Alerting Rules
```yaml
# alerts.yml
groups:
  - name: cerberus_alerts
    rules:
      - alert: HighAPIUsage
        expr: api_usage_percentage > 80
        for: 5m
        annotations:
          summary: "API usage above 80%"
      
      - alert: RpcProviderDown
        expr: rpc_provider_health == 0
        for: 1m
        annotations:
          summary: "RPC provider is down"
```

---

## 🔄 Updates & Maintenance

### Update System
```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
./scripts/deploy-production.sh restart
```

### Backup Data
```bash
# Backup PostgreSQL
docker exec postgres pg_dump -U postgres cerberus > backup.sql

# Backup Qdrant
docker exec qdrant tar -czf /tmp/qdrant_backup.tar.gz /qdrant/storage

# Backup configuration
cp infrastructure/.env backup/.env.$(date +%Y%m%d)
```

### Log Management
```bash
# View logs
docker-compose -f infrastructure/docker-compose.core.yml logs -f cerebro-bff

# Log rotation
sudo logrotate -f /etc/logrotate.d/docker-containers
```

---

## 🐛 Troubleshooting

### Common Issues

#### Services Not Starting
```bash
# Check Docker status
sudo systemctl status docker

# Check available resources
free -h
df -h

# Restart Docker
sudo systemctl restart docker
```

#### API Key Issues
```bash
# Verify API keys
curl -H "Authorization: Bearer $HELIUS_API_KEY" \
     https://api.helius.xyz/v1/health

# Check environment variables
docker exec cerebro-bff env | grep API_KEY
```

#### Database Connection Issues
```bash
# Check PostgreSQL
docker exec postgres psql -U postgres -c "SELECT version();"

# Reset database
docker-compose -f infrastructure/docker-compose.core.yml restart postgres
```

#### High Memory Usage
```bash
# Check memory usage
docker stats

# Restart services
docker-compose -f infrastructure/docker-compose.core.yml restart
```

### Performance Optimization

#### Database Tuning
```sql
-- PostgreSQL optimization
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
```

#### Redis Optimization
```bash
# Redis memory optimization
docker exec redis redis-cli CONFIG SET maxmemory 512mb
docker exec redis redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

---

## 📈 Scaling

### Horizontal Scaling
```yaml
# docker-compose.scale.yml
version: '3.8'
services:
  cerebro-bff:
    deploy:
      replicas: 3
  
  hft-ninja:
    deploy:
      replicas: 2
```

### Load Balancing
```nginx
upstream cerebro_backend {
    server localhost:3000;
    server localhost:3001;
    server localhost:3002;
}

server {
    location /api/ {
        proxy_pass http://cerebro_backend;
    }
}
```

---

## 🔒 Security Hardening

### Container Security
```bash
# Run containers as non-root
docker run --user 1000:1000 cerebro-bff

# Limit container resources
docker run --memory=512m --cpus=1.0 cerebro-bff
```

### Network Security
```bash
# Create isolated network
docker network create --driver bridge cerberus-network

# Use custom network
docker-compose --network cerberus-network up
```

### Secret Management
```bash
# Use Docker secrets
echo "api_key_value" | docker secret create helius_api_key -

# Mount secrets in containers
docker service create --secret helius_api_key cerebro-bff
```

---

## 📞 Support

### Getting Help
- **Documentation**: Check all docs in `/docs` directory
- **Issues**: [GitHub Issues](https://github.com/SynergiaOS/Cerebrosso/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SynergiaOS/Cerebrosso/discussions)

### Emergency Procedures
```bash
# Emergency stop
./scripts/deploy-production.sh stop

# Emergency restart
./scripts/deploy-production.sh restart

# Check system status
./scripts/deploy-production.sh status
```
