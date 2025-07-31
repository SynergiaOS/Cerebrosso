# 👑 Agent-Strateg (CEO) - Strategic Decision Making Agent

**Główny decydent i koordynator strategiczny w architekturze Hive Mind dla Cerberus Phoenix v3.0**

## 🎯 Przegląd

Agent-Strateg to CEO całego systemu Swarmagentic - najważniejszy agent odpowiedzialny za strategiczne podejmowanie decyzji, dekompozycję celów i koordynację działań innych agentów. Posiada 40% wagi w końcowych decyzjach systemu.

### ✨ Kluczowe Funkcje

- **🎯 Goal Decomposition** - Rozkład wysokopoziomowych celów na wykonalne zadania
- **📋 Task Delegation** - Inteligentne przydzielanie zadań wyspecjalizowanym agentom
- **🔬 Decision Synthesis** - Synteza końcowych decyzji z raportów wszystkich agentów
- **📈 Strategy Planning** - Długoterminowe planowanie strategii tradingowych
- **🛡️ Risk Management** - Zarządzanie ryzykiem i kontrola limitów
- **🧠 AI Orchestration** - Koordynacja wielu modeli AI (GPT-4, Claude-3, Llama3)

### 🎯 Rola w Hierarchii

```
👑 Agent-Strateg (CEO) - 40% wagi decyzyjnej
├── 🔬 Agent-Analityk - 25% wagi (analiza jakościowa)
├── 🧮 Agent-Quant - 30% wagi (analiza ilościowa)
└── 🛡️ Agent-Nadzorca - 5% wagi + veto power
```

## 🏗️ Architektura

### 🧠 Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    AGENT-STRATEG (CEO)                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │ Goal Decomposer │  │ Task Delegator  │                  │
│  │ 🎯 Rozkład      │  │ 📋 Przydzielanie│                  │
│  │    celów        │  │    zadań        │                  │
│  └─────────────────┘  └─────────────────┘                  │
│                                                             │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │Decision Synth.  │  │Strategy Planner │                  │
│  │ 🔬 Synteza      │  │ 📈 Planowanie   │                  │
│  │    decyzji      │  │    strategii    │                  │
│  └─────────────────┘  └─────────────────┘                  │
│                                                             │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │ AI Manager      │  │ Risk Manager    │                  │
│  │ 🧠 Modele AI    │  │ 🛡️ Zarządzanie  │                  │
│  │                 │  │    ryzykiem     │                  │
│  └─────────────────┘  └─────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### 🔄 Workflow Procesu Decyzyjnego

1. **📥 Otrzymanie Celu** - Cel wysokopoziomowy (np. "Przeanalizuj token XYZ")
2. **🎯 Goal Decomposition** - Rozkład na pod-cele:
   - Analiza danych rynkowych → Agent-Quant
   - Analiza sentymentu → Agent-Analityk  
   - Ocena bezpieczeństwa → Agent-Nadzorca
3. **📋 Task Delegation** - Przydzielenie zadań agentom
4. **⏳ Oczekiwanie na Raporty** - Zbieranie wyników od agentów
5. **🔬 Decision Synthesis** - Synteza końcowej decyzji
6. **✅ Wykonanie Decyzji** - Przekazanie do HFT-Ninja

## 🚀 Quick Start

### 1. Uruchomienie z Docker Compose

```bash
# Z głównego katalogu projektu
docker-compose up agent-strateg
```

### 2. Uruchomienie Development

```bash
cd services/agent-strateg

# Kopiuj konfigurację
cp .env.example .env

# Edytuj zmienne środowiskowe (szczególnie API keys)
nano .env

# Uruchom w trybie development
cargo run
```

### 3. Sprawdzenie Statusu

```bash
# Health check
curl http://localhost:8100/health

# Status agenta
curl http://localhost:8100/status

# Metryki wydajności
curl http://localhost:8100/metrics

# Lista aktywnych celów
curl http://localhost:8100/goals
```

## 📡 API Endpoints

### 🔍 Monitoring
- `GET /health` - Health check agenta
- `GET /status` - Status i stan agenta
- `GET /metrics` - Metryki wydajności

### 🎯 Zarządzanie Celami
- `GET /goals` - Lista aktywnych celów
- `POST /goals` - Utworzenie nowego celu
- `POST /goals/{id}/decompose` - Dekompozycja celu

### 🔬 Podejmowanie Decyzji
- `POST /decisions` - Synteza decyzji z raportów agentów
- `GET /decisions/history` - Historia decyzji

### 📈 Strategia
- `GET /strategies` - Lista strategii
- `POST /strategies` - Utworzenie nowej strategii

## 🔧 Konfiguracja

### Zmienne Środowiskowe

```bash
# Server
STRATEG_HOST=0.0.0.0
STRATEG_PORT=8100

# Swarm Communication
SWARM_COORDINATOR_URL=http://localhost:8090
STRATEG_AGENT_ID=strateg_1

# AI Models
STRATEG_PRIMARY_MODEL=gpt-4
STRATEG_BACKUP_MODEL=claude-3
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key

# Strategy Settings
STRATEG_DECISION_WEIGHT=0.4
STRATEG_RISK_TOLERANCE=0.3
STRATEG_MAX_GOALS=5

# Risk Management
STRATEG_MAX_POSITION_SOL=10.0
STRATEG_STOP_LOSS_PCT=0.05
STRATEG_TAKE_PROFIT_PCT=0.15
```

### Modele AI

Agent-Strateg używa hierarchii modeli AI:

1. **GPT-4** (Primary) - Strategiczne decyzje wysokiego poziomu
2. **Claude-3** (Backup) - Alternatywa dla GPT-4
3. **Llama3** (Local) - Lokalne operacje i fallback

## 🎯 Specjalizacje Agenta

### 🎯 Goal Decomposition
- **Input**: Cel wysokopoziomowy (np. "Analyze token ABC")
- **Process**: AI-powered dekompozycja na wykonalne zadania
- **Output**: Lista pod-celów z przypisanymi typami agentów

### 📋 Task Delegation
- **Strategie delegacji**:
  - `BestAvailable` - Najlepszy dostępny agent
  - `LoadBalanced` - Równomierne rozłożenie
  - `SpecializationBased` - Na podstawie specjalizacji
  - `PerformanceBased` - Na podstawie historycznej wydajności

### 🔬 Decision Synthesis
- **Weighted Decision Making** - Ważone głosowanie agentów
- **Confidence Thresholds** - Progi pewności decyzji
- **Risk Assessment Integration** - Uwzględnienie oceny ryzyka
- **Rationale Generation** - Generowanie uzasadnień

## 📊 Metryki & Monitoring

### Kluczowe Metryki

- **Decision Accuracy**: Procent poprawnych decyzji
- **Goal Completion Rate**: Wskaźnik ukończenia celów
- **Delegation Success Rate**: Skuteczność delegacji
- **Average Response Time**: Średni czas odpowiedzi
- **AI Model Performance**: Wydajność modeli AI

### Alerty

System generuje alerty gdy:
- Decision accuracy < 84.8%
- Goal completion rate < 90%
- Delegation success rate < 95%
- Response time > 5 sekund

## 🛡️ Risk Management

### Risk Levels
- **VeryLow** (0.1) - Minimalne ryzyko
- **Low** (0.3) - Niskie ryzyko
- **Medium** (0.5) - Średnie ryzyko
- **High** (0.7) - Wysokie ryzyko
- **VeryHigh** (0.9) - Bardzo wysokie ryzyko

### Risk Controls
- **Position Size Limits** - Maksymalny rozmiar pozycji
- **Stop Loss** - Automatyczne stop-loss orders
- **Daily Loss Limits** - Limity dziennych strat
- **Confidence Thresholds** - Minimalna pewność dla transakcji

## 🔄 Integration z Innymi Agentami

### Komunikacja
- **SwarmCoordinator** - Centralna rejestracja i koordynacja
- **Agent-Analityk** - Otrzymuje zadania analizy jakościowej
- **Agent-Quant** - Otrzymuje zadania analizy ilościowej
- **Agent-Nadzorca** - Otrzymuje zadania bezpieczeństwa

### Message Types
- `AgentRegistration` - Rejestracja w systemie
- `TaskAssignment` - Przydzielenie zadania
- `TaskResult` - Wynik zadania
- `CollaborationRequest` - Żądanie współpracy

## 🧪 Development

### Struktura Projektu

```
src/
├── lib.rs                  # Główna biblioteka
├── main.rs                 # Entry point
├── config.rs               # Konfiguracja
├── agent_strateg.rs        # Główny agent
├── goal_decomposition.rs   # Dekompozycja celów
├── task_delegation.rs      # Delegacja zadań
├── decision_synthesis.rs   # Synteza decyzji
├── strategy_planning.rs    # Planowanie strategii
├── swarm_communication.rs  # Komunikacja Swarm
├── ai_models.rs           # Zarządzanie AI
├── risk_management.rs     # Zarządzanie ryzykiem
└── metrics.rs             # Metryki
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

## 🔐 Bezpieczeństwo

- **API Key Management** - Bezpieczne przechowywanie kluczy AI
- **Input Validation** - Walidacja wszystkich danych wejściowych
- **Rate Limiting** - Ochrona przed nadużyciami
- **Audit Logging** - Szczegółowe logi decyzji
- **Risk Controls** - Wielopoziomowe kontrole ryzyka

## 📚 Przykłady Użycia

### Dekompozycja Celu

```json
POST /goals
{
  "title": "Analyze Token XYZ",
  "description": "Comprehensive analysis of new token XYZ",
  "priority": "High",
  "context": {
    "token_address": "So11111111111111111111111111111111111111112",
    "market_cap": 1000000,
    "volume_24h": 50000
  }
}
```

### Synteza Decyzji

```json
POST /decisions
{
  "agent_responses": [
    {
      "agent_type": "Analityk",
      "confidence": 0.85,
      "recommendation": "BUY",
      "reasoning": "Strong community sentiment"
    },
    {
      "agent_type": "Quant", 
      "confidence": 0.78,
      "recommendation": "BUY",
      "reasoning": "Favorable technical indicators"
    }
  ]
}
```

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Implement changes
4. Add comprehensive tests
5. Submit pull request

## 📄 License

MIT License - see [LICENSE](../../LICENSE) file for details.

---

**👑 Agent-Strateg - Leading the Hive Mind to Victory**
