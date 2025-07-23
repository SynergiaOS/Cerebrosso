//! 🧠 Intelligent Cache - Adaptive TTL Based on Token Volatility
//! 
//! Smart caching system that adjusts TTL based on token characteristics,
//! volatility, and trading patterns for optimal API usage reduction.

use anyhow::{Result, anyhow};
use redis::{Client as RedisClient, AsyncCommands};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc};

/// 🎯 Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub access_count: u32,
    pub volatility_score: f64,
    pub ttl_seconds: u64,
    pub cache_tier: CacheTier,
}

/// 🏆 Cache tiers based on data importance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CacheTier {
    Hot,        // High-frequency access, short TTL
    Warm,       // Medium-frequency access, medium TTL
    Cold,       // Low-frequency access, long TTL
    Frozen,     // Static data, very long TTL
}

/// 📊 Token volatility metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
    pub holder_change_rate: f64,
    pub liquidity_stability: f64,
    pub trading_frequency: f64,
}

/// 🧠 Intelligent Cache Manager
pub struct IntelligentCache {
    redis_client: RedisClient,
    base_ttl: u64,
    max_ttl: u64,
    min_ttl: u64,
    volatility_threshold: f64,
}

impl IntelligentCache {
    /// 🚀 Initialize intelligent cache
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis_client = RedisClient::open(redis_url)?;
        
        Ok(Self {
            redis_client,
            base_ttl: 300,        // 5 minutes base TTL
            max_ttl: 3600,       // 1 hour max TTL
            min_ttl: 60,         // 1 minute min TTL
            volatility_threshold: 0.1, // 10% volatility threshold
        })
    }

    /// 💾 Store data with intelligent TTL
    pub async fn set_with_intelligence(
        &self,
        key: &str,
        data: &serde_json::Value,
        volatility_metrics: Option<VolatilityMetrics>,
    ) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Calculate intelligent TTL
        let (ttl, cache_tier) = self.calculate_intelligent_ttl(volatility_metrics.as_ref());
        
        let cache_entry = CacheEntry {
            data: data.clone(),
            created_at: Utc::now(),
            access_count: 0,
            volatility_score: volatility_metrics.as_ref()
                .map(|v| self.calculate_volatility_score(v))
                .unwrap_or(0.5),
            ttl_seconds: ttl,
            cache_tier,
        };
        
        let serialized = serde_json::to_string(&cache_entry)?;
        let _: () = conn.set_ex(key, serialized, ttl).await?;
        
        debug!("💾 Cached {} with TTL {}s (tier: {:?})", key, ttl, cache_entry.cache_tier);
        Ok(())
    }

    /// 🎯 Get data and update access patterns
    pub async fn get_with_intelligence(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let cached: Option<String> = conn.get(key).await?;
        
        match cached {
            Some(serialized) => {
                let mut cache_entry: CacheEntry = serde_json::from_str(&serialized)?;
                cache_entry.access_count += 1;
                
                // Update access pattern
                let updated_serialized = serde_json::to_string(&cache_entry)?;
                let remaining_ttl: i64 = conn.ttl(key).await?;
                
                if remaining_ttl > 0 {
                    let _: () = conn.set_ex(key, updated_serialized, remaining_ttl as u64).await?;
                }
                
                debug!("🎯 Cache hit for {} (access count: {})", key, cache_entry.access_count);
                Ok(Some(cache_entry.data))
            }
            None => {
                debug!("❌ Cache miss for {}", key);
                Ok(None)
            }
        }
    }

    /// 📊 Calculate intelligent TTL based on volatility
    fn calculate_intelligent_ttl(&self, volatility_metrics: Option<&VolatilityMetrics>) -> (u64, CacheTier) {
        match volatility_metrics {
            Some(metrics) => {
                let volatility_score = self.calculate_volatility_score(metrics);
                
                if volatility_score > 0.8 {
                    // High volatility - short TTL
                    (self.min_ttl, CacheTier::Hot)
                } else if volatility_score > 0.5 {
                    // Medium volatility - medium TTL
                    (self.base_ttl, CacheTier::Warm)
                } else if volatility_score > 0.2 {
                    // Low volatility - long TTL
                    (self.base_ttl * 2, CacheTier::Cold)
                } else {
                    // Very stable - very long TTL
                    (self.max_ttl, CacheTier::Frozen)
                }
            }
            None => {
                // Default TTL for unknown volatility
                (self.base_ttl, CacheTier::Warm)
            }
        }
    }

    /// 📈 Calculate volatility score from metrics
    fn calculate_volatility_score(&self, metrics: &VolatilityMetrics) -> f64 {
        let price_weight = 0.4;
        let volume_weight = 0.3;
        let holder_weight = 0.2;
        let liquidity_weight = 0.1;
        
        let price_volatility = (metrics.price_change_24h.abs() / 100.0).min(1.0);
        let volume_volatility = (metrics.volume_change_24h.abs() / 100.0).min(1.0);
        let holder_volatility = (metrics.holder_change_rate.abs() / 10.0).min(1.0);
        let liquidity_volatility = 1.0 - metrics.liquidity_stability.min(1.0);
        
        let weighted_score = price_volatility * price_weight
            + volume_volatility * volume_weight
            + holder_volatility * holder_weight
            + liquidity_volatility * liquidity_weight;
        
        weighted_score.min(1.0)
    }

    /// 🔥 Cache warming for popular tokens
    pub async fn warm_cache(&self, popular_tokens: &[String]) -> Result<()> {
        info!("🔥 Warming cache for {} popular tokens", popular_tokens.len());
        
        for token_address in popular_tokens {
            let key = format!("token_popular:{}", token_address);
            
            // Check if already cached
            if self.get_with_intelligence(&key).await?.is_none() {
                // Fetch and cache popular token data
                let mock_data = serde_json::json!({
                    "address": token_address,
                    "popular": true,
                    "warmed_at": Utc::now()
                });
                
                // Popular tokens get longer TTL
                let cache_entry = CacheEntry {
                    data: mock_data,
                    created_at: Utc::now(),
                    access_count: 0,
                    volatility_score: 0.3, // Assume popular tokens are more stable
                    ttl_seconds: self.base_ttl * 3, // 15 minutes for popular tokens
                    cache_tier: CacheTier::Warm,
                };
                
                let mut conn = self.redis_client.get_async_connection().await?;
                let serialized = serde_json::to_string(&cache_entry)?;
                let _: () = conn.set_ex(&key, serialized, cache_entry.ttl_seconds).await?;
            }
        }
        
        Ok(())
    }

    /// 🧹 Cache invalidation based on events
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<u32> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Get keys matching pattern
        let keys: Vec<String> = conn.keys(pattern).await?;
        let count = keys.len() as u32;
        
        if !keys.is_empty() {
            let _: () = conn.del(&keys).await?;
            info!("🧹 Invalidated {} cache entries matching pattern: {}", count, pattern);
        }
        
        Ok(count)
    }

    /// 📊 Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Get all cache keys
        let all_keys: Vec<String> = conn.keys("*").await?;
        let mut stats = CacheStats::default();
        
        stats.total_entries = all_keys.len() as u32;
        
        // Analyze cache entries
        for key in &all_keys {
            if let Ok(Some(serialized)) = conn.get::<_, Option<String>>(key).await {
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&serialized) {
                    stats.total_access_count += cache_entry.access_count;
                    
                    match cache_entry.cache_tier {
                        CacheTier::Hot => stats.hot_tier_count += 1,
                        CacheTier::Warm => stats.warm_tier_count += 1,
                        CacheTier::Cold => stats.cold_tier_count += 1,
                        CacheTier::Frozen => stats.frozen_tier_count += 1,
                    }
                    
                    // Calculate age
                    let age = Utc::now().signed_duration_since(cache_entry.created_at);
                    if age.num_seconds() > 0 {
                        stats.avg_age_seconds += age.num_seconds() as u64;
                    }
                }
            }
        }
        
        if stats.total_entries > 0 {
            stats.avg_age_seconds /= stats.total_entries as u64;
            stats.avg_access_count = stats.total_access_count as f64 / stats.total_entries as f64;
        }
        
        Ok(stats)
    }

    /// 🎯 Adaptive cache management
    pub async fn optimize_cache(&self) -> Result<()> {
        info!("🎯 Running adaptive cache optimization");
        
        let stats = self.get_cache_stats().await?;
        
        // If cache is getting full, clean up cold entries
        if stats.total_entries > 10000 {
            let cleaned = self.cleanup_cold_entries().await?;
            info!("🧹 Cleaned up {} cold cache entries", cleaned);
        }
        
        // Promote frequently accessed entries
        self.promote_hot_entries().await?;
        
        Ok(())
    }

    /// ❄️ Cleanup cold cache entries
    async fn cleanup_cold_entries(&self) -> Result<u32> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let keys: Vec<String> = conn.keys("*").await?;
        let mut cleaned = 0;
        
        for key in keys {
            if let Ok(Some(serialized)) = conn.get::<_, Option<String>>(&key).await {
                if let Ok(cache_entry) = serde_json::from_str::<CacheEntry>(&serialized) {
                    // Remove entries that are old and rarely accessed
                    let age = Utc::now().signed_duration_since(cache_entry.created_at);
                    if age.num_hours() > 2 && cache_entry.access_count < 2 {
                        let _: () = conn.del(&key).await?;
                        cleaned += 1;
                    }
                }
            }
        }
        
        Ok(cleaned)
    }

    /// 🔥 Promote frequently accessed entries
    async fn promote_hot_entries(&self) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let keys: Vec<String> = conn.keys("*").await?;
        
        for key in keys {
            if let Ok(Some(serialized)) = conn.get::<_, Option<String>>(&key).await {
                if let Ok(mut cache_entry) = serde_json::from_str::<CacheEntry>(&serialized) {
                    // Promote entries with high access count
                    if cache_entry.access_count > 10 && cache_entry.cache_tier != CacheTier::Hot {
                        cache_entry.cache_tier = CacheTier::Hot;
                        cache_entry.ttl_seconds = self.min_ttl; // Shorter TTL for hot data
                        
                        let updated_serialized = serde_json::to_string(&cache_entry)?;
                        let _: () = conn.set_ex(&key, updated_serialized, cache_entry.ttl_seconds).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// 📊 Cache statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheStats {
    pub total_entries: u32,
    pub hot_tier_count: u32,
    pub warm_tier_count: u32,
    pub cold_tier_count: u32,
    pub frozen_tier_count: u32,
    pub total_access_count: u32,
    pub avg_access_count: f64,
    pub avg_age_seconds: u64,
}
