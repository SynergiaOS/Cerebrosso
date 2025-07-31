//! 🚀 HFT-Ninja v2.0 - Cerberus Phoenix Ultra-Low Latency Engine
//! 
//! Implementacja strategii "Certainty-First HFT" z modułami:
//! - Fee & Tip Optimizer
//! - Redundant RPC Broadcaster  
//! - Transaction Simulator & Backrunner

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};

mod execution;
mod rpc;
mod simulation;
mod config;
mod cerebro_integration;
mod monitoring;

use execution::fee_optimizer::FeeOptimizer;
use config::Config;
use cerebro_integration::{CerebroClient, TokenProfile, AITradingDecision};
use monitoring::{TradingMetricsManager, DashboardApi};
use monitoring::dashboard_api::DashboardState;

/// 🏗️ Application State
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub fee_optimizer: Arc<FeeOptimizer>,
    pub cerebro_client: Arc<CerebroClient>,
    pub metrics_manager: Arc<TradingMetricsManager>,
}

/// 🏥 Health Response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    timestamp: i64,
}

/// 💰 Fee Optimization Request
#[derive(Deserialize)]
struct FeeOptimizationRequest {
    strategy: String,
    amount_sol: f64,
    urgency_level: Option<u8>, // 1-10 scale
}

/// 💰 Fee Optimization Response
#[derive(Serialize)]
struct FeeOptimizationResponse {
    optimal_tip_lamports: u64,
    confidence_score: f64,
    estimated_inclusion_time_ms: u64,
    strategy_multiplier: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 📝 Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting HFT-Ninja v2.0 - Certainty-First HFT Engine");

    // 🔧 Load configuration
    let config = Arc::new(Config::from_env()?);
    
    // 🏗️ Initialize core modules
    let fee_optimizer = Arc::new(FeeOptimizer::new(config.clone()).await?);
    
    // Initialize Cerebro-BFF Integration
    let cerebro_client = Arc::new(CerebroClient::new(config.clone())?);

    // Initialize Trading Metrics Manager
    let metrics_manager = Arc::new(TradingMetricsManager::new()?);

    // 🏗️ Build application state
    let app_state = AppState {
        config: config.clone(),
        fee_optimizer,
        cerebro_client,
        metrics_manager: metrics_manager.clone(),
    };

    // 🌐 Setup routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/optimize-fee", post(optimize_fee))
        .route("/api/analyze-token", post(analyze_token_with_ai))
        .route("/api/metrics", get(get_metrics))
        .with_state(app_state);

    // Add dashboard routes
    let dashboard_state = DashboardState {
        metrics_manager: metrics_manager.clone(),
        performance_analyzer: Arc::new(monitoring::PerformanceAnalyzer::new(0.05)), // 5% risk-free rate
    };

    let app = app.merge(DashboardApi::create_router(dashboard_state));

    // 🚀 Start server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 HFT-Ninja v2.0 listening on http://{}", addr);
    info!("💰 Fee optimization endpoint: http://{}/api/optimize-fee", addr);
    info!("📊 Metrics endpoint: http://{}/api/metrics", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// 🏥 Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "hft-ninja-v2".to_string(),
        version: "2.0.0".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// 💰 Fee optimization endpoint
async fn optimize_fee(
    State(state): State<AppState>,
    Json(request): Json<FeeOptimizationRequest>,
) -> Result<Json<FeeOptimizationResponse>, StatusCode> {
    info!("💰 Fee optimization request for strategy: {}", request.strategy);

    match state.fee_optimizer.get_optimal_jito_tip(&request.strategy, request.amount_sol, request.urgency_level).await {
        Ok((tip_lamports, confidence, inclusion_time, multiplier)) => {
            Ok(Json(FeeOptimizationResponse {
                optimal_tip_lamports: tip_lamports,
                confidence_score: confidence,
                estimated_inclusion_time_ms: inclusion_time,
                strategy_multiplier: multiplier,
            }))
        }
        Err(e) => {
            error!("❌ Fee optimization failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 📊 Metrics endpoint
async fn get_metrics() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "active",
        "uptime_seconds": 0, // TODO: Implement uptime tracking
        "requests_processed": 0, // TODO: Implement request counter
        "average_response_time_ms": 0.0 // TODO: Implement response time tracking
    }))
}

/// 🧠 AI Token Analysis endpoint
async fn analyze_token_with_ai(
    State(state): State<AppState>,
    Json(token_profile): Json<TokenProfile>,
) -> Result<Json<AITradingDecision>, StatusCode> {
    info!("🧠 AI analysis request for token: {}", token_profile.symbol);

    match state.cerebro_client.get_trading_decision(token_profile, "piranha_surf").await {
        Ok(decision) => {
            info!("✅ AI decision: {:?} with {:.1}% confidence",
                  decision.action, decision.confidence * 100.0);
            Ok(Json(decision))
        }
        Err(e) => {
            error!("❌ AI analysis failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
