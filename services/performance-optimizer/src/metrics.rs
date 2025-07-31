//! 📈 Performance Optimizer Metrics

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizerMetrics {
    pub latency: LatencyStats,
    pub throughput: ThroughputStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyStats {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub avg_ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThroughputStats {
    pub requests_per_second: f64,
    pub total_requests: u64,
    pub error_rate: f64,
}

impl OptimizerMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
