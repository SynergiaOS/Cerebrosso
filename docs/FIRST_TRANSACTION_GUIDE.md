# 🚀 **PRZEWODNIK PIERWSZEJ TRANSAKCJI - CERBERUS PHOENIX v2.0**

## 🎯 **Przegląd**

Ten przewodnik przeprowadzi Cię przez pierwszą transakcję z wykorzystaniem **Cerberus Phoenix v2.0** z integracją **Helius API Pro** i **QuickNode Premium**.

## 🔧 **Wymagania Wstępne**

### **✅ API Keys (SKONFIGUROWANE)**
- **Helius API Pro**: `40a78e4c-bdd0-4338-877a-aa7d56a5f5a0`
- **QuickNode Premium**: `QN_5ca5bc11920f47e6892ed21e8c306a07`
- **QuickNode RPC**: `https://distinguished-blue-glade.solana-devnet.quiknode.pro/...`

### **✅ System Requirements**
- Docker & Docker Compose
- Rust & Cargo
- Minimum 4GB RAM
- Połączenie internetowe

## 🚀 **Krok 1: Uruchomienie Systemu**

```bash
# 🦈 Deploy complete Cerberus Phoenix v2.0
./deploy-cerberus-v2.sh

# ✅ Sprawdź status wszystkich serwisów
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

**Oczekiwane serwisy:**
- `cerberus-cerebro-bff` (port 8080)
- `cerberus-hft-ninja` (port 8090)
- `cerberus-qdrant` (port 6333)
- `cerberus-vault` (port 8200)
- `cerberus-grafana` (port 3001)
- `cerberus-prometheus` (port 9090)

## 🧪 **Krok 2: Testowanie Premium API**

```bash
# 🧪 Uruchom kompletne testy
./test-premium-api.sh
```

**Oczekiwane rezultaty:**
- ✅ Health check passed
- ✅ Helius API Pro connection successful
- ✅ QuickNode Premium connection successful
- ✅ Network latency <100ms (idealnie)

## 💰 **Krok 3: Przygotowanie Wallet**

### **Devnet Setup (BEZPIECZNE TESTOWANIE)**
```bash
# 🌊 Ustaw devnet
export SOLANA_RPC_URL=https://api.devnet.solana.com
export ENVIRONMENT=devnet

# 💧 Pobierz devnet SOL (darmowe)
solana airdrop 2

# ✅ Sprawdź balance
solana balance
```

### **Konfiguracja Wallet w Vault**
```bash
# 🔐 Dodaj keypair do Vault (opcjonalne)
vault kv put secret/solana/devnet private_key="your_base58_private_key"
```

## 🦈 **Krok 4: Pierwsza Transakcja - Piranha Strategy**

### **Test 1: Symulacja Decyzji**
```bash
# 🧠 Test decision engine z bezpiecznymi sygnałami
curl -X POST http://localhost:8080/api/v1/decision/test \
  -H "Content-Type: application/json" \
  -d '{
    "signals": [
      {
        "signal_type": "high_liquidity",
        "value": 0.8,
        "tf_idf_weight": 0.75,
        "importance_score": 0.8
      },
      {
        "signal_type": "team_verified",
        "value": 1.0,
        "tf_idf_weight": 0.87,
        "importance_score": 0.9
      },
      {
        "signal_type": "contract_verified",
        "value": 1.0,
        "tf_idf_weight": 0.82,
        "importance_score": 0.85
      }
    ]
  }'
```

**Oczekiwana odpowiedź:**
```json
{
  "action": "Execute",
  "confidence": 0.85,
  "risk_score": 0.15,
  "reasoning": [
    "📊 Rule: high_liquidity + team_verified → execute (confidence: 0.85)"
  ]
}
```

### **Test 2: Skanowanie Okazji**
```bash
# 🦈 Skanuj okazje tradingowe
curl -X POST http://localhost:8080/api/v1/piranha/scan \
  -H "Content-Type: application/json" \
  -d '{
    "max_opportunities": 3,
    "min_confidence": 0.7
  }'
```

### **Test 3: Pierwsza Transakcja (BARDZO MAŁA KWOTA)**
```bash
# 💰 Wykonaj pierwszą transakcję z minimalną kwotą
curl -X POST http://localhost:8080/api/v1/piranha/execute \
  -H "Content-Type: application/json" \
  -d '{
    "token_mint": "So11111111111111111111111111111111111111112",
    "amount_sol": 0.001,
    "strategy": "test_run",
    "max_slippage": 0.01,
    "use_jito": false,
    "dry_run": true
  }'
```

**Parametry bezpieczeństwa:**
- `amount_sol: 0.001` - Tylko 0.001 SOL (bardzo mała kwota)
- `dry_run: true` - Symulacja bez prawdziwej transakcji
- `use_jito: false` - Bez Jito Bundles dla pierwszego testu

## 📊 **Krok 5: Monitoring i Analiza**

### **Real-time Monitoring**
```bash
# 📊 Grafana Dashboard
open http://localhost:3001

# 📈 Prometheus Metrics
open http://localhost:9090

# 📝 Live logs
docker logs cerberus-cerebro-bff -f
```

### **Kluczowe Metryki do Śledzenia**
- **Execution Time**: <100ms dla HFT
- **Success Rate**: >85% dla strategii
- **Risk Score**: <0.3 dla bezpiecznych transakcji
- **Network Latency**: <200ms dla QuickNode

## 🛡️ **Krok 6: Bezpieczeństwo i Circuit Breakers**

### **Test Emergency Stop**
```bash
# 🚨 Test emergency circuit breaker
curl -X POST http://localhost:8080/api/v1/emergency/test \
  -H "Content-Type: application/json" \
  -d '{
    "rug_pull_score": 0.95,
    "confidence": 0.98
  }'
```

**Oczekiwana odpowiedź:**
```json
{
  "action": "EmergencyExit",
  "triggered": true,
  "reason": "High rug pull risk detected"
}
```

### **Sprawdzenie Position Limits**
```bash
# 💰 Test position sizing
curl -X POST http://localhost:8080/api/v1/piranha/position-size \
  -H "Content-Type: application/json" \
  -d '{
    "portfolio_size": 1.0,
    "risk_tolerance": 0.1,
    "expected_profit": 0.15
  }'
```

## 🎯 **Krok 7: Analiza Rezultatów**

### **Sprawdzenie TF-IDF Weights**
```bash
# 📊 Sprawdź wagi sygnałów
curl -X POST http://localhost:8080/api/v1/context/tfidf \
  -H "Content-Type: application/json" \
  -d '{
    "signals": [
      "rug_pull_detected",
      "team_verified", 
      "high_liquidity",
      "contract_verified"
    ]
  }'
```

**Oczekiwane wagi:**
- `rug_pull_detected`: 0.94 (najwyższa)
- `team_verified`: 0.87
- `contract_verified`: 0.82
- `high_liquidity`: 0.75

### **Performance Analysis**
```bash
# 📈 Sprawdź performance metrics
curl http://localhost:8080/api/v1/metrics/performance
```

## 🚀 **Krok 8: Przejście do Prawdziwego Tradingu**

### **Gdy Testy Przejdą Pomyślnie:**

1. **Zwiększ kwoty stopniowo:**
   ```bash
   # Start z 0.01 SOL
   amount_sol: 0.01
   
   # Potem 0.05 SOL
   amount_sol: 0.05
   
   # Maksymalnie 0.1 SOL (limit dla małego portfela)
   amount_sol: 0.1
   ```

2. **Włącz Jito Bundles:**
   ```bash
   use_jito: true
   ```

3. **Wyłącz dry_run:**
   ```bash
   dry_run: false
   ```

4. **Monitor w real-time:**
   ```bash
   # Śledź wszystkie transakcje
   docker logs cerberus-cerebro-bff -f | grep "TRANSACTION"
   ```

## 🚨 **Emergency Procedures**

### **Jeśli Coś Pójdzie Nie Tak:**

1. **Natychmiastowe zatrzymanie:**
   ```bash
   # 🛑 Stop wszystkich kontenerów
   docker stop $(docker ps -q --filter "name=cerberus")
   ```

2. **Emergency exit:**
   ```bash
   # 🚨 Emergency stop API
   curl -X POST http://localhost:8080/api/v1/emergency/stop
   ```

3. **Restart systemu:**
   ```bash
   # 🔄 Pełny restart
   ./deploy-cerberus-v2.sh
   ```

## 📋 **Checklist Pierwszej Transakcji**

- [ ] ✅ System uruchomiony i wszystkie serwisy działają
- [ ] ✅ Premium API (Helius + QuickNode) połączone
- [ ] ✅ Wallet skonfigurowany z devnet SOL
- [ ] ✅ Testy symulacji przeszły pomyślnie
- [ ] ✅ Emergency procedures przetestowane
- [ ] ✅ Monitoring włączony (Grafana + Prometheus)
- [ ] ✅ Pierwsza transakcja z dry_run=true
- [ ] ✅ Analiza rezultatów pozytywna
- [ ] ✅ Gotowy do prawdziwego tradingu

## 🎉 **Gratulacje!**

Jeśli wszystkie kroki przeszły pomyślnie, masz teraz działający **Cerberus Phoenix v2.0** z najzaawansowanszymi funkcjami AI trading na Solanie!

**🦈⚡ READY TO DOMINATE SOLANA TRADING! ⚡🦈**

---

## 📞 **Support**

W przypadku problemów:
1. Sprawdź logi: `docker logs cerberus-cerebro-bff`
2. Sprawdź status: `docker ps`
3. Restart: `./deploy-cerberus-v2.sh`
4. Test API: `./test-premium-api.sh`
