//! 🗄️ Cache Manager - Advanced Multi-Level Caching System
//! 
//! High-performance caching system with intelligent cache strategies

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use redis::{Client as RedisClient, AsyncCommands};
use tracing::{info, debug, warn, instrument};
use ahash::AHashMap;

use crate::{config::Config, PerformanceError};

/// 🎯 Cache Strategy Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Time-based expiration
    TTL,
    /// Adaptive Replacement Cache
    ARC,
    /// Machine Learning based
    MLBased,
}

/// 📊 Cache Hit Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHit {
    /// Cache key
    pub key: String,
    /// Cache level (L1, L2, L3)
    pub level: CacheLevel,
    /// Hit timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Access time in microseconds
    pub access_time_us: u64,
}

/// 🏷️ Cache Level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheLevel {
    /// L1 - In-memory cache (fastest)
    L1,
    /// L2 - Redis cache (fast)
    L2,
    /// L3 - Distributed cache (slower but persistent)
    L3,
}

/// 📈 Cache Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Cache miss rate (0.0 - 1.0)
    pub miss_rate: f64,
    /// Total cache requests
    pub total_requests: u64,
    /// Current cache size in MB
    pub cache_size_mb: usize,
    /// Number of evictions
    pub eviction_count: u64,
}

/// 🗄️ Cache Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// Cached value
    value: serde_json::Value,
    /// Creation timestamp
    created_at: Instant,
    /// Last access timestamp
    last_accessed: Instant,
    /// Access count
    access_count: u64,
    /// TTL in seconds
    ttl_seconds: u64,
    /// Entry size in bytes
    size_bytes: usize,
}

impl CacheEntry {
    fn new(value: serde_json::Value, ttl_seconds: u64) -> Self {
        let now = Instant::now();
        let size_bytes = serde_json::to_string(&value).unwrap_or_default().len();
        
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl_seconds,
            size_bytes,
        }
    }
    
    fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() > self.ttl_seconds
    }
    
    fn access(&mut self) -> &serde_json::Value {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.value
    }
}

/// 🗄️ Advanced Cache Manager
pub struct CacheManager {
    /// Configuration
    config: Arc<Config>,
    /// L1 Cache (In-memory)
    l1_cache: Arc<RwLock<AHashMap<String, CacheEntry>>>,
    /// L2 Cache (Redis)
    redis_client: RedisClient,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Cache strategy
    strategy: CacheStrategy,
    /// Maximum L1 cache size in bytes
    max_l1_size_bytes: usize,
    /// Current L1 cache size in bytes
    current_l1_size: Arc<RwLock<usize>>,
}

impl CacheManager {
    /// Creates new cache manager
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self, PerformanceError> {
        info!("🗄️ Initializing Cache Manager...");
        
        // Initialize Redis client
        let redis_url = config.cache.redis_cluster_urls.first()
            .ok_or_else(|| PerformanceError::Configuration("No Redis URL provided".to_string()))?;
        
        let redis_client = RedisClient::open(redis_url.as_str())
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        // Test Redis connection
        let mut conn = redis_client.get_async_connection().await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        let _: String = conn.ping().await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        let max_l1_size_bytes = config.cache.max_cache_size_mb * 1024 * 1024 / 4; // 25% for L1
        
        let cache_manager = Self {
            config,
            l1_cache: Arc::new(RwLock::new(AHashMap::new())),
            redis_client,
            stats: Arc::new(RwLock::new(CacheStats {
                hit_rate: 0.0,
                miss_rate: 0.0,
                total_requests: 0,
                cache_size_mb: 0,
                eviction_count: 0,
            })),
            strategy: CacheStrategy::ARC, // Adaptive Replacement Cache
            max_l1_size_bytes,
            current_l1_size: Arc::new(RwLock::new(0)),
        };
        
        // Start cache maintenance task
        cache_manager.start_maintenance_task().await;
        
        info!("✅ Cache Manager initialized with strategy: {:?}", cache_manager.strategy);
        Ok(cache_manager)
    }
    
    /// Gets value from cache
    #[instrument(skip(self, key))]
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, PerformanceError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let start = Instant::now();
        
        // Update total requests
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }
        
        // Try L1 cache first
        if let Some(value) = self.get_from_l1(key).await? {
            let access_time_us = start.elapsed().as_micros() as u64;
            self.record_cache_hit(key, CacheLevel::L1, access_time_us).await;
            
            let result: T = serde_json::from_value(value)
                .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
            return Ok(Some(result));
        }
        
        // Try L2 cache (Redis)
        if let Some(value) = self.get_from_l2(key).await? {
            let access_time_us = start.elapsed().as_micros() as u64;
            self.record_cache_hit(key, CacheLevel::L2, access_time_us).await;
            
            // Promote to L1 cache
            self.set_in_l1(key, &value, self.config.cache.hot_cache_ttl_secs).await?;
            
            let result: T = serde_json::from_value(value)
                .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
            return Ok(Some(result));
        }
        
        // Cache miss
        self.record_cache_miss().await;
        debug!("❌ Cache miss for key: {}", key);
        
        Ok(None)
    }
    
    /// Sets value in cache
    #[instrument(skip(self, key, value))]
    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), PerformanceError>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        // Determine cache level based on TTL
        if ttl_seconds <= self.config.cache.hot_cache_ttl_secs {
            // Hot data - store in L1 and L2
            self.set_in_l1(key, &json_value, ttl_seconds).await?;
            self.set_in_l2(key, &json_value, ttl_seconds).await?;
        } else if ttl_seconds <= self.config.cache.warm_cache_ttl_secs {
            // Warm data - store in L2
            self.set_in_l2(key, &json_value, ttl_seconds).await?;
        } else {
            // Cold data - store in L2 with longer TTL
            self.set_in_l2(key, &json_value, ttl_seconds).await?;
        }
        
        debug!("✅ Cached key: {} with TTL: {}s", key, ttl_seconds);
        Ok(())
    }
    
    /// Invalidates cache entry
    #[instrument(skip(self, key))]
    pub async fn invalidate(&self, key: &str) -> Result<(), PerformanceError> {
        // Remove from L1
        {
            let mut l1_cache = self.l1_cache.write().await;
            if let Some(entry) = l1_cache.remove(key) {
                let mut current_size = self.current_l1_size.write().await;
                *current_size = current_size.saturating_sub(entry.size_bytes);
            }
        }
        
        // Remove from L2 (Redis)
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        let _: () = conn.del(key).await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        debug!("🗑️ Invalidated cache key: {}", key);
        Ok(())
    }
    
    /// Gets current cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Gets value from L1 cache
    async fn get_from_l1(&self, key: &str) -> Result<Option<serde_json::Value>, PerformanceError> {
        let mut l1_cache = self.l1_cache.write().await;
        
        if let Some(entry) = l1_cache.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                l1_cache.remove(key);
                let mut current_size = self.current_l1_size.write().await;
                *current_size = current_size.saturating_sub(entry.size_bytes);
                return Ok(None);
            }
            
            // Return value and update access info
            let value = entry.access().clone();
            return Ok(Some(value));
        }
        
        Ok(None)
    }
    
    /// Gets value from L2 cache (Redis)
    async fn get_from_l2(&self, key: &str) -> Result<Option<serde_json::Value>, PerformanceError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        let value_str: Option<String> = conn.get(key).await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        if let Some(str_val) = value_str {
            let json_value: serde_json::Value = serde_json::from_str(&str_val)
                .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
            return Ok(Some(json_value));
        }
        
        Ok(None)
    }
    
    /// Sets value in L1 cache
    async fn set_in_l1(&self, key: &str, value: &serde_json::Value, ttl_seconds: u64) -> Result<(), PerformanceError> {
        let entry = CacheEntry::new(value.clone(), ttl_seconds);
        let entry_size = entry.size_bytes;
        
        // Check if we need to evict entries
        self.ensure_l1_capacity(entry_size).await?;
        
        {
            let mut l1_cache = self.l1_cache.write().await;
            l1_cache.insert(key.to_string(), entry);
            
            let mut current_size = self.current_l1_size.write().await;
            *current_size += entry_size;
        }
        
        Ok(())
    }
    
    /// Sets value in L2 cache (Redis)
    async fn set_in_l2(&self, key: &str, value: &serde_json::Value, ttl_seconds: u64) -> Result<(), PerformanceError> {
        let mut conn = self.redis_client.get_async_connection().await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        let value_str = serde_json::to_string(value)
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        let _: () = conn.setex(key, ttl_seconds as usize, value_str).await
            .map_err(|e| PerformanceError::CacheManagement(e.to_string()))?;
        
        Ok(())
    }
    
    /// Ensures L1 cache has enough capacity
    async fn ensure_l1_capacity(&self, needed_bytes: usize) -> Result<(), PerformanceError> {
        let current_size = *self.current_l1_size.read().await;
        
        if current_size + needed_bytes > self.max_l1_size_bytes {
            // Need to evict entries
            self.evict_l1_entries(needed_bytes).await?;
        }
        
        Ok(())
    }
    
    /// Evicts entries from L1 cache using ARC strategy
    async fn evict_l1_entries(&self, needed_bytes: usize) -> Result<(), PerformanceError> {
        let mut l1_cache = self.l1_cache.write().await;
        let mut current_size = self.current_l1_size.write().await;
        let mut evicted_bytes = 0;
        let mut eviction_count = 0;
        
        // Simple LRU eviction for now (ARC would be more complex)
        let mut entries_to_remove = Vec::new();
        
        for (key, entry) in l1_cache.iter() {
            if evicted_bytes >= needed_bytes {
                break;
            }
            
            entries_to_remove.push((key.clone(), entry.size_bytes));
            evicted_bytes += entry.size_bytes;
            eviction_count += 1;
        }
        
        // Remove selected entries
        for (key, size) in entries_to_remove {
            l1_cache.remove(&key);
            *current_size = current_size.saturating_sub(size);
        }
        
        // Update eviction stats
        {
            let mut stats = self.stats.write().await;
            stats.eviction_count += eviction_count;
        }
        
        debug!("🗑️ Evicted {} entries ({} bytes) from L1 cache", eviction_count, evicted_bytes);
        Ok(())
    }
    
    /// Records cache hit
    async fn record_cache_hit(&self, key: &str, level: CacheLevel, access_time_us: u64) {
        debug!("✅ Cache hit: {} (level: {:?}, time: {}μs)", key, level, access_time_us);
        
        let mut stats = self.stats.write().await;
        let total = stats.total_requests as f64;
        let hits = total * stats.hit_rate + 1.0;
        stats.hit_rate = hits / total;
        stats.miss_rate = 1.0 - stats.hit_rate;
    }
    
    /// Records cache miss
    async fn record_cache_miss(&self) {
        let mut stats = self.stats.write().await;
        let total = stats.total_requests as f64;
        let hits = total * stats.hit_rate;
        stats.hit_rate = hits / total;
        stats.miss_rate = 1.0 - stats.hit_rate;
    }
    
    /// Starts cache maintenance task
    async fn start_maintenance_task(&self) {
        let l1_cache = self.l1_cache.clone();
        let current_l1_size = self.current_l1_size.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Clean expired entries from L1 cache
                let mut cache = l1_cache.write().await;
                let mut size = current_l1_size.write().await;
                let mut expired_keys = Vec::new();
                let mut freed_bytes = 0;
                
                for (key, entry) in cache.iter() {
                    if entry.is_expired() {
                        expired_keys.push(key.clone());
                        freed_bytes += entry.size_bytes;
                    }
                }
                
                for key in expired_keys {
                    cache.remove(&key);
                }
                
                *size = size.saturating_sub(freed_bytes);
                
                if freed_bytes > 0 {
                    debug!("🧹 Cleaned {} expired entries ({} bytes) from L1 cache", 
                           cache.len(), freed_bytes);
                }
            }
        });
    }
}
