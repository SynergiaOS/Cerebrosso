# 🔐 **PRZEWODNIK BEZPIECZEŃSTWA PRIVATE KEY - CERBERUS PHOENIX v2.0**

## 🚨 **KRYTYCZNE ZASADY BEZPIECZEŃSTWA**

### **❌ NIGDY NIE RÓB TEGO:**
- ❌ **Hardcode private key w kodzie**
- ❌ **Commit private key do Git**
- ❌ **Przechowuj w plain text files**
- ❌ **Wysyłaj przez email/chat**
- ❌ **Loguj private key w aplikacji**

### **✅ ZAWSZE RÓB TO:**
- ✅ **Używaj HashiCorp Vault**
- ✅ **Szyfruj w spoczynku (encryption at rest)**
- ✅ **Rotuj klucze regularnie**
- ✅ **Monitoruj dostęp**
- ✅ **Backup z szyfrowaniem**

---

## 🏗️ **ARCHITEKTURA BEZPIECZEŃSTWA**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HFT-Ninja     │    │  Cerebro-BFF    │    │   Vault Server  │
│                 │    │                 │    │                 │
│ 1. Request key  │───▶│ 2. Auth check   │───▶│ 3. Decrypt key  │
│ 4. Use for TX   │◀───│ 5. Return key   │◀───│ 6. Audit log    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

---

## 🔧 **1. KONFIGURACJA VAULT (POZIOM ENTERPRISE)**

### **A. Inicjalizacja Vault**
```bash
# 🔐 Inicjalizuj Vault (tylko raz!)
docker exec -it cerberus-vault vault operator init

# Zapisz BEZPIECZNIE:
# - 5 Unseal Keys
# - 1 Root Token
```

### **B. Unseal Vault**
```bash
# 🔓 Unseal Vault (po każdym restarcie)
docker exec -it cerberus-vault vault operator unseal <KEY1>
docker exec -it cerberus-vault vault operator unseal <KEY2>
docker exec -it cerberus-vault vault operator unseal <KEY3>
```

### **C. Konfiguracja Secrets Engine**
```bash
# 🗝️ Włącz KV secrets engine
docker exec -it cerberus-vault vault secrets enable -path=solana kv-v2

# 🔑 Stwórz policy dla HFT
docker exec -it cerberus-vault vault policy write hft-policy - <<EOF
path "solana/data/trading/*" {
  capabilities = ["read"]
}
path "solana/data/backup/*" {
  capabilities = ["read", "create", "update"]
}
EOF
```

---

## 🔑 **2. PRZECHOWYWANIE PRIVATE KEY**

### **A. Dodanie Private Key do Vault**
```bash
# 🔐 Dodaj private key (DEVNET - bezpieczne testowanie)
docker exec -it cerberus-vault vault kv put solana/trading/devnet \
  private_key="YOUR_BASE58_PRIVATE_KEY_HERE" \
  public_key="YOUR_PUBLIC_KEY_HERE" \
  network="devnet" \
  purpose="hft_trading" \
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# 🔐 Dodaj private key (MAINNET - produkcja)
docker exec -it cerberus-vault vault kv put solana/trading/mainnet \
  private_key="YOUR_MAINNET_PRIVATE_KEY" \
  public_key="YOUR_MAINNET_PUBLIC_KEY" \
  network="mainnet" \
  purpose="hft_trading" \
  max_daily_volume="1.0" \
  emergency_stop="true"
```

### **B. Weryfikacja Przechowywania**
```bash
# ✅ Sprawdź czy klucz jest bezpiecznie przechowywany
docker exec -it cerberus-vault vault kv get solana/trading/devnet

# ✅ Lista wszystkich kluczy
docker exec -it cerberus-vault vault kv list solana/trading/
```

---

## 🛡️ **3. BEZPIECZNE POBIERANIE KLUCZA W APLIKACJI**

### **A. Rust Code (HFT-Ninja)**
```rust
use reqwest;
use serde_json::Value;
use std::env;

pub struct VaultClient {
    base_url: String,
    token: String,
}

impl VaultClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(VaultClient {
            base_url: env::var("VAULT_URL")
                .unwrap_or_else(|_| "http://vault:8200".to_string()),
            token: env::var("VAULT_TOKEN")?,
        })
    }

    pub async fn get_private_key(&self, network: &str) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/solana/data/trading/{}", self.base_url, network);
        
        let response = reqwest::Client::new()
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Vault error: {}", response.status()).into());
        }

        let json: Value = response.json().await?;
        let private_key = json["data"]["data"]["private_key"]
            .as_str()
            .ok_or("Private key not found")?;

        // 🔒 NIGDY nie loguj private key!
        log::info!("Successfully retrieved private key for network: {}", network);
        
        Ok(private_key.to_string())
    }
}
```

### **B. Environment Variables (Bezpieczne)**
```bash
# 🔐 .env file (NIE commituj do Git!)
VAULT_URL=http://localhost:8200
VAULT_TOKEN=hvs.XXXXXXXXXXXXXXXXXXXXXXXX
SOLANA_NETWORK=devnet
```

---

## 🔄 **4. ROTACJA KLUCZY (SECURITY BEST PRACTICE)**

### **A. Automatyczna Rotacja**
```bash
# 🔄 Script rotacji kluczy (uruchamiaj co 30 dni)
#!/bin/bash
# rotate-keys.sh

OLD_KEY=$(docker exec -it cerberus-vault vault kv get -field=private_key solana/trading/devnet)
NEW_KEYPAIR=$(solana-keygen new --no-bip39-passphrase --silent --outfile /dev/stdout)

# Backup starego klucza
docker exec -it cerberus-vault vault kv put solana/backup/$(date +%Y%m%d) \
  old_private_key="$OLD_KEY" \
  rotated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Ustaw nowy klucz
docker exec -it cerberus-vault vault kv put solana/trading/devnet \
  private_key="$NEW_KEYPAIR" \
  rotated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

echo "✅ Key rotation completed successfully"
```

### **B. Emergency Key Rotation**
```bash
# 🚨 Emergency rotation (w przypadku kompromitacji)
#!/bin/bash
# emergency-rotate.sh

echo "🚨 EMERGENCY KEY ROTATION INITIATED"

# 1. Zatrzymaj trading
curl -X POST http://localhost:8080/api/v1/emergency/stop

# 2. Wygeneruj nowy klucz
NEW_KEYPAIR=$(solana-keygen new --no-bip39-passphrase --silent --outfile /dev/stdout)

# 3. Backup kompromitowanego klucza
docker exec -it cerberus-vault vault kv put solana/compromised/$(date +%Y%m%d_%H%M%S) \
  compromised_key="$(docker exec -it cerberus-vault vault kv get -field=private_key solana/trading/devnet)" \
  incident_time="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  reason="security_breach"

# 4. Ustaw nowy klucz
docker exec -it cerberus-vault vault kv put solana/trading/devnet \
  private_key="$NEW_KEYPAIR" \
  emergency_rotation="true" \
  rotated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# 5. Restart systemu z nowym kluczem
docker restart cerberus-hft-ninja cerberus-cerebro-bff

echo "✅ Emergency rotation completed. System restarted with new key."
```

---

## 📊 **5. MONITORING I AUDITING**

### **A. Vault Audit Logs**
```bash
# 📝 Włącz audit logging
docker exec -it cerberus-vault vault audit enable file file_path=/vault/logs/audit.log

# 📊 Sprawdź kto i kiedy dostępował klucze
docker exec -it cerberus-vault tail -f /vault/logs/audit.log | grep "solana/trading"
```

### **B. Monitoring Dostępu**
```bash
# 🔍 Script monitoringu dostępu
#!/bin/bash
# monitor-key-access.sh

while true; do
    # Sprawdź ostatnie dostępy do kluczy
    RECENT_ACCESS=$(docker exec -it cerberus-vault vault audit list -format=json | \
        jq '.[] | select(.path | contains("solana/trading"))')
    
    if [ ! -z "$RECENT_ACCESS" ]; then
        echo "🔍 Key access detected at $(date)"
        echo "$RECENT_ACCESS" | jq .
        
        # Wyślij alert (opcjonalnie)
        # curl -X POST "https://hooks.slack.com/..." -d "Key access detected"
    fi
    
    sleep 60
done
```

---

## 🚨 **6. EMERGENCY PROCEDURES**

### **A. Kompromitacja Klucza**
```bash
# 🚨 NATYCHMIASTOWE DZIAŁANIA:

# 1. STOP wszystkich transakcji
curl -X POST http://localhost:8080/api/v1/emergency/stop

# 2. Zmień klucz (patrz: Emergency Key Rotation)
./emergency-rotate.sh

# 3. Sprawdź transakcje na blockchain
solana transaction-history <COMPROMISED_PUBLIC_KEY>

# 4. Przenieś pozostałe środki na nowy wallet
solana transfer --from <OLD_KEYPAIR> <NEW_PUBLIC_KEY> ALL
```

### **B. Utrata Dostępu do Vault**
```bash
# 🔓 Recovery z Unseal Keys
docker exec -it cerberus-vault vault operator unseal <UNSEAL_KEY_1>
docker exec -it cerberus-vault vault operator unseal <UNSEAL_KEY_2>
docker exec -it cerberus-vault vault operator unseal <UNSEAL_KEY_3>

# 🔑 Recovery z Root Token
export VAULT_TOKEN=<ROOT_TOKEN>
docker exec -it cerberus-vault vault auth -method=token
```

---

## 💰 **7. STRATEGIA DLA MAŁEGO PORTFELA**

### **A. Multi-Wallet Strategy**
```bash
# 🎯 Strategia dla małego portfela (1-10 SOL)

# Wallet 1: Trading (80% środków)
docker exec -it cerberus-vault vault kv put solana/trading/main \
  private_key="<TRADING_KEY>" \
  purpose="daily_trading" \
  max_position="0.1" \
  stop_loss="0.05"

# Wallet 2: Emergency (20% środków)  
docker exec -it cerberus-vault vault kv put solana/emergency/backup \
  private_key="<BACKUP_KEY>" \
  purpose="emergency_funds" \
  access_restricted="true"
```

### **B. Position Limits**
```rust
// 💰 Limits dla małego portfela
pub struct PortfolioLimits {
    pub max_position_size: f64,    // 0.1 SOL max per trade
    pub daily_loss_limit: f64,     // 0.05 SOL max daily loss
    pub emergency_threshold: f64,  // 0.02 SOL emergency stop
}
```

---

## ✅ **8. CHECKLIST BEZPIECZEŃSTWA**

- [ ] ✅ Private key przechowywany w Vault
- [ ] ✅ Vault unseal keys bezpiecznie zapisane (offline)
- [ ] ✅ Root token bezpiecznie przechowywany
- [ ] ✅ Audit logging włączony
- [ ] ✅ Monitoring dostępu skonfigurowany
- [ ] ✅ Emergency procedures przetestowane
- [ ] ✅ Backup strategy zaimplementowana
- [ ] ✅ Key rotation schedule ustalony
- [ ] ✅ Position limits skonfigurowane
- [ ] ✅ Multi-wallet strategy wdrożona

**🔐 PAMIĘTAJ: Bezpieczeństwo private key to podstawa sukcesu w HFT trading! 🔐**
