//! 🎯 Sniper Profile Engine - Fast Token Filtering & Analysis
//! 
//! Quick filtering system for incoming webhook tokens before sending to Cerebro-BFF.
//! Designed for <10ms analysis time per token.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenProfile {
    pub mint: String,
    pub score: f64,
    pub signals: Vec<TradingSignal>,
    pub risk_level: RiskLevel,
    pub analysis_timestamp: i64,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,      // 0.0-0.3
    Medium,   // 0.3-0.6  
    High,     // 0.6-0.8
    Extreme,  // 0.8-1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    SendToCerebro,
    Monitor,
    Ignore,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub signal_type: SignalType,
    pub strength: f64,     // 0.0-1.0
    pub confidence: f64,   // 0.0-1.0
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    VolumeSpike,
    PriceMovement,
    LiquidityChange,
    NewListing,
    WhaleActivity,
    SocialSentiment,
}

pub struct SniperProfileEngine {
    // Filtering thresholds
    min_volume_usd: f64,
    min_liquidity_usd: f64,
    max_risk_score: f64,
    min_opportunity_score: f64,
    
    // Performance tracking
    tokens_processed: std::sync::atomic::AtomicU64,
    tokens_passed: std::sync::atomic::AtomicU64,
}

impl Clone for SniperProfileEngine {
    fn clone(&self) -> Self {
        Self {
            min_volume_usd: self.min_volume_usd,
            min_liquidity_usd: self.min_liquidity_usd,
            max_risk_score: self.max_risk_score,
            min_opportunity_score: self.min_opportunity_score,
            tokens_processed: std::sync::atomic::AtomicU64::new(
                self.tokens_processed.load(std::sync::atomic::Ordering::Relaxed)
            ),
            tokens_passed: std::sync::atomic::AtomicU64::new(
                self.tokens_passed.load(std::sync::atomic::Ordering::Relaxed)
            ),
        }
    }
}

impl SniperProfileEngine {
    pub fn new() -> Self {
        Self {
            min_volume_usd: 1000.0,        // $1K minimum volume
            min_liquidity_usd: 5000.0,     // $5K minimum liquidity
            max_risk_score: 0.75,          // Max 75% risk
            min_opportunity_score: 0.6,    // Min 60% opportunity
            tokens_processed: std::sync::atomic::AtomicU64::new(0),
            tokens_passed: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 🎯 Main analysis function - designed for <10ms execution
    pub fn analyze_token(&self, token_data: &serde_json::Value) -> Option<TokenProfile> {
        let start = std::time::Instant::now();
        self.tokens_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Quick extraction of key metrics
        let mint = token_data["mint"].as_str()?.to_string();
        let volume_usd = self.extract_volume_usd(token_data)?;
        let liquidity_usd = self.extract_liquidity_usd(token_data);
        
        debug!("🔍 Analyzing token: {} (volume: ${:.0}, liquidity: ${:.0})", 
               mint, volume_usd, liquidity_usd);

        // Fast filtering - fail fast approach
        if volume_usd < self.min_volume_usd {
            debug!("❌ Token {} filtered: volume too low ({:.0} < {:.0})", 
                   mint, volume_usd, self.min_volume_usd);
            return None;
        }

        if liquidity_usd < self.min_liquidity_usd {
            debug!("❌ Token {} filtered: liquidity too low ({:.0} < {:.0})", 
                   mint, liquidity_usd, self.min_liquidity_usd);
            return None;
        }

        // Calculate risk and opportunity scores
        let risk_score = self.calculate_risk_score(token_data);
        let opportunity_score = self.calculate_opportunity_score(token_data, volume_usd, liquidity_usd);

        if risk_score > self.max_risk_score {
            debug!("❌ Token {} filtered: risk too high ({:.2} > {:.2})", 
                   mint, risk_score, self.max_risk_score);
            return None;
        }

        if opportunity_score < self.min_opportunity_score {
            debug!("❌ Token {} filtered: opportunity too low ({:.2} < {:.2})", 
                   mint, opportunity_score, self.min_opportunity_score);
            return None;
        }

        // Generate signals
        let signals = self.extract_signals(token_data, volume_usd, liquidity_usd);
        let risk_level = self.map_risk_level(risk_score);
        let recommended_action = self.determine_action(&risk_level, opportunity_score, &signals);

        let profile = TokenProfile {
            mint: mint.clone(),
            score: opportunity_score,
            signals,
            risk_level: risk_level.clone(),
            analysis_timestamp: Utc::now().timestamp(),
            recommended_action: recommended_action.clone(),
        };

        let duration = start.elapsed();
        self.tokens_passed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        info!("✅ Token {} passed analysis: score={:.2}, risk={:?}, action={:?} ({}μs)", 
              mint, opportunity_score, risk_level, recommended_action, duration.as_micros());

        Some(profile)
    }

    fn extract_volume_usd(&self, data: &serde_json::Value) -> Option<f64> {
        // Try multiple possible fields from Helius webhook
        data["volume_usd"].as_f64()
            .or_else(|| data["volume_24h_usd"].as_f64())
            .or_else(|| data["transaction"]["volume_usd"].as_f64())
            .or_else(|| {
                // Fallback: estimate from transaction count and average size
                let tx_count = data["transaction_count"].as_f64().unwrap_or(1.0);
                let avg_tx_size = data["avg_transaction_size_usd"].as_f64().unwrap_or(100.0);
                Some(tx_count * avg_tx_size)
            })
    }

    fn extract_liquidity_usd(&self, data: &serde_json::Value) -> f64 {
        data["liquidity_usd"].as_f64()
            .or_else(|| data["pool_liquidity_usd"].as_f64())
            .unwrap_or(0.0)
    }

    fn calculate_risk_score(&self, data: &serde_json::Value) -> f64 {
        let mut risk_score: f64 = 0.0;

        // Age risk (newer = riskier)
        if let Some(created_at) = data["created_at"].as_str() {
            if let Ok(created) = DateTime::parse_from_rfc3339(created_at) {
                let age_hours = (Utc::now() - created.with_timezone(&Utc)).num_hours();
                risk_score += match age_hours {
                    0..=1 => 0.4,      // Very new
                    2..=24 => 0.2,     // New
                    25..=168 => 0.1,   // Week old
                    _ => 0.0,          // Established
                };
            }
        }

        // Volatility risk
        if let Some(price_change_24h) = data["price_change_24h_percent"].as_f64() {
            risk_score += match price_change_24h.abs() {
                x if x > 100.0 => 0.3,  // Extreme volatility
                x if x > 50.0 => 0.2,   // High volatility
                x if x > 20.0 => 0.1,   // Medium volatility
                _ => 0.0,               // Stable
            };
        }

        // Liquidity risk (lower liquidity = higher risk)
        let liquidity = self.extract_liquidity_usd(data);
        risk_score += match liquidity {
            x if x < 1000.0 => 0.3,     // Very low liquidity
            x if x < 5000.0 => 0.2,     // Low liquidity
            x if x < 20000.0 => 0.1,    // Medium liquidity
            _ => 0.0,                   // Good liquidity
        };

        risk_score.min(1.0)
    }

    fn calculate_opportunity_score(&self, data: &serde_json::Value, volume_usd: f64, liquidity_usd: f64) -> f64 {
        let mut score: f64 = 0.0;

        // Volume score (higher volume = better opportunity)
        score += match volume_usd {
            x if x > 100000.0 => 0.3,   // Excellent volume
            x if x > 50000.0 => 0.25,   // Good volume
            x if x > 10000.0 => 0.2,    // Decent volume
            x if x > 1000.0 => 0.1,     // Minimum volume
            _ => 0.0,
        };

        // Liquidity score
        score += match liquidity_usd {
            x if x > 100000.0 => 0.3,   // Excellent liquidity
            x if x > 50000.0 => 0.25,   // Good liquidity
            x if x > 20000.0 => 0.2,    // Decent liquidity
            x if x > 5000.0 => 0.1,     // Minimum liquidity
            _ => 0.0,
        };

        // Price momentum
        if let Some(price_change) = data["price_change_24h_percent"].as_f64() {
            score += match price_change {
                x if x > 20.0 => 0.2,       // Strong upward momentum
                x if x > 10.0 => 0.15,      // Good momentum
                x if x > 5.0 => 0.1,        // Positive momentum
                x if x > -5.0 => 0.05,      // Stable
                _ => 0.0,                   // Declining
            };
        }

        // Transaction activity
        if let Some(tx_count) = data["transaction_count_24h"].as_f64() {
            score += match tx_count {
                x if x > 1000.0 => 0.2,     // Very active
                x if x > 500.0 => 0.15,     // Active
                x if x > 100.0 => 0.1,      // Moderate activity
                x if x > 50.0 => 0.05,      // Some activity
                _ => 0.0,
            };
        }

        score.min(1.0)
    }

    fn extract_signals(&self, data: &serde_json::Value, volume_usd: f64, _liquidity_usd: f64) -> Vec<TradingSignal> {
        let mut signals = Vec::new();

        // Volume spike signal
        if volume_usd > 50000.0 {
            signals.push(TradingSignal {
                signal_type: SignalType::VolumeSpike,
                strength: (volume_usd / 50000.0).min(1.0),
                confidence: 0.8,
                source: "volume_analysis".to_string(),
            });
        }

        // Price movement signal
        if let Some(price_change) = data["price_change_24h_percent"].as_f64() {
            if price_change.abs() > 10.0 {
                signals.push(TradingSignal {
                    signal_type: SignalType::PriceMovement,
                    strength: (price_change.abs() / 100.0).min(1.0),
                    confidence: 0.7,
                    source: "price_analysis".to_string(),
                });
            }
        }

        // New listing signal
        if let Some(created_at) = data["created_at"].as_str() {
            if let Ok(created) = DateTime::parse_from_rfc3339(created_at) {
                let age_hours = (Utc::now() - created.with_timezone(&Utc)).num_hours();
                if age_hours < 24 {
                    signals.push(TradingSignal {
                        signal_type: SignalType::NewListing,
                        strength: ((24 - age_hours) as f64 / 24.0).max(0.0),
                        confidence: 0.6,
                        source: "age_analysis".to_string(),
                    });
                }
            }
        }

        signals
    }

    fn map_risk_level(&self, score: f64) -> RiskLevel {
        match score {
            s if s < 0.3 => RiskLevel::Low,
            s if s < 0.6 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::Extreme,
        }
    }

    fn determine_action(&self, risk_level: &RiskLevel, opportunity_score: f64, signals: &[TradingSignal]) -> RecommendedAction {
        match risk_level {
            RiskLevel::Extreme => RecommendedAction::Ignore,
            RiskLevel::High => {
                if opportunity_score > 0.8 && signals.len() >= 2 {
                    RecommendedAction::Alert
                } else {
                    RecommendedAction::Monitor
                }
            },
            RiskLevel::Medium => {
                if opportunity_score > 0.7 {
                    RecommendedAction::SendToCerebro
                } else {
                    RecommendedAction::Monitor
                }
            },
            RiskLevel::Low => {
                if opportunity_score > 0.6 {
                    RecommendedAction::SendToCerebro
                } else {
                    RecommendedAction::Monitor
                }
            }
        }
    }

    pub fn get_stats(&self) -> (u64, u64, f64) {
        let processed = self.tokens_processed.load(std::sync::atomic::Ordering::Relaxed);
        let passed = self.tokens_passed.load(std::sync::atomic::Ordering::Relaxed);
        let pass_rate = if processed > 0 { passed as f64 / processed as f64 } else { 0.0 };
        (processed, passed, pass_rate)
    }
}

impl Default for SniperProfileEngine {
    fn default() -> Self {
        Self::new()
    }
}
