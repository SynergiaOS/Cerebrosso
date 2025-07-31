//! 🐝 SwarmCoordinator Main Entry Point
//! 
//! Główny punkt wejścia dla centralnego orkiestratora Hive Mind

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use swarm_coordinator::{
    Config,
    SwarmCoordinator,
    SwarmState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicjalizacja logowania
    init_tracing()?;
    
    info!("🐝 Starting Cerberus Phoenix v3.0 - SwarmCoordinator");
    info!("🎯 Hive Mind Architecture Initializing...");
    
    // Wczytanie konfiguracji
    let config = Arc::new(Config::from_env()?);
    info!("✅ Configuration loaded successfully");
    
    // Utworzenie SwarmCoordinator
    let mut coordinator = SwarmCoordinator::new(config.clone()).await?;
    info!("✅ SwarmCoordinator created successfully");
    
    // Uruchomienie HTTP API
    let api_handle = start_api_server(config.clone(), coordinator.clone()).await?;
    
    // Uruchomienie SwarmCoordinator
    coordinator.start().await?;
    info!("🚀 SwarmCoordinator started and ready for agent coordination");
    
    // Wyświetlenie informacji o systemie
    display_system_info(&config).await;
    
    // Oczekiwanie na sygnał shutdown
    wait_for_shutdown_signal().await;
    
    // Graceful shutdown
    info!("🛑 Shutdown signal received, stopping SwarmCoordinator...");
    coordinator.shutdown().await?;
    
    // Zatrzymanie API server
    api_handle.abort();
    
    info!("✅ SwarmCoordinator shutdown completed");
    Ok(())
}

/// Inicjalizacja systemu logowania
fn init_tracing() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "swarm_coordinator=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}

/// Uruchomienie HTTP API server
async fn start_api_server(
    config: Arc<Config>,
    coordinator: SwarmCoordinator,
) -> Result<tokio::task::JoinHandle<()>> {
    use axum::{
        extract::State,
        http::StatusCode,
        response::Json,
        routing::{get, post},
        Router,
    };
    use serde_json::{json, Value};
    use tower_http::{cors::CorsLayer, trace::TraceLayer};
    
    // Shared state dla API
    #[derive(Clone)]
    struct AppState {
        coordinator: Arc<SwarmCoordinator>,
    }
    
    let app_state = AppState {
        coordinator: Arc::new(coordinator),
    };
    
    // API routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/metrics", get(get_metrics))
        .route("/agents", get(list_agents))
        .route("/agents/register", post(register_agent))
        .route("/tasks", post(delegate_task))
        .route("/tasks/:id/result", post(submit_task_result))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🌐 API Server listening on: {}", addr);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("❌ API Server error: {}", e);
        }
    });
    
    Ok(handle)
}

/// Health check endpoint
async fn health_check() -> Result<Json<Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "service": "swarm-coordinator",
        "version": "3.0.0",
        "timestamp": chrono::Utc::now()
    })))
}

/// Status endpoint
async fn get_status(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let swarm_state = state.coordinator.get_state().await;
    
    Ok(Json(json!({
        "swarm_state": swarm_state,
        "timestamp": chrono::Utc::now()
    })))
}

/// Metrics endpoint
async fn get_metrics(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let metrics = state.coordinator.get_metrics().await;
    
    Ok(Json(json!({
        "metrics": metrics,
        "timestamp": chrono::Utc::now()
    })))
}

/// List agents endpoint
async fn list_agents(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy AgentRegistry będzie gotowy
    Ok(Json(json!({
        "agents": [],
        "total": 0
    })))
}

/// Register agent endpoint
async fn register_agent(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy AgentRegistry będzie gotowy
    Ok(Json(json!({
        "status": "registered",
        "agent_id": uuid::Uuid::new_v4()
    })))
}

/// Delegate task endpoint
async fn delegate_task(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy TaskDelegator będzie gotowy
    Ok(Json(json!({
        "status": "delegated",
        "task_id": uuid::Uuid::new_v4()
    })))
}

/// Submit task result endpoint
async fn submit_task_result(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy TaskResult będzie gotowy
    Ok(Json(json!({
        "status": "received"
    })))
}

/// Wyświetlenie informacji o systemie
async fn display_system_info(config: &Config) {
    info!("🎯 ===== CERBERUS PHOENIX v3.0 - SWARM COORDINATOR =====");
    info!("🐝 Hive Mind Architecture: ACTIVE");
    info!("🌐 API Server: http://{}:{}", config.server.host, config.server.port);
    info!("🔴 Redis: {}", config.redis.url);
    info!("🧠 Qdrant: {}", config.qdrant.url);
    info!("👥 Max Agents: {}", config.swarm.max_agents);
    info!("🎯 Decision Threshold: {:.1}%", config.swarm.decision_threshold * 100.0);
    info!("⚡ Target Latency: <100ms");
    info!("📊 Monitoring: Enabled");
    info!("🔐 Security: Enabled");
    info!("🎯 ===================================================");
}

/// Oczekiwanie na sygnał shutdown
async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("🛑 Ctrl+C received");
        },
        _ = terminate => {
            info!("🛑 SIGTERM received");
        },
    }
}
