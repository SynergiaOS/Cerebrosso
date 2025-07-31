//! 👑 Agent-Strateg - CEO agenta w architekturze Hive Mind
//! 
//! Główny decydent i koordynator strategiczny odpowiedzialny za:
//! - Goal decomposition (dekompozycja celów)
//! - Task delegation (delegacja zadań)
//! - Decision synthesis (synteza decyzji)
//! - Strategy planning (planowanie strategiczne)

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tracing::{info, debug, warn, error, instrument};

use crate::{
    config::Config,
    goal_decomposition::{GoalDecomposer, Goal, GoalStatus},
    task_delegation::{TaskDelegator, TaskAssignment},
    decision_synthesis::{DecisionSynthesizer, Decision, DecisionConfidence},
    strategy_planning::{StrategyPlanner, TradingStrategy},
    swarm_communication::{SwarmClient, SwarmMessage, MessageType},
    ai_models::{AIModelManager, AIResponse},
    metrics::{StrategMetrics, DecisionMetrics},
    risk_management::{RiskManager, RiskAssessment},
};

/// 🎯 Stan Agent-Strateg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategState {
    /// Agent inicjalizuje się
    Initializing,
    /// Agent jest aktywny i gotowy
    Active,
    /// Agent analizuje sytuację
    Analyzing,
    /// Agent planuje strategię
    Planning,
    /// Agent deleguje zadania
    Delegating,
    /// Agent syntetyzuje decyzje
    Synthesizing,
    /// Agent jest w trybie maintenance
    Maintenance,
    /// Agent jest wyłączony
    Shutdown,
}

/// ❌ Błędy Agent-Strateg
#[derive(Debug, thiserror::Error)]
pub enum StrategError {
    #[error("Goal decomposition error: {0}")]
    GoalDecomposition(String),
    
    #[error("Task delegation error: {0}")]
    TaskDelegation(String),
    
    #[error("Decision synthesis error: {0}")]
    DecisionSynthesis(String),
    
    #[error("Strategy planning error: {0}")]
    StrategyPlanning(String),
    
    #[error("Communication error: {0}")]
    Communication(String),
    
    #[error("AI model error: {0}")]
    AIModel(String),
    
    #[error("Risk management error: {0}")]
    RiskManagement(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Goal not found: {0}")]
    GoalNotFound(Uuid),
    
    #[error("Insufficient confidence: {0}")]
    InsufficientConfidence(f64),
}

/// 👑 Główny Agent-Strateg
pub struct AgentStrateg {
    /// Konfiguracja agenta
    config: Arc<Config>,
    
    /// Dekompozycja celów
    goal_decomposer: Arc<RwLock<GoalDecomposer>>,
    
    /// Delegacja zadań
    task_delegator: Arc<RwLock<TaskDelegator>>,
    
    /// Synteza decyzji
    decision_synthesizer: Arc<RwLock<DecisionSynthesizer>>,
    
    /// Planowanie strategiczne
    strategy_planner: Arc<RwLock<StrategyPlanner>>,
    
    /// Komunikacja ze Swarm
    swarm_client: Arc<SwarmClient>,
    
    /// Zarządzanie modelami AI
    ai_manager: Arc<AIModelManager>,
    
    /// Zarządzanie ryzykiem
    risk_manager: Arc<RiskManager>,
    
    /// Metryki agenta
    metrics: Arc<RwLock<StrategMetrics>>,
    
    /// Aktualny stan agenta
    state: Arc<RwLock<StrategState>>,
    
    /// Aktywne cele
    active_goals: Arc<RwLock<HashMap<Uuid, Goal>>>,
    
    /// Historia decyzji
    decision_history: Arc<RwLock<Vec<Decision>>>,
    
    /// Kanał do zatrzymania agenta
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl AgentStrateg {
    /// Tworzy nowego Agent-Strateg
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self, StrategError> {
        info!("👑 Initializing Agent-Strateg (CEO)...");
        
        // Walidacja konfiguracji
        config.validate()
            .map_err(|e| StrategError::Configuration(e.to_string()))?;
        
        // Inicjalizacja komponentów
        let goal_decomposer = Arc::new(RwLock::new(
            GoalDecomposer::new(config.clone()).await
                .map_err(|e| StrategError::GoalDecomposition(e.to_string()))?
        ));
        
        let task_delegator = Arc::new(RwLock::new(
            TaskDelegator::new(config.clone()).await
                .map_err(|e| StrategError::TaskDelegation(e.to_string()))?
        ));
        
        let decision_synthesizer = Arc::new(RwLock::new(
            DecisionSynthesizer::new(config.clone()).await
                .map_err(|e| StrategError::DecisionSynthesis(e.to_string()))?
        ));
        
        let strategy_planner = Arc::new(RwLock::new(
            StrategyPlanner::new(config.clone()).await
                .map_err(|e| StrategError::StrategyPlanning(e.to_string()))?
        ));
        
        let swarm_client = Arc::new(
            SwarmClient::new(config.clone()).await
                .map_err(|e| StrategError::Communication(e.to_string()))?
        );
        
        let ai_manager = Arc::new(
            AIModelManager::new(config.clone()).await
                .map_err(|e| StrategError::AIModel(e.to_string()))?
        );
        
        let risk_manager = Arc::new(
            RiskManager::new(config.clone()).await
                .map_err(|e| StrategError::RiskManagement(e.to_string()))?
        );
        
        let metrics = Arc::new(RwLock::new(StrategMetrics::new()));
        let state = Arc::new(RwLock::new(StrategState::Initializing));
        let active_goals = Arc::new(RwLock::new(HashMap::new()));
        let decision_history = Arc::new(RwLock::new(Vec::new()));
        
        info!("✅ Agent-Strateg initialized successfully");
        
        Ok(AgentStrateg {
            config,
            goal_decomposer,
            task_delegator,
            decision_synthesizer,
            strategy_planner,
            swarm_client,
            ai_manager,
            risk_manager,
            metrics,
            state,
            active_goals,
            decision_history,
            shutdown_tx: None,
        })
    }
    
    /// Uruchamia Agent-Strateg
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<(), StrategError> {
        info!("🚀 Starting Agent-Strateg...");
        
        // Ustawienie stanu na aktywny
        {
            let mut state = self.state.write().await;
            *state = StrategState::Active;
        }
        
        // Rejestracja w SwarmCoordinator
        self.register_with_swarm().await?;
        
        // Uruchomienie pętli heartbeat
        self.start_heartbeat_loop().await?;
        
        // Uruchomienie pętli przetwarzania wiadomości
        self.start_message_processing_loop().await?;
        
        // Uruchomienie pętli strategicznego planowania
        self.start_strategic_planning_loop().await?;
        
        info!("✅ Agent-Strateg started successfully");
        Ok(())
    }
    
    /// Zatrzymuje Agent-Strateg
    #[instrument(skip(self))]
    pub async fn shutdown(&mut self) -> Result<(), StrategError> {
        info!("🛑 Shutting down Agent-Strateg...");
        
        // Ustawienie stanu na shutdown
        {
            let mut state = self.state.write().await;
            *state = StrategState::Shutdown;
        }
        
        // Wysłanie sygnału shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        // Zakończenie wszystkich aktywnych celów
        self.complete_all_goals().await?;
        
        // Wyrejestrowanie ze Swarm
        self.unregister_from_swarm().await?;
        
        info!("✅ Agent-Strateg shutdown completed");
        Ok(())
    }
    
    /// Dekomponuje cel na pod-cele i zadania
    #[instrument(skip(self, goal))]
    pub async fn decompose_goal(&self, goal: Goal) -> Result<Vec<Goal>, StrategError> {
        debug!("🎯 Decomposing goal: {}", goal.title);
        
        // Zmień stan na analizowanie
        {
            let mut state = self.state.write().await;
            *state = StrategState::Analyzing;
        }
        
        // Analiza celu przez AI
        let ai_analysis = self.ai_manager
            .analyze_goal(&goal).await
            .map_err(|e| StrategError::AIModel(e.to_string()))?;
        
        // Dekompozycja celu
        let sub_goals = {
            let mut decomposer = self.goal_decomposer.write().await;
            decomposer.decompose_goal(goal.clone(), ai_analysis).await
                .map_err(|e| StrategError::GoalDecomposition(e.to_string()))?
        };
        
        // Dodaj do aktywnych celów
        {
            let mut active_goals = self.active_goals.write().await;
            active_goals.insert(goal.id, goal);
            for sub_goal in &sub_goals {
                active_goals.insert(sub_goal.id, sub_goal.clone());
            }
        }
        
        // Zmień stan z powrotem na aktywny
        {
            let mut state = self.state.write().await;
            *state = StrategState::Active;
        }
        
        info!("✅ Goal decomposed into {} sub-goals", sub_goals.len());
        Ok(sub_goals)
    }
    
    /// Deleguje zadania do innych agentów
    #[instrument(skip(self, goal))]
    pub async fn delegate_tasks(&self, goal: &Goal) -> Result<Vec<TaskAssignment>, StrategError> {
        debug!("📋 Delegating tasks for goal: {}", goal.title);
        
        // Zmień stan na delegowanie
        {
            let mut state = self.state.write().await;
            *state = StrategState::Delegating;
        }
        
        // Planowanie delegacji przez AI
        let delegation_plan = self.ai_manager
            .plan_task_delegation(goal).await
            .map_err(|e| StrategError::AIModel(e.to_string()))?;
        
        // Delegacja zadań
        let assignments = {
            let mut delegator = self.task_delegator.write().await;
            delegator.delegate_tasks(goal, delegation_plan).await
                .map_err(|e| StrategError::TaskDelegation(e.to_string()))?
        };
        
        // Wysłanie zadań do SwarmCoordinator
        for assignment in &assignments {
            let message = SwarmMessage::new_task_assignment(assignment.clone());
            self.swarm_client.send_message(message).await
                .map_err(|e| StrategError::Communication(e.to_string()))?;
        }
        
        // Zmień stan z powrotem na aktywny
        {
            let mut state = self.state.write().await;
            *state = StrategState::Active;
        }
        
        info!("✅ Delegated {} tasks", assignments.len());
        Ok(assignments)
    }
    
    /// Syntetyzuje decyzje z wyników innych agentów
    #[instrument(skip(self, agent_responses))]
    pub async fn synthesize_reports(&self, agent_responses: Vec<AIResponse>) -> Result<Decision, StrategError> {
        debug!("🔬 Synthesizing {} agent responses", agent_responses.len());
        
        // Zmień stan na syntezowanie
        {
            let mut state = self.state.write().await;
            *state = StrategState::Synthesizing;
        }
        
        // Ocena ryzyka
        let risk_assessment = self.risk_manager
            .assess_responses(&agent_responses).await
            .map_err(|e| StrategError::RiskManagement(e.to_string()))?;
        
        // Synteza decyzji
        let decision = {
            let mut synthesizer = self.decision_synthesizer.write().await;
            synthesizer.synthesize_decision(agent_responses, risk_assessment).await
                .map_err(|e| StrategError::DecisionSynthesis(e.to_string()))?
        };
        
        // Sprawdź próg pewności
        if decision.confidence.value() < self.config.ai.decision_threshold {
            return Err(StrategError::InsufficientConfidence(decision.confidence.value()));
        }
        
        // Zapisz w historii
        {
            let mut history = self.decision_history.write().await;
            history.push(decision.clone());
            
            // Ogranicz rozmiar historii
            if history.len() > 1000 {
                history.remove(0);
            }
        }
        
        // Aktualizuj metryki
        {
            let mut metrics = self.metrics.write().await;
            metrics.record_decision(&decision);
        }
        
        // Zmień stan z powrotem na aktywny
        {
            let mut state = self.state.write().await;
            *state = StrategState::Active;
        }
        
        info!("✅ Decision synthesized with confidence: {:.2}", decision.confidence.value());
        Ok(decision)
    }
    
    /// Pobiera aktualny stan agenta
    pub async fn get_state(&self) -> StrategState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Pobiera metryki agenta
    pub async fn get_metrics(&self) -> StrategMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Pobiera aktywne cele
    pub async fn get_active_goals(&self) -> Vec<Goal> {
        let goals = self.active_goals.read().await;
        goals.values().cloned().collect()
    }
    
    /// Rejestruje agenta w SwarmCoordinator
    async fn register_with_swarm(&self) -> Result<(), StrategError> {
        let registration_message = SwarmMessage::new_agent_registration(
            self.config.swarm.agent_id.clone(),
            "Strateg".to_string(),
            vec!["DecisionMaking".to_string(), "StrategyPlanning".to_string()],
        );
        
        self.swarm_client.send_message(registration_message).await
            .map_err(|e| StrategError::Communication(e.to_string()))?;
        
        info!("✅ Registered with SwarmCoordinator");
        Ok(())
    }
    
    /// Wyrejestrowuje agenta ze SwarmCoordinator
    async fn unregister_from_swarm(&self) -> Result<(), StrategError> {
        let unregistration_message = SwarmMessage::new_agent_unregistration(
            self.config.swarm.agent_id.clone(),
        );
        
        self.swarm_client.send_message(unregistration_message).await
            .map_err(|e| StrategError::Communication(e.to_string()))?;
        
        info!("✅ Unregistered from SwarmCoordinator");
        Ok(())
    }
    
    /// Uruchamia pętlę heartbeat
    async fn start_heartbeat_loop(&self) -> Result<(), StrategError> {
        let swarm_client = self.swarm_client.clone();
        let agent_id = self.config.swarm.agent_id.clone();
        let interval_ms = self.config.swarm.heartbeat_interval_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(interval_ms)
            );
            
            loop {
                interval.tick().await;
                
                let heartbeat = SwarmMessage::new_heartbeat(agent_id.clone());
                if let Err(e) = swarm_client.send_message(heartbeat).await {
                    error!("❌ Failed to send heartbeat: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Uruchamia pętlę przetwarzania wiadomości
    async fn start_message_processing_loop(&self) -> Result<(), StrategError> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
    
    /// Uruchamia pętlę strategicznego planowania
    async fn start_strategic_planning_loop(&self) -> Result<(), StrategError> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
    
    /// Kończy wszystkie aktywne cele
    async fn complete_all_goals(&self) -> Result<(), StrategError> {
        let mut goals = self.active_goals.write().await;
        for (goal_id, mut goal) in goals.drain() {
            goal.status = GoalStatus::Cancelled;
            debug!("🚫 Cancelled goal: {}", goal_id);
        }
        Ok(())
    }
}
