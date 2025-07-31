//! 🔧 Advanced Monitoring Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub tracing: TracingConfig,
    pub anomaly_detection: AnomalyDetectionConfig,
    pub metrics: MetricsConfig,
    pub alerting: AlertingConfig,
    pub performance: PerformanceConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub jaeger_endpoint: String,
    pub sampling_rate: f64,
    pub service_name: String,
    pub enable_console_export: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionConfig {
    pub enable_real_time: bool,
    pub detection_window_minutes: u64,
    pub sensitivity_threshold: f64,
    pub model_update_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval_seconds: u64,
    pub retention_days: u64,
    pub influxdb_url: String,
    pub prometheus_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enable_email: bool,
    pub enable_slack: bool,
    pub enable_webhook: bool,
    pub webhook_url: String,
    pub alert_cooldown_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_analysis: bool,
    pub analysis_window_hours: u64,
    pub bottleneck_threshold: f64,
    pub enable_recommendations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_port: u16,
    pub log_level: String,
    pub health_check_interval_seconds: u64,
}
