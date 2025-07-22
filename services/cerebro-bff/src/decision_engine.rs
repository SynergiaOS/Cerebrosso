//! 🧠 Decision Engine - Advanced Rule-Based Decision Making
//! 
//! Implements sophisticated decision logic with Apriori rules,
//! anti-pattern detection, and risk mitigation strategies.

use crate::context_engine::{ContextEngine, WeightedSignal, AprioriRule};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, debug};
use chrono::{DateTime, Utc};

/// 🎯 Trading decision with confidence and reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    pub action: DecisionAction,
    pub confidence: f64,
    pub risk_score: f64,
    pub reasoning: Vec<String>,
    pub applied_rules: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub stop_loss_triggers: Vec<String>,
}

/// 🚦 Possible trading actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionAction {
    Execute,
    Reject,
    Hold,
    ReducePosition,
    EmergencyExit,
}

/// 🛡️ Risk mitigation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRule {
    pub id: String,
    pub condition: String,
    pub action: DecisionAction,
    pub confidence_threshold: f64,
    pub priority: u8, // 1-10, 10 = highest priority
    pub created_at: DateTime<Utc>,
    pub success_rate: f64,
}

/// 🧠 Advanced Decision Engine
pub struct DecisionEngine {
    context_engine: Arc<ContextEngine>,
    risk_rules: Vec<RiskRule>,
    apriori_confidence_threshold: f64,
    max_risk_tolerance: f64,
}

impl DecisionEngine {
    /// 🚀 Initialize Decision Engine
    pub fn new(context_engine: Arc<ContextEngine>) -> Self {
        let mut engine = Self {
            context_engine,
            risk_rules: Vec::new(),
            apriori_confidence_threshold: 0.7,
            max_risk_tolerance: 0.15,
        };
        
        // Initialize default risk rules
        engine.initialize_default_rules();
        
        info!("🧠 Decision Engine initialized with {} risk rules", engine.risk_rules.len());
        engine
    }

    /// 🎯 Make trading decision based on signals and rules
    pub async fn make_decision(&self, signals: &[WeightedSignal]) -> Result<TradingDecision> {
        info!("🎯 Making trading decision with {} signals", signals.len());
        
        // 1. Filter semantic noise
        let filtered_signals = self.context_engine
            .filter_semantic_noise(signals, 0.3).await;
        
        // 2. Get Apriori rules
        let apriori_rules = self.get_applicable_apriori_rules(&filtered_signals).await?;
        
        // 3. Apply risk rules
        let risk_assessment = self.assess_risk(&filtered_signals, &apriori_rules).await?;
        
        // 4. Check for emergency conditions
        if let Some(emergency_decision) = self.check_emergency_conditions(&filtered_signals).await? {
            return Ok(emergency_decision);
        }
        
        // 5. Apply anti-pattern detection
        let anti_patterns = self.detect_anti_patterns(&filtered_signals).await?;
        
        // 6. Make final decision
        let decision = self.synthesize_decision(
            &filtered_signals,
            &apriori_rules,
            &risk_assessment,
            &anti_patterns
        ).await?;
        
        info!("🎯 Decision made: {:?} with confidence {:.2}", 
              decision.action, decision.confidence);
        
        Ok(decision)
    }

    /// 🔍 Get applicable Apriori rules for current signals
    async fn get_applicable_apriori_rules(&self, signals: &[WeightedSignal]) -> Result<Vec<AprioriRule>> {
        let signal_types: Vec<String> = signals.iter()
            .map(|s| s.signal_type.clone())
            .collect();
        
        // This would normally query the context engine for stored rules
        // For now, we'll create some example rules based on your specification
        let mut applicable_rules = Vec::new();
        
        // Rule 1: Rug pull detection
        if signal_types.iter().any(|s| s.contains("rug_pull")) {
            applicable_rules.push(AprioriRule {
                antecedent: vec!["rug_pull_detected".to_string()],
                consequent: "reject".to_string(),
                confidence: 0.94,
                support: 0.15,
                lift: 2.1,
                created_at: Utc::now(),
                last_validated: Utc::now(),
                success_count: 47,
                total_count: 50,
            });
        }
        
        // Rule 2: Anonymous team + high risk
        if signal_types.iter().any(|s| s.contains("team_anonymous")) &&
           signal_types.iter().any(|s| s.contains("high_risk")) {
            applicable_rules.push(AprioriRule {
                antecedent: vec!["team_anonymous".to_string(), "high_risk".to_string()],
                consequent: "reject".to_string(),
                confidence: 0.87,
                support: 0.08,
                lift: 1.8,
                created_at: Utc::now(),
                last_validated: Utc::now(),
                success_count: 23,
                total_count: 26,
            });
        }
        
        // Rule 3: High volume + good liquidity = execute
        if signal_types.iter().any(|s| s.contains("high_volume")) &&
           signal_types.iter().any(|s| s.contains("good_liquidity")) {
            applicable_rules.push(AprioriRule {
                antecedent: vec!["high_volume".to_string(), "good_liquidity".to_string()],
                consequent: "execute".to_string(),
                confidence: 0.73,
                support: 0.12,
                lift: 1.4,
                created_at: Utc::now(),
                last_validated: Utc::now(),
                success_count: 31,
                total_count: 42,
            });
        }
        
        debug!("🔍 Found {} applicable Apriori rules", applicable_rules.len());
        Ok(applicable_rules)
    }

    /// 🛡️ Assess overall risk based on signals and rules
    async fn assess_risk(&self, signals: &[WeightedSignal], rules: &[AprioriRule]) -> Result<f64> {
        let mut total_risk = 0.0;
        let mut risk_factors = 0;
        
        // Risk from signals
        for signal in signals {
            if signal.signal_type.contains("rug_pull") || 
               signal.signal_type.contains("risk") ||
               signal.signal_type.contains("suspicious") {
                total_risk += signal.importance_score * signal.tf_idf_weight;
                risk_factors += 1;
            }
        }
        
        // Risk from rules
        for rule in rules {
            if rule.consequent == "reject" && rule.confidence > self.apriori_confidence_threshold {
                total_risk += rule.confidence * 0.5; // Weight rule confidence
                risk_factors += 1;
            }
        }
        
        let normalized_risk = if risk_factors > 0 {
            (total_risk / risk_factors as f64).min(1.0)
        } else {
            0.0
        };
        
        debug!("🛡️ Risk assessment: {:.3} from {} factors", normalized_risk, risk_factors);
        Ok(normalized_risk)
    }

    /// 🚨 Check for emergency conditions requiring immediate action
    async fn check_emergency_conditions(&self, signals: &[WeightedSignal]) -> Result<Option<TradingDecision>> {
        for signal in signals {
            // Emergency: High confidence rug pull detection
            if signal.signal_type.contains("rug_pull") && 
               signal.importance_score > 0.9 &&
               signal.tf_idf_weight > 0.8 {
                return Ok(Some(TradingDecision {
                    action: DecisionAction::EmergencyExit,
                    confidence: signal.importance_score,
                    risk_score: 1.0,
                    reasoning: vec![
                        "🚨 EMERGENCY: High confidence rug pull detected".to_string(),
                        format!("Signal: {} (confidence: {:.2})", signal.signal_type, signal.importance_score)
                    ],
                    applied_rules: vec!["emergency_rug_pull_exit".to_string()],
                    timestamp: Utc::now(),
                    stop_loss_triggers: vec!["rug_pull_confirmed".to_string()],
                }));
            }
        }
        
        Ok(None)
    }

    /// 🔍 Detect anti-patterns that should block trading
    async fn detect_anti_patterns(&self, signals: &[WeightedSignal]) -> Result<Vec<String>> {
        let mut anti_patterns = Vec::new();
        
        // Anti-pattern 1: Anonymous team + new token + low liquidity
        let has_anonymous_team = signals.iter().any(|s| s.signal_type.contains("team_anonymous"));
        let has_new_token = signals.iter().any(|s| s.signal_type.contains("new_token"));
        let has_low_liquidity = signals.iter().any(|s| s.signal_type.contains("low_liquidity"));
        
        if has_anonymous_team && has_new_token && has_low_liquidity {
            anti_patterns.push("anonymous_new_low_liquidity".to_string());
        }
        
        // Anti-pattern 2: Rapid price movement without volume
        let has_rapid_price = signals.iter().any(|s| s.signal_type.contains("rapid_price"));
        let has_low_volume = signals.iter().any(|s| s.signal_type.contains("low_volume"));
        
        if has_rapid_price && has_low_volume {
            anti_patterns.push("price_pump_no_volume".to_string());
        }
        
        debug!("🔍 Detected {} anti-patterns", anti_patterns.len());
        Ok(anti_patterns)
    }

    /// 🎯 Synthesize final decision from all inputs
    async fn synthesize_decision(
        &self,
        signals: &[WeightedSignal],
        rules: &[AprioriRule],
        risk_score: &f64,
        anti_patterns: &[String]
    ) -> Result<TradingDecision> {
        let mut reasoning = Vec::new();
        let mut applied_rules = Vec::new();
        let mut confidence = 0.5; // Base confidence
        
        // If risk is too high, reject
        if *risk_score > self.max_risk_tolerance {
            return Ok(TradingDecision {
                action: DecisionAction::Reject,
                confidence: 1.0 - risk_score,
                risk_score: *risk_score,
                reasoning: vec![
                    format!("🛡️ Risk score {:.2} exceeds tolerance {:.2}", 
                           risk_score, self.max_risk_tolerance)
                ],
                applied_rules: vec!["max_risk_tolerance".to_string()],
                timestamp: Utc::now(),
                stop_loss_triggers: vec!["risk_threshold_exceeded".to_string()],
            });
        }
        
        // If anti-patterns detected, reject
        if !anti_patterns.is_empty() {
            return Ok(TradingDecision {
                action: DecisionAction::Reject,
                confidence: 0.9,
                risk_score: *risk_score,
                reasoning: vec![
                    format!("🔍 Anti-patterns detected: {}", anti_patterns.join(", "))
                ],
                applied_rules: anti_patterns.to_vec(),
                timestamp: Utc::now(),
                stop_loss_triggers: anti_patterns.to_vec(),
            });
        }
        
        // Apply Apriori rules
        for rule in rules {
            if rule.confidence > self.apriori_confidence_threshold {
                applied_rules.push(format!("apriori_{}", rule.consequent));
                reasoning.push(format!(
                    "📊 Rule: {} → {} (confidence: {:.2})",
                    rule.antecedent.join(" + "), rule.consequent, rule.confidence
                ));
                
                match rule.consequent.as_str() {
                    "execute" => {
                        confidence += rule.confidence * 0.3;
                        return Ok(TradingDecision {
                            action: DecisionAction::Execute,
                            confidence: confidence.min(1.0),
                            risk_score: *risk_score,
                            reasoning,
                            applied_rules,
                            timestamp: Utc::now(),
                            stop_loss_triggers: vec!["risk_increase".to_string()],
                        });
                    },
                    "reject" => {
                        return Ok(TradingDecision {
                            action: DecisionAction::Reject,
                            confidence: rule.confidence,
                            risk_score: *risk_score,
                            reasoning,
                            applied_rules,
                            timestamp: Utc::now(),
                            stop_loss_triggers: vec![],
                        });
                    },
                    _ => {}
                }
            }
        }
        
        // Default: Hold if uncertain
        Ok(TradingDecision {
            action: DecisionAction::Hold,
            confidence: 0.5,
            risk_score: *risk_score,
            reasoning: vec!["🤔 Insufficient confidence for execute/reject decision".to_string()],
            applied_rules: vec!["default_hold".to_string()],
            timestamp: Utc::now(),
            stop_loss_triggers: vec!["risk_increase".to_string()],
        })
    }

    /// 🛡️ Initialize default risk rules
    fn initialize_default_rules(&mut self) {
        self.risk_rules = vec![
            RiskRule {
                id: "rug_pull_high_confidence".to_string(),
                condition: "rug_pull_potential > 0.8".to_string(),
                action: DecisionAction::Reject,
                confidence_threshold: 0.9,
                priority: 10,
                created_at: Utc::now(),
                success_rate: 0.94,
            },
            RiskRule {
                id: "anonymous_team_new_token".to_string(),
                condition: "team_anonymous = true AND token_age < 24h".to_string(),
                action: DecisionAction::Reject,
                confidence_threshold: 0.7,
                priority: 8,
                created_at: Utc::now(),
                success_rate: 0.87,
            },
            RiskRule {
                id: "low_liquidity_high_risk".to_string(),
                condition: "liquidity < 10000 AND risk_score > 0.6".to_string(),
                action: DecisionAction::ReducePosition,
                confidence_threshold: 0.6,
                priority: 6,
                created_at: Utc::now(),
                success_rate: 0.73,
            },
        ];
    }
}
