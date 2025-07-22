# 🐺 Projekt Cerberus Phoenix v2.0

**Autonomiczny, samodoskonalący się ekosystem do operacji on-chain na Solanie**

[![CI/CD](https://github.com/SynergiaOS/Cerebros/workflows/CI/badge.svg)](https://github.com/SynergiaOS/Cerebros/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🚀 One-Command Deploy

```bash
# Uruchomienie całego stosu na Oracle Cloud Free Tier
make deploy-cloud

# Uruchomienie lokalnie (development)
make dev

# Pełny restart systemu
make phoenix-restart
```

## 🏗️ Architektura Phoenix v2.0

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   🥷 HFT-Ninja   │◄──►│  🧠 Cerebro-BFF  │◄──►│  🖥️ Dashboard   │
│   (Rust Core)   │    │   (Rust/Axum)   │    │ (React/Next.js) │
│                 │    │                 │    │                 │
│ • Jito Bundles  │    │ • AI Logic      │    │ • Real-time UI  │
│ • MEV Execution │    │ • Context Engine│    │ • Monitoring    │
│ • <100ms Latency│    │ • LLM Interface │    │ • Controls      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │  ⚙️ Kestra       │
                    │  (Orchestrator) │
                    │                 │
                    │ • Data Flows    │
                    │ • Scheduling    │
                    │ • Learning Loop │
                    └─────────────────┘
```

## 🎯 Kluczowe Komponenty

| Komponent | Technologia | Rola |
|-----------|-------------|------|
| **Infrastruktura** | Terraform (OCI) | Automatyczne tworzenie darmowej VM 4 OCPU / 24 GB RAM |
| **Orkiestracja** | Docker Compose | Uruchomienie całego stosu na jednej VM |
| **Obrazy Bazowe** | Wolfi + Apko | Ultrabezpieczne, SBOM-first, minimalistyczne obrazy |
| **Egzekutor** | hft-ninja (Rust) | Błyskawiczna egzekucja transakcji (Jito Bundles) |
| **Mózg & BFF** | cerebro-bff (Rust/Axum) | API, logika AI, orkiestracja agentów |
| **Modele LLM** | FinLlama + Deepseek-Math | Zewnętrzne serwery inferencyjne |
| **Pamięć AI** | Qdrant | Wysokowydajna baza wektorowa |
| **Orkiestrator** | Kestra | Zarządzanie przepływami i uczeniem się |
| **Strażnik API** | Traefik | Bezpieczna brama, automatyczne HTTPS |

## 📊 Cele Wydajnościowe

- **Daily ROI**: 5% (0.4 SOL z 8 SOL)
- **Execution Latency**: <100ms średnio, <200ms 99th percentile
- **Strategy Success**: >85% sandwich, >90% arbitrage
- **System Uptime**: >99.9%

## 🛠️ Szybki Start

### Wymagania
- Docker & Docker Compose
- Terraform (dla cloud deployment)
- Make
- Git

### Development Setup
```bash
git clone https://github.com/SynergiaOS/Cerebros.git
cd Cerebros
make dev-setup
make dev
```

### Production Deployment
```bash
make deploy-cloud
```

## 📁 Struktura Projektu

```
cerberus-phoenix/
├── 🚀 infrastructure/          # Infrastruktura jako kod
│   ├── terraform/             # Oracle Cloud Free Tier
│   ├── docker-compose.yml     # Orkiestracja całego stosu
│   ├── apko/                  # Manifesty ultralekkich obrazów
│   └── kestra/                # Definicje przepływów
└── 📦 services/
    ├── 🥷 hft-ninja/           # Rdzeń egzekucyjny (Rust)
    ├── 🧠 cerebro-bff/        # BFF i logika AI (Rust/Axum)
    └── 🖥️ dashboard/           # Interface użytkownika (React)
```

## 🔄 Przepływ End-to-End

1. **Zbieranie Danych**: Kestra → Oumi/Scrapy APIs → cerebro-bff
2. **Kontekstualizacja**: cerebro-bff → FinLlama embeddings → Qdrant
3. **Wykrycie Sygnału**: hft-ninja/Oumi → sygnał → cerebro-bff
4. **Decyzja AI**: cerebro-bff → Qdrant context + LLM → decyzja
5. **Wykonanie**: hft-ninja → Jito Bundle → blockchain
6. **Nauka**: wynik → cerebro-bff → aktualizacja kontekstu

## 🚨 Bezpieczeństwo

- **Wolfi Linux**: Minimalistyczny, bezpieczny OS
- **Apko**: Deklaratywne budowanie obrazów z SBOM
- **Vault Integration**: Zarządzanie sekretami
- **Circuit Breakers**: Automatyczne zatrzymanie przy stratach
- **Multi-layer Monitoring**: Prometheus + Grafana + Alerting

## 📈 Monitoring & Alerting

- **Grafana Dashboards**: Real-time performance metrics
- **Prometheus Metrics**: System i trading KPIs
- **Real-time Alerts**: Critical events i anomalie
- **P&L Tracking**: Szczegółowe śledzenie zysków/strat

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/SynergiaOS/Cerebros/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SynergiaOS/Cerebros/discussions)
- **Documentation**: [Wiki](https://github.com/SynergiaOS/Cerebros/wiki)

---

**🥷 Built with ❤️ for the Solana ecosystem**
