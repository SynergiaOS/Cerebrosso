//! 🚀 Pump.fun Token Scanner - Real-time New Token Detection
//! 
//! Advanced scanner for pump.fun platform with real-time webhook integration,
//! risk analysis, and automated trading signal generation.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};

use crate::helius_client::HeliusClient;
use crate::qdrant_client::{QdrantClient, TokenRiskAnalysis};

/// 🎯 Pump.fun token discovery event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunTokenEvent {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub creator: String,
    pub initial_liquidity: f64,
    pub market_cap: f64,
    pub created_at: DateTime<Utc>,
    pub pump_fun_url: String,
    pub metadata_uri: String,
}

/// 📊 Token analysis result with trading signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysisResult {
    pub token_event: PumpFunTokenEvent,
    pub risk_analysis: TokenRiskAnalysis,
    pub trading_signals: Vec<TradingSignal>,
    pub recommendation: TradingRecommendation,
    pub confidence_score: f64,
    pub analyzed_at: DateTime<Utc>,
}

/// 📈 Trading signal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingSignal {
    StrongBuy { reason: String, confidence: f64 },
    Buy { reason: String, confidence: f64 },
    Hold { reason: String, confidence: f64 },
    Sell { reason: String, confidence: f64 },
    StrongSell { reason: String, confidence: f64 },
    Avoid { reason: String, confidence: f64 },
}

/// 🎯 Trading recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingRecommendation {
    HighPotential { max_investment: f64, stop_loss: f64, take_profit: f64 },
    ModerateRisk { max_investment: f64, stop_loss: f64, take_profit: f64 },
    HighRisk { max_investment: f64, stop_loss: f64, take_profit: f64 },
    Avoid { reason: String },
}

/// 🔍 Scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub min_liquidity_usd: f64,
    pub max_risk_score: f64,
    pub min_confidence: f64,
    pub scan_interval_ms: u64,
    pub max_tokens_per_minute: u32,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 1000.0,
            max_risk_score: 0.7,
            min_confidence: 0.6,
            scan_interval_ms: 5000,
            max_tokens_per_minute: 20,
        }
    }
}

/// 🚀 Pump.fun Token Scanner
pub struct PumpFunScanner {
    config: ScannerConfig,
    helius_client: Arc<HeliusClient>,
    qdrant_client: Arc<QdrantClient>,
    discovered_tokens: Arc<RwLock<HashMap<String, TokenAnalysisResult>>>,
    scan_stats: Arc<Mutex<ScanStats>>,
}

/// 📊 Scanner statistics
#[derive(Debug, Clone, Default)]
pub struct ScanStats {
    pub total_tokens_discovered: u64,
    pub tokens_analyzed: u64,
    pub high_potential_tokens: u64,
    pub avoided_tokens: u64,
    pub last_scan_time: Option<Instant>,
    pub avg_analysis_time_ms: f64,
}

impl PumpFunScanner {
    /// 🚀 Initialize pump.fun scanner
    pub fn new(
        config: ScannerConfig,
        helius_client: Arc<HeliusClient>,
        qdrant_client: Arc<QdrantClient>,
    ) -> Self {
        Self {
            config,
            helius_client,
            qdrant_client,
            discovered_tokens: Arc::new(RwLock::new(HashMap::new())),
            scan_stats: Arc::new(Mutex::new(ScanStats::default())),
        }
    }

    /// 🔍 Process new token from webhook
    pub async fn process_token_event(&self, event: PumpFunTokenEvent) -> Result<TokenAnalysisResult> {
        let start_time = Instant::now();
        
        info!("🔍 Processing new pump.fun token: {} ({})", event.name, event.token_address);
        
        // Quick filters
        if event.initial_liquidity < self.config.min_liquidity_usd {
            debug!("❌ Token {} rejected: liquidity too low ({:.2})", event.token_address, event.initial_liquidity);
            return Err(anyhow!("Liquidity below minimum threshold"));
        }
        
        // Create metadata for risk analysis
        let metadata = serde_json::json!({
            "name": event.name,
            "symbol": event.symbol,
            "description": event.description,
            "creator": event.creator,
            "initial_liquidity": event.initial_liquidity,
            "market_cap": event.market_cap,
            "created_at": event.created_at,
            "platform": "pump.fun",
            "holder_count": 1, // Initial holder is creator
            "liquidity_usd": event.initial_liquidity,
            "volume_24h": 0.0,
            "is_verified": false,
            "team_doxxed": false
        });
        
        // Perform risk analysis
        let risk_analysis = self.qdrant_client
            .analyze_token_risk(&event.token_address, &metadata)
            .await?;
        
        // Generate trading signals
        let trading_signals = self.generate_trading_signals(&event, &risk_analysis).await?;
        
        // Create recommendation
        let recommendation = self.create_recommendation(&event, &risk_analysis, &trading_signals).await?;
        
        // Calculate overall confidence
        let confidence_score = self.calculate_confidence(&risk_analysis, &trading_signals);
        
        let analysis_result = TokenAnalysisResult {
            token_event: event.clone(),
            risk_analysis,
            trading_signals,
            recommendation,
            confidence_score,
            analyzed_at: Utc::now(),
        };
        
        // Store result
        {
            let mut discovered = self.discovered_tokens.write().await;
            discovered.insert(event.token_address.clone(), analysis_result.clone());
        }
        
        // Update statistics
        {
            let mut stats = self.scan_stats.lock().await;
            stats.tokens_analyzed += 1;
            stats.last_scan_time = Some(start_time);
            
            let analysis_time = start_time.elapsed().as_millis() as f64;
            if stats.avg_analysis_time_ms == 0.0 {
                stats.avg_analysis_time_ms = analysis_time;
            } else {
                stats.avg_analysis_time_ms = stats.avg_analysis_time_ms * 0.9 + analysis_time * 0.1;
            }
            
            match &analysis_result.recommendation {
                TradingRecommendation::HighPotential { .. } => stats.high_potential_tokens += 1,
                TradingRecommendation::Avoid { .. } => stats.avoided_tokens += 1,
                _ => {}
            }
        }
        
        info!("✅ Token analysis complete: {} - {:.2}% risk, {:.2}% confidence", 
              event.token_address, 
              analysis_result.risk_analysis.overall_risk_score * 100.0,
              confidence_score * 100.0);
        
        Ok(analysis_result)
    }

    /// 📈 Generate trading signals based on analysis
    async fn generate_trading_signals(&self, event: &PumpFunTokenEvent, risk: &TokenRiskAnalysis) -> Result<Vec<TradingSignal>> {
        let mut signals = Vec::new();
        
        // Risk-based signals
        if risk.overall_risk_score < 0.3 {
            signals.push(TradingSignal::Buy {
                reason: "Low risk profile detected".to_string(),
                confidence: 1.0 - risk.overall_risk_score,
            });
        } else if risk.overall_risk_score > 0.8 {
            signals.push(TradingSignal::Avoid {
                reason: "High risk profile - potential rug pull".to_string(),
                confidence: risk.overall_risk_score,
            });
        }
        
        // Liquidity signals
        if event.initial_liquidity > 50000.0 {
            signals.push(TradingSignal::Buy {
                reason: "Strong initial liquidity".to_string(),
                confidence: 0.7,
            });
        } else if event.initial_liquidity < 5000.0 {
            signals.push(TradingSignal::Sell {
                reason: "Low liquidity - exit risk".to_string(),
                confidence: 0.8,
            });
        }
        
        // Market cap signals
        if event.market_cap < 100000.0 && risk.overall_risk_score < 0.5 {
            signals.push(TradingSignal::StrongBuy {
                reason: "Low market cap with acceptable risk".to_string(),
                confidence: 0.8,
            });
        }
        
        // Keyword analysis signals
        let description_lower = event.description.to_lowercase();
        let high_risk_keywords = ["moon", "100x", "gem", "pump", "rocket"];
        let keyword_count = high_risk_keywords.iter()
            .filter(|&keyword| description_lower.contains(keyword))
            .count();
        
        if keyword_count >= 3 {
            signals.push(TradingSignal::Avoid {
                reason: "Multiple pump keywords detected".to_string(),
                confidence: 0.9,
            });
        }
        
        // If no signals generated, default to hold
        if signals.is_empty() {
            signals.push(TradingSignal::Hold {
                reason: "Neutral analysis - monitor for changes".to_string(),
                confidence: 0.5,
            });
        }
        
        Ok(signals)
    }

    /// 🎯 Create trading recommendation
    async fn create_recommendation(&self, event: &PumpFunTokenEvent, risk: &TokenRiskAnalysis, signals: &[TradingSignal]) -> Result<TradingRecommendation> {
        // Count signal types
        let mut buy_signals = 0;
        let mut sell_signals = 0;
        let mut avoid_signals = 0;
        
        for signal in signals {
            match signal {
                TradingSignal::StrongBuy { .. } | TradingSignal::Buy { .. } => buy_signals += 1,
                TradingSignal::Sell { .. } | TradingSignal::StrongSell { .. } => sell_signals += 1,
                TradingSignal::Avoid { .. } => avoid_signals += 1,
                _ => {}
            }
        }
        
        // High risk or avoid signals
        if avoid_signals > 0 || risk.overall_risk_score > self.config.max_risk_score {
            return Ok(TradingRecommendation::Avoid {
                reason: format!("Risk score: {:.2}%, avoid signals: {}", risk.overall_risk_score * 100.0, avoid_signals),
            });
        }
        
        // Calculate position sizing based on risk
        let base_investment = 0.1; // 0.1 SOL base
        let risk_multiplier = 1.0 - risk.overall_risk_score;
        let max_investment = base_investment * risk_multiplier;
        
        // Set stop loss and take profit based on risk
        let stop_loss = if risk.overall_risk_score > 0.5 { 0.15 } else { 0.25 }; // 15-25% stop loss
        let take_profit = if risk.overall_risk_score < 0.3 { 3.0 } else { 2.0 }; // 200-300% take profit
        
        if buy_signals > sell_signals && risk.overall_risk_score < 0.4 {
            Ok(TradingRecommendation::HighPotential {
                max_investment,
                stop_loss,
                take_profit,
            })
        } else if buy_signals > 0 && risk.overall_risk_score < 0.6 {
            Ok(TradingRecommendation::ModerateRisk {
                max_investment: max_investment * 0.5,
                stop_loss,
                take_profit: take_profit * 0.8,
            })
        } else {
            Ok(TradingRecommendation::HighRisk {
                max_investment: max_investment * 0.25,
                stop_loss: 0.1, // Tight stop loss
                take_profit: 1.5, // Conservative take profit
            })
        }
    }

    /// 📊 Calculate overall confidence score
    fn calculate_confidence(&self, risk: &TokenRiskAnalysis, signals: &[TradingSignal]) -> f64 {
        let risk_confidence = risk.confidence_level;
        
        let signal_confidence: f64 = signals.iter()
            .map(|signal| match signal {
                TradingSignal::StrongBuy { confidence, .. } => *confidence,
                TradingSignal::Buy { confidence, .. } => *confidence,
                TradingSignal::Hold { confidence, .. } => *confidence,
                TradingSignal::Sell { confidence, .. } => *confidence,
                TradingSignal::StrongSell { confidence, .. } => *confidence,
                TradingSignal::Avoid { confidence, .. } => *confidence,
            })
            .sum::<f64>() / signals.len() as f64;
        
        (risk_confidence * 0.6 + signal_confidence * 0.4).min(1.0)
    }

    /// 📊 Get scanner statistics
    pub async fn get_stats(&self) -> ScanStats {
        self.scan_stats.lock().await.clone()
    }

    /// 🔍 Get discovered tokens
    pub async fn get_discovered_tokens(&self) -> HashMap<String, TokenAnalysisResult> {
        self.discovered_tokens.read().await.clone()
    }

    /// 🎯 Get high potential tokens
    pub async fn get_high_potential_tokens(&self) -> Vec<TokenAnalysisResult> {
        let discovered = self.discovered_tokens.read().await;
        discovered.values()
            .filter(|result| matches!(result.recommendation, TradingRecommendation::HighPotential { .. }))
            .filter(|result| result.confidence_score >= self.config.min_confidence)
            .cloned()
            .collect()
    }

    /// 🧹 Cleanup old tokens (older than 24 hours)
    pub async fn cleanup_old_tokens(&self) {
        let mut discovered = self.discovered_tokens.write().await;
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        
        let initial_count = discovered.len();
        discovered.retain(|_, result| result.analyzed_at > cutoff);
        let removed_count = initial_count - discovered.len();
        
        if removed_count > 0 {
            info!("🧹 Cleaned up {} old token analyses", removed_count);
        }
    }
}
