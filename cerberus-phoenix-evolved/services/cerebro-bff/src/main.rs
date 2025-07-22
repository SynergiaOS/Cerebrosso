//! 🧠 Cerebro BFF - MVP Trading Decision API

use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use uuid::Uuid;

// MVP Structures
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub service: String,
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub qdrant_connection: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnipeResponse {
    pub id: Uuid,
    pub confidence: f32,
    pub action: String,
    pub reasoning: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnipeRequest {
    pub token_address: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsResponse {
    pub total_requests: u64,
    pub successful_decisions: u64,
    pub avg_confidence: f32,
    pub uptime_seconds: u64,
}

/// 🏥 Health check endpoint
async fn health_check() -> Result<ResponseJson<HealthResponse>, StatusCode> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = HealthResponse {
        service: "cerebro-bff".to_string(),
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        qdrant_connection: true, // Simplified for MVP
    };

    Ok(ResponseJson(response))
}

/// 🎯 Snipe trigger endpoint
async fn trigger_snipe(Json(request): Json<SnipeRequest>) -> Result<ResponseJson<SnipeResponse>, StatusCode> {
    info!("🎯 Snipe triggered for token: {}", request.token_address);

    // MVP: Generate mock decision with high confidence
    let decision_id = Uuid::new_v4();
    let confidence = 0.95; // High confidence for demo
    
    let response = SnipeResponse {
        id: decision_id,
        confidence,
        action: "BUY".to_string(),
        reasoning: format!("High-confidence snipe opportunity detected for token {}", request.token_address),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    info!("✅ Snipe decision generated: ID={}, Confidence={}", decision_id, confidence);
    Ok(ResponseJson(response))
}

/// 📊 Metrics endpoint
async fn metrics() -> Result<ResponseJson<MetricsResponse>, StatusCode> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = MetricsResponse {
        total_requests: 42,
        successful_decisions: 38,
        avg_confidence: 0.87,
        uptime_seconds: uptime,
    };

    Ok(ResponseJson(response))
}

/// 🚀 Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("🧠 Starting Cerebro BFF MVP v{}", env!("CARGO_PKG_VERSION"));

    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/trigger/snipe", post(trigger_snipe))
        .route("/metrics", get(metrics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Cerebro BFF MVP running on {}", addr);
    info!("📊 Endpoints: /health, /trigger/snipe, /metrics");

    axum::serve(listener, app).await?;

    Ok(())
}
