//! 💰 Secure Wallet Manager
//! 
//! Manages Solana wallets with private keys stored securely in Infisical

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error, instrument};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use bs58;

use crate::secure_config::SecureConfigManager;
use crate::SecureConfigTrait;

/// 💰 Wallet Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletType {
    Trading,
    FeePayer,
    Backup,
}

/// 💰 Wallet Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub wallet_type: WalletType,
    pub public_key: String,
    pub balance_sol: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 💰 Transaction Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub signature: String,
    pub success: bool,
    pub error: Option<String>,
    pub slot: Option<u64>,
    pub confirmation_status: String,
}

/// 💰 Secure Wallet Manager
pub struct WalletManager {
    config_manager: Arc<dyn SecureConfigTrait>,
    trading_keypair: Arc<tokio::sync::RwLock<Option<Keypair>>>,
    fee_payer_keypair: Arc<tokio::sync::RwLock<Option<Keypair>>>,
    backup_keypair: Arc<tokio::sync::RwLock<Option<Keypair>>>,
}

impl WalletManager {
    /// 🚀 Initialize Wallet Manager
    #[instrument(skip(config_manager))]
    pub fn new(config_manager: Arc<dyn SecureConfigTrait>) -> Self {
        info!("💰 Initializing Secure Wallet Manager");
        
        Self {
            config_manager,
            trading_keypair: Arc::new(tokio::sync::RwLock::new(None)),
            fee_payer_keypair: Arc::new(tokio::sync::RwLock::new(None)),
            backup_keypair: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// 🔐 Load All Wallets from Infisical
    #[instrument(skip(self))]
    pub async fn load_wallets(&self) -> Result<()> {
        info!("🔐 Loading wallets from secure storage");

        let config = self.config_manager.load_config().await?;

        // Load trading wallet
        let trading_keypair = self.keypair_from_private_key(&config.trading_wallet_private_key)?;
        {
            let mut keypair = self.trading_keypair.write().await;
            *keypair = Some(trading_keypair);
        }
        info!("✅ Trading wallet loaded: {}", self.get_trading_public_key().await?);

        // Load fee payer wallet
        let fee_payer_keypair = self.keypair_from_private_key(&config.fee_payer_private_key)?;
        {
            let mut keypair = self.fee_payer_keypair.write().await;
            *keypair = Some(fee_payer_keypair);
        }
        info!("✅ Fee payer wallet loaded: {}", self.get_fee_payer_public_key().await?);

        // Load backup wallet if available
        if let Some(backup_private_key) = &config.backup_wallet_private_key {
            let backup_keypair = self.keypair_from_private_key(backup_private_key)?;
            {
                let mut keypair = self.backup_keypair.write().await;
                *keypair = Some(backup_keypair);
            }
            info!("✅ Backup wallet loaded: {}", self.get_backup_public_key().await?.unwrap_or_default());
        }

        info!("✅ All wallets loaded successfully");
        Ok(())
    }

    /// 🔑 Get Trading Wallet Public Key
    pub async fn get_trading_public_key(&self) -> Result<String> {
        let keypair = self.trading_keypair.read().await;
        match keypair.as_ref() {
            Some(kp) => Ok(kp.pubkey().to_string()),
            None => Err(anyhow!("Trading wallet not loaded")),
        }
    }

    /// 🔑 Get Fee Payer Public Key
    pub async fn get_fee_payer_public_key(&self) -> Result<String> {
        let keypair = self.fee_payer_keypair.read().await;
        match keypair.as_ref() {
            Some(kp) => Ok(kp.pubkey().to_string()),
            None => Err(anyhow!("Fee payer wallet not loaded")),
        }
    }

    /// 🔑 Get Backup Wallet Public Key
    pub async fn get_backup_public_key(&self) -> Result<Option<String>> {
        let keypair = self.backup_keypair.read().await;
        Ok(keypair.as_ref().map(|kp| kp.pubkey().to_string()))
    }

    /// ✍️ Sign Transaction with Trading Wallet
    #[instrument(skip(self, transaction))]
    pub async fn sign_transaction_trading(&self, mut transaction: Transaction) -> Result<Transaction> {
        let keypair = self.trading_keypair.read().await;
        match keypair.as_ref() {
            Some(kp) => {
                transaction.sign(&[kp], transaction.message.recent_blockhash);
                info!("✅ Transaction signed with trading wallet");
                Ok(transaction)
            }
            None => Err(anyhow!("Trading wallet not loaded")),
        }
    }

    /// ✍️ Sign Transaction with Fee Payer
    #[instrument(skip(self, transaction))]
    pub async fn sign_transaction_fee_payer(&self, mut transaction: Transaction) -> Result<Transaction> {
        let keypair = self.fee_payer_keypair.read().await;
        match keypair.as_ref() {
            Some(kp) => {
                transaction.sign(&[kp], transaction.message.recent_blockhash);
                info!("✅ Transaction signed with fee payer wallet");
                Ok(transaction)
            }
            None => Err(anyhow!("Fee payer wallet not loaded")),
        }
    }

    /// ✍️ Sign Transaction with Multiple Wallets
    #[instrument(skip(self, transaction))]
    pub async fn sign_transaction_multi(&self, mut transaction: Transaction, wallet_types: Vec<WalletType>) -> Result<Transaction> {
        let mut signers = Vec::new();

        for wallet_type in wallet_types {
            match wallet_type {
                WalletType::Trading => {
                    let keypair = self.trading_keypair.read().await;
                    if let Some(kp) = keypair.as_ref() {
                        signers.push(kp);
                    } else {
                        return Err(anyhow!("Trading wallet not loaded"));
                    }
                }
                WalletType::FeePayer => {
                    let keypair = self.fee_payer_keypair.read().await;
                    if let Some(kp) = keypair.as_ref() {
                        signers.push(kp);
                    } else {
                        return Err(anyhow!("Fee payer wallet not loaded"));
                    }
                }
                WalletType::Backup => {
                    let keypair = self.backup_keypair.read().await;
                    if let Some(kp) = keypair.as_ref() {
                        signers.push(kp);
                    } else {
                        return Err(anyhow!("Backup wallet not loaded"));
                    }
                }
            }
        }

        if signers.is_empty() {
            return Err(anyhow!("No signers available"));
        }

        transaction.sign(&signers, transaction.message.recent_blockhash);
        info!("✅ Transaction signed with {} wallets", signers.len());
        Ok(transaction)
    }

    /// 📊 Get Wallet Information
    #[instrument(skip(self))]
    pub async fn get_wallet_info(&self, wallet_type: WalletType) -> Result<WalletInfo> {
        let public_key = match wallet_type {
            WalletType::Trading => self.get_trading_public_key().await?,
            WalletType::FeePayer => self.get_fee_payer_public_key().await?,
            WalletType::Backup => {
                self.get_backup_public_key().await?
                    .ok_or_else(|| anyhow!("Backup wallet not configured"))?
            }
        };

        // TODO: Fetch actual balance from RPC
        let balance_sol = 0.0; // Placeholder

        Ok(WalletInfo {
            wallet_type,
            public_key,
            balance_sol,
            last_updated: chrono::Utc::now(),
        })
    }

    /// 📊 Get All Wallet Information
    pub async fn get_all_wallet_info(&self) -> Result<Vec<WalletInfo>> {
        let mut wallets = Vec::new();

        // Trading wallet
        if let Ok(info) = self.get_wallet_info(WalletType::Trading).await {
            wallets.push(info);
        }

        // Fee payer wallet
        if let Ok(info) = self.get_wallet_info(WalletType::FeePayer).await {
            wallets.push(info);
        }

        // Backup wallet (if configured)
        if let Ok(info) = self.get_wallet_info(WalletType::Backup).await {
            wallets.push(info);
        }

        Ok(wallets)
    }

    /// 🔄 Rotate Wallet Keys (Emergency)
    #[instrument(skip(self))]
    pub async fn rotate_wallet_keys(&self, wallet_type: WalletType) -> Result<String> {
        warn!("🔄 Rotating wallet keys for {:?}", wallet_type);

        // Generate new keypair
        let new_keypair = Keypair::new();
        let new_public_key = new_keypair.pubkey().to_string();
        let new_private_key = bs58::encode(new_keypair.secret()).into_string();

        // Update in Infisical
        let secret_key = match wallet_type {
            WalletType::Trading => "TRADING_WALLET_PRIVATE_KEY",
            WalletType::FeePayer => "FEE_PAYER_PRIVATE_KEY", 
            WalletType::Backup => "BACKUP_WALLET_PRIVATE_KEY",
        };

        // TODO: Implement secret update in Infisical
        // This would require additional Infisical API endpoints

        // Update local keypair
        match wallet_type {
            WalletType::Trading => {
                let mut keypair = self.trading_keypair.write().await;
                *keypair = Some(new_keypair);
            }
            WalletType::FeePayer => {
                let mut keypair = self.fee_payer_keypair.write().await;
                *keypair = Some(new_keypair);
            }
            WalletType::Backup => {
                let mut keypair = self.backup_keypair.write().await;
                *keypair = Some(new_keypair);
            }
        }

        warn!("⚠️ Wallet keys rotated for {:?}. New public key: {}", wallet_type, new_public_key);
        Ok(new_public_key)
    }

    /// 🔐 Create Keypair from Private Key
    fn keypair_from_private_key(&self, private_key: &str) -> Result<Keypair> {
        let decoded = bs58::decode(private_key)
            .into_vec()
            .map_err(|e| anyhow!("Failed to decode private key: {}", e))?;

        if decoded.len() != 64 {
            return Err(anyhow!("Invalid private key length: expected 64 bytes, got {}", decoded.len()));
        }

        Keypair::from_bytes(&decoded)
            .map_err(|e| anyhow!("Failed to create keypair: {}", e))
    }

    /// 🔍 Validate Wallet Configuration
    #[instrument(skip(self))]
    pub async fn validate_wallets(&self) -> Result<WalletValidationReport> {
        info!("🔍 Validating wallet configuration");

        let mut report = WalletValidationReport::new();

        // Check if wallets are loaded
        let trading_loaded = self.trading_keypair.read().await.is_some();
        let fee_payer_loaded = self.fee_payer_keypair.read().await.is_some();
        let backup_loaded = self.backup_keypair.read().await.is_some();

        report.trading_wallet_loaded = trading_loaded;
        report.fee_payer_loaded = fee_payer_loaded;
        report.backup_wallet_loaded = backup_loaded;

        if !trading_loaded {
            report.issues.push("Trading wallet not loaded".to_string());
        }

        if !fee_payer_loaded {
            report.issues.push("Fee payer wallet not loaded".to_string());
        }

        // Check for key duplication
        if trading_loaded && fee_payer_loaded {
            let trading_pubkey = self.get_trading_public_key().await?;
            let fee_payer_pubkey = self.get_fee_payer_public_key().await?;
            
            if trading_pubkey == fee_payer_pubkey {
                report.issues.push("Trading and fee payer wallets are the same".to_string());
            }
        }

        report.is_valid = report.issues.is_empty();
        info!("✅ Wallet validation completed: {} issues found", report.issues.len());
        Ok(report)
    }

    /// 📊 Get Wallet Summary
    pub async fn get_wallet_summary(&self) -> serde_json::Value {
        let trading_loaded = self.trading_keypair.read().await.is_some();
        let fee_payer_loaded = self.fee_payer_keypair.read().await.is_some();
        let backup_loaded = self.backup_keypair.read().await.is_some();

        let trading_pubkey = if trading_loaded {
            self.get_trading_public_key().await.ok()
        } else {
            None
        };

        let fee_payer_pubkey = if fee_payer_loaded {
            self.get_fee_payer_public_key().await.ok()
        } else {
            None
        };

        let backup_pubkey = if backup_loaded {
            self.get_backup_public_key().await.ok().flatten()
        } else {
            None
        };

        serde_json::json!({
            "wallets_loaded": {
                "trading": trading_loaded,
                "fee_payer": fee_payer_loaded,
                "backup": backup_loaded
            },
            "public_keys": {
                "trading": trading_pubkey,
                "fee_payer": fee_payer_pubkey,
                "backup": backup_pubkey
            },
            "total_wallets": vec![trading_loaded, fee_payer_loaded, backup_loaded].iter().filter(|&&x| x).count()
        })
    }
}

/// 🔍 Wallet Validation Report
#[derive(Debug, Clone)]
pub struct WalletValidationReport {
    pub is_valid: bool,
    pub trading_wallet_loaded: bool,
    pub fee_payer_loaded: bool,
    pub backup_wallet_loaded: bool,
    pub issues: Vec<String>,
}

impl WalletValidationReport {
    fn new() -> Self {
        Self {
            is_valid: false,
            trading_wallet_loaded: false,
            fee_payer_loaded: false,
            backup_wallet_loaded: false,
            issues: Vec::new(),
        }
    }

    pub fn get_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "is_valid": self.is_valid,
            "wallets_status": {
                "trading": self.trading_wallet_loaded,
                "fee_payer": self.fee_payer_loaded,
                "backup": self.backup_wallet_loaded
            },
            "issues": self.issues,
            "total_issues": self.issues.len()
        })
    }
}
