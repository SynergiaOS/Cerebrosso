//! 🐺 Projekt Cerberus Phoenix v2.0 - HFT-Ninja
//! 
//! Ultrawydajny egzekutor transakcji Solana z obsługą MEV i Jito Bundles.
//! Zaprojektowany dla latencji <100ms i wysokiej przepustowości.

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn, error};
use uuid::Uuid;

mod strategies;
mod execution;
mod jito;
mod metrics;
mod nemotron_profit_engine;
mod solana;
mod rpc_load_balancer;
mod webhook_handler;

use hft_ninja::config::Config;
use execution::{ExecutionEngine, ExecutionRequest, ExecutionResult};
use nemotron_profit_engine::{NemotronProfitEngine, create_nemotron_request};
use metrics::MetricsCollector;
use webhook_handler::{WebhookState, WebhookMetrics, RateLimiter, handle_helius_webhook, get_webhook_metrics};

/// 🏗️ Główna struktura aplikacji z NVIDIA Nemotron integration
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub execution_engine: Arc<ExecutionEngine>,
    pub metrics: Arc<MetricsCollector>,
    pub webhook_state: WebhookState,
    pub nemotron_engine: Arc<NemotronProfitEngine>,
}

/// 📊 Struktura odpowiedzi health check
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    solana_connection: bool,
    jito_connection: bool,
}

/// 🎯 Struktura żądania wykrywania sygnałów
#[derive(Deserialize)]
struct DetectSignalsRequest {
    context_id: String,
    strategies: Vec<String>,
    risk_level: String,
}

/// 📈 Struktura odpowiedzi z sygnałami
#[derive(Serialize)]
struct DetectSignalsResponse {
    signals: Vec<TradingSignal>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// 🎯 Struktura sygnału tradingowego z enhanced profit calculation
#[derive(Serialize, Deserialize, Clone)]
pub struct TradingSignal {
    pub id: Uuid,
    pub strategy: String,
    pub token_address: String,
    pub signal_type: String,
    pub confidence: f64,
    pub estimated_profit: f64,           // Base profit estimation
    pub estimated_profit_nemotron: Option<f64>, // NVIDIA Nemotron enhanced estimation
    pub profit_confidence: f64,          // Confidence in profit estimation
    pub max_potential_profit: f64,       // Maximum possible profit
    pub min_expected_profit: f64,        // Minimum expected profit
    pub risk_score: f64,
    pub execution_priority: u8,
    pub market_conditions: String,       // Current market state
    pub volatility_factor: f64,          // Market volatility impact
}

/// ⚡ Struktura żądania egzekucji
#[derive(Deserialize)]
struct ExecuteRequest {
    decision: serde_json::Value,
    execution_mode: String,
    priority_fee: String,
    slippage_tolerance: f64,
}

/// 🔥 Piranha Surf Analysis Request
#[derive(Deserialize)]
struct PiranhaAnalysisRequest {
    pool_address: String,
    token_address: String,
    liquidity_sol: Option<f64>,
    volume_24h: Option<f64>,
}

/// 🎯 Piranha Surf Execution Request
#[derive(Deserialize)]
struct PiranhaExecuteRequest {
    token_address: String,
    amount_sol: f64,
    max_slippage: Option<f64>,
}

/// 📊 Struktura odpowiedzi egzekucji
#[derive(Serialize)]
struct ExecuteResponse {
    transaction_id: String,
    status: String,
    estimated_completion: chrono::DateTime<chrono::Utc>,
}

/// 🚀 Główna funkcja aplikacji
#[tokio::main]
async fn main() -> Result<()> {
    // 📊 Inicjalizacja tracingu
    println!("🐺 Uruchamianie HFT-Ninja v2.0...");

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("🐺 Uruchamianie HFT-Ninja v2.0...");

    // 🔧 Ładowanie konfiguracji
    let config = Arc::new(Config::load()?);
    info!("✅ Konfiguracja załadowana");

    // 🚀 Inicjalizacja komponentów
    let execution_engine = Arc::new(ExecutionEngine::new(config.clone()).await?);
    let metrics = Arc::new(MetricsCollector::new());

    info!("✅ Silnik egzekucji zainicjalizowany");

    // 🎣 Inicjalizacja Webhook State
    let helius_auth_token = std::env::var("HELIUS_AUTH_TOKEN")
        .unwrap_or_else(|_| "default_token".to_string());
    let kestra_trigger_url = std::env::var("KESTRA_TRIGGER_URL")
        .unwrap_or_else(|_| "http://kestra:8080/api/v1/executions/trigger".to_string());
    let cerebro_bff_url = std::env::var("CEREBRO_BFF_URL")
        .unwrap_or_else(|_| "http://cerebro-bff:8080".to_string());

    let webhook_state = WebhookState {
        helius_auth_token,
        kestra_trigger_url,
        cerebro_bff_url,
        metrics: Arc::new(WebhookMetrics::default()),
        rate_limiter: Arc::new(tokio::sync::RwLock::new(RateLimiter::new(100))), // 100 req/min
        sniper_engine: hft_ninja::SniperProfileEngine::new(config.sniper.clone()),
    };

    info!("✅ Webhook handler zainicjalizowany");

    // 🧠 Inicjalizacja NVIDIA Nemotron Profit Engine
    let nemotron_url = std::env::var("NVIDIA_NEMOTRON_URL")
        .unwrap_or_else(|_| "http://nemotron:11434".to_string());
    let nemotron_engine = Arc::new(NemotronProfitEngine::new(nemotron_url));
    info!("🧠 NVIDIA Nemotron Profit Engine zainicjalizowany");

    // 🏗️ Tworzenie stanu aplikacji z enhanced profit calculation
    let app_state = AppState {
        config: config.clone(),
        execution_engine,
        metrics,
        webhook_state: webhook_state.clone(),
        nemotron_engine,
    };

    // 🌐 Konfiguracja routingu
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/signals/detect", post(detect_signals))
        .route("/api/v1/execute", post(execute_transaction))
        .route("/api/v1/track/:transaction_id", get(track_transaction))
        .route("/api/v1/performance/summary", get(performance_summary))
        .route("/api/v1/strategies/update", post(update_strategies))
        // 🔥 Piranha Surf Strategy Endpoints
        .route("/piranha/analyze", post(piranha_analyze_pool))
        .route("/piranha/execute", post(piranha_execute_snipe))
        .route("/piranha/positions", get(piranha_get_positions))
        // 🎣 Helius Webhook Endpoints
        .route("/webhooks/helius", post(handle_helius_webhook_wrapper))
        .route("/webhooks/metrics", get(get_webhook_metrics_wrapper))
        .route("/metrics", get(metrics::export_metrics))
        .route("/test/sniper", get(test_sniper_engine))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // 🚀 Uruchomienie serwera
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr).await?;

    info!("🚀 HFT-Ninja uruchomiony na {}", addr);
    info!("📊 Metryki dostępne na /metrics");
    info!("🔍 Health check dostępny na /health");
    println!("🚀 HFT-Ninja uruchomiony na {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 🏥 Health check endpoint
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // TODO: Sprawdzenie połączeń z Solana i Jito
    let solana_connection = true; // state.execution_engine.check_solana_connection().await;
    let jito_connection = true;   // state.execution_engine.check_jito_connection().await;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        solana_connection,
        jito_connection,
    };

    Ok(Json(response))
}

/// 🎯 Wykrywanie sygnałów tradingowych z NVIDIA Nemotron Enhanced Profit Calculation
async fn detect_signals(
    State(state): State<AppState>,
    Json(request): Json<DetectSignalsRequest>,
) -> Result<Json<DetectSignalsResponse>, StatusCode> {
    info!("🔍 Wykrywanie sygnałów dla kontekstu: {} z Nemotron enhancement", request.context_id);

    // 🧠 Enhanced profit calculation using NVIDIA Nemotron
    let token_address = "So11111111111111111111111111111111111111112".to_string();
    let market_data = serde_json::json!({
        "price": 0.000123,
        "volume_24h": 150000.0,
        "liquidity_usd": 45000.0,
        "price_change_24h": 0.15,
        "volume_spike": 2.3,
        "depth_ratio": 1.2
    });

    let trading_signals = vec![
        serde_json::json!({
            "signal_type": "memecoin_launch",
            "strength": 0.85,
            "social_sentiment": 0.78,
            "whale_activity": 0.65
        })
    ];

    // Create Nemotron request
    let nemotron_request = create_nemotron_request(
        token_address.clone(),
        market_data,
        trading_signals,
    );

    // Get enhanced profit calculation from Nemotron
    let (enhanced_profit, profit_confidence, max_profit, min_profit) =
        match state.nemotron_engine.calculate_enhanced_profit(nemotron_request).await {
            Ok(nemotron_response) => {
                if state.nemotron_engine.validate_profit_response(&nemotron_response) {
                    info!("🧠 Nemotron enhanced profit: {:.4} (confidence: {:.2})",
                          nemotron_response.enhanced_profit_estimate,
                          nemotron_response.confidence_score);
                    (
                        Some(nemotron_response.enhanced_profit_estimate),
                        nemotron_response.confidence_score,
                        nemotron_response.max_potential_profit,
                        nemotron_response.min_expected_profit,
                    )
                } else {
                    warn!("⚠️ Invalid Nemotron response, using fallback");
                    (None, 0.5, 0.01, 0.001)
                }
            }
            Err(e) => {
                warn!("⚠️ Nemotron calculation failed: {}, using fallback", e);
                (None, 0.5, 0.01, 0.001)
            }
        };

    let signals = vec![
        TradingSignal {
            id: Uuid::new_v4(),
            strategy: "piranha_surf_memecoin_snipe".to_string(),
            token_address,
            signal_type: "memecoin_launch_opportunity".to_string(),
            confidence: 0.85,
            estimated_profit: 0.003,                    // Base calculation
            estimated_profit_nemotron: enhanced_profit, // NVIDIA Nemotron enhanced
            profit_confidence,                          // Nemotron confidence
            max_potential_profit: max_profit,           // Nemotron max profit
            min_expected_profit: min_profit,            // Nemotron min profit
            risk_score: 0.2,
            execution_priority: 1,
            market_conditions: "high_volatility_bullish".to_string(),
            volatility_factor: 1.3,                     // 30% above normal volatility
        }
    ];

    let response = DetectSignalsResponse {
        signals,
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(response))
}

/// ⚡ Egzekucja transakcji
async fn execute_transaction(
    State(state): State<AppState>,
    Json(request): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, StatusCode> {
    info!("⚡ Egzekucja transakcji w trybie: {}", request.execution_mode);

    // TODO: Implementacja egzekucji przez Jito Bundle
    let transaction_id = Uuid::new_v4().to_string();
    
    let response = ExecuteResponse {
        transaction_id,
        status: "submitted".to_string(),
        estimated_completion: chrono::Utc::now() + chrono::Duration::seconds(30),
    };

    Ok(Json(response))
}

/// 📊 Śledzenie transakcji
async fn track_transaction(
    State(state): State<AppState>,
    axum::extract::Path(transaction_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📊 Śledzenie transakcji: {}", transaction_id);

    // TODO: Implementacja śledzenia transakcji
    let response = serde_json::json!({
        "transaction_id": transaction_id,
        "status": "confirmed",
        "profit_loss": 0.0025,
        "execution_time_ms": 95,
        "gas_used": 5000
    });

    Ok(Json(response))
}

/// 📈 Podsumowanie wydajności
async fn performance_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📈 Generowanie podsumowania wydajności");

    // TODO: Implementacja zbierania metryk wydajności
    let response = serde_json::json!({
        "daily_roi": 0.045,
        "total_trades": 127,
        "successful_trades": 108,
        "success_rate": 0.85,
        "avg_execution_time_ms": 87,
        "total_profit_sol": 0.34
    });

    Ok(Json(response))
}

/// 🔧 Aktualizacja strategii
async fn update_strategies(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔧 Aktualizacja strategii");

    // TODO: Implementacja aktualizacji strategii
    let response = serde_json::json!({
        "status": "updated",
        "deployed_count": 3,
        "rollback_enabled": true
    });

    Ok(Json(response))
}

// 🔥 PIRANHA SURF STRATEGY HANDLERS

/// 🔍 Analyze pool for Piranha Surf opportunity
async fn piranha_analyze_pool(
    State(_state): State<AppState>,
    Json(request): Json<PiranhaAnalysisRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔍 Piranha analyzing pool: {}", request.pool_address);

    // Create mock Solana client for analysis
    let solana_client = solana::SolanaClient::new(
        "https://api.devnet.solana.com",
        "confirmed"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analysis = solana_client
        .analyze_pool(&request.pool_address, &request.token_address)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("✅ Pool analysis complete: {:?}", analysis.action);

    Ok(Json(serde_json::to_value(analysis).unwrap()))
}

/// 🎯 Execute Piranha Surf snipe
async fn piranha_execute_snipe(
    State(_state): State<AppState>,
    Json(request): Json<PiranhaExecuteRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔥 Executing Piranha snipe: {} SOL on {}", request.amount_sol, request.token_address);

    // Create mock analysis for execution
    let mock_analysis = solana::PoolAnalysis {
        pool_address: format!("pool_{}", &request.token_address[0..8]),
        token_address: request.token_address.clone(),
        liquidity_sol: 25.0,
        volume_24h: 150.0,
        age_seconds: 45,
        risk_score: 0.2,
        action: solana::PiranhaAction::Snipe { amount_sol: request.amount_sol },
        confidence: 0.85,
    };

    let mut solana_client = solana::SolanaClient::new(
        "https://api.devnet.solana.com",
        "confirmed"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let signature = solana_client
        .execute_piranha_snipe(&mock_analysis)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("✅ Piranha snipe executed: {}", signature);

    Ok(Json(serde_json::json!({
        "success": true,
        "signature": signature.to_string(),
        "token_address": request.token_address,
        "amount_sol": request.amount_sol,
        "strategy": "piranha_surf",
        "timestamp": chrono::Utc::now().timestamp()
    })))
}

/// 📊 Get active Piranha positions
async fn piranha_get_positions(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📊 Fetching active Piranha positions");

    let solana_client = solana::SolanaClient::new(
        "https://api.devnet.solana.com",
        "confirmed"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let positions = solana_client.get_active_positions();

    Ok(Json(serde_json::json!({
        "active_positions": positions.len(),
        "positions": positions,
        "strategy": "piranha_surf",
        "timestamp": chrono::Utc::now().timestamp()
    })))
}

// 🎣 Webhook wrapper functions
async fn handle_helius_webhook_wrapper(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<webhook_handler::HeliusWebhookPayload>,
) -> impl axum::response::IntoResponse {
    handle_helius_webhook(
        State(state.webhook_state),
        headers,
        Json(payload),
    ).await
}

async fn get_webhook_metrics_wrapper(
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    get_webhook_metrics(State(state.webhook_state)).await
}

// Dodaj test endpoint
async fn test_sniper_engine(State(state): State<AppState>) -> impl IntoResponse {
    let test_tokens = vec![
        // Token 1: Good fundamentals but medium metrics
        serde_json::json!({
            "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "volume_usd": 15000.0,
            "liquidity_usd": 25000.0,
            "price_change_24h_percent": 12.5,
            "transaction_count_24h": 150.0,
            "created_at": "2024-01-15T10:00:00Z",
            "dev_allocation_percent": 5.0,  // Good - low dev allocation
            "has_freeze_function": false,    // Good - no freeze
            "holder_count": 120.0,          // Good - decent holders
            "is_verified": true,            // Good - verified
            "team_doxxed": false,           // Neutral
            "listing_platform": "raydium"
        }),
        // Token 2: Red flags - should be filtered
        serde_json::json!({
            "mint": "So11111111111111111111111111111111111111112",
            "volume_usd": 500.0,  // Too low
            "liquidity_usd": 2000.0,  // Too low
            "price_change_24h_percent": 5.0,
            "transaction_count_24h": 25.0,
            "created_at": "2024-01-10T10:00:00Z",
            "dev_allocation_percent": 80.0,  // RED FLAG - high dev allocation
            "has_freeze_function": true,     // RED FLAG - has freeze
            "holder_count": 15.0,           // RED FLAG - low holders
            "is_verified": false,           // RED FLAG - not verified
            "metadata": {
                "name": "SafeMoon🚀💎",
                "description": "Guaranteed 100x returns to the MOON! Lambo incoming!"
            }
        }),
        // Token 3: Excellent opportunity - pump.fun gem
        serde_json::json!({
            "mint": "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            "volume_usd": 75000.0,  // High volume
            "liquidity_usd": 120000.0,  // High liquidity
            "price_change_24h_percent": 25.0,  // Strong momentum
            "transaction_count_24h": 500.0,
            "created_at": "2024-01-15T08:00:00Z",  // Recent
            "dev_allocation_percent": 2.0,   // EXCELLENT - very low dev allocation
            "has_freeze_function": false,    // EXCELLENT - no freeze
            "holder_count": 350.0,          // EXCELLENT - many holders
            "is_verified": true,            // EXCELLENT - verified
            "team_doxxed": true,            // EXCELLENT - doxxed team
            "listing_platform": "pump.fun", // GOOD - pump.fun listing
            "metadata": {
                "name": "SolanaDoge",
                "description": "Community-driven meme token on Solana"
            }
        }),
        // Token 4: Suspicious metadata test
        serde_json::json!({
            "mint": "SuspiciousToken123456789012345678901234567890",
            "volume_usd": 25000.0,
            "liquidity_usd": 15000.0,
            "price_change_24h_percent": 45.0,
            "transaction_count_24h": 200.0,
            "created_at": "2024-01-15T07:00:00Z",
            "dev_allocation_percent": 15.0,  // Moderate dev allocation
            "has_freeze_function": false,
            "holder_count": 80.0,
            "is_verified": false,
            "metadata": {
                "name": "SAFE🚀INU💎MOON",
                "description": "100% SAFE guaranteed 1000x returns! Lambo guaranteed!"
            }
        })
    ];

    let mut results = Vec::new();
    
    for token in test_tokens {
        let result = match state.webhook_state.sniper_engine.analyze_token(&token) {
            Some(profile) => serde_json::json!({
                "mint": token["mint"],
                "status": "passed",
                "profile": profile
            }),
            None => serde_json::json!({
                "mint": token["mint"],
                "status": "filtered_out"
            })
        };
        results.push(result);
    }

    let (processed, passed, pass_rate) = state.webhook_state.sniper_engine.get_stats();

    Json(serde_json::json!({
        "test_results": results,
        "engine_stats": {
            "tokens_processed": processed,
            "tokens_passed": passed,
            "pass_rate": format!("{:.1}%", pass_rate * 100.0)
        }
    }))
}

/// 🧪 Test execution engine endpoint
async fn test_execution_engine(
    State(state): State<AppState>,
    Json(request): Json<ExecutionRequest>,
) -> Result<Json<ExecutionResult>, StatusCode> {
    info!("🧪 Testing execution engine with mode: {:?}", request.execution_mode);

    match state.execution_engine.execute_trade(request).await {
        Ok(result) => {
            info!("✅ Execution test successful: {:?}", result.success);
            Ok(Json(result))
        },
        Err(e) => {
            error!("❌ Execution test failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
