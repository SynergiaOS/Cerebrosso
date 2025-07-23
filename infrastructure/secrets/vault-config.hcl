# 🔐 HashiCorp Vault Configuration for Cerberus Phoenix v2.0
ui = true

storage "file" {
  path = "/vault/data"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 1
}

disable_mlock = true
default_lease_ttl = "300s"
max_lease_ttl = "300s"
api_addr = "http://0.0.0.0:8200"
