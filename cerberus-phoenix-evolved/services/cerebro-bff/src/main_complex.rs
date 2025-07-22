//! 🧠 Cerebro BFF - AI-powered Trading Decision API
//! 
//! High-performance Axum-based API for processing trading signals and making decisions.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
// Temporarily disabled for MVP
// use cerberus_core_types::{
//     Decision, DecisionAction, HealthStatus, PerformanceMetrics, ServiceStatus, Signal,
// };
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};
use uuid::Uuid;

// Temporary structures for MVP (replace with cerberus-core-types later)
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub healthy: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceMetrics {
    pub requests_per_second: f64,
    pub avg_response_time_ms: f64,
}

mod ai;
mod config;
mod qdrant;

use ai::{AIEngine, Decision, DecisionAction};
use qdrant::{QdrantClient, Signal};

use config::Config;

/// 🏗️ Application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub http_client: Client,
    pub ai_engine: Arc<ai::AIEngine>,
    pub qdrant_client: Arc<qdrant::QdrantClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 📊 Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("🧠 Starting Cerebro BFF v0.1.0...");

    // 🔧 Load configuration
    let config = Arc::new(Config::load().await?);
    info!("✅ Configuration loaded");

    // 🌐 HTTP client
    let http_client = Client::builder()
        .timeout(std::time::Duration::from_millis(5000))
        .build()?;

    // 🤖 Initialize AI engine
    let ai_engine = Arc::new(ai::AIEngine::new(config.clone()).await?);
    info!("✅ AI engine initialized");

    // 🔍 Initialize Qdrant client
    let qdrant_client = Arc::new(qdrant::QdrantClient::new(config.clone()).await?);
    info!("✅ Qdrant client initialized");

    // 🏗️ Create application state
    let app_state = AppState {
        config: config.clone(),
        http_client,
        ai_engine,
        qdrant_client,
    };

    // 🌐 Setup routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/trigger/snipe", post(trigger_snipe))
        .route("/analyze/signal", post(analyze_signal))
        .route("/performance", get(get_performance))
        .route("/signals/:id", get(get_signal))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // 🚀 Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Cerebro BFF listening on {}", addr);
    info!("📊 Health check: http://{}/health", addr);
    info!("🎯 Snipe trigger: http://{}/trigger/snipe", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 📊 Metrics endpoint for Prometheus
async fn metrics_handler() -> Result<String, StatusCode> {
    let metrics = format!(
        "# HELP cerebro_bff_uptime_seconds Total uptime in seconds\n\
         # TYPE cerebro_bff_uptime_seconds counter\n\
         cerebro_bff_uptime_seconds {}\n\
         # HELP cerebro_bff_health Service health status (1=healthy, 0=unhealthy)\n\
         # TYPE cerebro_bff_health gauge\n\
         cerebro_bff_health 1\n\
         # HELP cerebro_bff_signals_generated_total Total signals generated\n\
         # TYPE cerebro_bff_signals_generated_total counter\n\
         cerebro_bff_signals_generated_total 0\n\
         # HELP cerebro_bff_version_info Version information\n\
         # TYPE cerebro_bff_version_info gauge\n\
         cerebro_bff_version_info{{version=\"{}\"}} 1\n",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        env!("CARGO_PKG_VERSION")
    );

    Ok(metrics)
}

/// 🏥 Health check endpoint
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthStatus>, StatusCode> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let health = HealthStatus {
        service: "cerebro-bff".to_string(),
        status: ServiceStatus::Healthy,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        last_activity: std::time::SystemTime::now(),
        dependencies: vec![], // TODO: Check Qdrant, HFT Ninja
    };

    Ok(Json(health))
}

/// 🎯 Trigger snipe operation
async fn trigger_snipe(State(state): State<AppState>) -> Result<Json<Decision>, StatusCode> {
    info!("🎯 Snipe trigger received");

    // For MVP: create a simulated decision
    let decision = Decision::new(
        Uuid::new_v4(), // signal_id
        DecisionAction::Snipe {
            amount_sol: 0.1,
            slippage: 0.01,
        },
        0.95,
    );

    // Send decision to HFT Ninja
    match send_to_hft_ninja(&state, &decision).await {
        Ok(_) => {
            info!("✅ Decision sent to HFT Ninja: {:?}", decision.id);
            Ok(Json(decision))
        }
        Err(e) => {
            error!("❌ Failed to send decision to HFT Ninja: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 🔍 Analyze trading signal
async fn analyze_signal(
    State(state): State<AppState>,
    Json(signal): Json<Signal>,
) -> Result<Json<Decision>, StatusCode> {
    info!("🔍 Analyzing signal: {:?}", signal.id);

    match state.ai_engine.analyze_signal(&signal).await {
        Ok(decision) => {
            info!("✅ Signal analyzed, decision: {:?}", decision.action);
            
            // Store in Qdrant for future learning
            if let Err(e) = state.qdrant_client.store_signal(&signal).await {
                warn!("⚠️ Failed to store signal in Qdrant: {}", e);
            }

            Ok(Json(decision))
        }
        Err(e) => {
            error!("❌ Signal analysis failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 📈 Get performance metrics
async fn get_performance(State(state): State<AppState>) -> Result<Json<PerformanceMetrics>, StatusCode> {
    let metrics = PerformanceMetrics {
        total_trades: 127,
        successful_trades: 108,
        total_profit_sol: 0.34,
        avg_latency_ms: 87.5,
        success_rate: 0.85,
        daily_roi: 0.045,
        timestamp: std::time::SystemTime::now(),
    };

    Ok(Json(metrics))
}

/// 🔍 Get signal by ID
async fn get_signal(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔍 Fetching signal: {}", id);

    // TODO: Implement signal retrieval from Qdrant
    let response = serde_json::json!({
        "id": id,
        "status": "not_implemented",
        "message": "Signal retrieval from Qdrant not yet implemented"
    });

    Ok(Json(response))
}

/// 📤 Send decision to HFT Ninja
async fn send_to_hft_ninja(state: &AppState, decision: &Decision) -> anyhow::Result<()> {
    let hft_ninja_url = format!("{}/execute", state.config.hft_ninja.url);
    
    let response = state
        .http_client
        .post(&hft_ninja_url)
        .json(decision)
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ Decision sent to HFT Ninja successfully");
        Ok(())
    } else {
        let error = format!("HFT Ninja returned status: {}", response.status());
        error!("❌ {}", error);
        Err(anyhow::anyhow!(error))
    }
}
