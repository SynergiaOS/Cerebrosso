//! 🔐 Secure Configuration Manager
//! 
//! Manages sensitive configuration using Infisical for secure storage

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error, instrument};
use async_trait::async_trait;

use crate::infisical_client::InfisicalClient;

/// 🔐 Secure Configuration for Cerberus Phoenix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfig {
    // 🔑 API Keys
    pub helius_api_key: String,
    pub quicknode_api_key: String,
    pub jito_api_key: String,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    
    // 💰 Wallet Configuration
    pub trading_wallet_private_key: String,
    pub fee_payer_private_key: String,
    pub backup_wallet_private_key: Option<String>,
    
    // 🌐 RPC Endpoints
    pub helius_rpc_url: String,
    pub quicknode_rpc_url: String,
    pub jito_rpc_url: String,
    
    // 🔐 Security Settings
    pub webhook_secret: String,
    pub jwt_secret: String,
    pub encryption_key: String,
    
    // 📊 External Services
    pub discord_webhook_url: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub slack_webhook_url: Option<String>,
    
    // 🎯 Trading Limits
    pub max_position_size_sol: f64,
    pub daily_loss_limit_sol: f64,
    pub emergency_stop_loss_percentage: f64,
}

/// 🔐 Secure Configuration Manager
pub struct SecureConfigManager {
    infisical_client: Arc<InfisicalClient>,
    cached_config: Arc<tokio::sync::RwLock<Option<SecureConfig>>>,
}

impl SecureConfigManager {
    /// 🚀 Initialize Secure Config Manager
    #[instrument(skip(infisical_client))]
    pub fn new(infisical_client: Arc<InfisicalClient>) -> Self {
        info!("🔐 Initializing Secure Configuration Manager");
        
        Self {
            infisical_client,
            cached_config: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// 🔐 Load Configuration from Infisical
    #[instrument(skip(self))]
    pub async fn load_config(&self) -> Result<SecureConfig> {
        info!("🔐 Loading secure configuration from Infisical");

        // Check if we have cached config
        {
            let cached = self.cached_config.read().await;
            if let Some(config) = cached.as_ref() {
                info!("✅ Using cached secure configuration");
                return Ok(config.clone());
            }
        }

        // Define all required secrets
        let required_secrets = vec![
            "HELIUS_API_KEY",
            "QUICKNODE_API_KEY", 
            "JITO_API_KEY",
            "TRADING_WALLET_PRIVATE_KEY",
            "FEE_PAYER_PRIVATE_KEY",
            "HELIUS_RPC_URL",
            "QUICKNODE_RPC_URL",
            "JITO_RPC_URL",
            "WEBHOOK_SECRET",
            "JWT_SECRET",
            "ENCRYPTION_KEY",
            "MAX_POSITION_SIZE_SOL",
            "DAILY_LOSS_LIMIT_SOL",
            "EMERGENCY_STOP_LOSS_PERCENTAGE",
        ];

        let optional_secrets = vec![
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "BACKUP_WALLET_PRIVATE_KEY",
            "DISCORD_WEBHOOK_URL",
            "TELEGRAM_BOT_TOKEN",
            "SLACK_WEBHOOK_URL",
        ];

        // Fetch all secrets
        let mut all_secrets = Vec::new();
        all_secrets.extend(required_secrets.iter());
        all_secrets.extend(optional_secrets.iter());

        let secrets = self.infisical_client.get_secrets(&all_secrets).await?;

        // Validate required secrets
        for &key in &required_secrets {
            if !secrets.contains_key(key) {
                return Err(anyhow!("Required secret '{}' not found in Infisical", key));
            }
        }

        // Build secure config
        let config = SecureConfig {
            // API Keys
            helius_api_key: secrets.get("HELIUS_API_KEY").unwrap().clone(),
            quicknode_api_key: secrets.get("QUICKNODE_API_KEY").unwrap().clone(),
            jito_api_key: secrets.get("JITO_API_KEY").unwrap().clone(),
            openai_api_key: secrets.get("OPENAI_API_KEY").cloned(),
            anthropic_api_key: secrets.get("ANTHROPIC_API_KEY").cloned(),
            
            // Wallet Configuration
            trading_wallet_private_key: secrets.get("TRADING_WALLET_PRIVATE_KEY").unwrap().clone(),
            fee_payer_private_key: secrets.get("FEE_PAYER_PRIVATE_KEY").unwrap().clone(),
            backup_wallet_private_key: secrets.get("BACKUP_WALLET_PRIVATE_KEY").cloned(),
            
            // RPC Endpoints
            helius_rpc_url: secrets.get("HELIUS_RPC_URL").unwrap().clone(),
            quicknode_rpc_url: secrets.get("QUICKNODE_RPC_URL").unwrap().clone(),
            jito_rpc_url: secrets.get("JITO_RPC_URL").unwrap().clone(),
            
            // Security Settings
            webhook_secret: secrets.get("WEBHOOK_SECRET").unwrap().clone(),
            jwt_secret: secrets.get("JWT_SECRET").unwrap().clone(),
            encryption_key: secrets.get("ENCRYPTION_KEY").unwrap().clone(),
            
            // External Services
            discord_webhook_url: secrets.get("DISCORD_WEBHOOK_URL").cloned(),
            telegram_bot_token: secrets.get("TELEGRAM_BOT_TOKEN").cloned(),
            slack_webhook_url: secrets.get("SLACK_WEBHOOK_URL").cloned(),
            
            // Trading Limits
            max_position_size_sol: secrets.get("MAX_POSITION_SIZE_SOL")
                .unwrap()
                .parse()
                .map_err(|e| anyhow!("Invalid MAX_POSITION_SIZE_SOL: {}", e))?,
            daily_loss_limit_sol: secrets.get("DAILY_LOSS_LIMIT_SOL")
                .unwrap()
                .parse()
                .map_err(|e| anyhow!("Invalid DAILY_LOSS_LIMIT_SOL: {}", e))?,
            emergency_stop_loss_percentage: secrets.get("EMERGENCY_STOP_LOSS_PERCENTAGE")
                .unwrap()
                .parse()
                .map_err(|e| anyhow!("Invalid EMERGENCY_STOP_LOSS_PERCENTAGE: {}", e))?,
        };

        // Cache the config
        {
            let mut cached = self.cached_config.write().await;
            *cached = Some(config.clone());
        }

        info!("✅ Secure configuration loaded successfully from Infisical");
        Ok(config)
    }

    /// 🔐 Get Specific Secret
    #[instrument(skip(self))]
    pub async fn get_secret(&self, key: &str) -> Result<String> {
        self.infisical_client.get_secret(key).await
    }

    /// 🔐 Refresh Configuration
    #[instrument(skip(self))]
    pub async fn refresh_config(&self) -> Result<SecureConfig> {
        info!("🔄 Refreshing secure configuration");
        
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
        info!("🔍 Validating secure configuration");
        
        let config = self.load_config().await?;
        let mut report = ValidationReport::new();

        // Validate API Keys
        report.check_api_key("HELIUS_API_KEY", &config.helius_api_key);
        report.check_api_key("QUICKNODE_API_KEY", &config.quicknode_api_key);
        report.check_api_key("JITO_API_KEY", &config.jito_api_key);

        // Validate Wallet Keys
        report.check_private_key("TRADING_WALLET_PRIVATE_KEY", &config.trading_wallet_private_key);
        report.check_private_key("FEE_PAYER_PRIVATE_KEY", &config.fee_payer_private_key);

        // Validate URLs
        report.check_url("HELIUS_RPC_URL", &config.helius_rpc_url);
        report.check_url("QUICKNODE_RPC_URL", &config.quicknode_rpc_url);
        report.check_url("JITO_RPC_URL", &config.jito_rpc_url);

        // Validate Trading Limits
        report.check_trading_limit("MAX_POSITION_SIZE_SOL", config.max_position_size_sol, 0.1, 100.0);
        report.check_trading_limit("DAILY_LOSS_LIMIT_SOL", config.daily_loss_limit_sol, 1.0, 1000.0);
        report.check_percentage("EMERGENCY_STOP_LOSS_PERCENTAGE", config.emergency_stop_loss_percentage, 5.0, 50.0);

        info!("✅ Configuration validation completed: {} issues found", report.issues.len());
        Ok(report)
    }

    /// 📊 Get Configuration Summary (without sensitive data)
    pub async fn get_config_summary(&self) -> Result<serde_json::Value> {
        let config = self.load_config().await?;
        
        Ok(serde_json::json!({
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
            }
        }))
    }
}

/// 🔍 Configuration Validation Report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub field: String,
    pub issue_type: IssueType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum IssueType {
    Missing,
    Invalid,
    Insecure,
    OutOfRange,
}

impl ValidationReport {
    fn new() -> Self {
        Self {
            issues: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn check_api_key(&mut self, field: &str, value: &str) {
        if value.is_empty() {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::Missing,
                message: "API key is empty".to_string(),
            });
        } else if value.len() < 20 {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::Invalid,
                message: "API key appears too short".to_string(),
            });
        }
    }

    fn check_private_key(&mut self, field: &str, value: &str) {
        if value.is_empty() {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::Missing,
                message: "Private key is empty".to_string(),
            });
        } else if value.len() < 32 {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::Invalid,
                message: "Private key appears too short".to_string(),
            });
        }
    }

    fn check_url(&mut self, field: &str, value: &str) {
        if value.is_empty() {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::Missing,
                message: "URL is empty".to_string(),
            });
        } else if !value.starts_with("http") {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::Invalid,
                message: "URL must start with http or https".to_string(),
            });
        }
    }

    fn check_trading_limit(&mut self, field: &str, value: f64, min: f64, max: f64) {
        if value < min || value > max {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::OutOfRange,
                message: format!("Value {} is outside safe range [{}, {}]", value, min, max),
            });
        }
    }

    fn check_percentage(&mut self, field: &str, value: f64, min: f64, max: f64) {
        if value < min || value > max {
            self.issues.push(ValidationIssue {
                field: field.to_string(),
                issue_type: IssueType::OutOfRange,
                message: format!("Percentage {} is outside safe range [{}%, {}%]", value, min, max),
            });
        }
    }

    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn get_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "is_valid": self.is_valid(),
            "total_issues": self.issues.len(),
            "total_warnings": self.warnings.len(),
            "issues": self.issues.iter().map(|issue| {
                serde_json::json!({
                    "field": issue.field,
                    "type": format!("{:?}", issue.issue_type),
                    "message": issue.message
                })
            }).collect::<Vec<_>>(),
            "warnings": self.warnings
        })
    }
}

/// 🔐 Secure Configuration Trait Implementation
#[async_trait]
impl crate::SecureConfigTrait for SecureConfigManager {
    async fn get_config_summary(&self) -> Result<serde_json::Value> {
        self.get_config_summary().await
    }

    async fn validate_config(&self) -> Result<ValidationReport> {
        self.validate_config().await
    }

    async fn load_config(&self) -> Result<SecureConfig> {
        self.load_config().await
    }
}
