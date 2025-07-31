//! 🔧 Performance Optimizer Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub optimization: OptimizationConfig,
    pub cache: CacheConfig,
    pub load_balancer: LoadBalancerConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub target_latency_ms: f64,
    pub target_accuracy: f64,
    pub target_throughput: f64,
    pub enable_caching: bool,
    pub enable_load_balancing: bool,
    pub enable_ml_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_cluster_urls: Vec<String>,
    pub max_cache_size_mb: usize,
    pub hot_cache_ttl_secs: u64,
    pub warm_cache_ttl_secs: u64,
    pub cold_cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub strategy: String,
    pub health_check_interval_secs: u64,
    pub max_failures: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_port: u16,
    pub monitoring_interval_secs: u64,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub latency_ms: f64,
    pub accuracy: f64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
}
