//! 📋 Task Delegation - Delegacja zadań do innych agentów
//! 
//! System inteligentnej delegacji zadań na podstawie możliwości agentów

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, debug, instrument};

use crate::{
    config::Config,
    goal_decomposition::Goal,
    ai_models::AIResponse,
};

/// 🎯 Strategia delegacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationStrategy {
    /// Deleguj do najlepszego dostępnego agenta
    BestAvailable,
    /// Deleguj równomiernie między agentami
    LoadBalanced,
    /// Deleguj na podstawie specjalizacji
    SpecializationBased,
    /// Deleguj na podstawie historycznej wydajności
    PerformanceBased,
}

/// 📋 Przydzielenie zadania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// ID przydzielenia
    pub id: Uuid,
    /// ID celu/zadania
    pub goal_id: Uuid,
    /// Typ agenta docelowego
    pub target_agent_type: String,
    /// Wymagane możliwości
    pub required_capabilities: Vec<String>,
    /// Priorytet zadania
    pub priority: u8,
    /// Dane zadania
    pub task_data: Value,
    /// Deadline wykonania
    pub deadline: DateTime<Utc>,
    /// Czas utworzenia
    pub created_at: DateTime<Utc>,
    /// Strategia delegacji
    pub strategy: DelegationStrategy,
    /// Metadane
    pub metadata: HashMap<String, String>,
}

/// 📊 Plan delegacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationPlan {
    /// Lista przydzieleń zadań
    pub assignments: Vec<TaskAssignment>,
    /// Szacowany czas wykonania
    pub estimated_duration_minutes: u32,
    /// Poziom pewności planu
    pub confidence: f64,
    /// Uzasadnienie planu
    pub rationale: String,
}

/// 📋 Delegator zadań
pub struct TaskDelegator {
    /// Konfiguracja
    config: Arc<Config>,
    /// Statystyki delegacji
    stats: DelegationStats,
}

/// 📊 Statystyki delegacji
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DelegationStats {
    pub total_delegations: u64,
    pub successful_delegations: u64,
    pub failed_delegations: u64,
    pub average_delegation_time_ms: f64,
    pub delegation_by_agent_type: HashMap<String, u64>,
}

impl TaskDelegator {
    /// Tworzy nowy delegator zadań
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("📋 Initializing TaskDelegator...");
        
        Ok(Self {
            config,
            stats: DelegationStats::default(),
        })
    }
    
    /// Deleguje zadania na podstawie celu i planu AI
    #[instrument(skip(self, goal, delegation_plan))]
    pub async fn delegate_tasks(
        &mut self,
        goal: &Goal,
        delegation_plan: DelegationPlan,
    ) -> Result<Vec<TaskAssignment>> {
        debug!("📋 Delegating tasks for goal: {}", goal.title);
        
        let start_time = std::time::Instant::now();
        let mut assignments = Vec::new();
        
        for assignment_template in delegation_plan.assignments {
            let assignment = TaskAssignment {
                id: Uuid::new_v4(),
                goal_id: goal.id,
                target_agent_type: assignment_template.target_agent_type.clone(),
                required_capabilities: assignment_template.required_capabilities,
                priority: assignment_template.priority,
                task_data: assignment_template.task_data,
                deadline: assignment_template.deadline,
                created_at: Utc::now(),
                strategy: assignment_template.strategy,
                metadata: assignment_template.metadata,
            };
            
            assignments.push(assignment);
            
            // Aktualizuj statystyki
            *self.stats.delegation_by_agent_type
                .entry(assignment_template.target_agent_type)
                .or_insert(0) += 1;
        }
        
        // Aktualizuj ogólne statystyki
        let duration = start_time.elapsed();
        self.update_stats(true, duration.as_millis() as f64);
        
        info!("✅ Delegated {} tasks", assignments.len());
        Ok(assignments)
    }
    
    /// Aktualizuje statystyki delegacji
    fn update_stats(&mut self, success: bool, duration_ms: f64) {
        self.stats.total_delegations += 1;
        
        if success {
            self.stats.successful_delegations += 1;
        } else {
            self.stats.failed_delegations += 1;
        }
        
        // Aktualizuj średni czas delegacji
        let total = self.stats.total_delegations as f64;
        self.stats.average_delegation_time_ms = 
            (self.stats.average_delegation_time_ms * (total - 1.0) + duration_ms) / total;
    }
    
    /// Pobiera statystyki delegacji
    pub fn get_stats(&self) -> &DelegationStats {
        &self.stats
    }
}
