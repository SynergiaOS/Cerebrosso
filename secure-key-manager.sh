#!/bin/bash
# 🔐 Secure Key Manager for Cerberus Phoenix v2.0
# Bezpieczne zarządzanie private keys z HashiCorp Vault

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}"
echo "🔐 SECURE KEY MANAGER - CERBERUS PHOENIX v2.0"
echo "=============================================="
echo -e "${NC}"

# Configuration
VAULT_CONTAINER="cerberus-vault"
VAULT_URL="http://localhost:8201"
VAULT_ADDR_INTERNAL="http://127.0.0.1:8200"

# Helper function for vault commands
vault_exec() {
    docker exec -e VAULT_ADDR="$VAULT_ADDR_INTERNAL" -e VAULT_TOKEN="cerberus-dev-token" "$VAULT_CONTAINER" vault "$@"
}

# Functions
check_vault_status() {
    echo -e "${BLUE}🔍 Checking Vault status...${NC}"
    
    if ! docker ps | grep -q "$VAULT_CONTAINER"; then
        echo -e "${RED}❌ Vault container is not running!${NC}"
        echo "Run: docker-compose up -d vault"
        exit 1
    fi
    
    # Check if Vault is sealed (skip for dev mode)
    VAULT_STATUS=$(vault_exec status -format=json 2>/dev/null || echo '{"sealed":false}')
    SEALED=$(echo $VAULT_STATUS | jq -r '.sealed // false')
    
    if [ "$SEALED" = "true" ]; then
        echo -e "${YELLOW}⚠️ Vault is sealed. You need to unseal it first.${NC}"
        echo "Run: ./secure-key-manager.sh unseal"
        return 1
    fi
    
    echo -e "${GREEN}✅ Vault is running and unsealed${NC}"
    return 0
}

init_vault() {
    echo -e "${YELLOW}🔐 Initializing Vault (ONLY RUN ONCE!)${NC}"
    echo -e "${RED}⚠️ SAVE THE OUTPUT SECURELY - YOU'LL NEED IT TO RECOVER!${NC}"
    
    read -p "Are you sure you want to initialize Vault? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Initialization cancelled."
        exit 0
    fi
    
    echo -e "${BLUE}Initializing Vault...${NC}"
    INIT_OUTPUT=$(docker exec $VAULT_CONTAINER vault operator init -format=json)
    
    echo -e "${GREEN}✅ Vault initialized successfully!${NC}"
    echo -e "${RED}🚨 SAVE THESE KEYS SECURELY (offline backup recommended):${NC}"
    echo "$INIT_OUTPUT" | jq -r '.unseal_keys_b64[]' | nl -v0 -s': Unseal Key ' -w1 -b'a'
    echo ""
    echo -e "${RED}Root Token: $(echo "$INIT_OUTPUT" | jq -r '.root_token')${NC}"
    echo ""
    echo -e "${YELLOW}📝 Save these keys in a secure location (NOT in this repository!)${NC}"
}

unseal_vault() {
    echo -e "${YELLOW}🔓 Unsealing Vault...${NC}"
    echo "You need 3 unseal keys to unseal the vault."
    
    for i in {1..3}; do
        echo -n "Enter unseal key $i: "
        read -s unseal_key
        echo ""
        
        docker exec $VAULT_CONTAINER vault operator unseal "$unseal_key" > /dev/null
        echo -e "${GREEN}✅ Unseal key $i accepted${NC}"
    done
    
    echo -e "${GREEN}🎉 Vault successfully unsealed!${NC}"
}

setup_secrets_engine() {
    echo -e "${BLUE}🗝️ Setting up secrets engine...${NC}"
    
    # Enable KV secrets engine for Solana keys
    vault_exec secrets enable -path=solana kv-v2 2>/dev/null || echo "Secrets engine already enabled"

    # Create policy for HFT trading
    cat > /tmp/hft-policy.hcl <<EOF
# Allow reading trading keys
path "solana/data/trading/*" {
  capabilities = ["read"]
}

# Allow backup operations
path "solana/data/backup/*" {
  capabilities = ["read", "create", "update"]
}

# Allow emergency operations
path "solana/data/emergency/*" {
  capabilities = ["read", "create", "update", "delete"]
}
EOF

    docker cp /tmp/hft-policy.hcl "$VAULT_CONTAINER":/tmp/hft-policy.hcl
    vault_exec policy write hft-policy /tmp/hft-policy.hcl
    rm -f /tmp/hft-policy.hcl
    
    echo -e "${GREEN}✅ Secrets engine and policies configured${NC}"
}

generate_keypair() {
    echo -e "${BLUE}🔑 Generating new Solana keypair...${NC}"
    
    # Generate new keypair
    KEYPAIR_OUTPUT=$(solana-keygen new --no-bip39-passphrase --silent --outfile /dev/stdout 2>/dev/null)
    PUBLIC_KEY=$(echo "$KEYPAIR_OUTPUT" | solana-keygen pubkey /dev/stdin 2>/dev/null)
    
    echo -e "${GREEN}✅ New keypair generated${NC}"
    echo -e "${BLUE}Public Key: $PUBLIC_KEY${NC}"
    
    return 0
}

store_key() {
    local network=$1
    local purpose=$2
    
    if [ -z "$network" ] || [ -z "$purpose" ]; then
        echo -e "${RED}❌ Usage: store_key <network> <purpose>${NC}"
        echo "Example: store_key devnet trading"
        return 1
    fi
    
    echo -e "${BLUE}🔐 Storing private key for $network ($purpose)...${NC}"
    
    # Generate new keypair
    echo "Generating new keypair..."
    TEMP_FILE=$(mktemp)
    solana-keygen new --no-bip39-passphrase --silent --force --outfile "$TEMP_FILE"

    PRIVATE_KEY=$(cat "$TEMP_FILE" | base58 -d | base58)
    PUBLIC_KEY=$(solana-keygen pubkey "$TEMP_FILE")

    # Clean up temp file immediately
    rm -f "$TEMP_FILE"

    # Store in Vault
    vault_exec kv put "solana/trading/$network" \
        private_key="$PRIVATE_KEY" \
        public_key="$PUBLIC_KEY" \
        network="$network" \
        purpose="$purpose" \
        created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        max_daily_volume="1.0" \
        emergency_stop="true"
    
    echo -e "${GREEN}✅ Private key stored securely in Vault${NC}"
    echo -e "${BLUE}Public Key: $PUBLIC_KEY${NC}"
    echo -e "${BLUE}Network: $network${NC}"
    echo -e "${BLUE}Purpose: $purpose${NC}"
    
    # Fund devnet wallet if it's devnet
    if [ "$network" = "devnet" ]; then
        echo -e "${YELLOW}💰 Funding devnet wallet...${NC}"
        solana airdrop 2 "$PUBLIC_KEY" --url https://api.devnet.solana.com || echo "Airdrop failed (rate limited?)"
    fi
}

retrieve_key() {
    local network=$1
    
    if [ -z "$network" ]; then
        echo -e "${RED}❌ Usage: retrieve_key <network>${NC}"
        echo "Example: retrieve_key devnet"
        return 1
    fi
    
    echo -e "${BLUE}🔍 Retrieving key information for $network...${NC}"
    
    KEY_DATA=$(vault_exec kv get -format=json "solana/trading/$network" 2>/dev/null)
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ No key found for network: $network${NC}"
        return 1
    fi
    
    PUBLIC_KEY=$(echo "$KEY_DATA" | jq -r '.data.data.public_key')
    CREATED_AT=$(echo "$KEY_DATA" | jq -r '.data.data.created_at')
    PURPOSE=$(echo "$KEY_DATA" | jq -r '.data.data.purpose')
    
    echo -e "${GREEN}✅ Key found${NC}"
    echo -e "${BLUE}Public Key: $PUBLIC_KEY${NC}"
    echo -e "${BLUE}Network: $network${NC}"
    echo -e "${BLUE}Purpose: $PURPOSE${NC}"
    echo -e "${BLUE}Created: $CREATED_AT${NC}"
    
    # Check balance
    if command -v solana &> /dev/null; then
        BALANCE=$(solana balance "$PUBLIC_KEY" --url "https://api.$network.solana.com" 2>/dev/null || echo "Unknown")
        echo -e "${BLUE}Balance: $BALANCE${NC}"
    fi
}

list_keys() {
    echo -e "${BLUE}📋 Listing all stored keys...${NC}"
    
    KEYS=$(vault_exec kv list -format=json solana/trading/ 2>/dev/null)
    
    if [ $? -ne 0 ]; then
        echo -e "${YELLOW}⚠️ No keys found or unable to access Vault${NC}"
        return 1
    fi
    
    echo "$KEYS" | jq -r '.[]' | while read -r network; do
        echo -e "${CYAN}🔑 Network: $network${NC}"
        retrieve_key "$network" | grep -E "(Public Key|Purpose|Created|Balance)" | sed 's/^/  /'
        echo ""
    done
}

rotate_key() {
    local network=$1
    
    if [ -z "$network" ]; then
        echo -e "${RED}❌ Usage: rotate_key <network>${NC}"
        echo "Example: rotate_key devnet"
        return 1
    fi
    
    echo -e "${YELLOW}🔄 Rotating key for $network...${NC}"
    echo -e "${RED}⚠️ This will replace the current key. Make sure no active trades are running!${NC}"
    
    read -p "Continue with key rotation? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Key rotation cancelled."
        return 0
    fi
    
    # Backup old key
    OLD_KEY_DATA=$(docker exec $VAULT_CONTAINER vault kv get -format=json "solana/trading/$network" 2>/dev/null)
    if [ $? -eq 0 ]; then
        BACKUP_PATH="solana/backup/$(date +%Y%m%d_%H%M%S)_$network"
        echo "$OLD_KEY_DATA" | jq '.data.data' | docker exec -i $VAULT_CONTAINER vault kv put "$BACKUP_PATH" -
        echo -e "${GREEN}✅ Old key backed up to: $BACKUP_PATH${NC}"
    fi
    
    # Generate and store new key
    store_key "$network" "trading"
    
    echo -e "${GREEN}🎉 Key rotation completed successfully!${NC}"
}

emergency_stop() {
    echo -e "${RED}🚨 EMERGENCY STOP INITIATED${NC}"

    # Stop trading via API
    curl -X POST http://localhost:8080/api/v1/emergency/stop 2>/dev/null || echo "API call failed"

    # Stop containers
    docker stop cerberus-hft-ninja cerberus-cerebro-bff 2>/dev/null || echo "Containers already stopped"

    echo -e "${RED}🛑 Emergency stop completed. All trading halted.${NC}"
}

sync_with_infisical() {
    echo -e "${BLUE}🔄 Syncing with Infisical...${NC}"

    # Check if Infisical is configured
    if [ ! -f ".infisical.json" ]; then
        echo -e "${YELLOW}⚠️ Infisical not configured. Run: make infisical-setup${NC}"
        return 1
    fi

    # Sync secrets
    ./scripts/infisical-sync.sh

    echo -e "${GREEN}✅ Infisical sync completed${NC}"
}

backup_to_infisical() {
    local network=$1

    if [ -z "$network" ]; then
        echo -e "${RED}❌ Usage: backup_to_infisical <network>${NC}"
        return 1
    fi

    echo -e "${BLUE}📤 Backing up key to Infisical...${NC}"

    # Get key from Vault
    KEY_DATA=$(docker exec $VAULT_CONTAINER vault kv get -format=json "solana/trading/$network" 2>/dev/null)

    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ No key found for network: $network${NC}"
        return 1
    fi

    PUBLIC_KEY=$(echo "$KEY_DATA" | jq -r '.data.data.public_key')
    PRIVATE_KEY=$(echo "$KEY_DATA" | jq -r '.data.data.private_key')

    # Store in Infisical
    if command -v infisical &> /dev/null; then
        infisical secrets set "SOLANA_PRIVATE_KEY_${network^^}" "$PRIVATE_KEY" --env=dev || echo "Infisical backup failed"
        infisical secrets set "SOLANA_PUBLIC_KEY_${network^^}" "$PUBLIC_KEY" --env=dev || echo "Infisical backup failed"
        echo -e "${GREEN}✅ Key backed up to Infisical${NC}"
    else
        echo -e "${YELLOW}⚠️ Infisical CLI not installed. Skipping backup.${NC}"
    fi
}

# Main menu
case "${1:-menu}" in
    "init")
        init_vault
        ;;
    "unseal")
        unseal_vault
        ;;
    "setup")
        check_vault_status && setup_secrets_engine
        ;;
    "store")
        check_vault_status && store_key "$2" "$3"
        ;;
    "get")
        check_vault_status && retrieve_key "$2"
        ;;
    "list")
        check_vault_status && list_keys
        ;;
    "rotate")
        check_vault_status && rotate_key "$2"
        ;;
    "emergency")
        emergency_stop
        ;;
    "sync")
        sync_with_infisical
        ;;
    "backup")
        check_vault_status && backup_to_infisical "$2"
        ;;
    "menu"|*)
        echo -e "${CYAN}🔐 Available commands:${NC}"
        echo ""
        echo -e "${YELLOW}Setup Commands:${NC}"
        echo "  init     - Initialize Vault (run only once!)"
        echo "  unseal   - Unseal Vault with unseal keys"
        echo "  setup    - Setup secrets engine and policies"
        echo ""
        echo -e "${YELLOW}Key Management:${NC}"
        echo "  store <network> <purpose> - Generate and store new key"
        echo "  get <network>            - Retrieve key information"
        echo "  list                     - List all stored keys"
        echo "  rotate <network>         - Rotate key for network"
        echo "  backup <network>         - Backup key to Infisical"
        echo ""
        echo -e "${YELLOW}Integration:${NC}"
        echo "  sync     - Sync with Infisical secrets"
        echo ""
        echo -e "${YELLOW}Emergency:${NC}"
        echo "  emergency - Emergency stop all trading"
        echo ""
        echo -e "${CYAN}Examples:${NC}"
        echo "  ./secure-key-manager.sh store devnet trading"
        echo "  ./secure-key-manager.sh get devnet"
        echo "  ./secure-key-manager.sh backup devnet"
        echo "  ./secure-key-manager.sh sync"
        ;;
esac
