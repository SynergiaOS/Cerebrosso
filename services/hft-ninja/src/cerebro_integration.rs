//! 🧠 Cerebro-BFF Integration - AI Decision Pipeline
//! 
//! Integruje HFT-Ninja z Cerebro-BFF Context Engine dla AI-driven trading decisions

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, debug, warn, error, instrument};

use crate::config::Config;

/// 🧠 Cerebro-BFF Integration Client
pub struct CerebroClient {
    client: Client,
    base_url: String,
    config: Arc<Config>,
}

/// 📊 Token Profile for AI Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenProfile {
    pub mint_address: String,
    pub symbol: String,
    pub name: String,
    pub market_cap: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub volume_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub holder_count: Option<u32>,
    pub dev_allocation_percentage: Option<f64>,
    pub freeze_authority: Option<bool>,
    pub mint_authority: Option<bool>,
    pub team_doxxed: Option<bool>,
    pub contract_verified: Option<bool>,
    pub risk_signals: Vec<String>,
    pub opportunity_signals: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 🎯 AI Trading Decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITradingDecision {
    pub action: TradingAction,
    pub confidence: f64,
    pub reasoning: String,
    pub risk_assessment: RiskLevel,
    pub position_size_percentage: f64,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
    pub urgency_level: u8, // 1-10 scale for fee optimization
    pub strategy_type: String,
}

/// 🎯 Trading Action Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingAction {
    Buy,
    Sell,
    Hold,
    Avoid,
}

/// ⚠️ Risk Assessment Levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 📡 Context Analysis Request
#[derive(Debug, Serialize)]
struct ContextAnalysisRequest {
    token_profile: TokenProfile,
    strategy_type: String,
    urgency_level: u8,
    market_conditions: MarketConditions,
}

/// 📊 Market Conditions
#[derive(Debug, Serialize)]
struct MarketConditions {
    overall_sentiment: String,
    volatility_index: f64,
    volume_trend: String,
    time_of_day: String,
}

impl CerebroClient {
    /// 🚀 Initialize Cerebro-BFF integration client
    #[instrument(skip(config))]
    pub fn new(config: Arc<Config>) -> Result<Self> {
        info!("🧠 Initializing Cerebro-BFF Integration Client");

        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(10000)) // 10s timeout
            .build()?;

        let base_url = std::env::var("CEREBRO_BFF_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        Ok(CerebroClient {
            client,
            base_url,
            config,
        })
    }

    /// 🎯 Get AI trading decision for token
    #[instrument(skip(self))]
    pub async fn get_trading_decision(
        &self,
        token_profile: TokenProfile,
        strategy_type: &str,
    ) -> Result<AITradingDecision> {
        info!("🎯 Requesting AI trading decision for token: {}", token_profile.symbol);

        // Prepare context analysis request
        let request = ContextAnalysisRequest {
            token_profile: token_profile.clone(),
            strategy_type: strategy_type.to_string(),
            urgency_level: self.calculate_urgency_level(&token_profile),
            market_conditions: self.get_current_market_conditions().await?,
        };

        // Send request to Cerebro-BFF
        let url = format!("{}/api/analyze-token", self.base_url);
        
        match self.client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<AITradingDecision>().await {
                    Ok(decision) => {
                        info!("✅ AI Decision received: {:?} with {:.2}% confidence", 
                              decision.action, decision.confidence * 100.0);
                        Ok(decision)
                    }
                    Err(e) => {
                        error!("❌ Failed to parse AI decision: {}", e);
                        // Fallback to rule-based decision
                        Ok(self.fallback_decision(&token_profile, strategy_type))
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️ Cerebro-BFF returned status: {}", response.status());
                Ok(self.fallback_decision(&token_profile, strategy_type))
            }
            Err(e) => {
                warn!("⚠️ Failed to connect to Cerebro-BFF: {}", e);
                Ok(self.fallback_decision(&token_profile, strategy_type))
            }
        }
    }

    /// 📊 Optimize context for specific AI agent
    #[instrument(skip(self))]
    pub async fn optimize_context_for_agent(
        &self,
        token_profile: &TokenProfile,
        agent_type: &str,
    ) -> Result<String> {
        debug!("📊 Optimizing context for agent: {}", agent_type);

        let url = format!("{}/api/optimize-context", self.base_url);
        
        let request = serde_json::json!({
            "token_profile": token_profile,
            "agent_type": agent_type,
            "max_tokens": self.get_max_tokens_for_agent(agent_type),
        });

        match self.client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.text().await {
                    Ok(optimized_context) => {
                        debug!("✅ Context optimized: {} chars", optimized_context.len());
                        Ok(optimized_context)
                    }
                    Err(e) => {
                        error!("❌ Failed to get optimized context: {}", e);
                        Ok(self.create_fallback_context(token_profile))
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️ Context optimization failed with status: {}", response.status());
                Ok(self.create_fallback_context(token_profile))
            }
            Err(e) => {
                warn!("⚠️ Failed to connect for context optimization: {}", e);
                Ok(self.create_fallback_context(token_profile))
            }
        }
    }

    /// 🔄 Update AI model with trading results (feedback loop)
    #[instrument(skip(self))]
    pub async fn update_trading_feedback(
        &self,
        token_address: &str,
        decision: &AITradingDecision,
        actual_result: &TradingResult,
    ) -> Result<()> {
        info!("🔄 Updating AI feedback for token: {}", token_address);

        let url = format!("{}/api/feedback", self.base_url);
        
        let feedback = serde_json::json!({
            "token_address": token_address,
            "original_decision": decision,
            "actual_result": actual_result,
            "timestamp": chrono::Utc::now(),
        });

        match self.client
            .post(&url)
            .json(&feedback)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                info!("✅ AI feedback updated successfully");
                Ok(())
            }
            Ok(response) => {
                warn!("⚠️ Feedback update failed with status: {}", response.status());
                Ok(()) // Non-critical failure
            }
            Err(e) => {
                warn!("⚠️ Failed to send feedback: {}", e);
                Ok(()) // Non-critical failure
            }
        }
    }

    /// ⚡ Calculate urgency level based on token characteristics
    fn calculate_urgency_level(&self, token_profile: &TokenProfile) -> u8 {
        let mut urgency = 5u8; // Base urgency

        // High risk signals increase urgency
        if token_profile.risk_signals.len() > 3 {
            urgency = urgency.saturating_add(2);
        }

        // High volume increases urgency
        if let Some(volume) = token_profile.volume_24h {
            if volume > 1_000_000.0 { // > $1M volume
                urgency = urgency.saturating_add(1);
            }
        }

        // Price volatility increases urgency
        if let Some(price_change) = token_profile.price_change_24h {
            if price_change.abs() > 50.0 { // > 50% price change
                urgency = urgency.saturating_add(2);
            }
        }

        urgency.min(10) // Cap at 10
    }

    /// 📊 Get current market conditions
    async fn get_current_market_conditions(&self) -> Result<MarketConditions> {
        // Simplified market conditions - in production would fetch real data
        Ok(MarketConditions {
            overall_sentiment: "neutral".to_string(),
            volatility_index: 0.5,
            volume_trend: "increasing".to_string(),
            time_of_day: chrono::Utc::now().format("%H:%M").to_string(),
        })
    }

    /// 🛡️ Fallback decision when AI is unavailable
    fn fallback_decision(&self, token_profile: &TokenProfile, strategy_type: &str) -> AITradingDecision {
        warn!("🛡️ Using fallback rule-based decision for {}", token_profile.symbol);

        // Simple rule-based logic
        let risk_score = token_profile.risk_signals.len() as f64;
        let opportunity_score = token_profile.opportunity_signals.len() as f64;

        let action = if risk_score > 3.0 {
            TradingAction::Avoid
        } else if opportunity_score > risk_score {
            TradingAction::Buy
        } else {
            TradingAction::Hold
        };

        let confidence = (opportunity_score / (risk_score + opportunity_score + 1.0)).min(0.8);

        AITradingDecision {
            action,
            confidence,
            reasoning: "Fallback rule-based decision due to AI unavailability".to_string(),
            risk_assessment: if risk_score > 2.0 { RiskLevel::High } else { RiskLevel::Medium },
            position_size_percentage: if confidence > 0.6 { 2.0 } else { 1.0 },
            stop_loss_percentage: Some(10.0),
            take_profit_percentage: Some(25.0),
            urgency_level: self.calculate_urgency_level(token_profile),
            strategy_type: strategy_type.to_string(),
        }
    }

    /// 📝 Create fallback context when optimization fails
    fn create_fallback_context(&self, token_profile: &TokenProfile) -> String {
        format!(
            "Token: {} ({})\nRisk Signals: {:?}\nOpportunity Signals: {:?}\nMarket Cap: {:?}\nVolume 24h: {:?}",
            token_profile.symbol,
            token_profile.mint_address,
            token_profile.risk_signals,
            token_profile.opportunity_signals,
            token_profile.market_cap,
            token_profile.volume_24h
        )
    }

    /// 🎯 Get max tokens for different agent types
    fn get_max_tokens_for_agent(&self, agent_type: &str) -> u32 {
        match agent_type {
            "FastDecisionAgent" => 500,
            "ContextAnalysisAgent" => 1500,
            "RiskAssessmentAgent" => 1000,
            "DeepAnalysisAgent" => 3000,
            _ => 1000,
        }
    }
}

/// 📈 Trading Result for feedback
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingResult {
    pub profit_loss_percentage: f64,
    pub execution_time_ms: u64,
    pub slippage_percentage: f64,
    pub success: bool,
    pub error_message: Option<String>,
}
