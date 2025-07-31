//! 📊 Advanced Monitoring - Main Application

use anyhow::Result;
use std::sync::Arc;
use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, instrument};

use advanced_monitoring::{
    config::Config,
    distributed_tracing::TracingManager,
    anomaly_detector::AnomalyDetector,
    metrics_collector::MetricsCollector,
    alert_manager::AlertManager,
    health_checker::{HealthChecker, HealthStatus},
    MonitoringConfig, MonitoringLevel, SystemHealthStatus,
};

#[derive(Clone)]
struct AppState {
    tracing_manager: Arc<TracingManager>,
    anomaly_detector: Arc<AnomalyDetector>,
    metrics_collector: Arc<MetricsCollector>,
    alert_manager: Arc<AlertManager>,
    health_checker: Arc<HealthChecker>,
    monitoring_config: MonitoringConfig,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("advanced_monitoring=debug,info")
        .init();

    info!("📊 Starting Advanced Monitoring v3.0...");

    let config = Arc::new(Config {
        server: advanced_monitoring::config::ServerConfig {
            port: 8700,
            host: "0.0.0.0".to_string(),
        },
        tracing: advanced_monitoring::config::TracingConfig {
            jaeger_endpoint: "http://jaeger:14268/api/traces".to_string(),
            sampling_rate: 0.1,
            service_name: "cerberus-monitoring".to_string(),
            enable_console_export: true,
        },
        anomaly_detection: advanced_monitoring::config::AnomalyDetectionConfig {
            enable_real_time: true,
            detection_window_minutes: 15,
            sensitivity_threshold: 0.8,
            model_update_interval_hours: 24,
        },
        metrics: advanced_monitoring::config::MetricsConfig {
            collection_interval_seconds: 15,
            retention_days: 90,
            influxdb_url: "http://influxdb:8086".to_string(),
            prometheus_port: 9700,
        },
        alerting: advanced_monitoring::config::AlertingConfig {
            enable_email: false,
            enable_slack: false,
            enable_webhook: true,
            webhook_url: "http://swarm-coordinator:8090/alerts".to_string(),
            alert_cooldown_minutes: 15,
        },
        performance: advanced_monitoring::config::PerformanceConfig {
            enable_analysis: true,
            analysis_window_hours: 24,
            bottleneck_threshold: 0.8,
            enable_recommendations: true,
        },
        monitoring: advanced_monitoring::config::MonitoringConfig {
            metrics_enabled: true,
            prometheus_port: 9700,
            log_level: "info".to_string(),
            health_check_interval_seconds: 30,
        },
    });

    let tracing_manager = Arc::new(TracingManager::new(config.clone()).await?);
    let anomaly_detector = Arc::new(AnomalyDetector::new(config.clone()).await?);
    let metrics_collector = Arc::new(MetricsCollector::new(config.clone()).await?);
    let alert_manager = Arc::new(AlertManager::new(config.clone()).await?);
    let health_checker = Arc::new(HealthChecker::new(config.clone()).await?);

    let app_state = AppState {
        tracing_manager,
        anomaly_detector,
        metrics_collector,
        alert_manager,
        health_checker,
        monitoring_config: MonitoringConfig::default(),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_monitoring_status))
        .route("/tracing/stats", get(get_tracing_stats))
        .route("/anomalies/recent", get(get_recent_anomalies))
        .route("/anomalies/stats", get(get_anomaly_stats))
        .route("/system/health", get(get_system_health))
        .route("/config", get(get_monitoring_config))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 Advanced Monitoring server started on {}", addr);
    info!("📊 Monitoring features enabled:");
    info!("  🔍 Distributed Tracing: Enabled (Jaeger)");
    info!("  🤖 AI Anomaly Detection: Enabled");
    info!("  📈 Metrics Collection: Enabled");
    info!("  🚨 Real-time Alerting: Enabled");
    info!("  ⚡ Performance Analysis: Enabled");
    info!("  🏥 Health Monitoring: Enabled");

    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument]
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "advanced-monitoring",
        "version": "3.0.0",
        "monitoring_level": "Enterprise",
        "timestamp": chrono::Utc::now()
    }))
}

#[instrument(skip(state))]
async fn get_monitoring_status(State(state): State<AppState>) -> Json<Value> {
    let tracing_stats = state.tracing_manager.get_stats().await;
    let anomaly_stats = state.anomaly_detector.get_stats().await;
    
    Json(json!({
        "service": "advanced-monitoring",
        "monitoring_level": state.monitoring_config.level,
        "tracing": {
            "active_traces": tracing_stats.active_traces,
            "total_traces": tracing_stats.total_traces,
            "avg_duration_us": tracing_stats.avg_duration_us,
            "error_rate": tracing_stats.error_rate
        },
        "anomaly_detection": {
            "total_anomalies": anomaly_stats.total_anomalies,
            "detection_accuracy": anomaly_stats.detection_accuracy,
            "avg_detection_time_ms": anomaly_stats.avg_detection_time_ms
        },
        "timestamp": chrono::Utc::now()
    }))
}

#[instrument(skip(state))]
async fn get_tracing_stats(State(state): State<AppState>) -> Json<Value> {
    let stats = state.tracing_manager.get_stats().await;
    Json(json!(stats))
}

#[instrument(skip(state))]
async fn get_recent_anomalies(State(state): State<AppState>) -> Json<Value> {
    let anomalies = state.anomaly_detector.get_recent_anomalies(10).await;
    Json(json!({
        "anomalies": anomalies,
        "count": anomalies.len()
    }))
}

#[instrument(skip(state))]
async fn get_anomaly_stats(State(state): State<AppState>) -> Json<Value> {
    let stats = state.anomaly_detector.get_stats().await;
    Json(json!(stats))
}

#[instrument(skip(state))]
async fn get_system_health(State(state): State<AppState>) -> Json<SystemHealthStatus> {
    // Mock system health status
    let mut service_statuses = std::collections::HashMap::new();
    service_statuses.insert("swarm-coordinator".to_string(), HealthStatus::Healthy);
    service_statuses.insert("agent-strateg".to_string(), HealthStatus::Healthy);
    service_statuses.insert("context-engine".to_string(), HealthStatus::Healthy);
    service_statuses.insert("synk".to_string(), HealthStatus::Healthy);
    service_statuses.insert("chainguardia".to_string(), HealthStatus::Healthy);
    service_statuses.insert("performance-optimizer".to_string(), HealthStatus::Healthy);
    service_statuses.insert("security-hardening".to_string(), HealthStatus::Healthy);
    service_statuses.insert("hft-ninja".to_string(), HealthStatus::Warning);
    service_statuses.insert("cerebro-bff".to_string(), HealthStatus::Healthy);
    
    let system_health = SystemHealthStatus {
        overall_status: HealthStatus::Healthy,
        service_statuses,
        active_alerts: 1,
        detected_anomalies: 0,
        performance_score: 0.94,
        last_check: chrono::Utc::now(),
    };
    
    Json(system_health)
}

#[instrument(skip(state))]
async fn get_monitoring_config(State(state): State<AppState>) -> Json<MonitoringConfig> {
    Json(state.monitoring_config.clone())
}
