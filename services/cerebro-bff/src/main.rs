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
use tracing::{info, warn, error};

// Simplified modules for now
mod config;
mod context_engine;
mod ai_agent;
mod qdrant_client;
mod vault_client;
mod infisical_client;
mod secure_config;
// mod wallet_manager; // Disabled due to Solana dependency conflicts
mod mock_infisical;
mod mock_secure_config;

use config::Config;
use context_engine::ContextEngine;
use ai_agent::AIAgent;
use infisical_client::{InfisicalClient, InfisicalClientBuilder};
use secure_config::SecureConfigManager;
// use wallet_manager::WalletManager; // Disabled due to Solana dependency conflicts
use mock_secure_config::MockSecureConfigManager;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// 🔐 Secure Configuration Trait
#[async_trait]
pub trait SecureConfigTrait: Send + Sync {
    async fn get_config_summary(&self) -> Result<serde_json::Value>;
    async fn validate_config(&self) -> Result<secure_config::ValidationReport>;
    async fn load_config(&self) -> Result<secure_config::SecureConfig>;
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub context_engine: Arc<ContextEngine>,
    pub ai_agent: Arc<AIAgent>,
    pub secure_config: Arc<dyn SecureConfigTrait>,
    // pub wallet_manager: Arc<WalletManager>, // Disabled due to Solana dependency conflicts
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    info!("🧠 Starting Cerebro-BFF v2.0...");

    // Load configuration
    let config = Config::load()?;
    
    // Initialize core components
    let context_engine = Arc::new(ContextEngine::new(Arc::new(config.clone())).await?);
    let ai_agent = Arc::new(AIAgent::new(&config)?);

    // Initialize Infisical Client (if configured)
    let infisical_client = if let (Ok(api_url), Ok(project_id), Ok(environment), Ok(client_id), Ok(client_secret)) = (
        std::env::var("INFISICAL_API_URL"),
        std::env::var("INFISICAL_PROJECT_ID"),
        std::env::var("INFISICAL_ENVIRONMENT"),
        std::env::var("INFISICAL_CLIENT_ID"),
        std::env::var("INFISICAL_CLIENT_SECRET"),
    ) {
        info!("🔐 Initializing Infisical integration");
        Some(Arc::new(InfisicalClientBuilder::new()
            .api_url(api_url)
            .project_id(project_id)
            .environment(environment)
            .client_credentials(client_id, client_secret)
            .cache_ttl(300) // 5 minutes
            .build()?))
    } else {
        warn!("⚠️ Infisical not configured - using environment variables only");
        None
    };

    // Initialize Secure Config Manager (with mock fallback)
    let secure_config: Arc<dyn SecureConfigTrait> = if let Some(client) = &infisical_client {
        Arc::new(SecureConfigManager::new(client.clone()))
    } else {
        warn!("⚠️ Using mock secure config manager for development");
        Arc::new(MockSecureConfigManager::new())
    };

    // Initialize Wallet Manager (disabled due to Solana dependency conflicts)
    // let wallet_manager = Arc::new(WalletManager::new(secure_config.clone()));

    let app_state = AppState {
        config,
        context_engine,
        ai_agent,
        secure_config,
        // wallet_manager,
    };

    // Setup routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/analyze", post(analyze_token))
        .route("/api/analyze-token", post(analyze_token_for_hft)) // HFT-Ninja integration
        .route("/api/optimize-context", post(optimize_context))
        .route("/api/feedback", post(update_feedback))
        .route("/api/decision", post(make_trading_decision))
        .route("/api/secure-config/summary", get(get_secure_config_summary))
        .route("/api/secure-config/validate", get(validate_secure_config))
        // .route("/api/wallets/summary", get(get_wallet_summary))
        // .route("/api/wallets/validate", get(validate_wallets))
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

// 📊 Token Profile from HFT-Ninja
#[derive(Debug, Deserialize)]
struct TokenProfile {
    mint_address: String,
    symbol: String,
    name: String,
    market_cap: Option<f64>,
    liquidity_usd: Option<f64>,
    volume_24h: Option<f64>,
    price_change_24h: Option<f64>,
    holder_count: Option<u32>,
    dev_allocation_percentage: Option<f64>,
    freeze_authority: Option<bool>,
    mint_authority: Option<bool>,
    team_doxxed: Option<bool>,
    contract_verified: Option<bool>,
    risk_signals: Vec<String>,
    opportunity_signals: Vec<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

// 🎯 AI Trading Decision Response
#[derive(Debug, Serialize)]
struct AITradingDecision {
    action: String,
    confidence: f64,
    reasoning: String,
    risk_assessment: String,
    position_size_percentage: f64,
    stop_loss_percentage: Option<f64>,
    take_profit_percentage: Option<f64>,
    urgency_level: u8,
    strategy_type: String,
}

// 🧠 HFT-Ninja Integration Endpoint
async fn analyze_token_for_hft(
    State(state): State<AppState>,
    Json(token_profile): Json<TokenProfile>,
) -> Result<Json<AITradingDecision>, StatusCode> {
    info!("🧠 AI analysis request from HFT-Ninja for token: {}", token_profile.symbol);

    // Use Context Engine for intelligent analysis
    let context_result = state.context_engine.get_weighted_context(
        &format!("analyze {} token with signals: {:?}", token_profile.symbol, token_profile.risk_signals),
        10
    ).await;

    let decision = match context_result {
        Ok(weighted_signals) => {
            info!("📊 Context Engine provided {} weighted signals", weighted_signals.len());

            // Apply Apriori rules for risk assessment
            let mut all_signals = token_profile.risk_signals.clone();
            all_signals.extend(token_profile.opportunity_signals.clone());

            let apriori_recommendations = state.context_engine.apply_apriori_rules(&all_signals).await
                .unwrap_or_else(|_| vec![]);

            // Intelligent decision making based on Context Engine
            let risk_score = token_profile.risk_signals.len() as f64;
            let opportunity_score = token_profile.opportunity_signals.len() as f64;

            // Enhanced risk assessment with Context Engine
            let enhanced_risk_score = if !apriori_recommendations.is_empty() {
                risk_score + 2.0 // Apriori rules detected high-risk patterns
            } else {
                risk_score
            };

            let action = if enhanced_risk_score > 3.0 {
                "Avoid"
            } else if opportunity_score > enhanced_risk_score {
                "Buy"
            } else {
                "Hold"
            };

            let confidence = if !weighted_signals.is_empty() {
                let avg_weight = weighted_signals.iter()
                    .map(|s| s.tf_idf_weight)
                    .sum::<f64>() / weighted_signals.len() as f64;
                (avg_weight / 5.0).min(0.95) // Normalize to 0-0.95
            } else {
                (opportunity_score / (enhanced_risk_score + opportunity_score + 1.0)).min(0.8)
            };

            let reasoning = if !apriori_recommendations.is_empty() {
                format!("Context Engine analysis with Apriori rules: {}. Risk signals: {:?}, Opportunity signals: {:?}",
                        apriori_recommendations.join(", "), token_profile.risk_signals, token_profile.opportunity_signals)
            } else {
                format!("Context Engine analysis: {} weighted signals processed. Risk: {}, Opportunity: {}",
                        weighted_signals.len(), risk_score, opportunity_score)
            };

            AITradingDecision {
                action: action.to_string(),
                confidence,
                reasoning,
                risk_assessment: if enhanced_risk_score > 3.0 { "High" } else if enhanced_risk_score > 1.0 { "Medium" } else { "Low" }.to_string(),
                position_size_percentage: if confidence > 0.7 { 3.0 } else if confidence > 0.5 { 2.0 } else { 1.0 },
                stop_loss_percentage: Some(if enhanced_risk_score > 2.0 { 5.0 } else { 10.0 }),
                take_profit_percentage: Some(if confidence > 0.8 { 50.0 } else { 25.0 }),
                urgency_level: calculate_urgency_level(&token_profile),
                strategy_type: "context_engine_analysis".to_string(),
            }
        }
        Err(e) => {
            error!("❌ Context Engine analysis failed: {}", e);
            // Fallback to simple rule-based decision
            simple_fallback_decision(&token_profile)
        }
    };

    info!("✅ AI Decision: {} with {:.1}% confidence", decision.action, decision.confidence * 100.0);
    Ok(Json(decision))
}

// 📊 Context Optimization Endpoint
async fn optimize_context(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<String>, StatusCode> {
    info!("📊 Context optimization request");

    if let Some(token_profile) = request.get("token_profile") {
        let query = format!("optimize context for token analysis: {}",
                          token_profile.get("symbol").and_then(|s| s.as_str()).unwrap_or("unknown"));

        match state.context_engine.get_weighted_context(&query, 5).await {
            Ok(signals) => {
                match state.context_engine.optimize_context_for_llm(&signals).await {
                    Ok(optimized) => {
                        info!("✅ Context optimized: {} characters", optimized.len());
                        Ok(Json(optimized))
                    }
                    Err(e) => {
                        error!("❌ Context optimization failed: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to get weighted context: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// 🔄 Feedback Update Endpoint
async fn update_feedback(
    State(state): State<AppState>,
    Json(feedback): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Feedback update received");

    // Update TF-IDF weights based on performance
    if let (Some(token_address), Some(actual_result)) = (
        feedback.get("token_address").and_then(|v| v.as_str()),
        feedback.get("actual_result")
    ) {
        if let Some(profit_loss) = actual_result.get("profit_loss_percentage").and_then(|v| v.as_f64()) {
            // Update weights based on performance
            let performance_delta = profit_loss / 100.0; // Normalize to -1.0 to 1.0

            if let Err(e) = state.context_engine.update_tf_idf_weights(token_address, performance_delta).await {
                error!("❌ Failed to update TF-IDF weights: {}", e);
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": "feedback_processed",
        "timestamp": chrono::Utc::now()
    })))
}

// ⚡ Calculate urgency level
fn calculate_urgency_level(token_profile: &TokenProfile) -> u8 {
    let mut urgency = 5u8;

    if token_profile.risk_signals.len() > 3 {
        urgency = urgency.saturating_add(2);
    }

    if let Some(volume) = token_profile.volume_24h {
        if volume > 1_000_000.0 {
            urgency = urgency.saturating_add(1);
        }
    }

    if let Some(price_change) = token_profile.price_change_24h {
        if price_change.abs() > 50.0 {
            urgency = urgency.saturating_add(2);
        }
    }

    urgency.min(10)
}

/// 🔐 Get Secure Configuration Summary
async fn get_secure_config_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔐 Getting secure configuration summary");

    match state.secure_config.get_config_summary().await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => {
            error!("❌ Failed to get config summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 🔍 Validate Secure Configuration
async fn validate_secure_config(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔍 Validating secure configuration");

    match state.secure_config.validate_config().await {
        Ok(report) => Ok(Json(report.get_summary())),
        Err(e) => {
            error!("❌ Failed to validate config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Wallet endpoints disabled due to Solana dependency conflicts
/*
/// 💰 Get Wallet Summary
async fn get_wallet_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("💰 Getting wallet summary");

    let summary = state.wallet_manager.get_wallet_summary().await;
    Ok(Json(summary))
}

/// 🔍 Validate Wallets
async fn validate_wallets(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔍 Validating wallet configuration");

    match state.wallet_manager.validate_wallets().await {
        Ok(report) => Ok(Json(report.get_summary())),
        Err(e) => {
            error!("❌ Failed to validate wallets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
*/

// 🛡️ Simple fallback decision
fn simple_fallback_decision(token_profile: &TokenProfile) -> AITradingDecision {
    let risk_score = token_profile.risk_signals.len() as f64;
    let opportunity_score = token_profile.opportunity_signals.len() as f64;

    let action = if risk_score > 3.0 {
        "Avoid"
    } else if opportunity_score > risk_score {
        "Buy"
    } else {
        "Hold"
    };

    let confidence = (opportunity_score / (risk_score + opportunity_score + 1.0)).min(0.8);

    AITradingDecision {
        action: action.to_string(),
        confidence,
        reasoning: "Simple fallback analysis due to Context Engine unavailability".to_string(),
        risk_assessment: if risk_score > 2.0 { "High" } else { "Medium" }.to_string(),
        position_size_percentage: if confidence > 0.6 { 2.0 } else { 1.0 },
        stop_loss_percentage: Some(10.0),
        take_profit_percentage: Some(25.0),
        urgency_level: calculate_urgency_level(token_profile),
        strategy_type: "fallback_analysis".to_string(),
    }
}
