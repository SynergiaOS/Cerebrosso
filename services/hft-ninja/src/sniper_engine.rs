//! 🎯 Sniper Profile Engine - Fast Token Filtering & Analysis
//! 
//! Quick filtering system for incoming webhook tokens before sending to Cerebro-BFF.
//! Designed for <10ms analysis time per token.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::config::SniperConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenProfile {
    pub mint: String,
    pub score: f64,
    pub signals: Vec<TradingSignal>,
    pub risk_level: RiskLevel,
    pub analysis_timestamp: i64,
    pub recommended_action: RecommendedAction,
    pub top_signals: Vec<TradingSignal>,
    pub potential_score: f64,
    pub risk_score: f64,
    pub weighted_score: f64,
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
    pub weight: f64,       // Signal weight from configuration
    pub weighted_strength: f64, // strength * weight * confidence
    pub signal_name: String, // Human-readable signal name for TF-IDF
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    VolumeSpike,
    PriceMovement,
    LiquidityChange,
    NewListing,
    WhaleActivity,
    SocialSentiment,
    // New signal types for enhanced analysis
    LowDevAllocation,
    NoFreezeFunction,
    HighLiquidity,
    VerifiedContract,
    DoxxedTeam,
    HighVolatility,
    LowHolderCount,
    SuspiciousMetadata,
    RugPullIndicators,
    PumpFunListing,
}

/// 📊 Market context for dynamic signal weighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContext {
    pub market_volatility: f64,      // Overall market volatility (0.0-1.0)
    pub memecoin_season: bool,       // Is it memecoin season?
    pub risk_appetite: f64,          // Current risk appetite (0.0-1.0)
    pub volume_trend: VolumeTrend,   // Market volume trend
    pub last_updated: i64,           // Timestamp of last update
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeTrend {
    Increasing,
    Stable,
    Decreasing,
}

/// 📈 Signal performance tracking for adaptive weighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPerformanceTracker {
    pub signal_success_rates: HashMap<String, f64>,  // Success rate per signal type
    pub signal_profit_impact: HashMap<String, f64>,  // Average profit impact per signal
    pub recent_performance: HashMap<String, Vec<f64>>, // Recent performance history
    pub last_updated: i64,
}

/// 🎯 Enhanced signal with dynamic weighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSignal {
    pub base_signal: TradingSignal,
    pub dynamic_weight: f64,         // Weight adjusted for market context
    pub confidence_adjusted: f64,    // Confidence adjusted for recent performance
    pub market_relevance: f64,       // How relevant this signal is in current market
}

pub struct SniperProfileEngine {
    // Configuration
    config: SniperConfig,

    // Performance tracking
    tokens_processed: std::sync::atomic::AtomicU64,
    tokens_passed: std::sync::atomic::AtomicU64,

    // Dynamic weighting system
    market_context: MarketContext,
    signal_performance_tracker: SignalPerformanceTracker,
}

impl Clone for SniperProfileEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            tokens_processed: std::sync::atomic::AtomicU64::new(
                self.tokens_processed.load(std::sync::atomic::Ordering::Relaxed)
            ),
            tokens_passed: std::sync::atomic::AtomicU64::new(
                self.tokens_passed.load(std::sync::atomic::Ordering::Relaxed)
            ),
            market_context: self.market_context.clone(),
            signal_performance_tracker: self.signal_performance_tracker.clone(),
        }
    }
}

impl SniperProfileEngine {
    pub fn new(config: SniperConfig) -> Self {
        Self {
            config,
            tokens_processed: std::sync::atomic::AtomicU64::new(0),
            tokens_passed: std::sync::atomic::AtomicU64::new(0),
            market_context: MarketContext::default(),
            signal_performance_tracker: SignalPerformanceTracker::default(),
        }
    }

    pub fn new_default() -> Self {
        use crate::config::Config;
        let config = Config::load().unwrap_or_else(|_| {
            tracing::warn!("Failed to load config, using defaults");
            Config::load().unwrap()
        });
        Self::new(config.sniper)
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
        if volume_usd < self.config.min_volume_usd {
            debug!("❌ Token {} filtered: volume too low ({:.0} < {:.0})",
                   mint, volume_usd, self.config.min_volume_usd);
            return None;
        }

        if liquidity_usd < self.config.min_liquidity_usd {
            debug!("❌ Token {} filtered: liquidity too low ({:.0} < {:.0})",
                   mint, liquidity_usd, self.config.min_liquidity_usd);
            return None;
        }

        // Calculate risk and opportunity scores
        let risk_score = self.calculate_risk_score(token_data);
        let opportunity_score = self.calculate_opportunity_score(token_data, volume_usd, liquidity_usd);

        if risk_score > self.config.max_risk_score {
            debug!("❌ Token {} filtered: risk too high ({:.2} > {:.2})",
                   mint, risk_score, self.config.max_risk_score);
            return None;
        }

        if opportunity_score < self.config.min_opportunity_score {
            debug!("❌ Token {} filtered: opportunity too low ({:.2} < {:.2})",
                   mint, opportunity_score, self.config.min_opportunity_score);
            return None;
        }

        // Generate weighted signals
        let signals = self.extract_weighted_signals(token_data, volume_usd, liquidity_usd);
        let (potential_score, risk_score_weighted, weighted_score) = self.calculate_weighted_scores(&signals);

        // Get top signals for Context Engine
        let top_signals = self.get_top_signals(&signals, self.config.top_signals_count);
        let risk_level = self.map_risk_level(risk_score);
        let recommended_action = self.determine_action(&risk_level, potential_score, &signals);

        let profile = TokenProfile {
            mint: mint.clone(),
            score: weighted_score,
            signals: signals.clone(),
            risk_level: risk_level.clone(),
            analysis_timestamp: Utc::now().timestamp(),
            recommended_action: recommended_action.clone(),
            top_signals,
            potential_score,
            risk_score: risk_score_weighted,
            weighted_score,
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
            let strength = (volume_usd / 50000.0).min(1.0);
            let confidence = 0.8;
            let weight = 0.7; // Default weight
            signals.push(TradingSignal {
                signal_type: SignalType::VolumeSpike,
                strength,
                confidence,
                source: "volume_analysis".to_string(),
                weight,
                weighted_strength: strength * weight * confidence,
                signal_name: "volume_spike".to_string(),
            });
        }

        // Price movement signal
        if let Some(price_change) = data["price_change_24h_percent"].as_f64() {
            if price_change.abs() > 10.0 {
                let strength = (price_change.abs() / 100.0).min(1.0);
                let confidence = 0.7;
                let weight = 0.6; // Default weight
                signals.push(TradingSignal {
                    signal_type: SignalType::PriceMovement,
                    strength,
                    confidence,
                    source: "price_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name: "price_momentum".to_string(),
                });
            }
        }

        // New listing signal
        if let Some(created_at) = data["created_at"].as_str() {
            if let Ok(created) = DateTime::parse_from_rfc3339(created_at) {
                let age_hours = (Utc::now() - created.with_timezone(&Utc)).num_hours();
                if age_hours < 24 {
                    let strength = ((24 - age_hours) as f64 / 24.0).max(0.0);
                    let confidence = 0.6;
                    let weight = 0.5; // Default weight
                    signals.push(TradingSignal {
                        signal_type: SignalType::NewListing,
                        strength,
                        confidence,
                        source: "age_analysis".to_string(),
                        weight,
                        weighted_strength: strength * weight * confidence,
                        signal_name: "new_listing".to_string(),
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

    /// 🎯 Extract weighted trading signals with configuration-based weights
    fn extract_weighted_signals(&self, data: &serde_json::Value, volume_usd: f64, liquidity_usd: f64) -> Vec<TradingSignal> {
        let mut signals = Vec::new();

        // Volume spike signal
        if volume_usd > 50000.0 {
            let signal_name = "volume_spike".to_string();
            let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.7);
            let strength = (volume_usd / 50000.0).min(1.0);
            let confidence = 0.8;

            signals.push(TradingSignal {
                signal_type: SignalType::VolumeSpike,
                strength,
                confidence,
                source: "volume_analysis".to_string(),
                weight,
                weighted_strength: strength * weight * confidence,
                signal_name,
            });
        }

        // Price movement signal
        if let Some(price_change) = data["price_change_24h_percent"].as_f64() {
            if price_change.abs() > 10.0 {
                let signal_name = "price_momentum".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.6);
                let strength = (price_change.abs() / 100.0).min(1.0);
                let confidence = 0.7;

                signals.push(TradingSignal {
                    signal_type: SignalType::PriceMovement,
                    strength,
                    confidence,
                    source: "price_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // High liquidity signal
        if liquidity_usd > 50000.0 {
            let signal_name = "high_liquidity".to_string();
            let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.7);
            let strength = (liquidity_usd / 100000.0).min(1.0);
            let confidence = 0.9;

            signals.push(TradingSignal {
                signal_type: SignalType::HighLiquidity,
                strength,
                confidence,
                source: "liquidity_analysis".to_string(),
                weight,
                weighted_strength: strength * weight * confidence,
                signal_name,
            });
        }

        // New listing signal
        if let Some(created_at) = data["created_at"].as_str() {
            if let Ok(created) = DateTime::parse_from_rfc3339(created_at) {
                let age_hours = (Utc::now() - created.with_timezone(&Utc)).num_hours();
                if age_hours < 24 {
                    let signal_name = "new_listing".to_string();
                    let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.5);
                    let strength = ((24 - age_hours) as f64 / 24.0).max(0.0);
                    let confidence = 0.6;

                    signals.push(TradingSignal {
                        signal_type: SignalType::NewListing,
                        strength,
                        confidence,
                        source: "age_analysis".to_string(),
                        weight,
                        weighted_strength: strength * weight * confidence,
                        signal_name,
                    });
                }
            }
        }

        // Risk signals (negative weights)
        if let Some(price_change) = data["price_change_24h_percent"].as_f64() {
            if price_change.abs() > 100.0 {
                let signal_name = "high_volatility".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(-0.3);
                let strength = (price_change.abs() / 200.0).min(1.0);
                let confidence = 0.8;

                signals.push(TradingSignal {
                    signal_type: SignalType::HighVolatility,
                    strength,
                    confidence,
                    source: "volatility_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // 🛡️ SECURITY SIGNALS - Based on memecoin research

        // Dev allocation check (critical for rug pull prevention)
        if let Some(dev_allocation) = data["dev_allocation_percent"].as_f64() {
            if dev_allocation < 10.0 {
                let signal_name = "low_dev_allocation".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.9);
                let strength = (10.0 - dev_allocation) / 10.0; // Higher strength for lower dev allocation
                let confidence = 0.95;

                signals.push(TradingSignal {
                    signal_type: SignalType::LowDevAllocation,
                    strength,
                    confidence,
                    source: "security_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            } else if dev_allocation > 50.0 {
                // High dev allocation is a red flag
                let signal_name = "rug_pull_indicators".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(-0.9);
                let strength = (dev_allocation - 50.0) / 50.0; // Higher strength for higher dev allocation
                let confidence = 0.9;

                signals.push(TradingSignal {
                    signal_type: SignalType::RugPullIndicators,
                    strength,
                    confidence,
                    source: "security_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // Freeze function check (no freeze = good)
        if let Some(has_freeze) = data["has_freeze_function"].as_bool() {
            if !has_freeze {
                let signal_name = "no_freeze_function".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.8);
                let strength = 1.0; // Full strength if no freeze function
                let confidence = 0.9;

                signals.push(TradingSignal {
                    signal_type: SignalType::NoFreezeFunction,
                    strength,
                    confidence,
                    source: "contract_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // Holder distribution analysis
        if let Some(holder_count) = data["holder_count"].as_f64() {
            if holder_count < 50.0 {
                // Low holder count is risky
                let signal_name = "low_holder_count".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(-0.4);
                let strength = (50.0 - holder_count) / 50.0;
                let confidence = 0.8;

                signals.push(TradingSignal {
                    signal_type: SignalType::LowHolderCount,
                    strength,
                    confidence,
                    source: "holder_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // Contract verification check
        if let Some(is_verified) = data["is_verified"].as_bool() {
            if is_verified {
                let signal_name = "verified_contract".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.8);
                let strength = 1.0;
                let confidence = 0.85;

                signals.push(TradingSignal {
                    signal_type: SignalType::VerifiedContract,
                    strength,
                    confidence,
                    source: "contract_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // Team doxxed check
        if let Some(team_doxxed) = data["team_doxxed"].as_bool() {
            if team_doxxed {
                let signal_name = "doxxed_team".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.6);
                let strength = 1.0;
                let confidence = 0.7;

                signals.push(TradingSignal {
                    signal_type: SignalType::DoxxedTeam,
                    strength,
                    confidence,
                    source: "team_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // Suspicious metadata check
        if let Some(metadata) = data["metadata"].as_object() {
            let mut suspicious_score: f64 = 0.0;

            // Check for suspicious patterns in name/description
            if let Some(name) = metadata.get("name").and_then(|v| v.as_str()) {
                if name.contains("🚀") || name.contains("💎") || name.contains("MOON") ||
                   name.to_lowercase().contains("safe") || name.to_lowercase().contains("inu") {
                    suspicious_score += 0.3;
                }
            }

            if let Some(description) = metadata.get("description").and_then(|v| v.as_str()) {
                if description.to_lowercase().contains("guaranteed") ||
                   description.to_lowercase().contains("100x") ||
                   description.to_lowercase().contains("lambo") {
                    suspicious_score += 0.4;
                }
            }

            if suspicious_score > 0.0 {
                let signal_name = "suspicious_metadata".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(-0.8);
                let strength = suspicious_score.min(1.0);
                let confidence = 0.7;

                signals.push(TradingSignal {
                    signal_type: SignalType::SuspiciousMetadata,
                    strength,
                    confidence,
                    source: "metadata_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        // Pump.fun listing signal (specific platform analysis)
        if let Some(platform) = data["listing_platform"].as_str() {
            if platform == "pump.fun" {
                let signal_name = "pump_fun_listing".to_string();
                let weight = self.config.signal_weights.get(&signal_name).copied().unwrap_or(0.6);
                let strength = 1.0;
                let confidence = 0.8;

                signals.push(TradingSignal {
                    signal_type: SignalType::PumpFunListing,
                    strength,
                    confidence,
                    source: "platform_analysis".to_string(),
                    weight,
                    weighted_strength: strength * weight * confidence,
                    signal_name,
                });
            }
        }

        signals
    }

    /// 📊 Calculate weighted scores from signals
    fn calculate_weighted_scores(&self, signals: &[TradingSignal]) -> (f64, f64, f64) {
        let mut potential_score = 0.0;
        let mut risk_score = 0.0;
        let mut total_weighted_strength = 0.0;

        for signal in signals {
            let weighted_strength = signal.weighted_strength;
            total_weighted_strength += weighted_strength.abs();

            if weighted_strength > 0.0 {
                potential_score += weighted_strength;
            } else {
                risk_score += weighted_strength.abs();
            }
        }

        // Normalize scores
        potential_score = potential_score.min(1.0);
        risk_score = risk_score.min(1.0);

        // Calculate overall weighted score (potential - risk)
        let weighted_score = (potential_score - risk_score * 0.5).max(0.0).min(1.0);

        (potential_score, risk_score, weighted_score)
    }

    /// 🔝 Get top N signals by weighted strength for Context Engine
    pub fn get_top_signals(&self, signals: &[TradingSignal], n: usize) -> Vec<TradingSignal> {
        let mut sorted_signals = signals.to_vec();
        sorted_signals.sort_by(|a, b| {
            b.weighted_strength.abs().partial_cmp(&a.weighted_strength.abs()).unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_signals.into_iter().take(n).collect()
    }

    /// 🎯 **NEW: Dynamic Signal Weighting** - Adjust weights based on market context
    pub fn calculate_dynamic_weights(&self, signals: &[TradingSignal]) -> Vec<EnhancedSignal> {
        let mut enhanced_signals = Vec::new();

        for signal in signals {
            let dynamic_weight = self.calculate_context_adjusted_weight(&signal.signal_name);
            let confidence_adjusted = self.calculate_performance_adjusted_confidence(&signal.signal_name, signal.confidence);
            let market_relevance = self.calculate_market_relevance(&signal.signal_name);

            enhanced_signals.push(EnhancedSignal {
                base_signal: signal.clone(),
                dynamic_weight,
                confidence_adjusted,
                market_relevance,
            });
        }

        enhanced_signals
    }

    /// 📊 Calculate weight adjustment based on market context
    fn calculate_context_adjusted_weight(&self, signal_name: &str) -> f64 {
        let base_weight = self.config.signal_weights.get(signal_name).copied().unwrap_or(0.5);
        let mut adjustment = 1.0;

        // Adjust based on market volatility
        match signal_name {
            "high_volatility" => {
                // In high volatility markets, volatility signals are less concerning
                adjustment *= 1.0 - (self.market_context.market_volatility * 0.3);
            },
            "volume_spike" | "price_momentum" => {
                // In volatile markets, volume and momentum signals are more important
                adjustment *= 1.0 + (self.market_context.market_volatility * 0.2);
            },
            "low_dev_allocation" | "no_freeze_function" => {
                // Security signals are always important, but more so in risky markets
                adjustment *= 1.0 + (self.market_context.risk_appetite * 0.1);
            },
            _ => {}
        }

        // Adjust for memecoin season
        if self.market_context.memecoin_season {
            match signal_name {
                "pump_fun_listing" | "new_listing" => adjustment *= 1.3,
                "social_sentiment" => adjustment *= 1.2,
                "rug_pull_indicators" => adjustment *= 1.4, // Extra caution during memecoin season
                _ => {}
            }
        }

        // Adjust based on volume trend
        match self.market_context.volume_trend {
            VolumeTrend::Increasing => {
                if signal_name == "volume_spike" { adjustment *= 0.8; } // Less significant in high volume periods
            },
            VolumeTrend::Decreasing => {
                if signal_name == "volume_spike" { adjustment *= 1.3; } // More significant in low volume periods
            },
            VolumeTrend::Stable => {}
        }

        (base_weight * adjustment).max(0.0).min(1.0)
    }

    /// 📈 Adjust confidence based on recent signal performance
    fn calculate_performance_adjusted_confidence(&self, signal_name: &str, base_confidence: f64) -> f64 {
        let success_rate = self.signal_performance_tracker
            .signal_success_rates
            .get(signal_name)
            .copied()
            .unwrap_or(0.5); // Default to neutral if no data

        // Adjust confidence based on recent performance
        let performance_adjustment = (success_rate - 0.5) * 0.4; // Max ±20% adjustment
        (base_confidence + performance_adjustment).max(0.1).min(1.0)
    }

    /// 🎯 Calculate how relevant this signal is in current market conditions
    fn calculate_market_relevance(&self, signal_name: &str) -> f64 {
        let mut relevance = 1.0;

        // Base relevance adjustments
        match signal_name {
            "volume_spike" | "price_momentum" => {
                relevance = 0.8 + (self.market_context.market_volatility * 0.4);
            },
            "low_dev_allocation" | "rug_pull_indicators" => {
                relevance = 0.9; // Always highly relevant for safety
            },
            "social_sentiment" => {
                relevance = if self.market_context.memecoin_season { 0.9 } else { 0.6 };
            },
            "pump_fun_listing" => {
                relevance = if self.market_context.memecoin_season { 0.95 } else { 0.7 };
            },
            _ => relevance = 0.7,
        }

        relevance.max(0.1).min(1.0)
    }

    /// 🔄 Update market context (would be called periodically)
    pub fn update_market_context(&mut self, new_context: MarketContext) {
        self.market_context = new_context;
        debug!("📊 Market context updated: volatility={:.2}, memecoin_season={}, risk_appetite={:.2}",
               self.market_context.market_volatility,
               self.market_context.memecoin_season,
               self.market_context.risk_appetite);
    }

    /// 📈 Update signal performance (would be called after trade outcomes)
    pub fn update_signal_performance(&mut self, signal_name: String, success: bool, profit_impact: f64) {
        // Update success rate
        let current_rate = self.signal_performance_tracker
            .signal_success_rates
            .get(&signal_name)
            .copied()
            .unwrap_or(0.5);

        let new_rate = current_rate * 0.9 + (if success { 1.0 } else { 0.0 }) * 0.1; // Exponential moving average
        self.signal_performance_tracker.signal_success_rates.insert(signal_name.clone(), new_rate);

        // Update profit impact
        let current_impact = self.signal_performance_tracker
            .signal_profit_impact
            .get(&signal_name)
            .copied()
            .unwrap_or(0.0);

        let new_impact = current_impact * 0.9 + profit_impact * 0.1;
        self.signal_performance_tracker.signal_profit_impact.insert(signal_name.clone(), new_impact);

        // Update recent performance history
        let recent = self.signal_performance_tracker
            .recent_performance
            .entry(signal_name.clone())
            .or_insert_with(Vec::new);

        recent.push(if success { 1.0 } else { 0.0 });
        if recent.len() > 20 { // Keep only last 20 results
            recent.remove(0);
        }

        self.signal_performance_tracker.last_updated = Utc::now().timestamp();

        debug!("📈 Signal performance updated: {} -> success_rate={:.2}, profit_impact={:.2}",
               signal_name, new_rate, new_impact);
    }
}

impl Default for SniperProfileEngine {
    fn default() -> Self {
        Self::new_default()
    }
}

impl Default for MarketContext {
    fn default() -> Self {
        Self {
            market_volatility: 0.5,
            memecoin_season: false,
            risk_appetite: 0.6,
            volume_trend: VolumeTrend::Stable,
            last_updated: Utc::now().timestamp(),
        }
    }
}

impl Default for SignalPerformanceTracker {
    fn default() -> Self {
        Self {
            signal_success_rates: HashMap::new(),
            signal_profit_impact: HashMap::new(),
            recent_performance: HashMap::new(),
            last_updated: Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_signal_weighting() {
        let mut engine = SniperProfileEngine::new_default();

        // Create test signals with weights matching config
        let signals = vec![
            TradingSignal {
                signal_type: SignalType::VolumeSpike,
                strength: 0.8,
                confidence: 0.9,
                source: "test".to_string(),
                weight: 0.7, // From config: volume_spike = 0.7
                weighted_strength: 0.8 * 0.7 * 0.9,
                signal_name: "volume_spike".to_string(),
            },
            TradingSignal {
                signal_type: SignalType::HighVolatility,
                strength: 0.6,
                confidence: 0.8,
                source: "test".to_string(),
                weight: -0.3, // From config: high_volatility = -0.3
                weighted_strength: 0.6 * (-0.3) * 0.8,
                signal_name: "high_volatility".to_string(),
            },
        ];

        // Test with default market context
        let enhanced_signals = engine.calculate_dynamic_weights(&signals);
        assert_eq!(enhanced_signals.len(), 2);

        println!("Default context - volume_spike weight: {:.3}", enhanced_signals[0].dynamic_weight);
        println!("Default context - high_volatility weight: {:.3}", enhanced_signals[1].dynamic_weight);

        // Test with high volatility market context
        let high_volatility_context = MarketContext {
            market_volatility: 0.9,
            memecoin_season: false,
            risk_appetite: 0.7,
            volume_trend: VolumeTrend::Increasing,
            last_updated: Utc::now().timestamp(),
        };

        engine.update_market_context(high_volatility_context);
        let enhanced_signals_volatile = engine.calculate_dynamic_weights(&signals);

        println!("High volatility context - volume_spike weight: {:.3}", enhanced_signals_volatile[0].dynamic_weight);
        println!("High volatility context - high_volatility weight: {:.3}", enhanced_signals_volatile[1].dynamic_weight);

        // Volume spike should have higher weight in volatile markets
        assert!(enhanced_signals_volatile[0].dynamic_weight > enhanced_signals[0].dynamic_weight,
                "Volume spike weight should increase in volatile markets: {:.3} > {:.3}",
                enhanced_signals_volatile[0].dynamic_weight, enhanced_signals[0].dynamic_weight);

        // High volatility signal should have lower negative impact in volatile markets (less negative)
        assert!(enhanced_signals_volatile[1].dynamic_weight > enhanced_signals[1].dynamic_weight,
                "High volatility signal should be less negative in volatile markets: {:.3} > {:.3}",
                enhanced_signals_volatile[1].dynamic_weight, enhanced_signals[1].dynamic_weight);
    }

    #[test]
    fn test_signal_performance_tracking() {
        let mut engine = SniperProfileEngine::new_default();

        // Update performance for a signal
        engine.update_signal_performance("volume_spike".to_string(), true, 0.15);
        engine.update_signal_performance("volume_spike".to_string(), true, 0.20);
        engine.update_signal_performance("volume_spike".to_string(), false, -0.05);

        // Check that success rate is updated
        let success_rate = engine.signal_performance_tracker
            .signal_success_rates
            .get("volume_spike")
            .unwrap();

        // Should be around 0.67 (2 successes out of 3)
        assert!(*success_rate > 0.6 && *success_rate < 0.8);

        // Check profit impact
        let profit_impact = engine.signal_performance_tracker
            .signal_profit_impact
            .get("volume_spike")
            .unwrap();

        assert!(*profit_impact > 0.0); // Should be positive overall
    }

    #[test]
    fn test_memecoin_season_adjustments() {
        let mut engine = SniperProfileEngine::new_default();

        let signals = vec![
            TradingSignal {
                signal_type: SignalType::PumpFunListing,
                strength: 0.8,
                confidence: 0.9,
                source: "test".to_string(),
                weight: 0.6,
                weighted_strength: 0.8 * 0.6 * 0.9,
                signal_name: "pump_fun_listing".to_string(),
            },
        ];

        // Test without memecoin season
        let normal_enhanced = engine.calculate_dynamic_weights(&signals);

        // Test with memecoin season
        let memecoin_context = MarketContext {
            market_volatility: 0.5,
            memecoin_season: true,
            risk_appetite: 0.6,
            volume_trend: VolumeTrend::Stable,
            last_updated: Utc::now().timestamp(),
        };

        engine.update_market_context(memecoin_context);
        let memecoin_enhanced = engine.calculate_dynamic_weights(&signals);

        // Pump.fun listing should have higher weight during memecoin season
        assert!(memecoin_enhanced[0].dynamic_weight > normal_enhanced[0].dynamic_weight);
        assert!(memecoin_enhanced[0].market_relevance > normal_enhanced[0].market_relevance);
    }
}
