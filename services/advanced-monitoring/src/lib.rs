//! 📊 Advanced Monitoring & Observability Library
//! 
//! Enterprise-grade monitoring with distributed tracing, AI anomaly detection, and comprehensive observability

pub mod config;
pub mod distributed_tracing;
pub mod anomaly_detector;
pub mod metrics_collector;
pub mod alert_manager;
pub mod dashboard_generator;
pub mod log_aggregator;
pub mod performance_analyzer;
pub mod health_checker;
pub mod metrics;

// Core exports
pub use config::Config;
pub use distributed_tracing::{TracingManager, TraceSpan, TraceContext};
pub use anomaly_detector::{AnomalyDetector, AnomalyType, AnomalyAlert, AnomalyModel};
pub use metrics_collector::{MetricsCollector, MetricType, MetricValue, MetricSeries};
pub use alert_manager::{AlertManager, Alert, AlertRule, AlertChannel};
pub use dashboard_generator::{DashboardGenerator, Dashboard, Widget, Chart};
pub use log_aggregator::{LogAggregator, LogEntry, LogLevel, LogFilter};
pub use performance_analyzer::{PerformanceAnalyzer, PerformanceReport, Bottleneck};
pub use health_checker::{HealthChecker, HealthStatus, HealthCheck};
pub use metrics::{MonitoringMetrics, SystemMetrics, ApplicationMetrics};

/// 🎯 Core Monitoring Result Type
pub type MonitoringResult<T> = Result<T, MonitoringError>;

/// ❌ Monitoring Error Types
#[derive(Debug, thiserror::Error)]
pub enum MonitoringError {
    #[error("Distributed tracing error: {0}")]
    DistributedTracing(String),
    
    #[error("Anomaly detection error: {0}")]
    AnomalyDetection(String),
    
    #[error("Metrics collection error: {0}")]
    MetricsCollection(String),
    
    #[error("Alert management error: {0}")]
    AlertManagement(String),
    
    #[error("Dashboard generation error: {0}")]
    DashboardGeneration(String),
    
    #[error("Log aggregation error: {0}")]
    LogAggregation(String),
    
    #[error("Performance analysis error: {0}")]
    PerformanceAnalysis(String),
    
    #[error("Health check error: {0}")]
    HealthCheck(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

/// 📊 Monitoring Level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum MonitoringLevel {
    /// Basic monitoring
    Basic = 1,
    /// Standard monitoring
    Standard = 2,
    /// Advanced monitoring
    Advanced = 3,
    /// Enterprise monitoring
    Enterprise = 4,
    /// Maximum monitoring
    Maximum = 5,
}

/// 📈 Monitoring Configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitoringConfig {
    /// Monitoring level
    pub level: MonitoringLevel,
    /// Enable distributed tracing
    pub enable_distributed_tracing: bool,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Enable real-time alerting
    pub enable_real_time_alerting: bool,
    /// Enable performance analysis
    pub enable_performance_analysis: bool,
    /// Metrics retention period in days
    pub metrics_retention_days: u64,
    /// Trace retention period in days
    pub trace_retention_days: u64,
    /// Log retention period in days
    pub log_retention_days: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            level: MonitoringLevel::Enterprise,
            enable_distributed_tracing: true,
            enable_anomaly_detection: true,
            enable_real_time_alerting: true,
            enable_performance_analysis: true,
            metrics_retention_days: 90,
            trace_retention_days: 30,
            log_retention_days: 365,
        }
    }
}

/// 📊 System Health Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemHealthStatus {
    /// Overall health status
    pub overall_status: HealthStatus,
    /// Service health statuses
    pub service_statuses: std::collections::HashMap<String, HealthStatus>,
    /// Active alerts count
    pub active_alerts: u32,
    /// Detected anomalies count
    pub detected_anomalies: u32,
    /// Performance score (0.0 - 1.0)
    pub performance_score: f64,
    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// 📊 Monitoring Constants
pub mod constants {
    /// Default metrics collection interval (seconds)
    pub const DEFAULT_METRICS_INTERVAL: u64 = 15;
    
    /// Default trace sampling rate (0.0 - 1.0)
    pub const DEFAULT_TRACE_SAMPLING_RATE: f64 = 0.1;
    
    /// Maximum metrics batch size
    pub const MAX_METRICS_BATCH_SIZE: usize = 1000;
    
    /// Default anomaly detection window (minutes)
    pub const DEFAULT_ANOMALY_WINDOW_MINUTES: u64 = 15;
    
    /// Maximum alert history size
    pub const MAX_ALERT_HISTORY: usize = 10000;
    
    /// Default dashboard refresh interval (seconds)
    pub const DEFAULT_DASHBOARD_REFRESH_INTERVAL: u64 = 30;
    
    /// Maximum log entry size (bytes)
    pub const MAX_LOG_ENTRY_SIZE: usize = 65536;
    
    /// Default health check interval (seconds)
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: u64 = 30;
    
    /// Performance analysis window (hours)
    pub const PERFORMANCE_ANALYSIS_WINDOW_HOURS: u64 = 24;
    
    /// Monitoring metrics update interval (seconds)
    pub const MONITORING_METRICS_INTERVAL: u64 = 60;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8700,
                host: "localhost".to_string(),
            },
            tracing: config::TracingConfig {
                jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
                sampling_rate: 0.1,
                service_name: "cerberus-monitoring".to_string(),
                enable_console_export: true,
            },
            anomaly_detection: config::AnomalyDetectionConfig {
                enable_real_time: true,
                detection_window_minutes: 15,
                sensitivity_threshold: 0.8,
                model_update_interval_hours: 24,
            },
            metrics: config::MetricsConfig {
                collection_interval_seconds: 15,
                retention_days: 90,
                influxdb_url: "http://localhost:8086".to_string(),
                prometheus_port: 9700,
            },
            alerting: config::AlertingConfig {
                enable_email: false,
                enable_slack: false,
                enable_webhook: true,
                webhook_url: "http://localhost:8090/alerts".to_string(),
                alert_cooldown_minutes: 15,
            },
            performance: config::PerformanceConfig {
                enable_analysis: true,
                analysis_window_hours: 24,
                bottleneck_threshold: 0.8,
                enable_recommendations: true,
            },
            monitoring: config::MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: 9700,
                log_level: "info".to_string(),
                health_check_interval_seconds: 30,
            },
        })
    }
    
    /// Create a mock system health status
    pub fn create_mock_health_status() -> SystemHealthStatus {
        let mut service_statuses = std::collections::HashMap::new();
        service_statuses.insert("swarm-coordinator".to_string(), HealthStatus::Healthy);
        service_statuses.insert("agent-strateg".to_string(), HealthStatus::Healthy);
        service_statuses.insert("context-engine".to_string(), HealthStatus::Healthy);
        service_statuses.insert("hft-ninja".to_string(), HealthStatus::Warning);
        
        SystemHealthStatus {
            overall_status: HealthStatus::Healthy,
            service_statuses,
            active_alerts: 2,
            detected_anomalies: 1,
            performance_score: 0.92,
            last_check: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        
        assert_eq!(config.level, MonitoringLevel::Enterprise);
        assert!(config.enable_distributed_tracing);
        assert!(config.enable_anomaly_detection);
        assert!(config.enable_real_time_alerting);
        assert_eq!(config.metrics_retention_days, 90);
    }
    
    #[tokio::test]
    async fn test_monitoring_level_ordering() {
        assert!(MonitoringLevel::Maximum > MonitoringLevel::Enterprise);
        assert!(MonitoringLevel::Enterprise > MonitoringLevel::Advanced);
        assert!(MonitoringLevel::Advanced > MonitoringLevel::Standard);
        assert!(MonitoringLevel::Standard > MonitoringLevel::Basic);
    }
    
    #[tokio::test]
    async fn test_monitoring_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.server.port, 8700);
        assert_eq!(config.tracing.service_name, "cerberus-monitoring");
        assert!(config.anomaly_detection.enable_real_time);
        assert_eq!(config.metrics.collection_interval_seconds, 15);
    }
    
    #[tokio::test]
    async fn test_health_status_creation() {
        let health_status = create_mock_health_status();
        
        assert_eq!(health_status.overall_status, HealthStatus::Healthy);
        assert_eq!(health_status.service_statuses.len(), 4);
        assert_eq!(health_status.active_alerts, 2);
        assert!(health_status.performance_score > 0.9);
    }
}
