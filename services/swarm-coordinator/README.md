# 🐝 SwarmCoordinator - Hive Mind Central Orchestrator

**Centralny orkiestrator architektury Hive Mind dla Cerberus Phoenix v3.0**

## 🎯 Przegląd

SwarmCoordinator to serce nowej architektury Swarmagentic - zaawansowany system zarządzania agentami AI, który przekształca Cerberus Phoenix w inteligentny ul z koordynacją hierarchiczną.

### ✨ Kluczowe Funkcje

- **🎯 Centralna Orkiestracja** - Zarządza wszystkimi agentami w systemie
- **📋 Inteligentna Delegacja Zadań** - Przydziela zadania najlepszym agentom
- **📡 Zaawansowana Komunikacja** - Real-time komunikacja między agentami
- **💾 System Pamięci** - Krótkoterminowa (Redis) i długoterminowa (Qdrant)
- **🔄 Pętla Uczenia** - Ciągłe doskonalenie na podstawie wyników
- **📊 Monitoring Wydajności** - Szczegółowe metryki i alerting

### 🎯 Cele Wydajnościowe

- **⚡ Latencja**: <100ms dla operacji krytycznych
- **🎯 Dokładność**: 84.8% accuracy w podejmowaniu decyzji
- **📈 Przepustowość**: Skalowanie od 4 do 40 agentów
- **🔄 Uczenie**: Ciągłe doskonalenie z feedback loop

## 🏗️ Architektura

### 🐝 Hive Mind Coordination Layer
```
┌─────────────────────────────────────────────────────────────┐
│                    SWARM COORDINATOR                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │Agent-Strateg│  │Agent-Analityk│  │Agent-Quant  │        │
│  │    (CEO)    │  │   (Qual)    │  │             │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                           │                                │
│  ┌─────────────┐         │         ┌─────────────┐        │
│  │Agent-Nadzorca│        │         │ Task Queue  │        │
│  │  (Guardian) │        │         │ & Metrics   │        │
│  └─────────────┘         │         └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### 🧠 Core Components

1. **AgentRegistry** - Zarządzanie agentami i ich statusami
2. **TaskDelegator** - Inteligentne przydzielanie zadań
3. **CommunicationHub** - Real-time komunikacja (WebSocket + Redis)
4. **MemoryStore** - Wielopoziomowy system pamięci
5. **FeedbackLoop** - Uczenie się z wyników
6. **MetricsCollector** - Monitoring i alerting

## 🚀 Quick Start

### 1. Uruchomienie z Docker Compose

```bash
# Z głównego katalogu projektu
docker-compose up swarm-coordinator
```

### 2. Uruchomienie Development

```bash
cd services/swarm-coordinator

# Kopiuj konfigurację
cp .env.example .env

# Edytuj zmienne środowiskowe
nano .env

# Uruchom w trybie development
cargo run
```

### 3. Sprawdzenie Statusu

```bash
# Health check
curl http://localhost:8090/health

# Status systemu
curl http://localhost:8090/status

# Metryki
curl http://localhost:8090/metrics
```

## 📡 API Endpoints

### 🔍 Monitoring
- `GET /health` - Health check
- `GET /status` - Status systemu Swarm
- `GET /metrics` - Metryki wydajności

### 🤖 Zarządzanie Agentami
- `GET /agents` - Lista agentów
- `POST /agents/register` - Rejestracja agenta
- `DELETE /agents/{id}` - Wyrejestrowanie agenta

### 📋 Zarządzanie Zadaniami
- `POST /tasks` - Delegacja zadania
- `GET /tasks/{id}` - Status zadania
- `POST /tasks/{id}/result` - Wynik zadania

### 🧠 System Pamięci
- `GET /memory/{key}` - Pobierz z pamięci
- `POST /memory` - Zapisz w pamięci
- `POST /memory/search` - Wyszukaj podobne

## 🔧 Konfiguracja

### Zmienne Środowiskowe

```bash
# Server
SWARM_HOST=0.0.0.0
SWARM_PORT=8090

# Redis (pamięć krótkoterminowa)
REDIS_URL=redis://localhost:6379

# Qdrant (pamięć długoterminowa)
QDRANT_URL=http://localhost:6333
QDRANT_COLLECTION=swarm_memory

# Swarm Settings
SWARM_MAX_AGENTS=40
SWARM_MIN_AGENTS=4
SWARM_DECISION_THRESHOLD=0.848

# Security
JWT_SECRET=your_secret_key
```

### Typy Agentów

1. **👑 Agent-Strateg (CEO)**
   - Główny decydent i koordynator
   - Waga decyzyjna: 40%
   - Max zadań: 10

2. **🔬 Agent-Analityk**
   - Analiza jakościowa i sentiment
   - Waga decyzyjna: 25%
   - Max zadań: 5

3. **🧮 Agent-Quant**
   - Analiza ilościowa i modelowanie
   - Waga decyzyjna: 30%
   - Max zadań: 8

4. **🛡️ Agent-Nadzorca (Guardian)**
   - Bezpieczeństwo i monitoring
   - Waga decyzyjna: 5% (+ veto power)
   - Max zadań: 3

## 📊 Monitoring & Metryki

### Kluczowe Metryki

- **Latencja**: Średni czas odpowiedzi systemu
- **Dokładność**: Procent poprawnych decyzji
- **Przepustowość**: Zadania/sekundę
- **Wykorzystanie**: CPU, pamięć, sieć
- **Sukces**: Wskaźnik udanych operacji

### Alerty

System automatycznie generuje alerty gdy:
- Latencja > 100ms
- Dokładność < 84.8%
- Wskaźnik sukcesu < 95%
- Agent nie odpowiada > 5s

## 🔄 Feedback Loop & Uczenie

### Proces Uczenia

1. **Zbieranie Danych** - Wyniki zadań, warunki rynkowe
2. **Analiza Wzorców** - Wykrywanie trendów i korelacji
3. **Aktualizacja Modeli** - Doskonalenie algorytmów
4. **Predykcje** - Przewidywanie wyników przyszłych zadań

### Typy Wzorców

- **SuccessPattern** - Warunki prowadzące do sukcesu
- **FailurePattern** - Przyczyny niepowodzeń
- **MarketPattern** - Wzorce rynkowe
- **AgentPerformancePattern** - Wydajność agentów
- **TemporalPattern** - Wzorce czasowe

## 🛠️ Development

### Struktura Projektu

```
src/
├── lib.rs              # Główna biblioteka
├── main.rs             # Entry point
├── config.rs           # Konfiguracja
├── swarm_coordinator.rs # Główny koordynator
├── agent_registry.rs   # Rejestr agentów
├── task_delegation.rs  # Delegacja zadań
├── communication.rs    # Hub komunikacyjny
├── memory_store.rs     # System pamięci
├── feedback_loop.rs    # Pętla uczenia
├── metrics.rs          # Metryki
└── agent_types.rs      # Typy agentów
```

### Testy

```bash
# Uruchom wszystkie testy
cargo test

# Testy z logami
cargo test -- --nocapture

# Testy wydajności
cargo bench
```

### Linting

```bash
# Sprawdź kod
cargo clippy

# Formatowanie
cargo fmt
```

## 🔐 Bezpieczeństwo

- **JWT Authentication** - Bezpieczna autoryzacja agentów
- **Rate Limiting** - Ochrona przed nadużyciami
- **Input Validation** - Walidacja wszystkich danych wejściowych
- **Secure Communication** - Szyfrowana komunikacja
- **Audit Logging** - Szczegółowe logi bezpieczeństwa

## 📈 Skalowanie

### Auto-Scaling

System automatycznie skaluje liczbę agentów na podstawie:
- Obciążenia systemu
- Długości kolejek zadań
- Dostępnych zasobów
- Celów wydajnościowych

### Load Balancing

- **Round Robin** - Równomierne rozłożenie zadań
- **Performance-Based** - Priorytet dla najlepszych agentów
- **Capability-Based** - Dopasowanie do możliwości agenta

## 🐛 Troubleshooting

### Częste Problemy

1. **Agent nie odpowiada**
   ```bash
   # Sprawdź logi
   docker logs swarm-coordinator
   
   # Sprawdź heartbeat
   curl http://localhost:8090/agents
   ```

2. **Wysokie opóźnienia**
   ```bash
   # Sprawdź metryki
   curl http://localhost:8090/metrics
   
   # Sprawdź Redis
   redis-cli ping
   ```

3. **Błędy pamięci**
   ```bash
   # Sprawdź Qdrant
   curl http://localhost:6333/collections
   
   # Sprawdź wykorzystanie
   docker stats swarm-coordinator
   ```

## 📚 Dokumentacja

- [API Reference](./docs/api.md)
- [Architecture Guide](./docs/architecture.md)
- [Deployment Guide](./docs/deployment.md)
- [Performance Tuning](./docs/performance.md)

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Make changes
4. Add tests
5. Submit pull request

## 📄 License

MIT License - see [LICENSE](../../LICENSE) file for details.

---

**🐝 SwarmCoordinator - Powering the Future of AI-Driven Trading**
