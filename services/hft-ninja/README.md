# 🚀 HFT-Ninja v2.0 - Cerberus Phoenix Ultra-Low Latency Engine

## 🎯 **Strategia "Certainty-First HFT"**

HFT-Ninja v2.0 implementuje strategię "Certainty-First HFT" - głęboką przebudowę rdzenia egzekucyjnego systemu Cerberus Phoenix, skupiającą się na pewności wykonania transakcji przy minimalnych kosztach.

---

## 🏗️ **Architektura Modułów**

### 💰 **Fee & Tip Optimizer** ✅ ZAIMPLEMENTOWANY
**Lokalizacja:** `src/execution/fee_optimizer.rs`

**Funkcjonalność:**
- Dynamiczne obliczanie optymalnego tip dla Jito Bundli
- Analiza percentyli z Jito API (50th, 75th, 95th, 99th)
- Strategiczne mnożniki dla różnych typów transakcji
- Jitter (+/- 5%) dla uniknięcia przewidywalności
- Cache z TTL dla optymalizacji wydajności

**Kluczowe Metody:**
```rust
// Główna metoda optymalizacji
pub async fn get_optimal_jito_tip(
    &self,
    strategy: &str,
    amount_sol: f64,
    urgency_level: Option<u8>,
) -> Result<(u64, f64, u64, f64)>

// Pobieranie danych z Jito API
async fn fetch_jito_tip_data(&self) -> Result<JitoTipResponse>

// Obliczanie percentyla
fn calculate_percentile_tip(&self, jito_data: &JitoTipResponse) -> Result<u64>
```

**Strategiczne Mnożniki:**
- **Piranha Surf:** 1.5x (agresywny sniping)
- **Sandwich Arbitrage:** 2.0x (maksymalny MEV)
- **Cross-DEX Arbitrage:** 1.2x (umiarkowany)
- **Liquidity Snipe:** 1.8x (wysokie priorytety)
- **Emergency Exit:** 0.5x (minimalne koszty)

### 🌐 **Redundant RPC Broadcaster** 🔄 PLANOWANY
**Lokalizacja:** `src/rpc/`

**Planowana Funkcjonalność:**
- Pula połączeń do wielu dostawców (Helius, QuickNode, Alchemy)
- Równoległe wysyłanie transakcji
- Automatyczny failover przy awariach
- Load balancing z priorytetami

### 🎯 **Transaction Simulator & Backrunner** 🔄 PLANOWANY
**Lokalizacja:** `src/simulation/`

**Planowana Funkcjonalność:**
- Pre-execution simulation
- Automatic profit protection
- Advanced MEV strategies
- Jito ShredStream integration

---

## 🔧 **Konfiguracja**

### **Zmienne Środowiskowe**
```bash
# Server
HFT_NINJA_PORT=8090
RUST_LOG=info

# Jito Integration
JITO_URL=https://mainnet.block-engine.jito.wtf
JITO_TIP_STREAM_URL=https://bundles-api-rest.jito.wtf/api/v1/bundles/tip_floor

# RPC Providers
HELIUS_API_KEY=your_helius_api_key_here
QUICKNODE_URL=https://your-quicknode-endpoint.com
ALCHEMY_API_KEY=your_alchemy_api_key_here

# Fee Optimizer
BASE_TIP_LAMPORTS=10000
TIP_PERCENTILE_TARGET=0.8
TIP_JITTER_PERCENTAGE=0.05
```

### **Strategiczne Parametry**
- **Percentile Target:** 0.8 (80th percentile dla optymalnej inkluzji)
- **Jitter:** ±5% dla uniknięcia przewidywalności
- **Cache TTL:** 30 sekund dla świeżości danych
- **Max Tip:** 1,000,000 lamports (0.001 SOL)

---

## 🚀 **Uruchomienie**

### **Development**
```bash
# Kopiuj konfigurację
cp .env.example .env

# Edytuj zmienne środowiskowe
nano .env

# Uruchom w trybie development
cargo run
```

### **Production**
```bash
# Build release
cargo build --release

# Uruchom
./target/release/hft-ninja
```

---

## 📊 **API Endpoints**

### **Fee Optimization**
```http
POST /api/optimize-fee
Content-Type: application/json

{
  "strategy": "PiranhaSurf",
  "amount_sol": 1.0,
  "urgency_level": 8
}
```

**Response:**
```json
{
  "optimal_tip_lamports": 15000,
  "confidence_score": 0.85,
  "estimated_inclusion_time_ms": 1200,
  "strategy_multiplier": 1.5
}
```

### **Health Check**
```http
GET /health
```

### **Metrics**
```http
GET /api/metrics
```

---

## 🎯 **Strategia Implementacji**

### **Faza 1: Core Refactoring** ✅ W TRAKCIE
1. ✅ **Fee & Tip Optimizer** - Zaimplementowany
2. 🔄 **RPC Layer Rebuild** - Planowany
3. 🔄 **Transaction Engine Overhaul** - Planowany

### **Faza 2: AI Enhancement** 🔄 PLANOWANA
1. **Congestion Forecaster** - Predykcja przeciążenia sieci
2. **MEV Predictor** - Analiza ShredStream

### **Faza 3: Swarm Reorganization** 🔄 PLANOWANA
1. **Drone Specialization** - Specjalizacja strategii
2. **Orchestration Enhancement** - Koordynacja roju

---

## 📈 **Oczekiwane Rezultaty**

### **Performance Improvements**
- **Latency Reduction:** 40-60% przez optymalizację RPC
- **Success Rate:** 25-35% wzrost przez lepszą optymalizację opłat
- **Profit Margins:** 15-25% poprawa przez strategie MEV

### **Reliability Enhancements**
- **Uptime:** 99.9% przez redundantną architekturę
- **Error Recovery:** Automatyczny failover i retry
- **Monitoring:** Real-time performance tracking

---

## 🔥 **Status Operacji "Phoenix Evolved"**

**Faza:** 1/3 – Przebudowa Rdzenia Egzekucyjnego  
**Postęp:** Fee & Tip Optimizer ✅ GOTOWY  
**Następny:** Redundant RPC Broadcaster  

**Motto:** *"Nie szybkość, a pewność. Nie siła, a precyzja. Budujemy skalpel, nie młot."*
