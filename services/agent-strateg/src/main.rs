//! 👑 Agent-Strateg Main Entry Point
//! 
//! Główny punkt wejścia dla Agent-Strateg (CEO) w architekturze Hive Mind

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agent_strateg::{
    Config,
    AgentStrateg,
    StrategState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Inicjalizacja logowania
    init_tracing()?;
    
    info!("👑 Starting Cerberus Phoenix v3.0 - Agent-Strateg (CEO)");
    info!("🎯 Strategic Decision Making & Goal Decomposition Agent");
    
    // Wczytanie konfiguracji
    let config = Arc::new(Config::from_env()?);
    info!("✅ Configuration loaded successfully");
    
    // Walidacja konfiguracji
    config.validate()?;
    info!("✅ Configuration validated");
    
    // Utworzenie Agent-Strateg
    let mut agent = AgentStrateg::new(config.clone()).await?;
    info!("✅ Agent-Strateg created successfully");
    
    // Uruchomienie HTTP API
    let api_handle = start_api_server(config.clone(), agent.clone()).await?;
    
    // Uruchomienie Agent-Strateg
    agent.start().await?;
    info!("🚀 Agent-Strateg started and ready for strategic coordination");
    
    // Wyświetlenie informacji o agencie
    display_agent_info(&config).await;
    
    // Oczekiwanie na sygnał shutdown
    wait_for_shutdown_signal().await;
    
    // Graceful shutdown
    info!("🛑 Shutdown signal received, stopping Agent-Strateg...");
    agent.shutdown().await?;
    
    // Zatrzymanie API server
    api_handle.abort();
    
    info!("✅ Agent-Strateg shutdown completed");
    Ok(())
}

/// Inicjalizacja systemu logowania
fn init_tracing() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent_strateg=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}

/// Uruchomienie HTTP API server
async fn start_api_server(
    config: Arc<Config>,
    agent: AgentStrateg,
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
        agent: Arc<AgentStrateg>,
    }
    
    let app_state = AppState {
        agent: Arc::new(agent),
    };
    
    // API routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/metrics", get(get_metrics))
        .route("/goals", get(list_goals))
        .route("/goals", post(create_goal))
        .route("/goals/:id/decompose", post(decompose_goal))
        .route("/decisions", post(synthesize_decision))
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
        "service": "agent-strateg",
        "version": "3.0.0",
        "role": "CEO",
        "timestamp": chrono::Utc::now()
    })))
}

/// Status endpoint
async fn get_status(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let agent_state = state.agent.get_state().await;
    
    Ok(Json(json!({
        "agent_state": agent_state,
        "agent_type": "Strateg",
        "role": "CEO",
        "decision_weight": 0.4,
        "timestamp": chrono::Utc::now()
    })))
}

/// Metrics endpoint
async fn get_metrics(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let metrics = state.agent.get_metrics().await;
    
    Ok(Json(json!({
        "metrics": metrics,
        "timestamp": chrono::Utc::now()
    })))
}

/// List goals endpoint
async fn list_goals(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let goals = state.agent.get_active_goals().await;
    
    Ok(Json(json!({
        "goals": goals,
        "total": goals.len()
    })))
}

/// Create goal endpoint
async fn create_goal(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy Goal creation będzie gotowy
    Ok(Json(json!({
        "status": "created",
        "goal_id": uuid::Uuid::new_v4()
    })))
}

/// Decompose goal endpoint
async fn decompose_goal(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy Goal decomposition będzie gotowy
    Ok(Json(json!({
        "status": "decomposed",
        "sub_goals": []
    })))
}

/// Synthesize decision endpoint
async fn synthesize_decision(State(_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    // Implementation będzie dodana gdy Decision synthesis będzie gotowy
    Ok(Json(json!({
        "status": "synthesized",
        "decision_id": uuid::Uuid::new_v4()
    })))
}

/// Wyświetlenie informacji o agencie
async fn display_agent_info(config: &Config) {
    info!("👑 ===== CERBERUS PHOENIX v3.0 - AGENT-STRATEG (CEO) =====");
    info!("🎯 Role: Strategic Decision Making & Goal Decomposition");
    info!("🌐 API Server: http://{}:{}", config.server.host, config.server.port);
    info!("🔗 SwarmCoordinator: {}", config.swarm.coordinator_url);
    info!("🤖 Agent ID: {}", config.swarm.agent_id);
    info!("⚖️ Decision Weight: {:.1}%", config.strategy.decision_weight * 100.0);
    info!("🎯 Max Concurrent Goals: {}", config.strategy.max_concurrent_goals);
    info!("🧠 Primary AI Model: {}", config.ai.primary_model);
    info!("🛡️ Risk Tolerance: {:.1}%", config.strategy.risk_tolerance * 100.0);
    info!("📊 Monitoring: Enabled");
    info!("🔐 Security: Enabled");
    info!("👑 =====================================================");
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
