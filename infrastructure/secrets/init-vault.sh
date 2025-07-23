#!/bin/bash
# 🔐 Vault Initialization Script for Cerberus Phoenix v2.0

set -e
VAULT_ADDR="http://localhost:8200"

echo "🔐 Initializing HashiCorp Vault for Cerberus Phoenix v2.0..."

# Wait for Vault to be ready
until curl -s $VAULT_ADDR/v1/sys/health > /dev/null 2>&1; do
  echo "Waiting for Vault..."
  sleep 2
done

# Initialize Vault if not already done
if ! vault status 2>/dev/null | grep -q "Initialized.*true"; then
  echo "🚀 Initializing Vault..."
  vault operator init -key-shares=1 -key-threshold=1 > /tmp/vault-init.txt
  
  # Extract and unseal
  UNSEAL_KEY=$(grep 'Unseal Key 1:' /tmp/vault-init.txt | awk '{print $NF}')
  ROOT_TOKEN=$(grep 'Initial Root Token:' /tmp/vault-init.txt | awk '{print $NF}')
  
  vault operator unseal $UNSEAL_KEY
  export VAULT_TOKEN=$ROOT_TOKEN
  
  # Enable secrets engine
  vault secrets enable -path=solana kv-v2
  
  echo "✅ Vault initialized successfully!"
  echo "🔑 Root token: $ROOT_TOKEN"
else
  echo "✅ Vault already initialized"
fi
