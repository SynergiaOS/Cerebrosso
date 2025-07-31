//! 🧠 Context Engine (CEM) - Main Application
//! 
//! Advanced Context Engine with dynamic memory system, Qdrant integration, and AI-powered learning

use anyhow::Result;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, error, instrument};
use uuid::Uuid;

use context_engine::{
    config::Config,
    context_engine::{ContextEngine, ContextRequest, ContextType, ContextState},
    feedback_loop::FeedbackData,
    metrics::ContextMetrics,
};

/// 🎯 Stan aplikacji
#[derive(Clone)]
struct AppState {
    context_engine: Arc<ContextEngine>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("context_engine=debug,info")
        .init();

    info!("🧠 Starting Context Engine (CEM) v3.0...");

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    info!("📋 Configuration loaded successfully");

    // Initialize Context Engine
    let context_engine = Arc::new(ContextEngine::new(config.clone()).await?);
    
    // Start Context Engine
    context_engine.start().await?;
    
    let app_state = AppState { context_engine };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/metrics", get(get_metrics))
        .route("/context/process", post(process_context))
        .route("/context/:id/feedback", post(record_feedback))
        .route("/memory/store", post(store_memory))
        .route("/memory/search", post(search_memory))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 Context Engine server started on {}", addr);
    info!("📊 Metrics available at http://{}/metrics", addr);
    info!("🏥 Health check at http://{}/health", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// 🏥 Health check endpoint
#[instrument]
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "context-engine",
        "version": "3.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

/// 📊 Status endpoint
#[instrument(skip(state))]
async fn get_status(State(state): State<AppState>) -> Json<Value> {
    let engine_state = state.context_engine.get_state().await;
    
    Json(json!({
        "service": "context-engine",
        "state": format!("{:?}", engine_state),
        "version": "3.0.0",
        "timestamp": chrono::Utc::now()
    }))
}

/// 📈 Metrics endpoint
#[instrument(skip(state))]
async fn get_metrics(State(state): State<AppState>) -> Json<ContextMetrics> {
    let metrics = state.context_engine.get_metrics().await;
    Json(metrics)
}

/// 🧠 Process context request
#[instrument(skip(state, request))]
async fn process_context(
    State(state): State<AppState>,
    Json(request): Json<ContextRequest>,
) -> Result<Json<Value>, StatusCode> {
    match state.context_engine.process_context_request(request).await {
        Ok(response) => Ok(Json(json!(response))),
        Err(e) => {
            error!("Failed to process context request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 📝 Record feedback
#[instrument(skip(state, feedback))]
async fn record_feedback(
    State(state): State<AppState>,
    Path(context_id): Path<Uuid>,
    Json(mut feedback): Json<FeedbackData>,
) -> Result<Json<Value>, StatusCode> {
    feedback.context_id = context_id;
    
    match state.context_engine.record_feedback(feedback).await {
        Ok(_) => Ok(Json(json!({
            "status": "success",
            "message": "Feedback recorded successfully"
        }))),
        Err(e) => {
            error!("Failed to record feedback: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 💾 Store memory endpoint
#[instrument(skip(state))]
async fn store_memory(
    State(_state): State<AppState>,
    Json(_memory_data): Json<Value>,
) -> Json<Value> {
    // Placeholder implementation
    Json(json!({
        "status": "success",
        "message": "Memory stored successfully"
    }))
}

/// 🔍 Search memory endpoint
#[instrument(skip(state))]
async fn search_memory(
    State(_state): State<AppState>,
    Json(_search_query): Json<Value>,
) -> Json<Value> {
    // Placeholder implementation
    Json(json!({
        "results": [],
        "total": 0
    }))
}
