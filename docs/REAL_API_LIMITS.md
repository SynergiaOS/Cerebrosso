# 📊 REAL API Limits - FREE Providers Only

## 🎯 PRAWDZIWE LIMITY (Zweryfikowane)

### **🌟 Helius API Pro - FREE TIER**
```bash
Monthly Limit: 100,000 requests (NOT 1M!)
RPM Limit: 10 requests/minute
Cost: FREE
Enhanced Data: YES (metadata, rug detection)
Webhooks: YES
Networks: Mainnet, Devnet
```

**Prawdziwy URL:**
- Mainnet: `https://api.helius.xyz/v1/rpc`
- Devnet: `https://api.helius.xyz/v1/rpc?cluster=devnet`

### **🔮 Alchemy - FREE TIER**
```bash
Monthly Limit: 100,000 requests
RPM Limit: NONE
Cost: FREE
Enhanced Data: NO (basic RPC only)
Webhooks: NO
Networks: Mainnet, Devnet
```

**Prawdziwy URL:**
- Mainnet: `https://solana-mainnet.g.alchemy.com/v2/Wu2Kqfk_50kW_Zs4ifjuf3c7afxLOs7R`
- Devnet: `https://solana-devnet.g.alchemy.com/v2/Wu2Kqfk_50kW_Zs4ifjuf3c7afxLOs7R`

### **🌐 Public Solana RPC - FREE**
```bash
Monthly Limit: UNLIMITED
RPM Limit: ~100 (rate limited)
Cost: FREE
Enhanced Data: NO
Webhooks: NO
Networks: Mainnet, Devnet
```

**Prawdziwy URL:**
- Mainnet: `https://api.mainnet-beta.solana.com`
- Devnet: `https://api.devnet.solana.com`

## ❌ PŁATNE DOSTAWCY (USUNIĘTE)

### **QuickNode - PŁATNY!**
```bash
FREE Tier: BRAK (tylko trial)
Minimum Plan: $9/month
NIE UŻYWAMY - za drogi!
```

### **Genesys - NIEPEWNY**
```bash
Status: Niejasne czy naprawdę darmowy
Dokumentacja: Słaba
NIE UŻYWAMY - ryzykowny
```

## 📊 ŁĄCZNE LIMITY DARMOWE

```bash
Helius FREE:     100,000 requests/month
Alchemy FREE:    100,000 requests/month  
Public RPC:      UNLIMITED (rate limited)
─────────────────────────────────────────
TOTAL FREE:      200,000+ requests/month
```

## 💰 PRAWDZIWE OSZCZĘDNOŚCI

### **Przed Multi-RPC (single provider):**
```bash
Helius Paid Plan: $99/month (1M requests)
LUB
Alchemy Paid Plan: $199/month (1M requests)
```

### **Po Multi-RPC (FREE providers):**
```bash
Helius FREE: $0/month (100k requests)
Alchemy FREE: $0/month (100k requests)
Public RPC: $0/month (unlimited)
─────────────────────────────────────
TOTAL: $0/month
OSZCZĘDNOŚCI: $99-199/month (100%!)
```

## 🎯 STRATEGIA OPTYMALIZACJI

### **1. Intelligent Routing**
```bash
1. Helius (enhanced data) - do analizy tokenów
2. Alchemy (no RPM limit) - do bulk requests
3. Public RPC (unlimited) - do basic queries
```

### **2. Request Distribution**
```bash
Helius: 30% (enhanced data requests)
Alchemy: 50% (bulk operations)
Public: 20% (basic queries)
```

### **3. Failover Strategy**
```bash
Primary: Helius (best data)
Secondary: Alchemy (reliable)
Fallback: Public RPC (always available)
```

## 🌐 DEVNET/MAINNET Support

### **Network Switching**
```rust
// Automatic network detection
let network = std::env::var("SOLANA_NETWORK")
    .unwrap_or_else(|_| "mainnet-beta".to_string());

match network.as_str() {
    "mainnet-beta" => use_mainnet_urls(),
    "devnet" => use_devnet_urls(),
    _ => use_mainnet_urls(), // default
}
```

### **Real Data Testing**
```bash
# Test MAINNET
SOLANA_NETWORK=mainnet-beta ./scripts/test-alchemy-api.sh

# Test DEVNET  
SOLANA_NETWORK=devnet ./scripts/test-alchemy-api.sh
```

## 🚨 WAŻNE UWAGI

### **1. Helius FREE Tier**
- ⚠️ **TYLKO 100k requests/month** (nie 1M!)
- ⚠️ **10 RPM limit** - bardzo restrykcyjny
- ✅ **Enhanced data** - najlepsze metadane
- ✅ **Webhooks** - real-time notifications

### **2. Alchemy FREE Tier**
- ✅ **100k requests/month**
- ✅ **No RPM limit** - można wysyłać szybko
- ❌ **No enhanced data** - tylko basic RPC
- ❌ **No webhooks**

### **3. Public RPC**
- ✅ **Unlimited requests**
- ⚠️ **Rate limited** - ~100 RPM
- ❌ **No enhanced data**
- ❌ **No webhooks**
- ⚠️ **Może być niestabilny**

## 🎯 REKOMENDACJE

### **Dla małego portfela (0.1 SOL):**
```bash
1. Używaj Helius do analizy ryzyka (enhanced data)
2. Używaj Alchemy do bulk operations (no RPM limit)
3. Używaj Public RPC do basic queries
4. Włącz webhooks Helius dla real-time data
```

### **Monitoring usage:**
```bash
# Sprawdzaj usage co godzinę
curl http://localhost:3000/api/v1/usage/report

# Alerty przy 80% limitu
API_USAGE_ALERT_THRESHOLD=0.8
```

### **Backup strategy:**
```bash
# Jeśli Helius osiągnie limit RPM
→ Przełącz na Alchemy (no RPM limit)

# Jeśli Alchemy osiągnie monthly limit  
→ Przełącz na Public RPC (unlimited)

# Jeśli Public RPC jest slow
→ Wróć do Helius (po minucie)
```

## 🔄 IMPLEMENTACJA

### **Environment Variables**
```bash
# Network selection
SOLANA_NETWORK=mainnet-beta  # or devnet

# FREE providers only
HELIUS_API_KEY=your_key_here
ALCHEMY_API_KEY=Wu2Kqfk_50kW_Zs4ifjuf3c7afxLOs7R

# Limits (REAL values)
HELIUS_MONTHLY_LIMIT=100000
ALCHEMY_MONTHLY_LIMIT=100000
```

### **Multi-RPC Configuration**
```bash
# Only FREE providers
RPC_ROUTING_STRATEGY=cost_optimized
ENABLE_RPC_FAILOVER=true
TOTAL_FREE_REQUESTS_MONTHLY=200000
```

**🎯 RESULT: 100% FREE infrastructure z 200k+ requests/month!**
