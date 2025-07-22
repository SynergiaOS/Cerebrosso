//! 🦈 Piranha Surf Strategy - Premium Small Portfolio Trading
//! 
//! Advanced trading strategy using Helius API Pro and QuickNode Premium
//! optimized for small portfolios with maximum risk mitigation.

use crate::helius_client::{HeliusClient, TokenAnalysis, MarketConditions};
use crate::quicknode_client::{QuickNodeClient, ExecutionRequest, ExecutionResult};
use crate::decision_engine::{DecisionEngine, TradingDecision, DecisionAction};
use crate::context_engine::{ContextEngine, WeightedSignal};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tracing::{info, warn, debug, error};
use chrono::{DateTime, Utc};

/// 🦈 Piranha Surf strategy for small portfolio optimization
pub struct PiranhaStrategy {
    helius_client: Arc<HeliusClient>,
    quicknode_client: Arc<QuickNodeClient>,
    decision_engine: Arc<DecisionEngine>,
    config: PiranhaConfig,
}

/// ⚙️ Configuration for Piranha strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiranhaConfig {
    pub max_position_size_sol: f64,
    pub min_liquidity_score: f64,
    pub max_rug_pull_score: f64,
    pub max_slippage: f64,
    pub use_jito_bundles: bool,
    pub emergency_exit_threshold: f64,
    pub profit_target: f64,
    pub stop_loss: f64,
}

/// 📊 Trading opportunity identified by Piranha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingOpportunity {
    pub token_mint: String,
    pub opportunity_type: OpportunityType,
    pub confidence_score: f64,
    pub expected_profit: f64,
    pub risk_score: f64,
    pub recommended_amount: f64,
    pub time_sensitive: bool,
    pub analysis: TokenAnalysis,
    pub reasoning: Vec<String>,
}

/// 🎯 Types of trading opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    LiquiditySnipe,
    EarlyEntry,
    ArbitragePlay,
    RecoveryTrade,
    EmergencyExit,
}

impl std::fmt::Display for OpportunityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpportunityType::LiquiditySnipe => write!(f, "LiquiditySnipe"),
            OpportunityType::EarlyEntry => write!(f, "EarlyEntry"),
            OpportunityType::ArbitragePlay => write!(f, "ArbitragePlay"),
            OpportunityType::RecoveryTrade => write!(f, "RecoveryTrade"),
            OpportunityType::EmergencyExit => write!(f, "EmergencyExit"),
        }
    }
}

/// 📈 Trading session result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSession {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub opportunities_found: u32,
    pub trades_executed: u32,
    pub total_profit_loss: f64,
    pub success_rate: f64,
    pub market_conditions: MarketConditions,
}

impl PiranhaStrategy {
    /// 🚀 Initialize Piranha Surf strategy
    pub fn new(
        helius_client: Arc<HeliusClient>,
        quicknode_client: Arc<QuickNodeClient>,
        decision_engine: Arc<DecisionEngine>,
        config: PiranhaConfig,
    ) -> Self {
        info!("🦈 Initializing Piranha Surf strategy with premium APIs");
        Self {
            helius_client,
            quicknode_client,
            decision_engine,
            config,
        }
    }

    /// 🔍 Scan for trading opportunities (Strategy Core)
    pub async fn scan_opportunities(&self) -> Result<Vec<TradingOpportunity>> {
        info!("🔍 Scanning for Piranha trading opportunities");
        
        // Get current market conditions
        let market_conditions = self.helius_client.get_market_conditions().await?;
        
        // Check if market conditions are favorable
        if !self.is_market_favorable(&market_conditions) {
            warn!("🌊 Market conditions unfavorable, skipping scan");
            return Ok(vec![]);
        }
        
        // Get list of new/trending tokens (mock implementation)
        let candidate_tokens = self.get_candidate_tokens().await?;
        
        let mut opportunities = Vec::new();
        
        for token_mint in candidate_tokens {
            match self.analyze_token_opportunity(&token_mint).await {
                Ok(Some(opportunity)) => {
                    info!("🎯 Found opportunity: {} with confidence {:.2}", 
                          opportunity.opportunity_type, opportunity.confidence_score);
                    opportunities.push(opportunity);
                },
                Ok(None) => {
                    debug!("❌ No opportunity found for token {}", token_mint);
                },
                Err(e) => {
                    warn!("⚠️ Error analyzing token {}: {}", token_mint, e);
                }
            }
        }
        
        // Sort by confidence score (highest first)
        opportunities.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        
        info!("🔍 Scan complete: found {} opportunities", opportunities.len());
        Ok(opportunities)
    }

    /// 🎯 Analyze individual token for opportunity
    async fn analyze_token_opportunity(&self, token_mint: &str) -> Result<Option<TradingOpportunity>> {
        debug!("🎯 Analyzing token opportunity: {}", token_mint);
        
        // Get comprehensive token analysis from Helius
        let analysis = self.helius_client.analyze_token_filtered(token_mint).await?;
        
        // Convert to weighted signals for decision engine
        let signals = self.convert_analysis_to_signals(&analysis).await?;
        
        // Make trading decision
        let decision = self.decision_engine.make_decision(&signals).await?;
        
        // Check if this creates a valid opportunity
        match decision.action {
            DecisionAction::Execute => {
                let opportunity = self.create_opportunity_from_analysis(
                    token_mint, 
                    &analysis, 
                    &decision
                ).await?;
                Ok(Some(opportunity))
            },
            DecisionAction::Reject => {
                debug!("❌ Token rejected by decision engine: {}", token_mint);
                Ok(None)
            },
            _ => {
                debug!("🤔 Token requires further analysis: {}", token_mint);
                Ok(None)
            }
        }
    }

    /// 🚀 Execute trading opportunity
    pub async fn execute_opportunity(&self, opportunity: &TradingOpportunity) -> Result<ExecutionResult> {
        info!("🚀 Executing {} opportunity for {}", 
              opportunity.opportunity_type, opportunity.token_mint);
        
        // Validate opportunity is still valid
        self.validate_opportunity_freshness(opportunity).await?;
        
        // Calculate execution parameters
        let amount_sol = self.calculate_position_size(opportunity)?;
        let use_jito = self.should_use_jito_bundle(opportunity);
        
        // Create execution request
        let execution_request = ExecutionRequest {
            strategy: format!("PiranhaSurf_{:?}", opportunity.opportunity_type),
            token_mint: opportunity.token_mint.clone(),
            amount_sol,
            max_slippage: self.config.max_slippage,
            priority_fee: self.calculate_priority_fee(opportunity).await?,
            use_jito,
            timeout_ms: if opportunity.time_sensitive { 5000 } else { 15000 },
        };
        
        // Execute via QuickNode Premium
        let result = self.quicknode_client.execute_transaction(execution_request).await?;
        
        if result.success {
            info!("✅ Opportunity executed successfully: tx_id={:?}, time={}ms", 
                  result.transaction_id, result.execution_time_ms);
        } else {
            error!("❌ Opportunity execution failed: {:?}", result.error_message);
        }
        
        Ok(result)
    }

    /// 🌊 Check if market conditions are favorable
    fn is_market_favorable(&self, conditions: &MarketConditions) -> bool {
        // Conservative approach for small portfolio
        conditions.overall_sentiment > 0.3 &&           // Not too bearish
        conditions.volatility_index < 0.8 &&            // Not too volatile
        conditions.rug_pull_rate_24h < 0.15 &&          // Low rug pull rate
        conditions.average_liquidity > 50000.0          // Decent liquidity
    }

    /// 📋 Get candidate tokens for analysis
    async fn get_candidate_tokens(&self) -> Result<Vec<String>> {
        // This would integrate with Helius API to get trending/new tokens
        // For now, return mock tokens
        Ok(vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "So11111111111111111111111111111111111111112".to_string(),   // SOL
            // Add more candidate tokens from Helius trending API
        ])
    }

    /// 🔄 Convert token analysis to weighted signals
    async fn convert_analysis_to_signals(&self, analysis: &TokenAnalysis) -> Result<Vec<WeightedSignal>> {
        let mut signals = Vec::new();
        
        // Rug pull signal
        signals.push(WeightedSignal {
            signal_type: "rug_pull_score".to_string(),
            value: analysis.rug_pull_score,
            tf_idf_weight: 0.94, // High weight for rug pull detection
            importance_score: analysis.rug_pull_score,
            timestamp: Utc::now(),
        });
        
        // Team verification signal
        signals.push(WeightedSignal {
            signal_type: if analysis.team_doxxed { "team_verified" } else { "team_anonymous" }.to_string(),
            value: if analysis.team_doxxed { 1.0 } else { 0.0 },
            tf_idf_weight: 0.87,
            importance_score: if analysis.team_doxxed { 0.8 } else { 0.2 },
            timestamp: Utc::now(),
        });
        
        // Liquidity signal
        signals.push(WeightedSignal {
            signal_type: if analysis.liquidity_score > 0.7 { "high_liquidity" } else { "low_liquidity" }.to_string(),
            value: analysis.liquidity_score,
            tf_idf_weight: 0.75,
            importance_score: analysis.liquidity_score,
            timestamp: Utc::now(),
        });
        
        // Volume signal
        if analysis.volume_24h > 100000.0 {
            signals.push(WeightedSignal {
                signal_type: "high_volume".to_string(),
                value: analysis.volume_24h / 1000000.0, // Normalize to millions
                tf_idf_weight: 0.73,
                importance_score: 0.8,
                timestamp: Utc::now(),
            });
        }
        
        // Contract verification signal
        signals.push(WeightedSignal {
            signal_type: if analysis.contract_verified { "contract_verified" } else { "contract_unverified" }.to_string(),
            value: if analysis.contract_verified { 1.0 } else { 0.0 },
            tf_idf_weight: 0.82,
            importance_score: if analysis.contract_verified { 0.7 } else { 0.3 },
            timestamp: Utc::now(),
        });
        
        debug!("🔄 Converted analysis to {} weighted signals", signals.len());
        Ok(signals)
    }

    /// 🎯 Create trading opportunity from analysis
    async fn create_opportunity_from_analysis(
        &self,
        token_mint: &str,
        analysis: &TokenAnalysis,
        decision: &TradingDecision,
    ) -> Result<TradingOpportunity> {
        
        // Determine opportunity type based on analysis
        let opportunity_type = self.classify_opportunity_type(analysis)?;
        
        // Calculate expected profit (conservative estimate)
        let expected_profit = self.estimate_profit(analysis, &opportunity_type)?;
        
        // Calculate recommended position size
        let recommended_amount = self.calculate_recommended_amount(analysis, expected_profit)?;
        
        // Check if time sensitive
        let time_sensitive = self.is_time_sensitive(&opportunity_type, analysis);
        
        Ok(TradingOpportunity {
            token_mint: token_mint.to_string(),
            opportunity_type,
            confidence_score: decision.confidence,
            expected_profit,
            risk_score: decision.risk_score,
            recommended_amount,
            time_sensitive,
            analysis: analysis.clone(),
            reasoning: decision.reasoning.clone(),
        })
    }

    /// 🏷️ Classify opportunity type
    fn classify_opportunity_type(&self, analysis: &TokenAnalysis) -> Result<OpportunityType> {
        // Token age in hours
        let token_age_hours = (Utc::now() - analysis.creation_time).num_hours();
        
        if analysis.rug_pull_score > self.config.emergency_exit_threshold {
            return Ok(OpportunityType::EmergencyExit);
        }
        
        if token_age_hours < 1 && analysis.liquidity_score > 0.8 {
            return Ok(OpportunityType::LiquiditySnipe);
        }
        
        if token_age_hours < 24 && analysis.rug_pull_score < 0.3 && analysis.team_doxxed {
            return Ok(OpportunityType::EarlyEntry);
        }
        
        if analysis.volume_24h > 500000.0 && analysis.liquidity_score > 0.6 {
            return Ok(OpportunityType::ArbitragePlay);
        }
        
        Ok(OpportunityType::RecoveryTrade)
    }

    /// 💰 Estimate profit potential
    fn estimate_profit(&self, analysis: &TokenAnalysis, opportunity_type: &OpportunityType) -> Result<f64> {
        let base_profit = match opportunity_type {
            OpportunityType::LiquiditySnipe => 0.15,    // 15% potential
            OpportunityType::EarlyEntry => 0.25,        // 25% potential
            OpportunityType::ArbitragePlay => 0.08,     // 8% potential
            OpportunityType::RecoveryTrade => 0.12,     // 12% potential
            OpportunityType::EmergencyExit => -0.05,    // 5% loss mitigation
        };
        
        // Adjust based on analysis quality
        let quality_multiplier = (analysis.metadata_quality + analysis.liquidity_score) / 2.0;
        let risk_adjustment = 1.0 - analysis.rug_pull_score;
        
        let adjusted_profit = base_profit * quality_multiplier * risk_adjustment;
        
        debug!("💰 Estimated profit: {:.2}% (base: {:.2}%, quality: {:.2}, risk_adj: {:.2})", 
               adjusted_profit * 100.0, base_profit * 100.0, quality_multiplier, risk_adjustment);
        
        Ok(adjusted_profit)
    }

    /// 📏 Calculate recommended position size
    fn calculate_recommended_amount(&self, analysis: &TokenAnalysis, expected_profit: f64) -> Result<f64> {
        // Kelly Criterion inspired sizing
        let win_probability = 1.0 - analysis.rug_pull_score;
        let win_amount = expected_profit.abs();
        let loss_amount = 1.0; // Assume 100% loss in worst case
        
        let kelly_fraction = (win_probability * win_amount - (1.0 - win_probability) * loss_amount) / win_amount;
        
        // Conservative sizing: use 25% of Kelly fraction, capped at max position size
        let recommended = (kelly_fraction * 0.25 * self.config.max_position_size_sol)
            .max(0.01) // Minimum 0.01 SOL
            .min(self.config.max_position_size_sol);
        
        debug!("📏 Recommended amount: {:.3} SOL (Kelly: {:.3})", recommended, kelly_fraction);
        
        Ok(recommended)
    }

    /// ⏰ Check if opportunity is time sensitive
    fn is_time_sensitive(&self, opportunity_type: &OpportunityType, analysis: &TokenAnalysis) -> bool {
        match opportunity_type {
            OpportunityType::LiquiditySnipe => true,
            OpportunityType::EmergencyExit => true,
            OpportunityType::ArbitragePlay => analysis.volume_24h > 1000000.0,
            _ => false,
        }
    }

    /// ✅ Validate opportunity is still fresh
    async fn validate_opportunity_freshness(&self, opportunity: &TradingOpportunity) -> Result<()> {
        // Re-analyze token to ensure conditions haven't changed
        let fresh_analysis = self.helius_client.analyze_token_filtered(&opportunity.token_mint).await?;
        
        // Check if rug pull score increased significantly
        if fresh_analysis.rug_pull_score > opportunity.analysis.rug_pull_score + 0.2 {
            return Err(anyhow!("Rug pull risk increased significantly"));
        }
        
        // Check if liquidity dropped
        if fresh_analysis.liquidity_score < opportunity.analysis.liquidity_score * 0.8 {
            return Err(anyhow!("Liquidity dropped significantly"));
        }
        
        Ok(())
    }

    /// 📊 Calculate position size for execution
    fn calculate_position_size(&self, opportunity: &TradingOpportunity) -> Result<f64> {
        // Use recommended amount, but apply additional safety checks
        let base_amount = opportunity.recommended_amount;
        
        // Reduce size for high-risk opportunities
        let risk_adjustment = if opportunity.risk_score > 0.7 {
            0.5 // Halve position size for high risk
        } else if opportunity.risk_score > 0.5 {
            0.75 // Reduce by 25% for medium risk
        } else {
            1.0 // Full size for low risk
        };
        
        let final_amount = base_amount * risk_adjustment;
        
        debug!("📊 Position size: {:.3} SOL (base: {:.3}, risk_adj: {:.2})", 
               final_amount, base_amount, risk_adjustment);
        
        Ok(final_amount)
    }

    /// 🎯 Determine if should use Jito bundle
    fn should_use_jito_bundle(&self, opportunity: &TradingOpportunity) -> bool {
        self.config.use_jito_bundles && (
            opportunity.time_sensitive ||
            matches!(opportunity.opportunity_type, OpportunityType::LiquiditySnipe | OpportunityType::ArbitragePlay)
        )
    }

    /// 💸 Calculate priority fee
    async fn calculate_priority_fee(&self, opportunity: &TradingOpportunity) -> Result<u64> {
        // Get current network metrics
        let metrics = self.quicknode_client.get_network_metrics().await?;
        
        // Base fee on network conditions and opportunity type
        let base_fee = if opportunity.time_sensitive {
            metrics.priority_fee_percentile_95 // High priority for time-sensitive
        } else {
            metrics.priority_fee_percentile_50 // Normal priority
        };
        
        // Adjust based on expected profit
        let profit_multiplier = if opportunity.expected_profit > 0.2 {
            1.5 // Higher fee for high-profit opportunities
        } else {
            1.0
        };
        
        let final_fee = (base_fee as f64 * profit_multiplier) as u64;
        
        debug!("💸 Priority fee: {} lamports (base: {}, multiplier: {:.1})", 
               final_fee, base_fee, profit_multiplier);
        
        Ok(final_fee)
    }
}

impl Default for PiranhaConfig {
    fn default() -> Self {
        Self {
            max_position_size_sol: 0.1,        // 0.1 SOL max per trade
            min_liquidity_score: 0.5,          // Minimum 50% liquidity score
            max_rug_pull_score: 0.3,           // Maximum 30% rug pull risk
            max_slippage: 0.05,                // 5% max slippage
            use_jito_bundles: true,             // Use Jito for MEV protection
            emergency_exit_threshold: 0.8,     // Exit if rug pull risk > 80%
            profit_target: 0.15,               // 15% profit target
            stop_loss: 0.1,                    // 10% stop loss
        }
    }
}
