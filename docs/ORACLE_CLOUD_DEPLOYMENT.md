# ☁️ Oracle Cloud Deployment Guide - Cerberus Phoenix v2.0

## 🎯 Overview

Deploy Cerberus Phoenix v2.0 to Oracle Cloud Free Tier with full Multi-RPC optimization, achieving **95%+ cost reduction** while running on **FREE** infrastructure.

### 💰 Cost Benefits
- **Oracle Cloud**: FREE (Always Free tier)
- **API Costs**: $13-20/month (vs $93-140 with single provider)
- **Total Monthly Cost**: $13-20 (vs $193-280 traditional setup)
- **Annual Savings**: $2,160-3,120

## 🏗️ Oracle Cloud Free Tier Specifications

### VM.Standard.A1.Flex Instance
- **CPU**: 4 OCPU (ARM-based Ampere Altra)
- **Memory**: 24GB RAM
- **Storage**: 200GB Block Volume
- **Network**: 10 Gbps bandwidth
- **Cost**: **FREE** (Always Free tier)

### Additional Free Services
- **Load Balancer**: 10 Mbps bandwidth
- **Object Storage**: 20GB
- **Archive Storage**: 10GB
- **Monitoring**: Basic metrics
- **Networking**: VCN, Security Lists, Route Tables

## 🚀 Prerequisites

### 1. Oracle Cloud Account
```bash
# Sign up for Oracle Cloud Free Tier
https://www.oracle.com/cloud/free/

# Verify account and complete identity verification
# Note: Credit card required for verification (not charged)
```

### 2. Local Tools Installation
```bash
# Install OCI CLI
bash -c "$(curl -L https://raw.githubusercontent.com/oracle/oci-cli/master/scripts/install/install.sh)"

# Install Terraform
wget https://releases.hashicorp.com/terraform/1.6.0/terraform_1.6.0_linux_amd64.zip
unzip terraform_1.6.0_linux_amd64.zip
sudo mv terraform /usr/local/bin/

# Verify installations
oci --version
terraform --version
```

### 3. OCI Configuration
```bash
# Configure OCI CLI
oci setup config

# Follow prompts to configure:
# - User OCID
# - Tenancy OCID
# - Region
# - Generate API key pair
```

## 🔧 Deployment Steps

### 1. Clone and Prepare
```bash
# Clone repository
git clone https://github.com/SynergiaOS/Cerebrosso.git
cd Cerebrosso

# Prepare Oracle Cloud configuration
cp infrastructure/.env.oracle-cloud infrastructure/.env
```

### 2. Configure API Keys
Edit `infrastructure/.env` with your API keys:
```bash
# Required for Multi-RPC optimization
HELIUS_API_KEY=your_helius_api_key_here
QUICKNODE_API_KEY=your_quicknode_api_key_here
ALCHEMY_API_KEY=your_alchemy_api_key_here
GENESYS_API_KEY=your_genesys_api_key_here

# Webhook configuration
WEBHOOK_BASE_URL=https://your-domain.com
```

### 3. Deploy to Oracle Cloud
```bash
# Run automated deployment
./scripts/deploy-oracle-cloud.sh

# The script will:
# ✅ Validate OCI configuration
# ✅ Generate SSH keys
# ✅ Deploy infrastructure with Terraform
# ✅ Configure the instance
# ✅ Install and start services
# ✅ Setup SSL (optional)
```

### 4. Post-Deployment Configuration
```bash
# SSH to the instance
./scripts/deploy-oracle-cloud.sh ssh

# Configure webhooks
cd /opt/cerberus-phoenix
./scripts/setup-helius-webhooks.py

# Verify system status
curl https://your-domain.com/api/v1/optimization/status
```

## 📊 Expected Performance

### Oracle Cloud A1 Flex Performance
- **CPU Performance**: ARM Ampere Altra (excellent for Rust)
- **Memory Bandwidth**: High-performance DDR4
- **Network**: 10 Gbps (more than sufficient for HFT)
- **Storage**: NVMe SSD (low latency)

### Cerberus Phoenix Performance
- **API Response Time**: <45ms average
- **Trading Execution**: <100ms latency
- **Concurrent Connections**: 500+ supported
- **Uptime**: 99.9% with auto-restart

## 🔒 Security Configuration

### Firewall Rules
```bash
# Automatically configured by deployment script
Port 22   - SSH access
Port 80   - HTTP (redirects to HTTPS)
Port 443  - HTTPS (API and webhooks)
Port 3000 - Cerebro-BFF (internal)
Port 8090 - HFT-Ninja (internal)
```

### SSL Certificate
```bash
# Automatic SSL with Let's Encrypt
sudo certbot --nginx -d your-domain.com

# Certificate auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

### Security Hardening
```bash
# Fail2ban for SSH protection
sudo systemctl enable fail2ban

# UFW firewall
sudo ufw enable

# Automatic security updates
sudo apt install unattended-upgrades
```

## 📈 Monitoring & Alerting

### Access Points
- **API Health**: `https://your-domain.com/health`
- **Grafana Dashboard**: `https://your-domain.com/grafana/`
- **Prometheus Metrics**: `https://your-domain.com:9090`
- **System Status**: `https://your-domain.com/`

### Key Metrics to Monitor
```bash
# Multi-RPC optimization
curl https://your-domain.com/api/v1/rpc/performance

# API usage and costs
curl https://your-domain.com/api/v1/usage/report

# Cache performance
curl https://your-domain.com/api/v1/cache/stats

# Trading performance
curl https://your-domain.com/api/v1/optimization/status
```

## 🔄 Maintenance

### System Updates
```bash
# SSH to instance
./scripts/deploy-oracle-cloud.sh ssh

# Update system
sudo apt update && sudo apt upgrade -y

# Update Cerberus Phoenix
cd /opt/cerberus-phoenix
git pull origin main
sudo systemctl restart cerberus-phoenix
```

### Backup Strategy
```bash
# Database backup
docker exec cerberus-postgres pg_dump -U postgres cerberus > backup.sql

# Configuration backup
cp /opt/cerberus-phoenix/infrastructure/.env backup/

# Automated backup script
sudo crontab -e
# Add: 0 2 * * * /opt/cerberus-phoenix/scripts/backup-data.sh
```

### Log Management
```bash
# View system logs
sudo journalctl -u cerberus-phoenix -f

# View application logs
docker-compose -f infrastructure/docker-compose.oracle-cloud.yml logs -f

# Log rotation
sudo logrotate -f /etc/logrotate.d/docker-containers
```

## 🚨 Troubleshooting

### Common Issues

#### Instance Not Accessible
```bash
# Check instance status in OCI Console
# Verify security list rules
# Check public IP assignment
```

#### Services Not Starting
```bash
# Check Docker status
sudo systemctl status docker

# Check service logs
sudo journalctl -u cerberus-phoenix -n 50

# Restart services
sudo systemctl restart cerberus-phoenix
```

#### High Memory Usage
```bash
# Check memory usage
free -h
docker stats

# Optimize Docker memory limits
# Edit docker-compose.oracle-cloud.yml
```

### Performance Optimization

#### ARM-Specific Optimizations
```bash
# Rust compilation for ARM
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

# Docker buildx for ARM
docker buildx create --use
docker buildx build --platform linux/arm64 .
```

#### Database Tuning
```sql
-- PostgreSQL optimization for ARM
ALTER SYSTEM SET shared_buffers = '2GB';
ALTER SYSTEM SET effective_cache_size = '6GB';
ALTER SYSTEM SET maintenance_work_mem = '512MB';
```

## 💡 Cost Optimization Tips

### Maximize Free Tier Benefits
1. **Use ARM instances**: Better price/performance ratio
2. **Optimize resource allocation**: Right-size containers
3. **Enable auto-scaling**: Scale down during low activity
4. **Monitor usage**: Stay within free tier limits

### Multi-RPC Cost Savings
1. **Configure all providers**: Maximize free tier usage
2. **Enable intelligent routing**: Automatic cost optimization
3. **Monitor API usage**: Real-time cost tracking
4. **Set up alerts**: Prevent unexpected charges

## 📞 Support

### Oracle Cloud Support
- **Documentation**: https://docs.oracle.com/en-us/iaas/
- **Community**: https://cloudcustomerconnect.oracle.com/
- **Free Tier FAQ**: https://www.oracle.com/cloud/free/faq/

### Cerberus Phoenix Support
- **Issues**: [GitHub Issues](https://github.com/SynergiaOS/Cerebrosso/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SynergiaOS/Cerebrosso/discussions)
- **Email**: synergiaos@outlook.com

---

## 🎉 Success Metrics

After successful deployment, you should achieve:

- ✅ **$0/month infrastructure cost** (Oracle Cloud Free Tier)
- ✅ **$13-20/month API costs** (vs $93-140 single provider)
- ✅ **95%+ cost reduction** through Multi-RPC optimization
- ✅ **99.9% uptime** with automatic failover
- ✅ **<100ms trading latency** on ARM architecture
- ✅ **2.2M+ free API requests/month** across all providers

**🥷 Ready to hunt for alpha on Solana with maximum efficiency and minimal costs!**
