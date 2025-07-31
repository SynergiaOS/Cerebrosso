//! 🔐 Mock Secure Configuration Manager
//! 
//! Provides mock secure configuration for development and testing

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, instrument};
use async_trait::async_trait;

use crate::mock_infisical::MockInfisicalClient;
use crate::secure_config::{SecureConfig, ValidationReport, ValidationIssue, IssueType};

/// 🔐 Mock Secure Configuration Manager
pub struct MockSecureConfigManager {
    mock_client: Arc<MockInfisicalClient>,
    cached_config: Arc<tokio::sync::RwLock<Option<SecureConfig>>>,
}

impl MockSecureConfigManager {
    /// 🚀 Initialize Mock Secure Config Manager
    #[instrument]
    pub fn new() -> Self {
        info!("🔐 Initializing Mock Secure Configuration Manager");
        
        Self {
            mock_client: Arc::new(MockInfisicalClient::new()),
            cached_config: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// 🔐 Load Configuration from Mock Store
    #[instrument(skip(self))]
    pub async fn load_config(&self) -> Result<SecureConfig> {
        info!("🔐 Loading mock secure configuration");

        // Check if we have cached config
        {
            let cached = self.cached_config.read().await;
            if let Some(config) = cached.as_ref() {
                info!("✅ Using cached mock configuration");
                return Ok(config.clone());
            }
        }

        // Load from mock client
        let secrets = self.mock_client.get_all_secrets().await?;

        // Build secure config
        let config = SecureConfig {
            // API Keys
            helius_api_key: secrets.get("HELIUS_API_KEY").unwrap_or(&"mock_helius_key".to_string()).clone(),
            quicknode_api_key: secrets.get("QUICKNODE_API_KEY").unwrap_or(&"mock_quicknode_key".to_string()).clone(),
            jito_api_key: secrets.get("JITO_API_KEY").unwrap_or(&"mock_jito_key".to_string()).clone(),
            openai_api_key: secrets.get("OPENAI_API_KEY").cloned(),
            anthropic_api_key: secrets.get("ANTHROPIC_API_KEY").cloned(),
            
            // Wallet Configuration
            trading_wallet_private_key: secrets.get("TRADING_WALLET_PRIVATE_KEY").unwrap_or(&"mock_trading_key".to_string()).clone(),
            fee_payer_private_key: secrets.get("FEE_PAYER_PRIVATE_KEY").unwrap_or(&"mock_fee_payer_key".to_string()).clone(),
            backup_wallet_private_key: secrets.get("BACKUP_WALLET_PRIVATE_KEY").cloned(),
            
            // RPC Endpoints
            helius_rpc_url: secrets.get("HELIUS_RPC_URL").unwrap_or(&"https://api.mainnet-beta.solana.com".to_string()).clone(),
            quicknode_rpc_url: secrets.get("QUICKNODE_RPC_URL").unwrap_or(&"https://api.mainnet-beta.solana.com".to_string()).clone(),
            jito_rpc_url: secrets.get("JITO_RPC_URL").unwrap_or(&"https://mainnet.block-engine.jito.wtf".to_string()).clone(),
            
            // Security Settings
            webhook_secret: secrets.get("WEBHOOK_SECRET").unwrap_or(&"mock_webhook_secret".to_string()).clone(),
            jwt_secret: secrets.get("JWT_SECRET").unwrap_or(&"mock_jwt_secret".to_string()).clone(),
            encryption_key: secrets.get("ENCRYPTION_KEY").unwrap_or(&"mock_encryption_key".to_string()).clone(),
            
            // External Services
            discord_webhook_url: secrets.get("DISCORD_WEBHOOK_URL").cloned(),
            telegram_bot_token: secrets.get("TELEGRAM_BOT_TOKEN").cloned(),
            slack_webhook_url: secrets.get("SLACK_WEBHOOK_URL").cloned(),
            
            // Trading Limits
            max_position_size_sol: secrets.get("MAX_POSITION_SIZE_SOL")
                .unwrap_or(&"1.0".to_string())
                .parse()
                .unwrap_or(1.0),
            daily_loss_limit_sol: secrets.get("DAILY_LOSS_LIMIT_SOL")
                .unwrap_or(&"5.0".to_string())
                .parse()
                .unwrap_or(5.0),
            emergency_stop_loss_percentage: secrets.get("EMERGENCY_STOP_LOSS_PERCENTAGE")
                .unwrap_or(&"15.0".to_string())
                .parse()
                .unwrap_or(15.0),
        };

        // Cache the config
        {
            let mut cached = self.cached_config.write().await;
            *cached = Some(config.clone());
        }

        info!("✅ Mock secure configuration loaded successfully");
        Ok(config)
    }

    /// 🔐 Get Specific Secret
    #[instrument(skip(self))]
    pub async fn get_secret(&self, key: &str) -> Result<String> {
        self.mock_client.get_secret(key).await
    }

    /// 🔐 Refresh Configuration
    #[instrument(skip(self))]
    pub async fn refresh_config(&self) -> Result<SecureConfig> {
        info!("🔄 Refreshing mock configuration");
        
        // Clear cache
        {
            let mut cached = self.cached_config.write().await;
            *cached = None;
        }

        // Reload config
        self.load_config().await
    }

    /// 🔐 Validate Configuration
    #[instrument(skip(self))]
    pub async fn validate_config(&self) -> Result<ValidationReport> {
        info!("🔍 Validating mock configuration");
        
        let config = self.load_config().await?;
        let mut report = ValidationReport {
            issues: Vec::new(),
            warnings: Vec::new(),
        };

        // Add mock validation warnings
        report.warnings.push("Using mock configuration for development".to_string());
        report.warnings.push("DO NOT use these keys in production".to_string());

        // Validate basic structure
        if config.helius_api_key.starts_with("mock_") {
            report.issues.push(ValidationIssue {
                field: "HELIUS_API_KEY".to_string(),
                issue_type: IssueType::Invalid,
                message: "Using mock API key".to_string(),
            });
        }

        if config.trading_wallet_private_key.starts_with("mock_") {
            report.issues.push(ValidationIssue {
                field: "TRADING_WALLET_PRIVATE_KEY".to_string(),
                issue_type: IssueType::Invalid,
                message: "Using mock private key".to_string(),
            });
        }

        // Trading limits validation
        if config.max_position_size_sol > 10.0 {
            report.issues.push(ValidationIssue {
                field: "MAX_POSITION_SIZE_SOL".to_string(),
                issue_type: IssueType::OutOfRange,
                message: "Position size too large for development".to_string(),
            });
        }

        info!("✅ Mock configuration validation completed: {} issues, {} warnings", 
              report.issues.len(), report.warnings.len());
        Ok(report)
    }

    /// 📊 Get Configuration Summary (without sensitive data)
    pub async fn get_config_summary(&self) -> Result<serde_json::Value> {
        let config = self.load_config().await?;
        
        Ok(serde_json::json!({
            "mode": "mock",
            "api_keys_configured": {
                "helius": !config.helius_api_key.is_empty(),
                "quicknode": !config.quicknode_api_key.is_empty(),
                "jito": !config.jito_api_key.is_empty(),
                "openai": config.openai_api_key.is_some(),
                "anthropic": config.anthropic_api_key.is_some(),
            },
            "wallets_configured": {
                "trading_wallet": !config.trading_wallet_private_key.is_empty(),
                "fee_payer": !config.fee_payer_private_key.is_empty(),
                "backup_wallet": config.backup_wallet_private_key.is_some(),
            },
            "external_services": {
                "discord": config.discord_webhook_url.is_some(),
                "telegram": config.telegram_bot_token.is_some(),
                "slack": config.slack_webhook_url.is_some(),
            },
            "trading_limits": {
                "max_position_size_sol": config.max_position_size_sol,
                "daily_loss_limit_sol": config.daily_loss_limit_sol,
                "emergency_stop_loss_percentage": config.emergency_stop_loss_percentage,
            },
            "warnings": [
                "This is a mock configuration for development only",
                "DO NOT use these credentials in production"
            ]
        }))
    }

    /// 🔧 Set Mock Secret (for testing)
    pub async fn set_mock_secret(&self, key: &str, value: &str) -> Result<()> {
        self.mock_client.set_secret(key, value).await?;
        
        // Clear cache to force reload
        {
            let mut cached = self.cached_config.write().await;
            *cached = None;
        }
        
        Ok(())
    }

    /// 📊 Get Mock Statistics
    pub async fn get_mock_stats(&self) -> serde_json::Value {
        self.mock_client.get_mock_stats().await
    }
}

impl Default for MockSecureConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 🔐 Secure Configuration Trait Implementation for Mock
#[async_trait]
impl crate::SecureConfigTrait for MockSecureConfigManager {
    async fn get_config_summary(&self) -> Result<serde_json::Value> {
        self.get_config_summary().await
    }

    async fn validate_config(&self) -> Result<crate::secure_config::ValidationReport> {
        self.validate_config().await
    }

    async fn load_config(&self) -> Result<crate::secure_config::SecureConfig> {
        self.load_config().await
    }
}
