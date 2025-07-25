//! 🎣 Helius Webhook Ingestor - Real-time Token Event Processing
//! 
//! Advanced webhook handler for Helius API Pro integration with:
//! - Secure signature validation
//! - Real-time token event processing  
//! - Kestra workflow triggering
//! - Metrics collection and monitoring
//! - Rate limiting and error handling

use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn, debug};
use tokio::time::{Duration, Instant};
use std::collections::HashMap;

// --- Enhanced Helius Webhook Structures ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeliusWebhookPayload {
    pub account_addresses: Vec<String>,
    pub transaction_types: Vec<String>,
    pub events: Vec<HeliusEvent>,
    pub webhook_type: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeliusEvent {
    pub native_transfers: Option<Vec<NativeTransfer>>,
    pub token_transfers: Option<Vec<TokenTransfer>>,
    pub transaction: HeliusTransaction,
    pub account_data: Option<Vec<AccountData>>,
    pub instructions: Option<Vec<Instruction>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeliusTransaction {
    pub signature: String,
    pub timestamp: u64,
    pub slot: Option<u64>,
    pub fee: Option<u64>,
    pub fee_payer: Option<String>,
    pub recent_blockhash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NativeTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub token_amount: f64,
    pub mint: String,
    pub token_standard: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountData {
    pub account: String,
    pub native_balance_change: Option<i64>,
    pub token_balance_changes: Option<Vec<TokenBalanceChange>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenBalanceChange {
    pub mint: String,
    pub raw_token_amount: TokenAmount,
    pub token_account: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenAmount {
    pub token_amount: String,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instruction {
    pub accounts: Vec<String>,
    pub data: String,
    pub program_id: String,
    pub inner_instructions: Option<Vec<InnerInstruction>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InnerInstruction {
    pub accounts: Vec<String>,
    pub data: String,
    pub program_id: String,
}

// --- Enhanced Application State ---
#[derive(Clone)]
pub struct WebhookState {
    pub helius_auth_token: String,
    pub kestra_trigger_url: String,
    pub cerebro_bff_url: String,
    pub metrics: Arc<WebhookMetrics>,
    pub rate_limiter: Arc<tokio::sync::RwLock<RateLimiter>>,
}

// --- Metrics Collection ---
#[derive(Debug, Default)]
pub struct WebhookMetrics {
    pub total_webhooks_received: std::sync::atomic::AtomicU64,
    pub successful_processing: std::sync::atomic::AtomicU64,
    pub failed_processing: std::sync::atomic::AtomicU64,
    pub kestra_triggers: std::sync::atomic::AtomicU64,
    pub cerebro_notifications: std::sync::atomic::AtomicU64,
    pub avg_processing_time_ms: std::sync::atomic::AtomicU64,
}

// --- Rate Limiting ---
#[derive(Debug)]
pub struct RateLimiter {
    pub requests: HashMap<String, Vec<Instant>>,
    pub max_requests_per_minute: usize,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: usize) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests_per_minute,
        }
    }
    
    pub fn is_allowed(&mut self, client_id: &str) -> bool {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);
        
        let requests = self.requests.entry(client_id.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests
        requests.retain(|&time| time > minute_ago);
        
        // Check if under limit
        if requests.len() < self.max_requests_per_minute {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

// --- Enhanced Webhook Handler ---
pub async fn handle_helius_webhook(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    Json(payload): Json<HeliusWebhookPayload>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    
    // Increment total webhooks counter
    state.metrics.total_webhooks_received.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    
    info!("🎣 Received Helius webhook with {} events", payload.events.len());
    debug!("Webhook payload: {:?}", payload);

    // 1. Rate limiting check
    let client_ip = headers.get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    {
        let mut rate_limiter = state.rate_limiter.write().await;
        if !rate_limiter.is_allowed(client_ip) {
            warn!("Rate limit exceeded for client: {}", client_ip);
            return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
        }
    }

    // 2. Enhanced signature validation
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str != format!("Bearer {}", state.helius_auth_token) {
                warn!("Invalid authorization token from {}", client_ip);
                state.metrics.failed_processing.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
            }
        } else {
            warn!("Invalid Authorization header format");
            state.metrics.failed_processing.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return (StatusCode::BAD_REQUEST, "Bad Request").into_response();
        }
    } else {
        warn!("Missing Authorization header");
        state.metrics.failed_processing.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // 3. Enhanced event processing and filtering
    let processed_events = process_and_filter_events(&payload).await;
    
    if processed_events.is_empty() {
        debug!("No relevant events found in webhook payload");
        return StatusCode::OK.into_response();
    }
    
    info!("🎯 Processing {} relevant events", processed_events.len());

    // 4. Parallel processing: Kestra + Cerebro-BFF
    let kestra_future = trigger_kestra_workflow(&state.kestra_trigger_url, &payload);
    let cerebro_future = notify_cerebro_bff(&state.cerebro_bff_url, &processed_events);
    
    let (kestra_result, cerebro_result) = tokio::join!(kestra_future, cerebro_future);

    // 5. Handle results and update metrics
    let mut success = true;
    
    match kestra_result {
        Ok(_) => {
            info!("✅ Successfully triggered Kestra workflow");
            state.metrics.kestra_triggers.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Err(e) => {
            error!("❌ Failed to trigger Kestra workflow: {}", e);
            success = false;
        }
    }
    
    match cerebro_result {
        Ok(_) => {
            info!("✅ Successfully notified Cerebro-BFF");
            state.metrics.cerebro_notifications.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Err(e) => {
            error!("❌ Failed to notify Cerebro-BFF: {}", e);
            success = false;
        }
    }

    // 6. Update processing metrics
    let processing_time = start_time.elapsed().as_millis() as u64;
    state.metrics.avg_processing_time_ms.store(processing_time, std::sync::atomic::Ordering::Relaxed);
    
    if success {
        state.metrics.successful_processing.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        info!("🚀 Webhook processed successfully in {}ms", processing_time);
        StatusCode::OK.into_response()
    } else {
        state.metrics.failed_processing.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        (StatusCode::INTERNAL_SERVER_ERROR, "Partial processing failure").into_response()
    }
}

// --- Event Processing and Filtering ---
async fn process_and_filter_events(payload: &HeliusWebhookPayload) -> Vec<ProcessedEvent> {
    let mut processed_events = Vec::new();
    
    for event in &payload.events {
        // Filter for relevant events (token creation, large transfers, etc.)
        if is_relevant_event(event) {
            if let Some(processed) = extract_trading_signals(event).await {
                processed_events.push(processed);
            }
        }
    }
    
    processed_events
}

fn is_relevant_event(event: &HeliusEvent) -> bool {
    // Check for token creation, large transfers, or pump.fun activity
    if let Some(token_transfers) = &event.token_transfers {
        for transfer in token_transfers {
            // Large transfer (>$1000 equivalent)
            if transfer.token_amount > 1000.0 {
                return true;
            }
        }
    }
    
    // Check for pump.fun program interactions
    if let Some(instructions) = &event.instructions {
        for instruction in instructions {
            if instruction.program_id == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" {
                return true; // pump.fun program
            }
        }
    }
    
    false
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessedEvent {
    pub event_type: String,
    pub token_mint: Option<String>,
    pub transaction_signature: String,
    pub timestamp: u64,
    pub trading_signals: Vec<TradingSignal>,
    pub risk_indicators: Vec<RiskIndicator>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradingSignal {
    pub signal_type: String,
    pub strength: f64,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RiskIndicator {
    pub risk_type: String,
    pub severity: f64,
    pub description: String,
}

async fn extract_trading_signals(event: &HeliusEvent) -> Option<ProcessedEvent> {
    let mut trading_signals = Vec::new();
    let mut risk_indicators = Vec::new();
    
    // Extract signals from token transfers
    if let Some(token_transfers) = &event.token_transfers {
        for transfer in token_transfers {
            // Large volume signal
            if transfer.token_amount > 10000.0 {
                trading_signals.push(TradingSignal {
                    signal_type: "large_volume".to_string(),
                    strength: (transfer.token_amount / 100000.0).min(1.0),
                    confidence: 0.8,
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("amount".to_string(), serde_json::Value::Number(
                            serde_json::Number::from_f64(transfer.token_amount).unwrap()
                        ));
                        map.insert("mint".to_string(), serde_json::Value::String(transfer.mint.clone()));
                        map
                    },
                });
            }
            
            // Risk indicator for suspicious patterns
            if transfer.from_user_account == transfer.to_user_account {
                risk_indicators.push(RiskIndicator {
                    risk_type: "self_transfer".to_string(),
                    severity: 0.7,
                    description: "Self-transfer detected - possible wash trading".to_string(),
                });
            }
        }
    }
    
    if !trading_signals.is_empty() || !risk_indicators.is_empty() {
        Some(ProcessedEvent {
            event_type: "token_activity".to_string(),
            token_mint: event.token_transfers.as_ref()
                .and_then(|transfers| transfers.first())
                .map(|t| t.mint.clone()),
            transaction_signature: event.transaction.signature.clone(),
            timestamp: event.transaction.timestamp,
            trading_signals,
            risk_indicators,
        })
    } else {
        None
    }
}

// --- External Service Integration ---
async fn trigger_kestra_workflow(
    kestra_url: &str,
    payload: &HeliusWebhookPayload,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(kestra_url)
        .timeout(Duration::from_secs(10))
        .json(&serde_json::json!({
            "webhook_data": payload,
            "source": "helius_webhook",
            "timestamp": chrono::Utc::now().timestamp()
        }))
        .send()
        .await
}

async fn notify_cerebro_bff(
    cerebro_url: &str,
    events: &[ProcessedEvent],
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post(format!("{}/api/v1/webhook/token-events", cerebro_url))
        .timeout(Duration::from_secs(5))
        .json(&serde_json::json!({
            "events": events,
            "source": "helius_webhook",
            "timestamp": chrono::Utc::now().timestamp()
        }))
        .send()
        .await
}

// --- Metrics Endpoint ---
pub async fn get_webhook_metrics(State(state): State<WebhookState>) -> impl IntoResponse {
    let metrics = &state.metrics;
    
    let response = serde_json::json!({
        "webhook_metrics": {
            "total_received": metrics.total_webhooks_received.load(std::sync::atomic::Ordering::Relaxed),
            "successful_processing": metrics.successful_processing.load(std::sync::atomic::Ordering::Relaxed),
            "failed_processing": metrics.failed_processing.load(std::sync::atomic::Ordering::Relaxed),
            "kestra_triggers": metrics.kestra_triggers.load(std::sync::atomic::Ordering::Relaxed),
            "cerebro_notifications": metrics.cerebro_notifications.load(std::sync::atomic::Ordering::Relaxed),
            "avg_processing_time_ms": metrics.avg_processing_time_ms.load(std::sync::atomic::Ordering::Relaxed),
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Json(response)
}
