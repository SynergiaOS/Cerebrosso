//! 🔐 Vault Client - Secure Key Management for Cerberus Phoenix v2.0
//!
//! Provides secure storage and retrieval of API keys, private keys, and other sensitive data.

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug, error};

/// 🔐 Vault client for secure key management
pub struct VaultClient {
    client: Client,
    base_url: String,
    token: String,
}

/// 🔑 Secret data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretData {
    pub data: HashMap<String, String>,
}

/// 📝 Vault response structure
#[derive(Debug, Deserialize)]
struct VaultResponse {
    data: Option<SecretData>,
}

/// 🔒 Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub key_version: u32,
}

impl VaultClient {
    /// 🚀 Create new Vault client
    pub fn new(vault_url: &str, token: &str) -> Self {
        info!("🔐 Initializing Vault client for URL: {}", vault_url);
        
        Self {
            client: Client::new(),
            base_url: vault_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    /// 🔍 Check Vault health
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/v1/sys/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    debug!("✅ Vault health check passed");
                } else {
                    warn!("⚠️ Vault health check failed: {}", response.status());
                }
                Ok(is_healthy)
            }
            Err(e) => {
                error!("❌ Vault health check error: {}", e);
                Ok(false)
            }
        }
    }

    /// 💾 Store secret in Vault
    pub async fn store_secret(&self, path: &str, data: &HashMap<String, String>) -> Result<()> {
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        
        let payload = serde_json::json!({
            "data": data
        });

        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("💾 Secret stored successfully at path: {}", path);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Failed to store secret: {}", error_text);
            Err(anyhow!("Failed to store secret: {}", error_text))
        }
    }

    /// 🔍 Retrieve secret from Vault
    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        
        let response = self.client
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if response.status().is_success() {
            let vault_response: VaultResponse = response.json().await?;
            
            match vault_response.data {
                Some(secret_data) => {
                    debug!("🔍 Secret retrieved successfully from path: {}", path);
                    Ok(secret_data.data)
                }
                None => {
                    warn!("⚠️ No data found at path: {}", path);
                    Ok(HashMap::new())
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Failed to retrieve secret: {}", error_text);
            Err(anyhow!("Failed to retrieve secret: {}", error_text))
        }
    }

    /// 🔒 Encrypt data using Vault transit engine
    pub async fn encrypt(&self, key_name: &str, plaintext: &str) -> Result<EncryptedData> {
        let url = format!("{}/v1/transit/encrypt/{}", self.base_url, key_name);
        
        let payload = serde_json::json!({
            "plaintext": base64::encode(plaintext)
        });

        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            if let Some(ciphertext) = result["data"]["ciphertext"].as_str() {
                debug!("🔒 Data encrypted successfully with key: {}", key_name);
                Ok(EncryptedData {
                    ciphertext: ciphertext.to_string(),
                    key_version: result["data"]["key_version"].as_u64().unwrap_or(1) as u32,
                })
            } else {
                Err(anyhow!("Invalid encryption response"))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Failed to encrypt data: {}", error_text);
            Err(anyhow!("Failed to encrypt data: {}", error_text))
        }
    }

    /// 🔓 Decrypt data using Vault transit engine
    pub async fn decrypt(&self, key_name: &str, encrypted_data: &EncryptedData) -> Result<String> {
        let url = format!("{}/v1/transit/decrypt/{}", self.base_url, key_name);
        
        let payload = serde_json::json!({
            "ciphertext": encrypted_data.ciphertext
        });

        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            
            if let Some(plaintext_b64) = result["data"]["plaintext"].as_str() {
                let plaintext = base64::decode(plaintext_b64)?;
                let decrypted = String::from_utf8(plaintext)?;
                debug!("🔓 Data decrypted successfully with key: {}", key_name);
                Ok(decrypted)
            } else {
                Err(anyhow!("Invalid decryption response"))
            }
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Failed to decrypt data: {}", error_text);
            Err(anyhow!("Failed to decrypt data: {}", error_text))
        }
    }

    /// 🔑 Create encryption key in Vault
    pub async fn create_key(&self, key_name: &str) -> Result<()> {
        let url = format!("{}/v1/transit/keys/{}", self.base_url, key_name);
        
        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if response.status().is_success() {
            info!("🔑 Encryption key created: {}", key_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Failed to create key: {}", error_text);
            Err(anyhow!("Failed to create key: {}", error_text))
        }
    }

    /// 🗑️ Delete secret from Vault
    pub async fn delete_secret(&self, path: &str) -> Result<()> {
        let url = format!("{}/v1/secret/data/{}", self.base_url, path);
        
        let response = self.client
            .delete(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await?;

        if response.status().is_success() {
            info!("🗑️ Secret deleted successfully from path: {}", path);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("❌ Failed to delete secret: {}", error_text);
            Err(anyhow!("Failed to delete secret: {}", error_text))
        }
    }
}

/// 🛡️ Secure configuration manager using Vault
pub struct SecureConfig {
    vault_client: VaultClient,
}

impl SecureConfig {
    /// 🚀 Initialize secure configuration
    pub fn new(vault_client: VaultClient) -> Self {
        Self { vault_client }
    }

    /// 🔑 Get API key securely
    pub async fn get_api_key(&self, service: &str) -> Result<String> {
        let secrets = self.vault_client.get_secret(&format!("api_keys/{}", service)).await?;
        
        secrets.get("key")
            .cloned()
            .ok_or_else(|| anyhow!("API key not found for service: {}", service))
    }

    /// 💾 Store API key securely
    pub async fn store_api_key(&self, service: &str, api_key: &str) -> Result<()> {
        let mut data = HashMap::new();
        data.insert("key".to_string(), api_key.to_string());
        data.insert("service".to_string(), service.to_string());
        data.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());
        
        self.vault_client.store_secret(&format!("api_keys/{}", service), &data).await
    }

    /// 🔐 Get private key securely
    pub async fn get_private_key(&self, wallet_name: &str) -> Result<String> {
        let secrets = self.vault_client.get_secret(&format!("wallets/{}", wallet_name)).await?;
        
        secrets.get("private_key")
            .cloned()
            .ok_or_else(|| anyhow!("Private key not found for wallet: {}", wallet_name))
    }

    /// 💾 Store private key securely (encrypted)
    pub async fn store_private_key(&self, wallet_name: &str, private_key: &str) -> Result<()> {
        // Encrypt the private key before storing
        let encrypted = self.vault_client.encrypt("cerberus_master_key", private_key).await?;
        
        let mut data = HashMap::new();
        data.insert("private_key".to_string(), encrypted.ciphertext);
        data.insert("key_version".to_string(), encrypted.key_version.to_string());
        data.insert("wallet_name".to_string(), wallet_name.to_string());
        data.insert("created_at".to_string(), chrono::Utc::now().to_rfc3339());
        
        self.vault_client.store_secret(&format!("wallets/{}", wallet_name), &data).await
    }
}
