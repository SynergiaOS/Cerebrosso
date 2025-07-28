// 📊 Market Data Module - Unified interface for market data providers
// Provides resilient, fault-tolerant access to Solana market data

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod mock_client;
pub mod resilient_client;

// Re-export main types
pub use mock_client::MockMarketDataClient;
pub use resilient_client::{ResilientMarketDataClient, RetryConfig};

/// Unified trait for all market data providers
#[async_trait]
pub trait MarketDataClient {
    /// Get detailed token data for a specific mint
    async fn get_token_data(&self, mint: &str) -> anyhow::Result<TokenData>;
    
    /// Get market snapshot for multiple tokens
    async fn get_market_snapshot(&self, tokens: Vec<String>) -> anyhow::Result<MarketSnapshot>;
    
    /// Get price history for a token
    async fn get_price_history(&self, mint: &str, hours: u32) -> anyhow::Result<Vec<(DateTime<Utc>, f64)>>;
    
    /// Health check for the client
    async fn health_check(&self) -> anyhow::Result<bool>;
}

/// Comprehensive token data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub price_sol: f64,
    pub liquidity_sol: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub holder_data: Option<HolderData>,
    pub price_data: Option<PriceData>,
}

/// Token holder distribution data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolderData {
    /// Percentage held by top 10 holders
    pub top_10_pct: f64,
    /// Percentage held by dev/team
    pub dev_pct: f64,
}

/// Price movement and historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// 24h price change percentage
    pub price_change_24h: f64,
    /// 1h price change percentage
    pub price_change_1h: f64,
    /// All-time high price
    pub ath: f64,
    /// All-time low price
    pub atl: f64,
}

/// Market snapshot containing multiple tokens and overall metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub timestamp: DateTime<Utc>,
    pub token_data: HashMap<String, TokenData>,
    pub market_metrics: MarketMetrics,
}

/// Overall market metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMetrics {
    pub total_volume_24h: f64,
    pub total_liquidity: f64,
    pub active_tokens: u32,
    pub trending_tokens: Vec<String>,
}

/// Market data client factory
pub struct MarketDataClientFactory;

impl MarketDataClientFactory {
    /// Create a resilient market data client based on environment configuration
    pub fn create_resilient_client() -> anyhow::Result<ResilientMarketDataClient> {
        // Check for real API credentials
        let helius_key = std::env::var("HELIUS_API_KEY").ok();
        let quicknode_url = std::env::var("QUICKNODE_RPC_URL").ok();
        
        // For now, use mock as primary until we implement real clients
        let primary = std::sync::Arc::new(MockMarketDataClient::new());
        
        let mut resilient_client = ResilientMarketDataClient::new(primary);
        
        // Configure retry settings based on environment
        let config = if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
            RetryConfig {
                max_retries: 5,
                base_delay_ms: 200,
                max_delay_ms: 10000,
                timeout_ms: 15000,
                use_mock_fallback: false, // No mock in production
            }
        } else {
            RetryConfig {
                max_retries: 3,
                base_delay_ms: 100,
                max_delay_ms: 5000,
                timeout_ms: 10000,
                use_mock_fallback: true, // Use mock in development
            }
        };
        
        resilient_client = resilient_client.with_config(config);
        
        tracing::info!("🏭 Created resilient market data client");
        if helius_key.is_none() && quicknode_url.is_none() {
            tracing::warn!("⚠️ No real API credentials found, using mock data");
        }
        
        Ok(resilient_client)
    }
    
    /// Create a pure mock client for testing
    pub fn create_mock_client() -> MockMarketDataClient {
        MockMarketDataClient::new()
    }
    
    /// Create a mock client with custom settings
    pub fn create_mock_client_with_settings(latency_ms: u64, failure_rate: f64) -> MockMarketDataClient {
        MockMarketDataClient::new()
            .with_latency(latency_ms)
            .with_failure_rate(failure_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_creates_resilient_client() {
        let client = MarketDataClientFactory::create_resilient_client();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_factory_creates_mock_client() {
        let client = MarketDataClientFactory::create_mock_client();
        let result = client.get_token_data("test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_client_with_custom_settings() {
        let client = MarketDataClientFactory::create_mock_client_with_settings(10, 0.0);
        let result = client.get_token_data("test").await;
        assert!(result.is_ok());
    }
}
