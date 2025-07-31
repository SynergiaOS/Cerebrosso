//! 📊 Monitoring Metrics

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    pub system: SystemMetrics,
    pub application: ApplicationMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub request_count: u64,
    pub error_rate: f64,
    pub response_time_p95: f64,
    pub active_connections: u32,
}

impl MonitoringMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
