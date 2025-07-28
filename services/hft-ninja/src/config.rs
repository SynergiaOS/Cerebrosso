//! 🔧 Konfiguracja HFT-Ninja

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub solana: SolanaConfig,
    pub jito: JitoConfig,
    pub strategies: StrategiesConfig,
    pub risk: RiskConfig,
    pub sniper: SniperConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperConfig {
    pub signal_weights: HashMap<String, f64>,
    pub min_volume_usd: f64,
    pub min_liquidity_usd: f64,
    pub max_risk_score: f64,
    pub min_opportunity_score: f64,
    pub top_signals_count: usize,
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
            sniper: SniperConfig {
                signal_weights: Self::default_signal_weights(),
                min_volume_usd: env::var("SNIPER_MIN_VOLUME_USD")
                    .unwrap_or_else(|_| "1000.0".to_string())
                    .parse()
                    .unwrap_or(1000.0),
                min_liquidity_usd: env::var("SNIPER_MIN_LIQUIDITY_USD")
                    .unwrap_or_else(|_| "5000.0".to_string())
                    .parse()
                    .unwrap_or(5000.0),
                max_risk_score: env::var("SNIPER_MAX_RISK_SCORE")
                    .unwrap_or_else(|_| "0.75".to_string())
                    .parse()
                    .unwrap_or(0.75),
                min_opportunity_score: env::var("SNIPER_MIN_OPPORTUNITY_SCORE")
                    .unwrap_or_else(|_| "0.6".to_string())
                    .parse()
                    .unwrap_or(0.6),
                top_signals_count: env::var("SNIPER_TOP_SIGNALS_COUNT")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
            },
        })
    }

    /// Default signal weights based on research and experience
    fn default_signal_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();

        // High-confidence signals (based on memecoin research)
        weights.insert("low_dev_allocation".to_string(), 0.9);
        weights.insert("no_freeze_function".to_string(), 0.8);
        weights.insert("high_liquidity".to_string(), 0.7);
        weights.insert("verified_contract".to_string(), 0.8);
        weights.insert("doxxed_team".to_string(), 0.6);

        // Volume and activity signals
        weights.insert("volume_spike".to_string(), 0.7);
        weights.insert("price_momentum".to_string(), 0.6);
        weights.insert("whale_activity".to_string(), 0.5);
        weights.insert("social_sentiment".to_string(), 0.4);

        // Risk signals (negative weights)
        weights.insert("high_volatility".to_string(), -0.3);
        weights.insert("low_holder_count".to_string(), -0.4);
        weights.insert("suspicious_metadata".to_string(), -0.8);
        weights.insert("rug_pull_indicators".to_string(), -0.9);

        // New listing signals
        weights.insert("new_listing".to_string(), 0.5);
        weights.insert("pump_fun_listing".to_string(), 0.6);

        weights
    }
}
