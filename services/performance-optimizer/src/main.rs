//! ⚡ Performance Optimizer - Main Application

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

use performance_optimizer::{
    config::Config,
    latency_optimizer::LatencyOptimizer,
    cache_manager::CacheManager,
    accuracy_enhancer::AccuracyEnhancer,
    load_balancer::LoadBalancer,
    PerformanceTargets,
};

#[derive(Clone)]
struct AppState {
    latency_optimizer: Arc<LatencyOptimizer>,
    cache_manager: Arc<CacheManager>,
    accuracy_enhancer: Arc<AccuracyEnhancer>,
    load_balancer: Arc<LoadBalancer>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("performance_optimizer=debug,info")
        .init();

    info!("⚡ Starting Performance Optimizer v3.0...");

    let config = Arc::new(Config {
        server: performance_optimizer::config::ServerConfig {
            port: 8500,
            host: "0.0.0.0".to_string(),
            workers: 8,
        },
        optimization: performance_optimizer::config::OptimizationConfig {
            target_latency_ms: 100.0,
            target_accuracy: 0.848,
            target_throughput: 1000.0,
            enable_caching: true,
            enable_load_balancing: true,
            enable_ml_optimization: true,
        },
        cache: performance_optimizer::config::CacheConfig {
            redis_cluster_urls: vec!["redis://redis:6379".to_string()],
            max_cache_size_mb: 1024,
            hot_cache_ttl_secs: 60,
            warm_cache_ttl_secs: 300,
            cold_cache_ttl_secs: 3600,
        },
        load_balancer: performance_optimizer::config::LoadBalancerConfig {
            strategy: "round_robin".to_string(),
            health_check_interval_secs: 5,
            max_failures: 5,
            timeout_ms: 5000,
        },
        monitoring: performance_optimizer::config::MonitoringConfig {
            metrics_enabled: true,
            prometheus_port: 9500,
            monitoring_interval_secs: 1,
            alert_thresholds: performance_optimizer::config::AlertThresholds {
                latency_ms: 150.0,
                accuracy: 0.8,
                cache_hit_rate: 0.9,
                error_rate: 0.05,
            },
        },
    });

    let latency_optimizer = Arc::new(LatencyOptimizer::new(config.clone()).await?);
    let cache_manager = Arc::new(CacheManager::new(config.clone()).await?);
    let accuracy_enhancer = Arc::new(AccuracyEnhancer::new(config.clone()).await?);
    let load_balancer = Arc::new(LoadBalancer::new(config.clone()).await?);

    let app_state = AppState {
        latency_optimizer,
        cache_manager,
        accuracy_enhancer,
        load_balancer,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/metrics", get(get_metrics))
        .route("/performance", get(get_performance))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 Performance Optimizer server started on {}", addr);
    info!("📊 Metrics available at http://{}/metrics", addr);
    info!("🎯 Target: <{}ms latency, {:.1}% accuracy", 
          config.optimization.target_latency_ms, 
          config.optimization.target_accuracy * 100.0);

    axum::serve(listener, app).await?;
    Ok(())
}

#[instrument]
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "performance-optimizer",
        "version": "3.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

#[instrument(skip(state))]
async fn get_status(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "service": "performance-optimizer",
        "targets": PerformanceTargets::default(),
        "version": "3.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

#[instrument(skip(state))]
async fn get_metrics(State(state): State<AppState>) -> Json<Value> {
    let latency_metrics = state.latency_optimizer.get_metrics().await;
    let accuracy_metrics = state.accuracy_enhancer.get_metrics().await;
    let cache_stats = state.cache_manager.get_stats().await;
    let load_balancer_health = state.load_balancer.get_health().await;
    
    Json(json!({
        "latency": latency_metrics,
        "accuracy": accuracy_metrics,
        "cache": cache_stats,
        "load_balancer": load_balancer_health,
        "timestamp": chrono::Utc::now()
    }))
}

#[instrument(skip(state))]
async fn get_performance(State(state): State<AppState>) -> Json<Value> {
    let latency_metrics = state.latency_optimizer.get_metrics().await;
    let accuracy_metrics = state.accuracy_enhancer.get_metrics().await;
    let targets = PerformanceTargets::default();
    
    let performance_score = calculate_performance_score(&latency_metrics, &accuracy_metrics, &targets);
    
    Json(json!({
        "performance_score": performance_score,
        "targets_met": {
            "latency": latency_metrics.p95_ms <= targets.target_latency_ms,
            "accuracy": accuracy_metrics.current_accuracy >= targets.target_accuracy
        },
        "timestamp": chrono::Utc::now()
    }))
}

fn calculate_performance_score(
    latency_metrics: &performance_optimizer::latency_optimizer::LatencyMetrics,
    accuracy_metrics: &performance_optimizer::accuracy_enhancer::AccuracyMetrics,
    targets: &PerformanceTargets,
) -> f64 {
    let latency_score = if latency_metrics.p95_ms <= targets.target_latency_ms {
        1.0
    } else {
        (targets.target_latency_ms / latency_metrics.p95_ms).min(1.0)
    };
    
    let accuracy_score = (accuracy_metrics.current_accuracy / targets.target_accuracy).min(1.0);
    
    (latency_score + accuracy_score) / 2.0
}
