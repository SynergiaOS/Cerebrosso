//! 🔗 Synk - Network State Synchronization Library
//! 
//! Advanced network state synchronization and monitoring for Solana blockchain

pub mod config;
pub mod network_monitor;
pub mod state_synchronizer;
pub mod rpc_manager;
pub mod block_tracker;
pub mod transaction_monitor;
pub mod account_watcher;
pub mod metrics;

// Core exports
pub use config::Config;
pub use network_monitor::{NetworkMonitor, NetworkState, NetworkHealth};
pub use state_synchronizer::{StateSynchronizer, SyncState, SyncError};
pub use rpc_manager::{RpcManager, RpcProvider, RpcHealth};
pub use block_tracker::{BlockTracker, BlockInfo, SlotInfo};
pub use transaction_monitor::{TransactionMonitor, TxStatus, TxEvent};
pub use account_watcher::{AccountWatcher, AccountChange, AccountSubscription};
pub use metrics::{SynkMetrics, NetworkMetrics, SyncMetrics};

/// 🎯 Core Synk Result Type
pub type SynkResult<T> = Result<T, SyncError>;

/// 🔗 Network State Types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkEvent {
    /// New block detected
    NewBlock(BlockInfo),
    /// New transaction detected
    NewTransaction(TxEvent),
    /// Account state changed
    AccountChanged(AccountChange),
    /// Network congestion detected
    CongestionAlert(f64),
    /// RPC provider status changed
    RpcStatusChanged(RpcHealth),
}

/// 📊 Synchronization Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncStatus {
    /// Synchronization starting
    Starting,
    /// Actively synchronizing
    Syncing { progress: f64 },
    /// Fully synchronized
    Synced,
    /// Synchronization paused
    Paused,
    /// Synchronization failed
    Failed(String),
}

/// 🎯 Synk Constants
pub mod constants {
    /// Default sync interval in milliseconds
    pub const DEFAULT_SYNC_INTERVAL_MS: u64 = 1000;
    
    /// Maximum blocks to sync in one batch
    pub const MAX_BLOCKS_PER_BATCH: usize = 100;
    
    /// Default RPC timeout in seconds
    pub const DEFAULT_RPC_TIMEOUT_SECS: u64 = 30;
    
    /// Maximum concurrent RPC requests
    pub const MAX_CONCURRENT_REQUESTS: usize = 50;
    
    /// Network health check interval
    pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 60;
    
    /// Default slot commitment level
    pub const DEFAULT_COMMITMENT: &str = "confirmed";
    
    /// Maximum retry attempts for failed requests
    pub const MAX_RETRY_ATTEMPTS: usize = 3;
    
    /// Backoff multiplier for retries
    pub const RETRY_BACKOFF_MULTIPLIER: f64 = 2.0;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8300,
                host: "localhost".to_string(),
            },
            network: config::NetworkConfig {
                cluster: "devnet".to_string(),
                rpc_urls: vec![
                    "https://api.devnet.solana.com".to_string(),
                    "https://devnet.helius-rpc.com".to_string(),
                ],
                websocket_url: "wss://api.devnet.solana.com".to_string(),
                commitment_level: "confirmed".to_string(),
                timeout_seconds: 30,
            },
            sync: config::SyncConfig {
                sync_interval_ms: 1000,
                batch_size: 100,
                max_concurrent_requests: 50,
                enable_real_time: true,
                enable_historical_sync: false,
            },
            monitoring: config::MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: 9300,
                health_check_interval_secs: 60,
                log_level: "info".to_string(),
            },
            redis: config::RedisConfig {
                url: "redis://localhost:6379".to_string(),
                key_prefix: "synk:".to_string(),
                ttl_seconds: 3600,
            },
        })
    }
    
    /// Create a mock block info
    pub fn create_mock_block_info() -> BlockInfo {
        BlockInfo {
            slot: 12345,
            block_hash: "test_hash".to_string(),
            parent_slot: 12344,
            block_time: Some(chrono::Utc::now().timestamp()),
            transaction_count: 100,
            block_height: Some(12345),
        }
    }
    
    /// Create a mock transaction event
    pub fn create_mock_tx_event() -> TxEvent {
        TxEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
            status: TxStatus::Confirmed,
            fee: 5000,
            compute_units_consumed: Some(200000),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_synk_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.server.port, 8300);
        assert_eq!(config.network.cluster, "devnet");
        assert_eq!(config.sync.sync_interval_ms, 1000);
    }
    
    #[tokio::test]
    async fn test_block_info_creation() {
        let block_info = create_mock_block_info();
        
        assert_eq!(block_info.slot, 12345);
        assert_eq!(block_info.parent_slot, 12344);
        assert_eq!(block_info.transaction_count, 100);
    }
    
    #[tokio::test]
    async fn test_tx_event_creation() {
        let tx_event = create_mock_tx_event();
        
        assert_eq!(tx_event.signature, "test_signature");
        assert_eq!(tx_event.slot, 12345);
        assert_eq!(tx_event.fee, 5000);
    }
}
