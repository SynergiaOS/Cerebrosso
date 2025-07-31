//! ⚡ Performance Optimizer - Ultra-Low Latency Optimization Library
//! 
//! Advanced performance optimization for achieving sub-100ms latency and 84.8% decision accuracy

pub mod config;
pub mod latency_optimizer;
pub mod cache_manager;
pub mod load_balancer;
pub mod accuracy_enhancer;
pub mod connection_pool;
pub mod request_router;
pub mod performance_monitor;
pub mod ml_optimizer;
pub mod metrics;

// Core exports
pub use config::Config;
pub use latency_optimizer::{LatencyOptimizer, LatencyTarget, LatencyMetrics};
pub use cache_manager::{CacheManager, CacheStrategy, CacheHit, CacheStats};
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy, ServerHealth};
pub use accuracy_enhancer::{AccuracyEnhancer, AccuracyTarget, AccuracyMetrics};
pub use connection_pool::{ConnectionPool, PoolConfig, PoolStats};
pub use request_router::{RequestRouter, RoutingStrategy, RouteMetrics};
pub use performance_monitor::{PerformanceMonitor, PerformanceAlert, PerformanceReport};
pub use ml_optimizer::{MLOptimizer, OptimizationModel, PredictionAccuracy};
pub use metrics::{OptimizerMetrics, LatencyStats, ThroughputStats};

/// 🎯 Core Performance Optimizer Result Type
pub type OptimizerResult<T> = Result<T, PerformanceError>;

/// ❌ Performance Error Types
#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    #[error("Latency optimization error: {0}")]
    LatencyOptimization(String),
    
    #[error("Cache management error: {0}")]
    CacheManagement(String),
    
    #[error("Load balancing error: {0}")]
    LoadBalancing(String),
    
    #[error("Accuracy enhancement error: {0}")]
    AccuracyEnhancement(String),
    
    #[error("Connection pool error: {0}")]
    ConnectionPool(String),
    
    #[error("Request routing error: {0}")]
    RequestRouting(String),
    
    #[error("Performance monitoring error: {0}")]
    PerformanceMonitoring(String),
    
    #[error("ML optimization error: {0}")]
    MLOptimization(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

/// ⚡ Performance Targets
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceTargets {
    /// Target latency in milliseconds
    pub target_latency_ms: f64,
    /// Target decision accuracy (0.0 - 1.0)
    pub target_accuracy: f64,
    /// Target throughput (requests per second)
    pub target_throughput: f64,
    /// Target cache hit rate (0.0 - 1.0)
    pub target_cache_hit_rate: f64,
    /// Target uptime (0.0 - 1.0)
    pub target_uptime: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            target_latency_ms: 100.0,
            target_accuracy: 0.848,
            target_throughput: 1000.0,
            target_cache_hit_rate: 0.95,
            target_uptime: 0.999,
        }
    }
}

/// 📊 Performance Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceStatus {
    /// Current latency metrics
    pub latency: LatencyMetrics,
    /// Current accuracy metrics
    pub accuracy: AccuracyMetrics,
    /// Current cache statistics
    pub cache: CacheStats,
    /// Current load balancer status
    pub load_balancer: ServerHealth,
    /// Performance targets
    pub targets: PerformanceTargets,
    /// Overall performance score (0.0 - 1.0)
    pub performance_score: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// ⚡ Performance Constants
pub mod constants {
    /// Target sub-100ms latency
    pub const TARGET_LATENCY_MS: f64 = 100.0;
    
    /// Target 84.8% decision accuracy (SWE Bench)
    pub const TARGET_ACCURACY: f64 = 0.848;
    
    /// Maximum cache size in MB
    pub const MAX_CACHE_SIZE_MB: usize = 1024;
    
    /// Default connection pool size
    pub const DEFAULT_POOL_SIZE: usize = 100;
    
    /// Maximum concurrent requests
    pub const MAX_CONCURRENT_REQUESTS: usize = 10000;
    
    /// Performance monitoring interval (seconds)
    pub const MONITORING_INTERVAL_SECS: u64 = 1;
    
    /// Cache TTL for hot data (seconds)
    pub const HOT_CACHE_TTL_SECS: u64 = 60;
    
    /// Cache TTL for warm data (seconds)
    pub const WARM_CACHE_TTL_SECS: u64 = 300;
    
    /// Cache TTL for cold data (seconds)
    pub const COLD_CACHE_TTL_SECS: u64 = 3600;
    
    /// Load balancer health check interval (seconds)
    pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 5;
    
    /// Circuit breaker failure threshold
    pub const CIRCUIT_BREAKER_THRESHOLD: usize = 5;
    
    /// Request timeout in milliseconds
    pub const REQUEST_TIMEOUT_MS: u64 = 5000;
    
    /// ML model retraining interval (hours)
    pub const ML_RETRAIN_INTERVAL_HOURS: u64 = 24;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8500,
                host: "localhost".to_string(),
                workers: 8,
            },
            optimization: config::OptimizationConfig {
                target_latency_ms: 100.0,
                target_accuracy: 0.848,
                target_throughput: 1000.0,
                enable_caching: true,
                enable_load_balancing: true,
                enable_ml_optimization: true,
            },
            cache: config::CacheConfig {
                redis_cluster_urls: vec!["redis://localhost:6379".to_string()],
                max_cache_size_mb: 1024,
                hot_cache_ttl_secs: 60,
                warm_cache_ttl_secs: 300,
                cold_cache_ttl_secs: 3600,
            },
            load_balancer: config::LoadBalancerConfig {
                strategy: "round_robin".to_string(),
                health_check_interval_secs: 5,
                max_failures: 5,
                timeout_ms: 5000,
            },
            monitoring: config::MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: 9500,
                monitoring_interval_secs: 1,
                alert_thresholds: config::AlertThresholds {
                    latency_ms: 150.0,
                    accuracy: 0.8,
                    cache_hit_rate: 0.9,
                    error_rate: 0.05,
                },
            },
        })
    }
    
    /// Create mock performance metrics
    pub fn create_mock_performance_metrics() -> PerformanceStatus {
        PerformanceStatus {
            latency: LatencyMetrics {
                p50_ms: 45.0,
                p95_ms: 85.0,
                p99_ms: 120.0,
                avg_ms: 52.0,
                max_ms: 150.0,
            },
            accuracy: AccuracyMetrics {
                current_accuracy: 0.852,
                rolling_accuracy: 0.848,
                confidence_score: 0.95,
                prediction_count: 1000,
            },
            cache: CacheStats {
                hit_rate: 0.94,
                miss_rate: 0.06,
                total_requests: 10000,
                cache_size_mb: 512,
                eviction_count: 100,
            },
            load_balancer: ServerHealth {
                healthy_servers: 8,
                total_servers: 10,
                avg_response_time_ms: 25.0,
                error_rate: 0.02,
            },
            targets: PerformanceTargets::default(),
            performance_score: 0.92,
            last_updated: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_performance_targets_default() {
        let targets = PerformanceTargets::default();
        
        assert_eq!(targets.target_latency_ms, 100.0);
        assert_eq!(targets.target_accuracy, 0.848);
        assert_eq!(targets.target_throughput, 1000.0);
        assert_eq!(targets.target_cache_hit_rate, 0.95);
        assert_eq!(targets.target_uptime, 0.999);
    }
    
    #[tokio::test]
    async fn test_performance_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.server.port, 8500);
        assert_eq!(config.optimization.target_latency_ms, 100.0);
        assert_eq!(config.optimization.target_accuracy, 0.848);
        assert!(config.optimization.enable_caching);
        assert!(config.optimization.enable_load_balancing);
    }
    
    #[tokio::test]
    async fn test_performance_metrics_creation() {
        let metrics = create_mock_performance_metrics();
        
        assert!(metrics.latency.p95_ms < 100.0);
        assert!(metrics.accuracy.current_accuracy > 0.848);
        assert!(metrics.cache.hit_rate > 0.9);
        assert!(metrics.performance_score > 0.9);
    }
    
    #[test]
    fn test_performance_constants() {
        assert_eq!(constants::TARGET_LATENCY_MS, 100.0);
        assert_eq!(constants::TARGET_ACCURACY, 0.848);
        assert_eq!(constants::MAX_CONCURRENT_REQUESTS, 10000);
        assert_eq!(constants::DEFAULT_POOL_SIZE, 100);
    }
}
