//! 🌟 Helius Webhook Handler - Real-time Token Discovery
//! 
//! Optimized webhook system for Helius API Pro to receive real-time notifications
//! about new tokens, eliminating polling and saving RPM limits.

use anyhow::{Result, anyhow};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::context_engine::ContextEngine;
use crate::ai_agent::AIAgent;

/// 🔔 Helius webhook payload for new token events
#[derive(Debug, Clone, Deserialize)]
pub struct HeliusWebhookPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub account_data: Vec<AccountData>,
    pub transaction: TransactionData,
    pub timestamp: DateTime<Utc>,
    pub slot: u64,
}

/// 📄 Account data from Helius webhook
#[derive(Debug, Clone, Deserialize)]
pub struct AccountData {
    pub account: String,
    pub native_balance_change: i64,
    pub token_balance_changes: Vec<TokenBalanceChange>,
}

/// 💰 Token balance change information
#[derive(Debug, Clone, Deserialize)]
pub struct TokenBalanceChange {
    pub mint: String,
    pub raw_token_amount: TokenAmount,
    pub token_account: String,
    pub user_account: String,
}

/// 🔢 Token amount details
#[derive(Debug, Clone, Deserialize)]
pub struct TokenAmount {
    pub token_amount: String,
    pub decimals: u8,
}

/// 📊 Transaction data from webhook
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionData {
    pub signature: String,
    pub fee: u64,
    pub fee_payer: String,
    pub recent_blockhash: String,
}

/// 🎯 Processed token discovery result
#[derive(Debug, Clone, Serialize)]
pub struct TokenDiscoveryResult {
    pub token_address: String,
    pub discovery_time: DateTime<Utc>,
    pub risk_score: f64,
    pub confidence: f64,
    pub action_recommended: String,
    pub reasoning: String,
    pub metadata: TokenMetadata,
}

/// 📋 Token metadata for analysis
#[derive(Debug, Clone, Serialize)]
pub struct TokenMetadata {
    pub is_pump_fun: bool,
    pub initial_liquidity: f64,
    pub holder_count_estimate: u32,
    pub creation_signature: String,
    pub potential_rug_signals: Vec<String>,
}

/// ⚙️ Advanced webhook configuration
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub pump_fun_program: String,
    pub boom_program: String,
    pub token_program: String,
    pub min_liquidity_threshold: f64,
    pub max_risk_threshold: f64,
    pub enable_real_time_filtering: bool,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            pump_fun_program: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
            boom_program: "BoomkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj".to_string(),
            token_program: "TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj".to_string(),
            min_liquidity_threshold: 1000.0,
            max_risk_threshold: 0.8,
            enable_real_time_filtering: true,
        }
    }
}

/// 🌟 Enhanced Helius webhook handler
pub struct HeliusWebhookHandler {
    context_engine: Arc<ContextEngine>,
    ai_agent: Arc<AIAgent>,
    config: WebhookConfig,
}

impl HeliusWebhookHandler {
    /// 🚀 Initialize webhook handler with default config
    pub fn new(context_engine: Arc<ContextEngine>, ai_agent: Arc<AIAgent>) -> Self {
        Self {
            context_engine,
            ai_agent,
            config: WebhookConfig::default(),
        }
    }

    /// 🚀 Initialize webhook handler with custom config
    pub fn with_config(context_engine: Arc<ContextEngine>, ai_agent: Arc<AIAgent>, config: WebhookConfig) -> Self {
        Self {
            context_engine,
            ai_agent,
            config,
        }
    }

    /// 🔔 Handle incoming webhook from Helius
    pub async fn handle_webhook(
        &self,
        payload: HeliusWebhookPayload,
    ) -> Result<TokenDiscoveryResult> {
        info!("🔔 Received Helius webhook: {} at slot {}", payload.event_type, payload.slot);

        // Extract new token information
        let new_tokens = self.extract_new_tokens(&payload).await?;
        
        if new_tokens.is_empty() {
            return Err(anyhow!("No new tokens found in webhook payload"));
        }

        // Process the first new token (can be extended for batch processing)
        let token_address = &new_tokens[0];
        info!("🆕 Processing new token: {}", token_address);

        // Analyze token risk using context engine
        let risk_analysis = self.analyze_token_risk(token_address, &payload).await?;
        
        // Make AI decision
        let ai_decision = self.make_ai_decision(token_address, &risk_analysis).await?;

        // Store in context for future reference
        self.store_token_context(token_address, &payload, &risk_analysis).await?;

        Ok(TokenDiscoveryResult {
            token_address: token_address.clone(),
            discovery_time: payload.timestamp,
            risk_score: risk_analysis.risk_score,
            confidence: ai_decision.confidence,
            action_recommended: ai_decision.action,
            reasoning: ai_decision.reasoning,
            metadata: risk_analysis.metadata,
        })
    }

    /// 🔍 Extract new token addresses from webhook payload
    async fn extract_new_tokens(&self, payload: &HeliusWebhookPayload) -> Result<Vec<String>> {
        let mut new_tokens = Vec::new();

        for account_data in &payload.account_data {
            for token_change in &account_data.token_balance_changes {
                // Check if this is a new token mint event
                if self.is_new_token_mint(&token_change.mint, payload).await? {
                    new_tokens.push(token_change.mint.clone());
                }
            }
        }

        Ok(new_tokens)
    }

    /// 🆕 Check if token mint represents a new token
    async fn is_new_token_mint(&self, mint: &str, payload: &HeliusWebhookPayload) -> Result<bool> {
        // Check if we've seen this token before in our context
        let existing_context = self.context_engine
            .search_similar(&format!("token_mint:{}", mint), 1)
            .await?;

        if !existing_context.is_empty() {
            debug!("Token {} already known", mint);
            return Ok(false);
        }

        // Additional checks for pump.fun or other new token patterns
        let is_pump_fun = self.detect_pump_fun_pattern(payload).await?;
        
        Ok(is_pump_fun || self.has_new_token_signatures(payload))
    }

    /// 🎪 Detect pump.fun token creation pattern
    async fn detect_pump_fun_pattern(&self, payload: &HeliusWebhookPayload) -> Result<bool> {
        // Look for pump.fun program interactions in transaction
        let pump_fun_program = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"; // pump.fun program ID
        
        // Check if transaction involves pump.fun program
        // This is a simplified check - in production, you'd parse the full transaction
        Ok(payload.transaction.signature.len() > 0) // Placeholder logic
    }

    /// ✍️ Check for new token creation signatures
    fn has_new_token_signatures(&self, payload: &HeliusWebhookPayload) -> bool {
        // Check for token mint creation patterns
        payload.account_data.iter().any(|account| {
            account.token_balance_changes.iter().any(|change| {
                // New token typically has initial mint to creator
                change.raw_token_amount.token_amount.parse::<u64>().unwrap_or(0) > 0
            })
        })
    }

    /// 🔍 Analyze token risk using context engine and local rules
    async fn analyze_token_risk(&self, token_address: &str, payload: &HeliusWebhookPayload) -> Result<RiskAnalysis> {
        info!("🔍 Analyzing risk for token: {}", token_address);

        // Extract metadata
        let metadata = self.extract_token_metadata(token_address, payload).await?;
        
        // Calculate risk score based on multiple factors
        let mut risk_score: f64 = 0.0;
        let mut risk_signals = Vec::new();

        // Factor 1: Pump.fun tokens have higher base risk
        if metadata.is_pump_fun {
            risk_score += 0.3;
            risk_signals.push("pump_fun_token".to_string());
        }

        // Factor 2: Low initial liquidity
        if metadata.initial_liquidity < 1.0 {
            risk_score += 0.4;
            risk_signals.push("low_initial_liquidity".to_string());
        }

        // Factor 3: Very few holders
        if metadata.holder_count_estimate < 10 {
            risk_score += 0.2;
            risk_signals.push("few_initial_holders".to_string());
        }

        // Factor 4: Check against known rug pull patterns in context
        let similar_rugs = self.context_engine
            .search_similar(&format!("rug_pull token_pattern"), 5)
            .await?;

        if !similar_rugs.is_empty() {
            risk_score += 0.1;
            risk_signals.push("similar_to_known_rugs".to_string());
        }

        // Normalize risk score to 0-1 range
        risk_score = risk_score.min(1.0);

        Ok(RiskAnalysis {
            risk_score,
            metadata,
            risk_signals,
        })
    }

    /// 📋 Extract token metadata from webhook payload
    async fn extract_token_metadata(&self, token_address: &str, payload: &HeliusWebhookPayload) -> Result<TokenMetadata> {
        Ok(TokenMetadata {
            is_pump_fun: self.detect_pump_fun_pattern(payload).await?,
            initial_liquidity: self.estimate_initial_liquidity(payload).await?,
            holder_count_estimate: self.estimate_holder_count(payload),
            creation_signature: payload.transaction.signature.clone(),
            potential_rug_signals: self.detect_rug_signals(payload).await?,
        })
    }

    /// 💰 Estimate initial liquidity from transaction data
    async fn estimate_initial_liquidity(&self, payload: &HeliusWebhookPayload) -> Result<f64> {
        // Simplified liquidity estimation based on SOL balance changes
        let total_sol_change: i64 = payload.account_data
            .iter()
            .map(|account| account.native_balance_change.abs())
            .sum();

        // Convert lamports to SOL (1 SOL = 1e9 lamports)
        Ok(total_sol_change as f64 / 1e9)
    }

    /// 👥 Estimate holder count from account data
    fn estimate_holder_count(&self, payload: &HeliusWebhookPayload) -> u32 {
        // Count unique accounts with token balance changes
        let unique_accounts: std::collections::HashSet<String> = payload.account_data
            .iter()
            .flat_map(|account| {
                account.token_balance_changes.iter().map(|change| change.user_account.clone())
            })
            .collect();

        unique_accounts.len() as u32
    }

    /// ⚠️ Detect potential rug pull signals
    async fn detect_rug_signals(&self, payload: &HeliusWebhookPayload) -> Result<Vec<String>> {
        let mut signals = Vec::new();

        // Signal 1: Large initial mint to single address
        for account_data in &payload.account_data {
            for token_change in &account_data.token_balance_changes {
                if let Ok(amount) = token_change.raw_token_amount.token_amount.parse::<u64>() {
                    if amount > 1_000_000_000 { // Large mint
                        signals.push("large_initial_mint".to_string());
                    }
                }
            }
        }

        // Signal 2: High transaction fee (potential bot)
        if payload.transaction.fee > 100_000 { // 0.0001 SOL
            signals.push("high_transaction_fee".to_string());
        }

        Ok(signals)
    }

    /// 🤖 Make AI decision based on risk analysis
    async fn make_ai_decision(&self, token_address: &str, risk_analysis: &RiskAnalysis) -> Result<AIDecision> {
        let context = format!(
            "New token discovered: {} with risk score: {:.2}, signals: {:?}",
            token_address, risk_analysis.risk_score, risk_analysis.risk_signals
        );

        // Use AI agent to make decision
        self.ai_agent.make_decision(&context, &[]).await
    }

    /// 💾 Store token context for future reference
    async fn store_token_context(&self, token_address: &str, payload: &HeliusWebhookPayload, risk_analysis: &RiskAnalysis) -> Result<()> {
        let context_data = serde_json::json!({
            "type": "token_discovery",
            "token_address": token_address,
            "discovery_time": payload.timestamp,
            "risk_score": risk_analysis.risk_score,
            "metadata": risk_analysis.metadata,
            "risk_signals": risk_analysis.risk_signals,
            "transaction_signature": payload.transaction.signature
        });

        self.context_engine
            .store_context(&format!("token_discovery:{}", token_address), &context_data)
            .await?;

        info!("💾 Stored context for token: {}", token_address);
        Ok(())
    }
}

/// 🔍 Risk analysis result
#[derive(Debug, Clone)]
struct RiskAnalysis {
    risk_score: f64,
    metadata: TokenMetadata,
    risk_signals: Vec<String>,
}

/// 🤖 AI decision structure (re-exported from ai_agent)
use crate::ai_agent::AIDecision;

/// 🔔 Webhook endpoint handler
pub async fn handle_helius_webhook(
    State(app_state): State<crate::AppState>,
    Json(payload): Json<HeliusWebhookPayload>,
) -> Result<ResponseJson<TokenDiscoveryResult>, StatusCode> {
    info!("🔔 Helius webhook received");

    match app_state.webhook_handler.handle_webhook(payload).await {
        Ok(result) => {
            info!("✅ Token discovery processed: {}", result.token_address);
            Ok(ResponseJson(result))
        }
        Err(e) => {
            error!("❌ Webhook processing failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
