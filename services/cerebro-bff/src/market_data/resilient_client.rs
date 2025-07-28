// 🛡️ Resilient Market Data Client with Retry Logic and Fallback
// Provides fault-tolerant access to market data with automatic failover

use super::{MarketDataClient, MarketSnapshot, TokenData};
use super::mock_client::MockMarketDataClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::time::{sleep, Duration, timeout};
use tracing::{warn, error, debug, info};

pub struct ResilientMarketDataClient {
    /// Primary client (e.g., HeliusClient)
    primary: Arc<dyn MarketDataClient + Send + Sync>,
    /// Fallback client (e.g., QuickNodeClient)
    fallback: Option<Arc<dyn MarketDataClient + Send + Sync>>,
    /// Mock client for ultimate fallback
    mock: Arc<MockMarketDataClient>,
    /// Retry configuration
    config: RetryConfig,
}

#[derive(Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay between retries (exponential backoff)
    pub base_delay_ms: u64,
    /// Maximum delay between retries
    pub max_delay_ms: u64,
    /// Request timeout
    pub timeout_ms: u64,
    /// Whether to use mock as ultimate fallback
    pub use_mock_fallback: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            timeout_ms: 10000,
            use_mock_fallback: true,
        }
    }
}

impl ResilientMarketDataClient {
    pub fn new(primary: Arc<dyn MarketDataClient + Send + Sync>) -> Self {
        Self {
            primary,
            fallback: None,
            mock: Arc::new(MockMarketDataClient::new()),
            config: RetryConfig::default(),
        }
    }

    pub fn with_fallback(mut self, fallback: Arc<dyn MarketDataClient + Send + Sync>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn with_config(mut self, config: RetryConfig) -> Self {
        self.config = config;
        self
    }

    /// Execute operation with retry logic and fallback
    async fn execute_with_resilience<T, F, Fut>(&self, operation_name: &str, operation: F) -> anyhow::Result<T>
    where
        F: Fn(Arc<dyn MarketDataClient + Send + Sync>) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
        T: Clone,
    {
        // Try primary client with retries
        if let Ok(result) = self.retry_operation(&operation, self.primary.clone(), operation_name).await {
            return Ok(result);
        }

        warn!("🔄 Primary client failed for {}, trying fallback", operation_name);

        // Try fallback client with retries
        if let Some(fallback) = &self.fallback {
            if let Ok(result) = self.retry_operation(&operation, fallback.clone(), operation_name).await {
                return Ok(result);
            }
        }

        warn!("🔄 Fallback client failed for {}, trying mock", operation_name);

        // Ultimate fallback to mock (if enabled)
        if self.config.use_mock_fallback {
            match self.retry_operation(&operation, self.mock.clone(), operation_name).await {
                Ok(result) => {
                    warn!("⚠️ Using mock data for {} due to all clients failing", operation_name);
                    Ok(result)
                }
                Err(e) => {
                    error!("❌ All clients failed for {}, including mock: {}", operation_name, e);
                    Err(anyhow::anyhow!("All market data clients failed for {}: {}", operation_name, e))
                }
            }
        } else {
            Err(anyhow::anyhow!("All market data clients failed for {}", operation_name))
        }
    }

    /// Retry operation with exponential backoff
    async fn retry_operation<T, F, Fut>(
        &self,
        operation: &F,
        client: Arc<dyn MarketDataClient + Send + Sync>,
        operation_name: &str,
    ) -> anyhow::Result<T>
    where
        F: Fn(Arc<dyn MarketDataClient + Send + Sync>) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            debug!("🔄 Attempt {} for {}", attempt + 1, operation_name);

            // Execute with timeout
            let result = timeout(
                Duration::from_millis(self.config.timeout_ms),
                operation(client.clone())
            ).await;

            match result {
                Ok(Ok(value)) => {
                    if attempt > 0 {
                        info!("✅ {} succeeded on attempt {}", operation_name, attempt + 1);
                    }
                    return Ok(value);
                }
                Ok(Err(e)) => {
                    warn!("⚠️ {} failed on attempt {}: {}", operation_name, attempt + 1, e);
                    last_error = Some(e);
                }
                Err(_) => {
                    warn!("⏰ {} timed out on attempt {}", operation_name, attempt + 1);
                    last_error = Some(anyhow::anyhow!("Operation timed out"));
                }
            }

            // Don't sleep after the last attempt
            if attempt < self.config.max_retries {
                let delay = self.calculate_backoff_delay(attempt);
                debug!("⏳ Waiting {}ms before retry", delay);
                sleep(Duration::from_millis(delay)).await;
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error")))
    }

    /// Calculate exponential backoff delay with jitter
    fn calculate_backoff_delay(&self, attempt: u32) -> u64 {
        let base_delay = self.config.base_delay_ms;
        let exponential_delay = base_delay * 2_u64.pow(attempt);
        let delay_with_cap = exponential_delay.min(self.config.max_delay_ms);
        
        // Add jitter (±25%)
        let jitter_range = delay_with_cap / 4;
        let jitter = rand::random::<u64>() % (jitter_range * 2);
        let jitter_offset = jitter.saturating_sub(jitter_range);
        
        delay_with_cap.saturating_add(jitter_offset)
    }
}

#[async_trait]
impl MarketDataClient for ResilientMarketDataClient {
    async fn get_token_data(&self, mint: &str) -> anyhow::Result<TokenData> {
        let mint = mint.to_string();
        self.execute_with_resilience(
            &format!("get_token_data({})", mint),
            |client| {
                let mint = mint.clone();
                async move { client.get_token_data(&mint).await }
            }
        ).await
    }

    async fn get_market_snapshot(&self, tokens: Vec<String>) -> anyhow::Result<MarketSnapshot> {
        self.execute_with_resilience(
            "get_market_snapshot",
            |client| {
                let tokens = tokens.clone();
                async move { client.get_market_snapshot(tokens).await }
            }
        ).await
    }

    async fn get_price_history(&self, mint: &str, hours: u32) -> anyhow::Result<Vec<(DateTime<Utc>, f64)>> {
        let mint = mint.to_string();
        self.execute_with_resilience(
            &format!("get_price_history({}, {}h)", mint, hours),
            |client| {
                let mint = mint.clone();
                async move { client.get_price_history(&mint, hours).await }
            }
        ).await
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        // Quick health check without full resilience (to avoid infinite loops)
        match timeout(Duration::from_millis(5000), self.primary.health_check()).await {
            Ok(Ok(true)) => Ok(true),
            _ => {
                if let Some(fallback) = &self.fallback {
                    match timeout(Duration::from_millis(5000), fallback.health_check()).await {
                        Ok(Ok(true)) => Ok(true),
                        _ => Ok(false), // At least mock is always available
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct FailingClient {
        fail_count: AtomicU32,
        max_failures: u32,
    }

    impl FailingClient {
        fn new(max_failures: u32) -> Self {
            Self {
                fail_count: AtomicU32::new(0),
                max_failures,
            }
        }
    }

    #[async_trait]
    impl MarketDataClient for FailingClient {
        async fn get_token_data(&self, _mint: &str) -> anyhow::Result<TokenData> {
            let count = self.fail_count.fetch_add(1, Ordering::SeqCst);
            if count < self.max_failures {
                Err(anyhow::anyhow!("Simulated failure {}", count + 1))
            } else {
                Ok(TokenData {
                    mint: "test".to_string(),
                    name: "Test Token".to_string(),
                    symbol: "TEST".to_string(),
                    price_sol: 0.001,
                    liquidity_sol: 1000.0,
                    volume_24h: 500.0,
                    market_cap: 10000.0,
                    holder_data: None,
                    price_data: None,
                })
            }
        }

        async fn get_market_snapshot(&self, _tokens: Vec<String>) -> anyhow::Result<MarketSnapshot> {
            unimplemented!()
        }

        async fn get_price_history(&self, _mint: &str, _hours: u32) -> anyhow::Result<Vec<(DateTime<Utc>, f64)>> {
            unimplemented!()
        }

        async fn health_check(&self) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_resilient_client_retry_success() {
        let failing_client = Arc::new(FailingClient::new(2)); // Fail 2 times, then succeed
        let resilient = ResilientMarketDataClient::new(failing_client)
            .with_config(RetryConfig {
                max_retries: 3,
                base_delay_ms: 10,
                ..Default::default()
            });

        let result = resilient.get_token_data("test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resilient_client_fallback_to_mock() {
        let always_failing = Arc::new(FailingClient::new(u32::MAX)); // Always fail
        let resilient = ResilientMarketDataClient::new(always_failing)
            .with_config(RetryConfig {
                max_retries: 1,
                base_delay_ms: 10,
                use_mock_fallback: true,
                ..Default::default()
            });

        let result = resilient.get_token_data("test").await;
        assert!(result.is_ok()); // Should succeed with mock
    }
}
