//! 🚀 HFT-Ninja Minimal - MVP dla natychmiastowego działania

use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn, error};

/// 📊 Application state
#[derive(Clone)]
struct AppState {
    cerebro_url: String,
    client: reqwest::Client,
}

/// 🎣 Helius webhook payload
#[derive(Debug, Deserialize)]
struct HeliusWebhook {
    #[serde(flatten)]
    data: serde_json::Value,
}

/// 📊 Analysis result
#[derive(Debug, Serialize)]
struct AnalysisResult {
    status: String,
    score: f64,
    action: String,
    timestamp: i64,
}

/// 🏥 Health response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    timestamp: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 📝 Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting HFT-Ninja Minimal MVP");

    // 🔧 Configuration
    let cerebro_url = std::env::var("CEREBRO_BFF_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    // 🏗️ Build app state
    let app_state = AppState {
        cerebro_url,
        client: reqwest::Client::new(),
    };

    // 🌐 Setup routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/webhook/helius", post(handle_webhook))
        .route("/api/analyze", post(analyze_token))
        .with_state(app_state);

    // 🚀 Start server
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("🚀 HFT-Ninja Minimal listening on http://{}", addr);
    info!("🎣 Webhook endpoint: http://{}/webhook/helius", addr);
    info!("🔍 Analysis endpoint: http://{}/api/analyze", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// 🏥 Health check endpoint
async fn health_check() -> ResponseJson<HealthResponse> {
    ResponseJson(HealthResponse {
        status: "healthy".to_string(),
        service: "hft-ninja-minimal".to_string(),
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}

/// 🎣 Webhook handler
async fn handle_webhook(
    State(state): State<AppState>,
    Json(payload): Json<HeliusWebhook>,
) -> Result<ResponseJson<AnalysisResult>, StatusCode> {
    info!("🎣 Received webhook");

    // 🔍 Simple token analysis
    let analysis = analyze_token_data(&payload.data);
    
    // 🧠 Send to Cerebro if score is high
    if analysis.score > 0.7 {
        if let Err(e) = send_to_cerebro(&state, &payload.data).await {
            warn!("Failed to send to Cerebro: {}", e);
        }
    }

    Ok(ResponseJson(analysis))
}

/// 🔍 Direct analysis endpoint
async fn analyze_token(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<ResponseJson<AnalysisResult>, StatusCode> {
    info!("🔍 Direct analysis request");

    let analysis = analyze_token_data(&payload);
    
    // 🧠 Send to Cerebro if score is high
    if analysis.score > 0.7 {
        if let Err(e) = send_to_cerebro(&state, &payload).await {
            warn!("Failed to send to Cerebro: {}", e);
        }
    }

    Ok(ResponseJson(analysis))
}

/// 📊 Simple token analysis logic
fn analyze_token_data(data: &serde_json::Value) -> AnalysisResult {
    let mut score = 0.0;
    let mut action = "ignore".to_string();

    // 📊 Check volume
    if let Some(volume) = data.get("volumeUsd").and_then(|v| v.as_f64()) {
        if volume > 1000.0 {
            score += 0.3;
        }
        if volume > 10000.0 {
            score += 0.2;
        }
    }

    // 💧 Check liquidity
    if let Some(liquidity) = data.get("liquidityUsd").and_then(|v| v.as_f64()) {
        if liquidity > 5000.0 {
            score += 0.3;
        }
        if liquidity > 50000.0 {
            score += 0.2;
        }
    }

    // 👥 Check holder count
    if let Some(holders) = data.get("holderCount").and_then(|v| v.as_u64()) {
        if holders > 50 {
            score += 0.2;
        }
        if holders > 200 {
            score += 0.1;
        }
    }

    // 🎯 Determine action
    action = match score {
        s if s > 0.8 => "buy".to_string(),
        s if s > 0.6 => "analyze".to_string(),
        s if s > 0.4 => "watch".to_string(),
        _ => "ignore".to_string(),
    };

    AnalysisResult {
        status: "analyzed".to_string(),
        score,
        action,
        timestamp: chrono::Utc::now().timestamp(),
    }
}

/// 🧠 Send data to Cerebro-BFF
async fn send_to_cerebro(state: &AppState, data: &serde_json::Value) -> Result<()> {
    let url = format!("{}/api/analyze", state.cerebro_url);
    
    let payload = serde_json::json!({
        "token_data": data,
        "source": "hft-ninja-minimal",
        "timestamp": chrono::Utc::now().timestamp()
    });

    let response = state.client
        .post(&url)
        .timeout(std::time::Duration::from_secs(10))
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        info!("✅ Successfully sent to Cerebro-BFF");
    } else {
        warn!("⚠️ Cerebro-BFF returned status: {}", response.status());
    }

    Ok(())
}
