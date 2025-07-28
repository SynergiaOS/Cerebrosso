// 🎭 Mock Market Data Client for Development & Testing
// Provides realistic mock data for Helius/QuickNode APIs without external dependencies

use super::{MarketDataClient, MarketSnapshot, TokenData, HolderData, PriceData};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

pub struct MockMarketDataClient {
    /// Simulate network latency (10-100ms)
    latency_ms: u64,
    /// Simulate occasional failures (5% failure rate)
    failure_rate: f64,
}

impl MockMarketDataClient {
    pub fn new() -> Self {
        Self {
            latency_ms: 50,
            failure_rate: 0.05,
        }
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    pub fn with_failure_rate(mut self, failure_rate: f64) -> Self {
        self.failure_rate = failure_rate;
        self
    }

    /// Simulate network delay
    async fn simulate_network_delay(&self) {
        use rand::Rng;
        let delay = {
            let mut rng = rand::thread_rng();
            rng.gen_range(self.latency_ms/2..=self.latency_ms*2)
        };
        sleep(Duration::from_millis(delay)).await;
    }

    /// Simulate occasional network failures
    fn should_fail(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.failure_rate
    }

    /// Generate realistic mock token data based on mint address
    fn generate_mock_token_data(&self, mint: &str) -> TokenData {
        use rand::Rng;

        // Use mint hash to generate consistent but varied data
        let mint_hash = mint.chars().map(|c| c as u32).sum::<u32>();
        let seed = mint_hash % 1000;

        let mut rng = rand::thread_rng();

        // Generate realistic price ranges based on seed
        let price_sol = match seed % 10 {
            0..=2 => rng.gen_range(0.000001..0.00001), // Micro cap
            3..=5 => rng.gen_range(0.00001..0.0001),   // Small cap
            6..=7 => rng.gen_range(0.0001..0.001),     // Mid cap
            8 => rng.gen_range(0.001..0.01),           // Large cap
            _ => rng.gen_range(0.01..0.1),             // Blue chip
        };

        let liquidity_sol = price_sol * rng.gen_range(10000.0..100000.0);
        let volume_24h = liquidity_sol * rng.gen_range(0.1..2.0);
        let market_cap = price_sol * rng.gen_range(1000000.0..10000000.0);

        TokenData {
            mint: mint.to_string(),
            name: format!("MockToken{}", &mint[..8]),
            symbol: format!("M{}", &mint[..4].to_uppercase()),
            price_sol,
            liquidity_sol,
            volume_24h,
            market_cap,
            holder_data: Some(HolderData {
                top_10_pct: rng.gen_range(0.15..0.45),
                dev_pct: rng.gen_range(0.01..0.15),
            }),
            price_data: Some(PriceData {
                price_change_24h: rng.gen_range(-0.5..0.5),
                price_change_1h: rng.gen_range(-0.1..0.1),
                ath: price_sol * rng.gen_range(1.0..10.0),
                atl: price_sol * rng.gen_range(0.1..1.0),
            }),
        }
    }
}

#[async_trait]
impl MarketDataClient for MockMarketDataClient {
    async fn get_token_data(&self, mint: &str) -> anyhow::Result<TokenData> {
        // Simulate network delay
        self.simulate_network_delay().await;

        // Simulate occasional failures
        if self.should_fail() {
            return Err(anyhow::anyhow!("Mock network failure for token {}", mint));
        }

        tracing::debug!("🎭 Mock: Fetching token data for {}", mint);
        Ok(self.generate_mock_token_data(mint))
    }

    async fn get_market_snapshot(&self, tokens: Vec<String>) -> anyhow::Result<MarketSnapshot> {
        // Simulate network delay
        self.simulate_network_delay().await;

        // Simulate occasional failures
        if self.should_fail() {
            return Err(anyhow::anyhow!("Mock network failure for market snapshot"));
        }

        tracing::debug!("🎭 Mock: Fetching market snapshot for {} tokens", tokens.len());

        let mut token_data = HashMap::new();
        for mint in tokens {
            token_data.insert(mint.clone(), self.generate_mock_token_data(&mint));
        }

        let mut rng = rand::thread_rng();
        Ok(MarketSnapshot {
            timestamp: Utc::now(),
            token_data,
            market_metrics: super::MarketMetrics {
                total_volume_24h: rng.gen_range(1000000.0..10000000.0),
                total_liquidity: rng.gen_range(10000000.0..100000000.0),
                active_tokens: rng.gen_range(1000..5000),
                trending_tokens: vec![
                    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    "So11111111111111111111111111111111111111112".to_string(),
                ],
            },
        })
    }

    async fn get_price_history(&self, mint: &str, hours: u32) -> anyhow::Result<Vec<(DateTime<Utc>, f64)>> {
        // Simulate network delay
        self.simulate_network_delay().await;

        // Simulate occasional failures
        if self.should_fail() {
            return Err(anyhow::anyhow!("Mock network failure for price history {}", mint));
        }

        tracing::debug!("🎭 Mock: Fetching price history for {} ({} hours)", mint, hours);

        let mut rng = rand::thread_rng();
        let base_price = rng.gen_range(0.0001..0.01);
        let mut prices = Vec::new();
        
        let now = Utc::now();
        for i in 0..hours {
            let timestamp = now - chrono::Duration::hours(i as i64);
            let price_variation = rng.gen_range(0.9..1.1);
            let price = base_price * price_variation;
            prices.push((timestamp, price));
        }

        prices.reverse(); // Oldest first
        Ok(prices)
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        // Mock always healthy with small delay
        sleep(Duration::from_millis(10)).await;
        tracing::debug!("🎭 Mock: Health check OK");
        Ok(true)
    }
}

impl Default for MockMarketDataClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_token_data() {
        let client = MockMarketDataClient::new();
        let result = client.get_token_data("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").await;
        
        assert!(result.is_ok());
        let token_data = result.unwrap();
        assert!(!token_data.name.is_empty());
        assert!(token_data.price_sol > 0.0);
    }

    #[tokio::test]
    async fn test_mock_market_snapshot() {
        let client = MockMarketDataClient::new();
        let tokens = vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            "So11111111111111111111111111111111111111112".to_string(),
        ];
        
        let result = client.get_market_snapshot(tokens.clone()).await;
        assert!(result.is_ok());
        
        let snapshot = result.unwrap();
        assert_eq!(snapshot.token_data.len(), tokens.len());
    }

    #[tokio::test]
    async fn test_mock_with_high_failure_rate() {
        let client = MockMarketDataClient::new().with_failure_rate(1.0); // Always fail
        let result = client.get_token_data("test").await;
        assert!(result.is_err());
    }
}
