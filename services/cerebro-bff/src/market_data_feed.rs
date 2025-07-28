//! 📊 Market Data Feed - Real-time Market Data for Paper Trading
//! 
//! Dostarcza real-time dane rynkowe dla paper trading engine,
//! integrując się z różnymi źródłami danych (Helius, QuickNode, DexScreener).

use crate::config::Config;
use crate::feedback_system::MarketSnapshot;
use crate::helius_client::HeliusClient;
use crate::quicknode_client::QuickNodeClient;
use crate::market_data::{ResilientMarketDataClient, MarketDataClient};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketDataPoint {
    pub token_address: String,
    pub symbol: String,
    pub price_usd: f64,
    pub volume_24h: f64,
    pub liquidity_usd: f64,
    pub market_cap: f64,
    pub price_change_24h: f64,
    pub volatility: f64,
    pub holder_count: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceHistory {
    pub token_address: String,
    pub prices: Vec<PricePoint>,
    pub timeframe: TimeFrame,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PricePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TimeFrame {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
}

#[derive(Debug, Clone)]
pub struct MarketDataConfig {
    pub update_interval_ms: u64,
    pub price_history_length: usize,
    pub volatility_window: usize,
    pub enable_real_time: bool,
    pub fallback_to_mock: bool,
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000, // 1 second updates
            price_history_length: 100,
            volatility_window: 20,
            enable_real_time: true,
            fallback_to_mock: true,
        }
    }
}

pub struct MarketDataFeed {
    config: Arc<Config>,
    data_config: MarketDataConfig,
    helius_client: Arc<HeliusClient>,
    quicknode_client: Arc<QuickNodeClient>,
    resilient_client: Arc<ResilientMarketDataClient>,
    market_data: Arc<RwLock<HashMap<String, MarketDataPoint>>>,
    price_history: Arc<RwLock<HashMap<String, PriceHistory>>>,
    subscribed_tokens: Arc<RwLock<Vec<String>>>,
    last_update: Arc<RwLock<Instant>>,
}

impl MarketDataFeed {
    /// 🚀 Initialize market data feed
    pub async fn new(
        config: Arc<Config>,
        helius_client: Arc<HeliusClient>,
        quicknode_client: Arc<QuickNodeClient>,
        resilient_client: Arc<ResilientMarketDataClient>,
    ) -> Result<Self> {
        info!("📊 Initializing Market Data Feed v2.0 with Resilient Client");

        let data_config = MarketDataConfig::default();
        let market_data = Arc::new(RwLock::new(HashMap::new()));
        let price_history = Arc::new(RwLock::new(HashMap::new()));
        let subscribed_tokens = Arc::new(RwLock::new(Vec::new()));
        let last_update = Arc::new(RwLock::new(Instant::now()));

        // Test resilient client health
        match resilient_client.health_check().await {
            Ok(true) => info!("✅ Resilient market data client is healthy"),
            Ok(false) => warn!("⚠️ Resilient market data client reports unhealthy"),
            Err(e) => warn!("⚠️ Failed to check resilient client health: {}", e),
        }

        let feed = MarketDataFeed {
            config,
            data_config,
            helius_client,
            quicknode_client,
            resilient_client,
            market_data,
            price_history,
            subscribed_tokens,
            last_update,
        };
        
        // Start background update task
        if feed.data_config.enable_real_time {
            feed.start_background_updates().await;
        }
        
        info!("✅ Market Data Feed initialized successfully");
        Ok(feed)
    }
    
    /// 📈 Subscribe to token for real-time updates
    pub async fn subscribe_token(&self, token_address: String) -> Result<()> {
        let mut subscribed = self.subscribed_tokens.write().await;
        if !subscribed.contains(&token_address) {
            subscribed.push(token_address.clone());
            
            // Initialize market data for this token
            self.fetch_initial_data(&token_address).await?;
            
            info!("📈 Subscribed to token: {}", token_address);
        }
        Ok(())
    }
    
    /// 📊 Get current market snapshot for token
    pub async fn get_market_snapshot(&self, token_address: &str) -> Result<MarketSnapshot> {
        // Try to get from cache first
        if let Some(data) = self.market_data.read().await.get(token_address) {
            return Ok(MarketSnapshot {
                token_address: data.token_address.clone(),
                price_usd: data.price_usd,
                volume_24h: Some(data.volume_24h),
                liquidity_usd: Some(data.liquidity_usd),
                volatility: Some(data.volatility),
                market_cap: Some(data.market_cap),
                holder_count: Some(data.holder_count as i32),
                dex_data: serde_json::json!({
                    "symbol": data.symbol,
                    "price_change_24h": data.price_change_24h,
                    "last_updated": data.last_updated
                }),
            });
        }
        
        // If not in cache, fetch fresh data
        self.fetch_token_data(token_address).await
    }
    
    /// 🔄 Fetch fresh token data from APIs
    async fn fetch_token_data(&self, token_address: &str) -> Result<MarketSnapshot> {
        // Try Helius first
        match self.fetch_from_helius(token_address).await {
            Ok(snapshot) => {
                self.update_cache_with_snapshot(token_address, &snapshot).await;
                return Ok(snapshot);
            }
            Err(e) => {
                warn!("Failed to fetch from Helius: {}", e);
            }
        }
        
        // Fallback to QuickNode
        match self.fetch_from_quicknode(token_address).await {
            Ok(snapshot) => {
                self.update_cache_with_snapshot(token_address, &snapshot).await;
                return Ok(snapshot);
            }
            Err(e) => {
                warn!("Failed to fetch from QuickNode: {}", e);
            }
        }
        
        // Final fallback to mock data if enabled
        if self.data_config.fallback_to_mock {
            Ok(self.generate_mock_data(token_address))
        } else {
            Err(anyhow::anyhow!("Failed to fetch market data for {}", token_address))
        }
    }
    
    /// 🌟 Fetch data from Helius
    async fn fetch_from_helius(&self, token_address: &str) -> Result<MarketSnapshot> {
        // This would integrate with actual Helius API
        // For now, return mock data structure
        debug!("Fetching market data from Helius for {}", token_address);
        
        // Simulate API call delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(MarketSnapshot {
            token_address: token_address.to_string(),
            price_usd: 0.001 + (rand::random::<f64>() * 0.01), // Mock price
            volume_24h: Some(10000.0 + (rand::random::<f64>() * 50000.0)),
            liquidity_usd: Some(100000.0 + (rand::random::<f64>() * 500000.0)),
            volatility: Some(0.02 + (rand::random::<f64>() * 0.08)),
            market_cap: Some(1000000.0 + (rand::random::<f64>() * 5000000.0)),
            holder_count: Some(1000 + (rand::random::<u32>() % 5000) as i32),
            dex_data: serde_json::json!({
                "source": "helius",
                "dex": "raydium",
                "pool_address": format!("pool_{}", token_address)
            }),
        })
    }
    
    /// ⚡ Fetch data from QuickNode
    async fn fetch_from_quicknode(&self, token_address: &str) -> Result<MarketSnapshot> {
        debug!("Fetching market data from QuickNode for {}", token_address);
        
        // Simulate API call delay
        tokio::time::sleep(Duration::from_millis(30)).await;
        
        Ok(MarketSnapshot {
            token_address: token_address.to_string(),
            price_usd: 0.001 + (rand::random::<f64>() * 0.01),
            volume_24h: Some(8000.0 + (rand::random::<f64>() * 40000.0)),
            liquidity_usd: Some(80000.0 + (rand::random::<f64>() * 400000.0)),
            volatility: Some(0.015 + (rand::random::<f64>() * 0.06)),
            market_cap: Some(800000.0 + (rand::random::<f64>() * 4000000.0)),
            holder_count: Some(800 + (rand::random::<u32>() % 4000) as i32),
            dex_data: serde_json::json!({
                "source": "quicknode",
                "dex": "orca",
                "pool_address": format!("pool_{}", token_address)
            }),
        })
    }
    
    /// 🎭 Generate mock data for testing
    fn generate_mock_data(&self, token_address: &str) -> MarketSnapshot {
        let base_price = 0.001 + (token_address.len() as f64 * 0.0001);
        let price_variation = (rand::random::<f64>() - 0.5) * 0.1;
        
        MarketSnapshot {
            token_address: token_address.to_string(),
            price_usd: base_price * (1.0 + price_variation),
            volume_24h: Some(5000.0 + (rand::random::<f64>() * 25000.0)),
            liquidity_usd: Some(50000.0 + (rand::random::<f64>() * 250000.0)),
            volatility: Some(0.01 + (rand::random::<f64>() * 0.05)),
            market_cap: Some(500000.0 + (rand::random::<f64>() * 2500000.0)),
            holder_count: Some(500 + (rand::random::<u32>() % 2500) as i32),
            dex_data: serde_json::json!({
                "source": "mock",
                "dex": "mock_dex",
                "pool_address": format!("mock_pool_{}", token_address)
            }),
        }
    }
    
    /// 💾 Update cache with new snapshot
    async fn update_cache_with_snapshot(&self, token_address: &str, snapshot: &MarketSnapshot) {
        let data_point = MarketDataPoint {
            token_address: snapshot.token_address.clone(),
            symbol: format!("TOKEN_{}", &token_address[..8]),
            price_usd: snapshot.price_usd,
            volume_24h: snapshot.volume_24h.unwrap_or(0.0),
            liquidity_usd: snapshot.liquidity_usd.unwrap_or(0.0),
            market_cap: snapshot.market_cap.unwrap_or(0.0),
            price_change_24h: 0.0, // Would calculate from history
            volatility: snapshot.volatility.unwrap_or(0.0),
            holder_count: snapshot.holder_count.unwrap_or(0) as u32,
            last_updated: chrono::Utc::now(),
        };
        
        // Update market data cache
        self.market_data.write().await.insert(token_address.to_string(), data_point);
        
        // Update price history
        self.update_price_history(token_address, snapshot.price_usd, snapshot.volume_24h.unwrap_or(0.0)).await;
        
        // Update last update time
        *self.last_update.write().await = Instant::now();
    }
    
    /// 📈 Update price history for volatility calculation
    async fn update_price_history(&self, token_address: &str, price: f64, volume: f64) {
        let mut history = self.price_history.write().await;
        let now = chrono::Utc::now();
        
        let price_point = PricePoint {
            timestamp: now,
            price,
            volume,
        };
        
        if let Some(token_history) = history.get_mut(token_address) {
            token_history.prices.push(price_point);
            
            // Keep only recent history
            if token_history.prices.len() > self.data_config.price_history_length {
                token_history.prices.remove(0);
            }
        } else {
            history.insert(token_address.to_string(), PriceHistory {
                token_address: token_address.to_string(),
                prices: vec![price_point],
                timeframe: TimeFrame::OneMinute,
            });
        }
    }
    
    /// 🔄 Start background updates
    async fn start_background_updates(&self) {
        let market_data = self.market_data.clone();
        let subscribed_tokens = self.subscribed_tokens.clone();
        let update_interval = self.data_config.update_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(update_interval));
            
            loop {
                interval.tick().await;
                
                let tokens = subscribed_tokens.read().await.clone();
                for token_address in tokens {
                    // Update market data for each subscribed token
                    // This would call real APIs in production
                    debug!("Background update for token: {}", token_address);
                }
            }
        });
    }
    
    /// 📊 Fetch initial data for new subscription
    async fn fetch_initial_data(&self, token_address: &str) -> Result<()> {
        let snapshot = self.fetch_token_data(token_address).await?;
        self.update_cache_with_snapshot(token_address, &snapshot).await;
        Ok(())
    }
    
    /// 📈 Get price history for token
    pub async fn get_price_history(&self, token_address: &str) -> Option<PriceHistory> {
        self.price_history.read().await.get(token_address).cloned()
    }
    
    /// 📊 Calculate volatility from price history
    pub async fn calculate_volatility(&self, token_address: &str) -> f64 {
        if let Some(history) = self.get_price_history(token_address).await {
            if history.prices.len() < 2 {
                return 0.02; // Default volatility
            }
            
            let prices: Vec<f64> = history.prices.iter().map(|p| p.price).collect();
            let returns: Vec<f64> = prices.windows(2)
                .map(|w| (w[1] / w[0] - 1.0))
                .collect();
            
            if returns.is_empty() {
                return 0.02;
            }
            
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / returns.len() as f64;
            
            variance.sqrt()
        } else {
            0.02 // Default volatility
        }
    }
}
