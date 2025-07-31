//! 🔐 Mock Infisical Client for Development
//! 
//! Provides a mock implementation of Infisical for testing and development

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};

/// 🔐 Mock Infisical Client
pub struct MockInfisicalClient {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl MockInfisicalClient {
    /// 🚀 Initialize Mock Infisical Client
    #[instrument]
    pub fn new() -> Self {
        info!("🔐 Initializing Mock Infisical Client for development");
        
        let mut secrets = HashMap::new();
        
        // Pre-populate with development secrets
        secrets.insert("HELIUS_API_KEY".to_string(), "dev_helius_key_12345".to_string());
        secrets.insert("QUICKNODE_API_KEY".to_string(), "dev_quicknode_key_67890".to_string());
        secrets.insert("JITO_API_KEY".to_string(), "dev_jito_key_abcdef".to_string());
        
        // Mock wallet keys (DO NOT USE IN PRODUCTION)
        secrets.insert("TRADING_WALLET_PRIVATE_KEY".to_string(), 
            "5J1F7GHaLNjmQBbTjm8pZAae5Qu5cocnCyc7b8Ep6XVdwdybRdqFS5BXzSuuXiZ9XK2unHNBxhVVy9lEbVDNzKJP".to_string());
        secrets.insert("FEE_PAYER_PRIVATE_KEY".to_string(), 
            "5J2G8HbMOkjnRCcUkn9qZBbf6Rv6dpdrDzd9cEq7YWEexezcSfqGT6CYtVvYjA0XL3voiICyxiWWz0mFcWEOzLKQ".to_string());
        
        // RPC URLs
        secrets.insert("HELIUS_RPC_URL".to_string(), "https://mainnet.helius-rpc.com".to_string());
        secrets.insert("QUICKNODE_RPC_URL".to_string(), "https://api.mainnet-beta.solana.com".to_string());
        secrets.insert("JITO_RPC_URL".to_string(), "https://mainnet.block-engine.jito.wtf".to_string());
        
        // Security settings
        secrets.insert("WEBHOOK_SECRET".to_string(), "dev_webhook_secret_123456789".to_string());
        secrets.insert("JWT_SECRET".to_string(), "dev_jwt_secret_abcdefghijklmnopqrstuvwxyz".to_string());
        secrets.insert("ENCRYPTION_KEY".to_string(), "dev_encryption_key_0123456789abcdef0123456789abcdef".to_string());
        
        // Trading limits
        secrets.insert("MAX_POSITION_SIZE_SOL".to_string(), "1.0".to_string());
        secrets.insert("DAILY_LOSS_LIMIT_SOL".to_string(), "5.0".to_string());
        secrets.insert("EMERGENCY_STOP_LOSS_PERCENTAGE".to_string(), "15.0".to_string());
        
        // Optional services
        secrets.insert("DISCORD_WEBHOOK_URL".to_string(), "https://discord.com/api/webhooks/dev".to_string());
        
        Self {
            secrets: Arc::new(RwLock::new(secrets)),
        }
    }

    /// 🔐 Get Secret by Key
    #[instrument(skip(self))]
    pub async fn get_secret(&self, key: &str) -> Result<String> {
        let secrets = self.secrets.read().await;
        
        match secrets.get(key) {
            Some(value) => {
                info!("✅ Mock secret retrieved: {}", key);
                Ok(value.clone())
            }
            None => {
                warn!("⚠️ Mock secret not found: {}", key);
                Err(anyhow::anyhow!("Secret '{}' not found in mock store", key))
            }
        }
    }

    /// 🔐 Get Multiple Secrets
    #[instrument(skip(self))]
    pub async fn get_secrets(&self, keys: &[&str]) -> Result<HashMap<String, String>> {
        let secrets = self.secrets.read().await;
        let mut results = HashMap::new();
        
        for &key in keys {
            if let Some(value) = secrets.get(key) {
                results.insert(key.to_string(), value.clone());
            }
        }
        
        info!("✅ Retrieved {}/{} mock secrets", results.len(), keys.len());
        Ok(results)
    }

    /// 🔐 Get All Secrets
    #[instrument(skip(self))]
    pub async fn get_all_secrets(&self) -> Result<HashMap<String, String>> {
        let secrets = self.secrets.read().await;
        info!("✅ Retrieved all {} mock secrets", secrets.len());
        Ok(secrets.clone())
    }

    /// 📊 Get Cache Statistics (mock)
    pub async fn get_cache_stats(&self) -> serde_json::Value {
        let secrets = self.secrets.read().await;
        
        serde_json::json!({
            "total_cached_secrets": secrets.len(),
            "expired_secrets": 0,
            "valid_secrets": secrets.len(),
            "cache_ttl_seconds": 300,
            "mock_mode": true
        })
    }

    /// 🧹 Cleanup Cache (mock - no-op)
    pub async fn cleanup_cache(&self) {
        info!("🧹 Mock cache cleanup (no-op)");
    }

    /// 🔐 Set Secret (for testing)
    #[instrument(skip(self))]
    pub async fn set_secret(&self, key: &str, value: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;
        secrets.insert(key.to_string(), value.to_string());
        info!("✅ Mock secret set: {}", key);
        Ok(())
    }

    /// 🗑️ Delete Secret (for testing)
    #[instrument(skip(self))]
    pub async fn delete_secret(&self, key: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;
        if secrets.remove(key).is_some() {
            info!("✅ Mock secret deleted: {}", key);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Secret '{}' not found", key))
        }
    }

    /// 📋 List All Secret Keys
    pub async fn list_secret_keys(&self) -> Vec<String> {
        let secrets = self.secrets.read().await;
        secrets.keys().cloned().collect()
    }

    /// 📊 Get Mock Statistics
    pub async fn get_mock_stats(&self) -> serde_json::Value {
        let secrets = self.secrets.read().await;
        let keys: Vec<&String> = secrets.keys().collect();
        
        serde_json::json!({
            "mode": "mock",
            "total_secrets": secrets.len(),
            "secret_keys": keys,
            "warning": "This is a mock implementation for development only",
            "security_notice": "DO NOT USE THESE KEYS IN PRODUCTION"
        })
    }
}

impl Default for MockInfisicalClient {
    fn default() -> Self {
        Self::new()
    }
}
