//! 📊 Metrics - Metryki Agent-Strateg
//! 
//! System zbierania i analizy metryk wydajności

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::decision_synthesis::Decision;

/// 📊 Główne metryki Agent-Strateg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategMetrics {
    pub decision_metrics: DecisionMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub goal_metrics: GoalMetrics,
    pub delegation_metrics: DelegationMetrics,
    pub last_updated: DateTime<Utc>,
}

impl StrategMetrics {
    pub fn new() -> Self {
        Self {
            decision_metrics: DecisionMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            goal_metrics: GoalMetrics::default(),
            delegation_metrics: DelegationMetrics::default(),
            last_updated: Utc::now(),
        }
    }
    
    pub fn record_decision(&mut self, decision: &Decision) {
        self.decision_metrics.total_decisions += 1;
        
        if decision.confidence.value() >= 0.8 {
            self.decision_metrics.high_confidence_decisions += 1;
        }
        
        let decision_type = format!("{:?}", decision.decision_type);
        *self.decision_metrics.decisions_by_type
            .entry(decision_type)
            .or_insert(0) += 1;
        
        self.last_updated = Utc::now();
    }
}

/// 🎯 Metryki decyzji
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionMetrics {
    pub total_decisions: u64,
    pub high_confidence_decisions: u64,
    pub successful_decisions: u64,
    pub failed_decisions: u64,
    pub average_confidence: f64,
    pub average_decision_time_ms: f64,
    pub decisions_by_type: HashMap<String, u64>,
}

/// ⚡ Metryki wydajności
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_response_time_ms: f64,
    pub goal_completion_rate: f64,
    pub delegation_success_rate: f64,
    pub ai_model_accuracy: f64,
    pub resource_utilization: f64,
}

/// 🎯 Metryki celów
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoalMetrics {
    pub total_goals: u64,
    pub completed_goals: u64,
    pub failed_goals: u64,
    pub average_goal_duration_minutes: f64,
    pub average_sub_goals_per_goal: f64,
}

/// 📋 Metryki delegacji
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DelegationMetrics {
    pub total_delegations: u64,
    pub successful_delegations: u64,
    pub failed_delegations: u64,
    pub average_delegation_time_ms: f64,
    pub delegations_by_agent_type: HashMap<String, u64>,
}
