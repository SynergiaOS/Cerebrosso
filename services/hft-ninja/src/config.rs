//! 🔧 Konfiguracja HFT-Ninja

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub solana: SolanaConfig,
    pub jito: JitoConfig,
    pub strategies: StrategiesConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoConfig {
    pub block_engine_url: String,
    pub tip_amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategiesConfig {
    pub enabled: Vec<String>,
    pub max_concurrent: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_position_size: f64,
    pub max_slippage: f64,
    pub stop_loss: f64,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Config {
            server: ServerConfig {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            },
            solana: SolanaConfig {
                rpc_url: env::var("SOLANA_RPC_URL")
                    .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
                commitment: env::var("SOLANA_COMMITMENT")
                    .unwrap_or_else(|_| "confirmed".to_string()),
            },
            jito: JitoConfig {
                block_engine_url: env::var("JITO_BLOCK_ENGINE_URL")
                    .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),
                tip_amount: env::var("JITO_TIP_AMOUNT")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()?,
            },
            strategies: StrategiesConfig {
                enabled: vec!["sandwich".to_string(), "arbitrage".to_string()],
                max_concurrent: 10,
            },
            risk: RiskConfig {
                max_position_size: 1.0,
                max_slippage: 0.01,
                stop_loss: 0.05,
            },
        })
    }
}
