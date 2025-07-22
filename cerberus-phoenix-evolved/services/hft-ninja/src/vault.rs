//! 🔐 Vault integration for secure secret management

use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{error, info};

pub struct VaultClient {
    client: vaultrs::client::VaultClient,
    mount_path: String,
}

impl VaultClient {
    pub fn new(vault_url: &str, token: &str, mount_path: &str) -> Result<Self> {
        let client = vaultrs::client::VaultClient::new(
            vaultrs::client::VaultClientSettingsBuilder::default()
                .address(vault_url)
                .token(token)
                .build()?,
        )?;

        Ok(Self {
            client,
            mount_path: mount_path.to_string(),
        })
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, Value>> {
        info!("🔐 Fetching secret from Vault: {}/{}", self.mount_path, path);

        match vaultrs::kv2::read(&self.client, &self.mount_path, path).await {
            Ok(secret) => {
                info!("✅ Secret retrieved successfully");
                Ok(secret)
            }
            Err(e) => {
                error!("❌ Failed to retrieve secret: {}", e);
                Err(anyhow!("Vault error: {}", e))
            }
        }
    }

    pub async fn store_secret(
        &self,
        path: &str,
        data: &HashMap<String, Value>,
    ) -> Result<()> {
        info!("🔐 Storing secret in Vault: {}/{}", self.mount_path, path);

        match vaultrs::kv2::set(&self.client, &self.mount_path, path, data).await {
            Ok(_) => {
                info!("✅ Secret stored successfully");
                Ok(())
            }
            Err(e) => {
                error!("❌ Failed to store secret: {}", e);
                Err(anyhow!("Vault error: {}", e))
            }
        }
    }

    pub async fn get_solana_keypair(&self) -> Result<String> {
        let secrets = self.get_secret("solana/keypair").await?;
        
        let private_key = secrets
            .get("private_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Private key not found in Vault"))?;

        Ok(private_key.to_string())
    }

    pub async fn health_check(&self) -> Result<bool> {
        match vaultrs::sys::health(&self.client).await {
            Ok(health) => Ok(health.initialized && !health.sealed),
            Err(e) => {
                error!("❌ Vault health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
