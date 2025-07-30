//! 🧠 Cerebro-BFF - AI Orchestration Service
//! Fixed version with proper error handling

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, error};

// Simplified modules for now
mod config;
mod context_engine;
mod ai_agent;
mod qdrant_client;
mod vault_client;

use config::Config;
use context_engine::ContextEngine;
use ai_agent::AIAgent;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub context_engine: Arc<ContextEngine>,
    pub ai_agent: Arc<AIAgent>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🧠 Starting Cerebro-BFF v2.0...");

    // Load configuration
    let config = Config::load()?;
    
    // Initialize core components
    let context_engine = Arc::new(ContextEngine::new(Arc::new(config.clone())).await?);
    let ai_agent = Arc::new(AIAgent::new(&config)?);

    let app_state = AppState {
        config,
        context_engine,
        ai_agent,
    };

    // Setup routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/analyze", post(analyze_token))
        .route("/api/decision", post(make_trading_decision))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("🚀 Cerebro-BFF listening on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "cerebro-bff",
        "version": "2.0.0",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

async fn analyze_token(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔍 Analyzing token: {:?}", payload.get("mint"));
    
    // TODO: Implement actual analysis
    Ok(Json(serde_json::json!({
        "status": "analyzed",
        "recommendation": "hold",
        "confidence": 0.75
    })))
}

async fn make_trading_decision(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🤖 Making trading decision for: {:?}", payload.get("mint"));
    
    // TODO: Implement actual decision making
    Ok(Json(serde_json::json!({
        "decision": "buy",
        "amount": 100.0,
        "confidence": 0.8,
        "reasoning": "Strong volume and momentum signals"
    })))
}
