# 🔐 Vault Configuration for Cerberus Phoenix v2.0

ui = true

storage "file" {
  path = "/vault/data"
}

listener "tcp" {
  address = "0.0.0.0:8200"
  tls_disable = true
}

# Development settings - DO NOT USE IN PRODUCTION
disable_mlock = true
api_addr = "http://0.0.0.0:8200"
cluster_addr = "http://0.0.0.0:8201"

# Enable KV secrets engine
path "secret/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

# Enable transit engine for encryption
path "transit/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
