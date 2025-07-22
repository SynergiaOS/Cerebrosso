# 🚀 Deployment Guide - Cerberus Phoenix v2.0

## 🎯 Quick Start (One-Command Deploy)

```bash
# 1. Sklonuj repozytorium
git clone https://github.com/SynergiaOS/Cerebros.git
cd Cerebros

# 2. Skopiuj i skonfiguruj zmienne środowiskowe
cp .env.example .env
# Edytuj .env i ustaw swoje klucze API

# 3. Uruchom cały stos lokalnie
make dev

# 4. Lub deploy na Oracle Cloud
make deploy-cloud
```

## 📋 Wymagania

### Lokalne Środowisko
- Docker & Docker Compose
- Make
- Git
- Node.js 18+ (dla developmentu dashboard)
- Rust 1.75+ (dla developmentu serwisów)

### Cloud Deployment
- Terraform
- Oracle Cloud Account (Free Tier)
- Klucze API OCI

## 🏗️ Architektura Deployment

```
┌─────────────────────────────────────────────────────────────┐
│                    Oracle Cloud VM                         │
│                  (4 OCPU, 24GB RAM)                       │
│                                                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Traefik   │  │  Dashboard  │  │ HFT-Ninja   │       │
│  │   (Proxy)   │  │ (React/Next)│  │   (Rust)    │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │ Cerebro-BFF │  │   Qdrant    │  │   Kestra    │       │
│  │   (Rust)    │  │ (Vector DB) │  │(Orchestrator)│       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                            │
│  ┌─────────────┐  ┌─────────────┐                        │
│  │ Prometheus  │  │   Grafana   │                        │
│  │ (Metrics)   │  │(Monitoring) │                        │
│  └─────────────┘  └─────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

## 🔧 Konfiguracja Środowiska

### 1. Zmienne Środowiskowe

Skopiuj `.env.example` jako `.env` i skonfiguruj:

```bash
# Kluczowe zmienne do ustawienia:
SOLANA_RPC_URL=https://api.devnet.solana.com
BIRDEYE_API_KEY=your_api_key
OUMI_API_KEY=your_api_key

# Dla Oracle Cloud:
TF_VAR_tenancy_ocid=ocid1.tenancy.oc1..xxx
TF_VAR_user_ocid=ocid1.user.oc1..xxx
TF_VAR_fingerprint=xx:xx:xx:xx
TF_VAR_private_key_path=~/.oci/oci_api_key.pem
TF_VAR_region=eu-frankfurt-1
TF_VAR_compartment_ocid=ocid1.compartment.oc1..xxx
TF_VAR_ssh_public_key="ssh-rsa AAAAB3..."
```

### 2. Oracle Cloud Setup

```bash
# Zainstaluj OCI CLI
curl -L https://raw.githubusercontent.com/oracle/oci-cli/master/scripts/install/install.sh | bash

# Skonfiguruj OCI
oci setup config

# Wygeneruj klucze SSH
ssh-keygen -t rsa -b 4096 -f ~/.ssh/cerberus_key
```

## 🚀 Deployment Options

### Option 1: Lokalne Środowisko (Development)

```bash
# Uruchom wszystkie serwisy lokalnie
make dev

# Sprawdź status
make status

# Zobacz logi
make logs

# Zatrzymaj
make dev-stop
```

**Dostępne endpointy:**
- Dashboard: http://localhost:3000
- Cerebro API: http://localhost:8080
- HFT-Ninja API: http://localhost:8081
- Grafana: http://localhost:3001
- Prometheus: http://localhost:9090
- Kestra: http://localhost:8082

### Option 2: Oracle Cloud Free Tier

```bash
# Deploy infrastruktury
make deploy-cloud

# Sprawdź status deployment
cd infrastructure/terraform
terraform output

# SSH do VM
ssh ubuntu@<PUBLIC_IP>

# Na VM uruchom serwisy
cd /opt/cerberus
make dev
```

### Option 3: Production Deployment

```bash
# Zbuduj obrazy produkcyjne
make build-apko

# Deploy z obrazami Apko
cd infrastructure
docker-compose -f docker-compose.yml up -d
```

## 📊 Monitoring & Observability

### Grafana Dashboards
- **URL**: http://localhost:3001 (local) lub http://CLOUD_IP:3001
- **Login**: admin / admin
- **Dashboards**: Automatycznie załadowane dla Cerberus

### Prometheus Metrics
- **URL**: http://localhost:9090
- **Targets**: Wszystkie serwisy automatycznie skonfigurowane
- **Retention**: 30 dni

### Kestra Workflows
- **URL**: http://localhost:8082
- **Flows**: Automatycznie załadowane z `infrastructure/kestra/flows/`

## 🔍 Troubleshooting

### Sprawdzenie Statusu Serwisów

```bash
# Status wszystkich kontenerów
make status

# Logi konkretnego serwisu
make logs-ninja    # HFT-Ninja
make logs-cerebro  # Cerebro-BFF

# Health check wszystkich serwisów
curl http://localhost:8080/health  # Cerebro-BFF
curl http://localhost:8081/health  # HFT-Ninja
curl http://localhost:3000/api/health  # Dashboard
```

### Częste Problemy

1. **Porty zajęte**
   ```bash
   # Sprawdź zajęte porty
   netstat -tulpn | grep :8080
   
   # Zatrzymaj konfliktujące serwisy
   make dev-stop
   ```

2. **Brak połączenia z Solana**
   ```bash
   # Sprawdź RPC endpoint
   curl -X POST -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
     https://api.devnet.solana.com
   ```

3. **Problemy z Docker**
   ```bash
   # Restart Docker
   sudo systemctl restart docker
   
   # Wyczyść cache
   make clean
   docker system prune -af
   ```

4. **Błędy kompilacji Rust**
   ```bash
   # Aktualizuj Rust
   rustup update
   
   # Wyczyść cache Cargo
   cd services/hft-ninja && cargo clean
   cd services/cerebro-bff && cargo clean
   ```

## 🔐 Security Considerations

### Produkcja
- Zmień domyślne hasła (Grafana, etc.)
- Skonfiguruj HTTPS z Let's Encrypt
- Ustaw firewall rules
- Używaj secrets management (Vault)
- Regularnie aktualizuj obrazy

### Development
- Nie commituj plików `.env`
- Używaj tylko testnet/devnet
- Ograniczaj dostęp do portów

## 📈 Scaling

### Horizontal Scaling
```bash
# Skaluj konkretny serwis
docker-compose up -d --scale hft-ninja=3

# Load balancing przez Traefik
# Automatycznie skonfigurowane
```

### Vertical Scaling
```bash
# Zwiększ zasoby w Oracle Cloud
cd infrastructure/terraform
# Edytuj main.tf - zwiększ OCPU/Memory
terraform apply
```

## 🔄 Updates & Maintenance

### Aktualizacja Systemu
```bash
# Pull najnowszych zmian
git pull origin main

# Restart z nowymi zmianami
make phoenix-restart
```

### Backup & Recovery
```bash
# Backup danych
docker-compose exec qdrant tar -czf /backup/qdrant-$(date +%Y%m%d).tar.gz /qdrant/storage
docker-compose exec postgres pg_dump -U kestra kestra > backup/kestra-$(date +%Y%m%d).sql

# Restore
# Procedury restore w dokumentacji operacyjnej
```

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/SynergiaOS/Cerebros/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SynergiaOS/Cerebros/discussions)
- **Documentation**: [Wiki](https://github.com/SynergiaOS/Cerebros/wiki)
