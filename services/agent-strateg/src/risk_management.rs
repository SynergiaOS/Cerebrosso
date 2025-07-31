//! 🛡️ Risk Management - Zarządzanie ryzykiem
//! 
//! System oceny i zarządzania ryzykiem dla Agent-Strateg

use anyhow::Result;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::{config::Config, ai_models::AIResponse};

/// ⚠️ Poziom ryzyka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Bardzo niskie ryzyko
    VeryLow,
    /// Niskie ryzyko
    Low,
    /// Średnie ryzyko
    Medium,
    /// Wysokie ryzyko
    High,
    /// Bardzo wysokie ryzyko
    VeryHigh,
}

impl RiskLevel {
    pub fn is_high(&self) -> bool {
        matches!(self, RiskLevel::High | RiskLevel::VeryHigh)
    }
    
    pub fn to_numeric(&self) -> f64 {
        match self {
            RiskLevel::VeryLow => 0.1,
            RiskLevel::Low => 0.3,
            RiskLevel::Medium => 0.5,
            RiskLevel::High => 0.7,
            RiskLevel::VeryHigh => 0.9,
        }
    }
}

/// 📊 Ocena ryzyka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
    pub recommended_position_size: f64,
    pub stop_loss_price: f64,
    pub take_profit_price: f64,
    pub max_loss_amount: f64,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

/// 🛡️ Zarządzanie ryzykiem
pub struct RiskManager {
    config: Arc<Config>,
    risk_limits: RiskLimits,
}

/// 📋 Limity ryzyka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub max_concurrent_positions: usize,
    pub min_confidence_threshold: f64,
    pub stop_loss_percentage: f64,
    pub take_profit_percentage: f64,
}

impl RiskManager {
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🛡️ Initializing RiskManager...");
        
        let risk_limits = RiskLimits {
            max_position_size: config.risk.max_position_size,
            max_daily_loss: config.risk.max_daily_loss,
            max_concurrent_positions: config.risk.max_concurrent_positions,
            min_confidence_threshold: config.risk.min_confidence_for_trade,
            stop_loss_percentage: config.risk.stop_loss_percentage,
            take_profit_percentage: config.risk.take_profit_percentage,
        };
        
        Ok(Self {
            config,
            risk_limits,
        })
    }
    
    pub async fn assess_responses(&self, responses: &[AIResponse]) -> Result<RiskAssessment> {
        let mut risk_score = 0.0;
        let mut risk_factors = Vec::new();
        
        // Analiza odpowiedzi agentów pod kątem ryzyka
        for response in responses {
            if response.confidence < 0.7 {
                risk_score += 0.2;
                risk_factors.push("Low agent confidence".to_string());
            }
            
            if response.content.contains("volatile") || response.content.contains("risky") {
                risk_score += 0.3;
                risk_factors.push("High volatility detected".to_string());
            }
        }
        
        // Określ poziom ryzyka
        let risk_level = match risk_score {
            s if s >= 0.8 => RiskLevel::VeryHigh,
            s if s >= 0.6 => RiskLevel::High,
            s if s >= 0.4 => RiskLevel::Medium,
            s if s >= 0.2 => RiskLevel::Low,
            _ => RiskLevel::VeryLow,
        };
        
        // Oblicz rekomendowany rozmiar pozycji
        let position_multiplier = match risk_level {
            RiskLevel::VeryLow => 1.0,
            RiskLevel::Low => 0.8,
            RiskLevel::Medium => 0.6,
            RiskLevel::High => 0.4,
            RiskLevel::VeryHigh => 0.2,
        };
        
        let recommended_position_size = self.risk_limits.max_position_size * position_multiplier;
        
        Ok(RiskAssessment {
            risk_level,
            risk_score,
            risk_factors,
            mitigation_strategies: vec![
                "Use stop-loss orders".to_string(),
                "Limit position size".to_string(),
                "Monitor market conditions".to_string(),
            ],
            recommended_position_size,
            stop_loss_price: 0.95, // 5% stop loss
            take_profit_price: 1.15, // 15% take profit
            max_loss_amount: recommended_position_size * self.risk_limits.stop_loss_percentage,
            confidence: 0.8,
            timestamp: Utc::now(),
        })
    }
    
    pub fn get_risk_limits(&self) -> &RiskLimits {
        &self.risk_limits
    }
}
