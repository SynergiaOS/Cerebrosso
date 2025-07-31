//! 🔧 Configuration module for HFT-Ninja v2.0

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::env;

/// 🔧 Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub jito_url: String,
    pub jito_tip_stream_url: String,
    pub rpc_providers: Vec<RpcProvider>,
    pub fee_optimizer: FeeOptimizerConfig,
    pub simulation: SimulationConfig,
}

/// 🌐 RPC Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcProvider {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub priority: u8,
    pub max_requests_per_second: u32,
    pub timeout_ms: u64,
}

/// 💰 Fee Optimizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeOptimizerConfig {
    pub base_tip_lamports: u64,
    pub percentile_target: f64, // 0.8 for 80th percentile
    pub jitter_percentage: f64, // 0.05 for ±5%
    pub strategy_multipliers: StrategyMultipliers,
    pub cache_ttl_seconds: u64,
    pub max_tip_lamports: u64,
}

/// 🎯 Strategy-specific multipliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMultipliers {
    pub piranha_surf: f64,
    pub sandwich_arbitrage: f64,
    pub cross_dex_arbitrage: f64,
    pub liquidity_snipe: f64,
    pub emergency_exit: f64,
}

/// 🎮 Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub enabled: bool,
    pub max_simulation_time_ms: u64,
    pub profit_threshold_percentage: f64,
}

impl Config {
    /// 🔧 Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file if present

        Ok(Config {
            port: env::var("HFT_NINJA_PORT")
                .unwrap_or_else(|_| "8090".to_string())
                .parse()?,
            
            jito_url: env::var("JITO_URL")
                .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),
            
            jito_tip_stream_url: env::var("JITO_TIP_STREAM_URL")
                .unwrap_or_else(|_| "https://bundles-api-rest.jito.wtf/api/v1/bundles/tip_floor".to_string()),
            
            rpc_providers: Self::load_rpc_providers()?,
            fee_optimizer: Self::load_fee_optimizer_config()?,
            simulation: Self::load_simulation_config()?,
        })
    }

    /// 🌐 Load RPC providers configuration
    fn load_rpc_providers() -> Result<Vec<RpcProvider>> {
        let mut providers = Vec::new();

        // Helius (Primary)
        if let Ok(helius_key) = env::var("HELIUS_API_KEY") {
            providers.push(RpcProvider {
                name: "Helius".to_string(),
                url: "https://mainnet.helius-rpc.com".to_string(),
                api_key: Some(helius_key),
                priority: 1,
                max_requests_per_second: 100,
                timeout_ms: 5000,
            });
        }

        // QuickNode (Secondary)
        if let Ok(quicknode_url) = env::var("QUICKNODE_URL") {
            providers.push(RpcProvider {
                name: "QuickNode".to_string(),
                url: quicknode_url,
                api_key: None,
                priority: 2,
                max_requests_per_second: 50,
                timeout_ms: 8000,
            });
        }

        // Alchemy (Tertiary)
        if let Ok(alchemy_key) = env::var("ALCHEMY_API_KEY") {
            providers.push(RpcProvider {
                name: "Alchemy".to_string(),
                url: format!("https://solana-mainnet.g.alchemy.com/v2/{}", alchemy_key),
                api_key: None,
                priority: 3,
                max_requests_per_second: 30,
                timeout_ms: 10000,
            });
        }

        // Public RPC (Fallback)
        providers.push(RpcProvider {
            name: "Public RPC".to_string(),
            url: "https://api.mainnet-beta.solana.com".to_string(),
            api_key: None,
            priority: 99,
            max_requests_per_second: 10,
            timeout_ms: 15000,
        });

        if providers.is_empty() {
            return Err(anyhow!("No RPC providers configured"));
        }

        Ok(providers)
    }

    /// 💰 Load fee optimizer configuration
    fn load_fee_optimizer_config() -> Result<FeeOptimizerConfig> {
        Ok(FeeOptimizerConfig {
            base_tip_lamports: env::var("BASE_TIP_LAMPORTS")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()?,
            
            percentile_target: env::var("TIP_PERCENTILE_TARGET")
                .unwrap_or_else(|_| "0.8".to_string())
                .parse()?,
            
            jitter_percentage: env::var("TIP_JITTER_PERCENTAGE")
                .unwrap_or_else(|_| "0.05".to_string())
                .parse()?,
            
            strategy_multipliers: StrategyMultipliers {
                piranha_surf: 1.5,
                sandwich_arbitrage: 2.0,
                cross_dex_arbitrage: 1.2,
                liquidity_snipe: 1.8,
                emergency_exit: 0.5,
            },
            
            cache_ttl_seconds: 30,
            max_tip_lamports: 1_000_000, // 0.001 SOL max tip
        })
    }

    /// 🎮 Load simulation configuration
    fn load_simulation_config() -> Result<SimulationConfig> {
        Ok(SimulationConfig {
            enabled: env::var("SIMULATION_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            
            max_simulation_time_ms: env::var("MAX_SIMULATION_TIME_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()?,
            
            profit_threshold_percentage: env::var("PROFIT_THRESHOLD_PERCENTAGE")
                .unwrap_or_else(|_| "0.02".to_string())
                .parse()?,
        })
    }
}
