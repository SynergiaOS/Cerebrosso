//! 🐺 Cerberus Phoenix Evolved - Core Types
//! 
//! Shared types for the entire Cerberus ecosystem.
//! Zero-copy, high-performance types for HFT trading.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::time::SystemTime;
use uuid::Uuid;

/// 🎯 Trading signal from various sources
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Signal {
    pub id: Uuid,
    pub source: SignalSource,
    pub token: Pubkey,
    pub pool_address: Pubkey,
    pub signal_type: SignalType,
    pub confidence: f32,
    pub timestamp: SystemTime,
    pub metadata: serde_json::Value,
}

/// 📊 Signal sources
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SignalSource {
    Mempool,
    Dex,
    Social,
    Technical,
    Arbitrage,
}

/// 🎯 Types of trading signals
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SignalType {
    NewListing,
    LiquidityAdd,
    LargeTransaction,
    PriceMovement,
    VolumeSpike,
    ArbitrageOpportunity,
}

/// 🧠 AI Decision from Cerebro
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Decision {
    pub id: Uuid,
    pub signal_id: Uuid,
    pub action: DecisionAction,
    pub confidence: f32,
    pub expected_profit: f64,
    pub risk_score: f32,
    pub timestamp: SystemTime,
    pub reasoning: String,
}

/// ⚡ Decision actions
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DecisionAction {
    Snipe { amount_sol: f64, slippage: f32 },
    Hold,
    Dump { percentage: f32 },
    Ignore,
}

/// 🚀 Execution result from HFT Ninja
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecutionResult {
    pub id: Uuid,
    pub decision_id: Uuid,
    pub bundle_id: Option<String>,
    pub success: bool,
    pub latency_ms: u64,
    pub profit_sol: f64,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
    pub timestamp: SystemTime,
}

/// 📦 Jito Bundle for MEV protection
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JitoBundle {
    pub id: Uuid,
    pub transactions: Vec<String>, // Base58 encoded transactions
    pub tip_lamports: u64,
    pub max_tip_lamports: u64,
    pub timestamp: SystemTime,
}

/// 🔍 Health check response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthStatus {
    pub service: String,
    pub status: ServiceStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub last_activity: SystemTime,
    pub dependencies: Vec<DependencyStatus>,
}

/// 📊 Service status
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 🔗 Dependency status
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DependencyStatus {
    pub name: String,
    pub status: ServiceStatus,
    pub latency_ms: Option<u64>,
    pub last_check: SystemTime,
}

/// ⚠️ Error types
#[derive(thiserror::Error, Debug)]
pub enum CerberusError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Execution failed: {0}")]
    Execution(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Slippage exceeded")]
    SlippageExceeded,
}

/// 📈 Performance metrics
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_profit_sol: f64,
    pub avg_latency_ms: f64,
    pub success_rate: f32,
    pub daily_roi: f32,
    pub timestamp: SystemTime,
}

impl Signal {
    pub fn new(
        source: SignalSource,
        token: Pubkey,
        pool_address: Pubkey,
        signal_type: SignalType,
        confidence: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            token,
            pool_address,
            signal_type,
            confidence,
            timestamp: SystemTime::now(),
            metadata: serde_json::Value::Null,
        }
    }
}

impl Decision {
    pub fn new(signal_id: Uuid, action: DecisionAction, confidence: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_id,
            action,
            confidence,
            expected_profit: 0.0,
            risk_score: 0.0,
            timestamp: SystemTime::now(),
            reasoning: String::new(),
        }
    }
}

impl ExecutionResult {
    pub fn success(decision_id: Uuid, latency_ms: u64, profit_sol: f64, tx_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_id,
            bundle_id: None,
            success: true,
            latency_ms,
            profit_sol,
            tx_hash: Some(tx_hash),
            error: None,
            timestamp: SystemTime::now(),
        }
    }

    pub fn failure(decision_id: Uuid, latency_ms: u64, error: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_id,
            bundle_id: None,
            success: false,
            latency_ms,
            profit_sol: 0.0,
            tx_hash: None,
            error: Some(error),
            timestamp: SystemTime::now(),
        }
    }
}
