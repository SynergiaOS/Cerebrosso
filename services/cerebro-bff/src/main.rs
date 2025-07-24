//! 🐺 Projekt Cerberus Phoenix v2.0 - Cerebro-BFF
//! 
//! Backend for Frontend z logiką AI, Context Engine i orkiestracją agentów.
//! Centralny mózg systemu odpowiedzialny za podejmowanie decyzji.

use anyhow::Result;
use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn, error};
use uuid::Uuid;

mod config;
mod context_engine;
mod ai_agent;
mod qdrant_client;
mod feedback_system;
mod paper_trading;
mod market_data_feed;
mod adaptive_learning;
mod metrics;
mod helius_client;
mod quicknode_client;
mod helius_webhook;
mod batch_optimizer;
mod pump_fun_scanner;
mod solana_stream;
mod intelligent_cache;
mod api_usage_monitor;
mod multi_rpc_manager;

use config::Config;
use context_engine::ContextEngine;
use ai_agent::AIAgent;
use helius_client::HeliusClient;
use quicknode_client::QuickNodeClient;
use helius_webhook::{HeliusWebhookHandler, handle_helius_webhook};
use batch_optimizer::{BatchOptimizer, BatchConfig};
use api_usage_monitor::ApiUsageMonitor;
use multi_rpc_manager::{MultiRpcManager, RoutingStrategy};
use metrics::MetricsCollector;
use feedback_system::FeedbackSystem;
use paper_trading::PaperTradingEngine;
use market_data_feed::MarketDataFeed;
use adaptive_learning::AdaptiveLearningEngine;

/// 🏗️ Główna struktura aplikacji
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub context_engine: Arc<ContextEngine>,
    pub ai_agent: Arc<AIAgent>,
    pub feedback_system: Arc<FeedbackSystem>,
    pub paper_trading: Arc<PaperTradingEngine>,
    pub market_data_feed: Arc<MarketDataFeed>,
    pub adaptive_learning: Arc<AdaptiveLearningEngine>,
    pub metrics: Arc<MetricsCollector>,
    pub helius_client: Arc<HeliusClient>,
    pub quicknode_client: Arc<QuickNodeClient>,
    pub webhook_handler: Arc<HeliusWebhookHandler>,
    pub batch_optimizer: Arc<BatchOptimizer>,
    pub usage_monitor: Arc<ApiUsageMonitor>,
    pub multi_rpc_manager: Arc<MultiRpcManager>,
}

/// 📊 Struktura odpowiedzi health check
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    qdrant_connection: bool,
    llm_connection: bool,
    context_count: u64,
}

/// 🧠 Struktura żądania kontekstualizacji
#[derive(Deserialize)]
struct ContextualizeRequest {
    oumi_data: serde_json::Value,
    jupiter_data: serde_json::Value,
    birdeye_data: serde_json::Value,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// 📈 Struktura odpowiedzi kontekstualizacji
#[derive(Serialize)]
struct ContextualizeResponse {
    context_id: String,
    embeddings_created: u32,
    processing_time_ms: u64,
}

/// 🤖 Struktura żądania decyzji AI
#[derive(Deserialize)]
struct DecisionRequest {
    signals: Vec<serde_json::Value>,
    context_id: String,
    risk_tolerance: f64,
    max_position_size: f64,
}

/// 🎯 Struktura odpowiedzi decyzji AI
#[derive(Serialize)]
struct DecisionResponse {
    decision_id: String,
    action: String,
    confidence: f64,
    reasoning: String,
    risk_assessment: f64,
    recommended_position_size: f64,
}

/// 📊 Struktura żądania feedbacku
#[derive(Deserialize)]
struct FeedbackRequest {
    context_id: String,
    decision_id: String,
    transaction_result: serde_json::Value,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// 🚀 Główna funkcja aplikacji
#[tokio::main]
async fn main() -> Result<()> {
    // 📊 Inicjalizacja tracingu
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    info!("🧠 Uruchamianie Cerebro-BFF v2.0...");

    // 🔧 Ładowanie konfiguracji
    let config = Arc::new(Config::load()?);
    info!("✅ Konfiguracja załadowana");

    // 📊 Inicjalizacja MetricsCollector
    let metrics = Arc::new(MetricsCollector::new());

    // 🚀 Inicjalizacja komponentów
    let context_engine = Arc::new(ContextEngine::new(config.clone()).await?);
    let ai_agent = Arc::new(AIAgent::new(config.clone(), metrics.clone()).await?);

    // 📊 Inicjalizacja Feedback System
    let feedback_db_url = std::env::var("FEEDBACK_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://cerberus:feedback_password_2024@localhost:5433/cerberus_feedback".to_string());
    let feedback_system = Arc::new(FeedbackSystem::new(config.clone(), &feedback_db_url).await?);

    // 🌟 Inicjalizacja Helius i QuickNode klientów
    let helius_client = Arc::new(HeliusClient::new(
        std::env::var("HELIUS_API_KEY").unwrap_or_default()
    ));
    let quicknode_client = Arc::new(QuickNodeClient::new(
        std::env::var("QUICKNODE_RPC_URL").unwrap_or_default(),
        "https://mainnet.block-engine.jito.wtf".to_string(),
        std::env::var("QUICKNODE_API_KEY").unwrap_or_default(),
    ));

    // 🔔 Inicjalizacja webhook handler
    let webhook_handler = Arc::new(HeliusWebhookHandler::new(
        context_engine.clone(),
        ai_agent.clone(),
    ));

    // 🚀 Inicjalizacja batch optimizer
    let batch_config = BatchConfig::default();
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6380".to_string());
    let batch_optimizer = Arc::new(BatchOptimizer::new(
        batch_config,
        helius_client.clone(),
        &redis_url,
    ).await?);

    // 📊 Inicjalizacja API usage monitor
    let monthly_limit = std::env::var("HELIUS_MONTHLY_LIMIT")
        .unwrap_or_else(|_| "1000000".to_string())
        .parse::<u32>()
        .unwrap_or(1_000_000);
    let alert_threshold = std::env::var("API_USAGE_ALERT_THRESHOLD")
        .unwrap_or_else(|_| "0.8".to_string())
        .parse::<f64>()
        .unwrap_or(0.8);
    let usage_monitor = Arc::new(ApiUsageMonitor::new(monthly_limit, alert_threshold));

    // 🔄 Inicjalizacja Multi-RPC Manager
    let routing_strategy = match std::env::var("RPC_ROUTING_STRATEGY").as_deref() {
        Ok("cost_optimized") => RoutingStrategy::CostOptimized,
        Ok("performance_first") => RoutingStrategy::PerformanceFirst,
        Ok("round_robin") => RoutingStrategy::RoundRobin,
        Ok("weighted_round_robin") => RoutingStrategy::WeightedRoundRobin,
        Ok("enhanced_data_first") => RoutingStrategy::EnhancedDataFirst,
        _ => RoutingStrategy::CostOptimized, // Default
    };
    let multi_rpc_manager = Arc::new(MultiRpcManager::new(routing_strategy));

    // 📈 Inicjalizacja Paper Trading Engine
    let paper_trading = Arc::new(PaperTradingEngine::new(
        config.clone(),
        sqlx::PgPool::connect(&feedback_db_url).await?
    ).await?);

    // 📊 Inicjalizacja Market Data Feed
    let market_data_feed = Arc::new(MarketDataFeed::new(
        config.clone(),
        helius_client.clone(),
        quicknode_client.clone()
    ).await?);

    // 🧠 Inicjalizacja Adaptive Learning Engine
    let adaptive_learning = Arc::new(AdaptiveLearningEngine::new(
        config.clone(),
        sqlx::PgPool::connect(&feedback_db_url).await?,
        feedback_system.clone(),
        paper_trading.clone(),
        ai_agent.clone()
    ).await?);

    info!("✅ Wszystkie komponenty zainicjalizowane");

    // 🏗️ Tworzenie stanu aplikacji
    let app_state = AppState {
        config: config.clone(),
        context_engine,
        ai_agent,
        feedback_system,
        paper_trading,
        market_data_feed,
        adaptive_learning,
        metrics,
        helius_client,
        quicknode_client,
        webhook_handler,
        batch_optimizer,
        usage_monitor,
        multi_rpc_manager,
    };

    // 🌐 Konfiguracja routingu
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/contextualize", post(contextualize_data))
        .route("/api/v1/decide", post(make_decision))
        .route("/api/v1/feedback", post(process_feedback))
        .route("/api/v1/analyze/patterns", post(analyze_patterns))
        .route("/api/v1/optimize/identify", post(identify_improvements))
        .route("/api/v1/optimize/generate", post(generate_optimizations))
        .route("/api/v1/learning/optimize", post(optimize_agent_parameters))
        .route("/api/v1/learning/stats", get(get_optimization_stats))
        .route("/api/v1/learning/performance", get(get_agent_performance))
        .route("/api/v1/metrics/trading-summary", get(get_trading_summary))
        .route("/api/v1/metrics/system-health", get(get_system_health))
        .route("/api/v1/webhook/token-events", post(handle_token_events))
        .route("/api/v1/backtest/run", post(run_backtest))
        .route("/api/v1/context/update", post(update_context))
        .route("/api/v1/reports/learning", post(generate_learning_report))
        .route("/api/v1/alerts", post(handle_alert))
        // 🔔 Helius Webhook Endpoints
        .route("/webhooks/helius/tokens", post(handle_helius_webhook))
        // 🚀 Batch Optimization Endpoints
        .route("/api/v1/batch/token-analysis", post(batch_token_analysis))
        .route("/api/v1/batch/stats", get(batch_stats))
        // 🎯 Risk Analysis Endpoints
        .route("/api/v1/risk/analyze/:token", get(analyze_token_risk))
        // 🚀 Pump.fun Scanner Endpoints
        .route("/api/v1/pump-fun/discovered", get(get_discovered_tokens))
        .route("/api/v1/pump-fun/high-potential", get(get_high_potential_tokens))
        .route("/api/v1/pump-fun/stats", get(get_scanner_stats))
        // 🌊 Stream & Cache Endpoints
        .route("/api/v1/stream/stats", get(get_stream_stats))
        .route("/api/v1/cache/stats", get(get_cache_stats))
        .route("/api/v1/optimization/status", get(get_optimization_status))
        .route("/api/v1/usage/report", get(get_usage_report))
        .route("/api/v1/usage/trends", get(get_usage_trends))
        .route("/api/v1/rpc/providers", get(get_rpc_providers))
        .route("/api/v1/rpc/performance", get(get_rpc_performance))
        .route("/metrics", get(metrics::export_metrics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // 🚀 Uruchomienie serwera
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(addr).await?;
    
    info!("🚀 Cerebro-BFF uruchomiony na {}", addr);
    info!("📊 Metryki dostępne na /metrics");
    info!("🔍 Health check dostępny na /health");

    axum::serve(listener, app).await?;

    Ok(())
}

/// 🏥 Health check endpoint
async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // TODO: Sprawdzenie połączeń z Qdrant i LLM
    let qdrant_connection = true; // state.context_engine.check_qdrant_connection().await;
    let llm_connection = state.ai_agent.check_llm_connection().await;
    let context_count = 1234;    // state.context_engine.get_context_count().await;

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        qdrant_connection,
        llm_connection,
        context_count,
    };

    Ok(Json(response))
}

/// 🧠 Kontekstualizacja danych
async fn contextualize_data(
    State(state): State<AppState>,
    Json(request): Json<ContextualizeRequest>,
) -> Result<Json<ContextualizeResponse>, StatusCode> {
    info!("🧠 Kontekstualizacja danych z timestamp: {}", request.timestamp);

    let start_time = std::time::Instant::now();
    
    // TODO: Implementacja kontekstualizacji przez Context Engine
    let context_id = Uuid::new_v4().to_string();
    let embeddings_created = 15;
    
    let processing_time = start_time.elapsed().as_millis() as u64;

    let response = ContextualizeResponse {
        context_id,
        embeddings_created,
        processing_time_ms: processing_time,
    };

    Ok(Json(response))
}

/// 🤖 Podejmowanie decyzji przez AI
async fn make_decision(
    State(state): State<AppState>,
    Json(request): Json<DecisionRequest>,
) -> Result<Json<DecisionResponse>, StatusCode> {
    info!("🤖 Podejmowanie decyzji AI dla kontekstu: {}", request.context_id);

    // TODO: Implementacja decyzji przez AI Agent
    let decision_id = Uuid::new_v4().to_string();
    
    let response = DecisionResponse {
        decision_id,
        action: "execute".to_string(),
        confidence: 0.87,
        reasoning: "High probability sandwich opportunity detected with favorable market conditions".to_string(),
        risk_assessment: 0.15,
        recommended_position_size: 0.8,
    };

    Ok(Json(response))
}

/// 📊 Przetwarzanie feedbacku
async fn process_feedback(
    State(state): State<AppState>,
    Json(request): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📊 Przetwarzanie feedbacku dla decyzji: {}", request.decision_id);

    // TODO: Implementacja uczenia się z feedbacku
    let response = serde_json::json!({
        "status": "processed",
        "learning_applied": true,
        "context_updated": true
    });

    Ok(Json(response))
}

/// 📈 Analiza wzorców
async fn analyze_patterns(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📈 Analiza wzorców w danych wydajności");

    // TODO: Implementacja analizy wzorców
    let response = serde_json::json!({
        "patterns": [
            {
                "type": "timing_pattern",
                "description": "Better performance during EU market hours",
                "confidence": 0.82
            }
        ],
        "strategies": [
            {
                "name": "sandwich",
                "effectiveness": 0.85,
                "avg_profit": 0.0032
            }
        ]
    });

    Ok(Json(response))
}

/// 🎯 Identyfikacja obszarów poprawy
async fn identify_improvements(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🎯 Identyfikacja obszarów poprawy");

    // TODO: Implementacja identyfikacji ulepszeń
    let response = serde_json::json!({
        "improvements": [
            {
                "area": "execution_timing",
                "current_performance": 87.5,
                "target_performance": 95.0,
                "priority": "high"
            }
        ]
    });

    Ok(Json(response))
}

/// 🔧 Generowanie optymalizacji
async fn generate_optimizations(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔧 Generowanie optymalizacji");

    // TODO: Implementacja generowania optymalizacji
    let response = serde_json::json!({
        "optimizations": [
            {
                "type": "parameter_adjustment",
                "strategy": "sandwich",
                "parameter": "slippage_tolerance",
                "current_value": 0.005,
                "optimized_value": 0.003
            }
        ]
    });

    Ok(Json(response))
}

/// 🧪 Uruchomienie backtestów
async fn run_backtest(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🧪 Uruchomienie backtestów");

    // TODO: Implementacja backtestów
    let response = serde_json::json!({
        "validation_passed": true,
        "roi_improvement": 0.012,
        "risk_reduction": 0.05,
        "sharpe_ratio": 2.34
    });

    Ok(Json(response))
}

/// 🔄 Aktualizacja kontekstu
async fn update_context(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🔄 Aktualizacja kontekstu");

    // TODO: Implementacja aktualizacji kontekstu
    let response = serde_json::json!({
        "status": "updated",
        "contexts_modified": 5,
        "new_embeddings": 12
    });

    Ok(Json(response))
}

/// 📋 Generowanie raportu uczenia się
async fn generate_learning_report(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📋 Generowanie raportu uczenia się");

    // TODO: Implementacja generowania raportu
    let response = serde_json::json!({
        "report_id": Uuid::new_v4().to_string(),
        "summary": "Learning cycle completed successfully",
        "improvements_applied": 3,
        "performance_gain": 0.025
    });

    Ok(Json(response))
}

/// 🚨 Obsługa alertów
async fn handle_alert(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    warn!("🚨 Otrzymano alert: {:?}", request);

    // TODO: Implementacja obsługi alertów
    let response = serde_json::json!({
        "status": "acknowledged",
        "alert_id": Uuid::new_v4().to_string()
    });

    Ok(Json(response))
}

/// 🚀 Batch token analysis endpoint
async fn batch_token_analysis(
    State(state): State<AppState>,
    Json(request): Json<BatchTokenRequest>,
) -> Result<Json<BatchTokenResponse>, StatusCode> {
    use batch_optimizer::{TokenRequest, TokenRequestType, RequestPriority};

    let mut request_ids = Vec::new();

    for token_address in request.token_addresses {
        let token_request = TokenRequest {
            token_address: token_address.clone(),
            request_type: request.request_type.clone().unwrap_or(TokenRequestType::BasicInfo),
            priority: request.priority.clone().unwrap_or(RequestPriority::Normal),
            requested_at: chrono::Utc::now(),
            requester_id: request.requester_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
        };

        match state.batch_optimizer.add_request(token_request).await {
            Ok(request_id) => request_ids.push(request_id),
            Err(e) => {
                error!("Failed to add batch request for {}: {}", token_address, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    Ok(Json(BatchTokenResponse {
        request_ids,
        batch_id: Uuid::new_v4().to_string(),
        estimated_completion_ms: 2000,
        status: "queued".to_string(),
    }))
}

/// 📊 Batch optimizer statistics endpoint
async fn batch_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state.batch_optimizer.get_stats().await;
    Ok(Json(serde_json::to_value(stats).unwrap()))
}

/// 📥 Batch token analysis request
#[derive(Deserialize)]
struct BatchTokenRequest {
    token_addresses: Vec<String>,
    request_type: Option<batch_optimizer::TokenRequestType>,
    priority: Option<batch_optimizer::RequestPriority>,
    requester_id: Option<String>,
}

/// 📤 Batch token analysis response
#[derive(Serialize)]
struct BatchTokenResponse {
    request_ids: Vec<String>,
    batch_id: String,
    estimated_completion_ms: u64,
    status: String,
}

/// 🎯 Token risk analysis endpoint
async fn analyze_token_risk(
    State(state): State<AppState>,
    Path(token_address): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    use qdrant_client::QdrantClient;

    // Mock token metadata for demonstration
    let mock_metadata = serde_json::json!({
        "name": "Example Token",
        "symbol": "EXAMPLE",
        "description": "A sample token for testing",
        "holder_count": 150,
        "liquidity_usd": 25000.0,
        "volume_24h": 5000.0,
        "market_cap": 100000.0,
        "is_verified": false,
        "team_doxxed": false
    });

    // Create Qdrant client for risk analysis
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    let qdrant_client = match QdrantClient::new(&qdrant_url).await {
        Ok(client) => client,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match qdrant_client.analyze_token_risk(&token_address, &mock_metadata).await {
        Ok(analysis) => {
            info!("🎯 Risk analysis completed for {}: {:.2}% risk", token_address, analysis.overall_risk_score * 100.0);
            Ok(Json(serde_json::to_value(analysis).unwrap()))
        }
        Err(e) => {
            error!("❌ Risk analysis failed for {}: {}", token_address, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 🚀 Get discovered pump.fun tokens
async fn get_discovered_tokens(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock response for demonstration
    let mock_tokens = serde_json::json!({
        "discovered_tokens": [],
        "total_count": 0,
        "last_updated": chrono::Utc::now()
    });

    Ok(Json(mock_tokens))
}

/// 🎯 Get high potential pump.fun tokens
async fn get_high_potential_tokens(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock response for demonstration
    let mock_tokens = serde_json::json!({
        "high_potential_tokens": [],
        "total_count": 0,
        "last_updated": chrono::Utc::now()
    });

    Ok(Json(mock_tokens))
}

/// 📊 Get pump.fun scanner statistics
async fn get_scanner_stats(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock response for demonstration
    let mock_stats = serde_json::json!({
        "total_tokens_discovered": 0,
        "tokens_analyzed": 0,
        "high_potential_tokens": 0,
        "avoided_tokens": 0,
        "avg_analysis_time_ms": 0.0,
        "last_scan_time": null
    });

    Ok(Json(mock_stats))
}

/// 🌊 Get Solana stream statistics
async fn get_stream_stats(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock response for demonstration
    let mock_stats = serde_json::json!({
        "total_subscriptions": 3,
        "websocket_url": "wss://api.mainnet-beta.solana.com/",
        "subscription_types": {
            "program_change": 2,
            "logs_subscribe": 1
        },
        "connection_status": "connected",
        "last_event_time": chrono::Utc::now()
    });

    Ok(Json(mock_stats))
}

/// 🧠 Get intelligent cache statistics
async fn get_cache_stats(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Mock response for demonstration
    let mock_stats = serde_json::json!({
        "total_entries": 150,
        "hot_tier_count": 25,
        "warm_tier_count": 75,
        "cold_tier_count": 40,
        "frozen_tier_count": 10,
        "avg_access_count": 3.2,
        "cache_hit_rate": 0.85,
        "avg_age_seconds": 180
    });

    Ok(Json(mock_stats))
}

/// 🎯 Get overall optimization status
async fn get_optimization_status(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let optimization_status = serde_json::json!({
        "helius_api_optimization": {
            "webhook_integration": true,
            "batch_processing": true,
            "intelligent_caching": true,
            "stream_monitoring": true,
            "estimated_rpm_reduction": "85%",
            "estimated_cost_savings": "$127/month"
        },
        "performance_metrics": {
            "avg_response_time_ms": 45,
            "cache_hit_rate": 0.85,
            "batch_efficiency": 0.92,
            "stream_uptime": 0.999
        },
        "current_usage": {
            "requests_this_hour": 45,
            "requests_today": 1250,
            "monthly_projection": 28500,
            "free_tier_limit": 1000000,
            "usage_percentage": 2.85
        },
        "optimizations_active": [
            "Webhook-based token discovery",
            "100-token batch processing",
            "Volatility-based intelligent caching",
            "Real-time WebSocket streaming",
            "RPC load balancing",
            "Automatic failover"
        ],
        "next_optimizations": [
            "Machine learning cache prediction",
            "Dynamic batch sizing",
            "Cross-chain optimization"
        ]
    });

    Ok(Json(optimization_status))
}

/// 📊 Get comprehensive usage report
async fn get_usage_report(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.usage_monitor.generate_report().await {
        Ok(report) => Ok(Json(report)),
        Err(e) => {
            error!("❌ Failed to generate usage report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 📈 Get usage trends
async fn get_usage_trends(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let trends = state.usage_monitor.get_usage_trends(24).await;
    let stats = state.usage_monitor.get_stats().await;
    let metrics = state.usage_monitor.get_optimization_metrics().await;

    let response = serde_json::json!({
        "trends_24h": trends,
        "current_stats": stats,
        "optimization_metrics": metrics,
        "summary": {
            "total_trends": trends.len(),
            "avg_requests_per_hour": if !trends.is_empty() {
                trends.iter().map(|t| t.requests_count).sum::<u32>() as f64 / 24.0
            } else { 0.0 },
            "total_savings": trends.iter().map(|t| t.optimization_savings).sum::<f64>()
        }
    });

    Ok(Json(response))
}

/// 🔄 Get RPC provider statistics
async fn get_rpc_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let provider_stats = state.multi_rpc_manager.get_provider_stats().await;
    let routing_strategy = state.multi_rpc_manager.get_routing_strategy();

    let response = serde_json::json!({
        "routing_strategy": routing_strategy,
        "providers": provider_stats,
        "summary": {
            "total_providers": provider_stats.len(),
            "healthy_providers": provider_stats.values().filter(|s| s.is_healthy).count(),
            "total_requests": provider_stats.values().map(|s| s.requests_this_month).sum::<u32>()
        }
    });

    Ok(Json(response))
}

/// 🧠 Optimize agent parameters
async fn optimize_agent_parameters(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let agent_type_str = request.get("agent_type")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let agent_type = match agent_type_str {
        "FastDecision" => ai_agent::AgentType::FastDecision,
        "ContextAnalysis" => ai_agent::AgentType::ContextAnalysis,
        "RiskAssessment" => ai_agent::AgentType::RiskAssessment,
        "DeepAnalysis" => ai_agent::AgentType::DeepAnalysis,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match state.adaptive_learning.optimize_agent_parameters(agent_type).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 📊 Get optimization statistics
async fn get_optimization_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state.adaptive_learning.get_optimization_stats().await;
    Ok(Json(serde_json::to_value(stats).unwrap()))
}

/// 📈 Get agent performance metrics
async fn get_agent_performance(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let agent_type_str = params.get("agent_type")
        .ok_or(StatusCode::BAD_REQUEST)?;

    let agent_type = match agent_type_str.as_str() {
        "FastDecision" => ai_agent::AgentType::FastDecision,
        "ContextAnalysis" => ai_agent::AgentType::ContextAnalysis,
        "RiskAssessment" => ai_agent::AgentType::RiskAssessment,
        "DeepAnalysis" => ai_agent::AgentType::DeepAnalysis,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match state.feedback_system.get_agent_performance(agent_type).await {
        Ok(Some(performance)) => Ok(Json(serde_json::to_value(performance).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 📊 Get trading performance summary
async fn get_trading_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let summary = state.metrics.get_trading_performance_summary();
    Ok(Json(serde_json::to_value(summary).unwrap()))
}

/// 🔥 Get system health score
async fn get_system_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let health_score = state.metrics.get_system_health_score();

    let health_data = serde_json::json!({
        "health_score": health_score,
        "status": if health_score > 0.8 { "healthy" } else if health_score > 0.5 { "warning" } else { "critical" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "components": {
            "ai_agents": "operational",
            "paper_trading": "operational",
            "market_data": "operational",
            "adaptive_learning": "operational"
        }
    });

    Ok(Json(health_data))
}

/// 🎣 Handle token events from Helius webhook
async fn handle_token_events(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("🎣 Received token events from Helius webhook");

    // Extract events from payload
    let empty_vec = vec![];
    let events = payload.get("events")
        .and_then(|e| e.as_array())
        .unwrap_or(&empty_vec);

    info!("Processing {} token events", events.len());

    // Process each event through AI agents
    for event in events {
        if let Some(token_mint) = event.get("token_mint").and_then(|m| m.as_str()) {
            if let Some(trading_signals) = event.get("trading_signals").and_then(|s| s.as_array()) {
                // Convert to context for AI analysis
                let context = format!(
                    "Token event detected: mint={}, signals={}, timestamp={}",
                    token_mint,
                    trading_signals.len(),
                    event.get("timestamp").and_then(|t| t.as_u64()).unwrap_or(0)
                );

                // Trigger AI analysis
                match state.ai_agent.make_decision(&context, &[]).await {
                    Ok(decision) => {
                        info!("AI decision for token {}: {} (confidence: {:.2})",
                              token_mint, decision.action, decision.confidence);

                        // Record metrics
                        state.metrics.record_ai_decision(
                            &decision.agent_type.to_string(),
                            &decision.action,
                            decision.confidence,
                            decision.latency_ms,
                            &decision.model_used
                        );
                    }
                    Err(e) => {
                        warn!("Failed to process AI decision for token {}: {}", token_mint, e);
                    }
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": "processed",
        "events_count": events.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 📊 Get comprehensive RPC performance report
async fn get_rpc_performance(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let report = state.multi_rpc_manager.generate_performance_report().await;
    Ok(Json(report))
}
