//! 🐝 SwarmCoordinator - Centralny orkiestrator Hive Mind
//! 
//! Główny komponent zarządzający całą architekturą Swarmagentic

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, debug, warn, error, instrument};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    agent_registry::{AgentRegistry, AgentInfo, AgentStatus},
    task_delegation::{TaskDelegator, Task, TaskStatus, TaskResult},
    communication::{CommunicationHub, AgentMessage},
    metrics::{SwarmMetrics, PerformanceMetrics},
    agent_types::{AgentType, SpecializedAgent},
    memory_store::MemoryStore,
    feedback_loop::FeedbackLoop,
};

/// 🎯 Stan całego systemu Swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwarmState {
    /// System inicjalizuje się
    Initializing,
    /// System jest aktywny i gotowy
    Active,
    /// System jest w trybie degradacji (część agentów niedostępna)
    Degraded,
    /// System jest w trybie maintenance
    Maintenance,
    /// System jest wyłączony
    Shutdown,
}

/// ❌ Błędy SwarmCoordinator
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("Agent registry error: {0}")]
    AgentRegistry(String),
    
    #[error("Task delegation error: {0}")]
    TaskDelegation(String),
    
    #[error("Communication error: {0}")]
    Communication(String),
    
    #[error("Memory store error: {0}")]
    MemoryStore(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Performance threshold exceeded: {0}")]
    PerformanceThreshold(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(Uuid),
    
    #[error("Task timeout: {0}")]
    TaskTimeout(Uuid),
}

/// 🐝 Główny SwarmCoordinator
pub struct SwarmCoordinator {
    /// Konfiguracja systemu
    config: Arc<Config>,
    
    /// Rejestr wszystkich agentów
    agent_registry: Arc<RwLock<AgentRegistry>>,
    
    /// Delegator zadań
    task_delegator: Arc<RwLock<TaskDelegator>>,
    
    /// Hub komunikacyjny
    communication_hub: Arc<CommunicationHub>,
    
    /// Magazyn pamięci
    memory_store: Arc<MemoryStore>,
    
    /// Pętla uczenia się
    feedback_loop: Arc<FeedbackLoop>,
    
    /// Metryki wydajności
    metrics: Arc<RwLock<SwarmMetrics>>,
    
    /// Aktualny stan systemu
    state: Arc<RwLock<SwarmState>>,
    
    /// Aktywne zadania
    active_tasks: Arc<DashMap<Uuid, Task>>,
    
    /// Kanał do zatrzymania systemu
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SwarmCoordinator {
    /// Tworzy nowy SwarmCoordinator
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self, CoordinatorError> {
        info!("🐝 Initializing SwarmCoordinator...");
        
        // Inicjalizacja komponentów
        let agent_registry = Arc::new(RwLock::new(
            AgentRegistry::new(config.clone()).await
                .map_err(|e| CoordinatorError::AgentRegistry(e.to_string()))?
        ));
        
        let task_delegator = Arc::new(RwLock::new(
            TaskDelegator::new(config.clone()).await
                .map_err(|e| CoordinatorError::TaskDelegation(e.to_string()))?
        ));
        
        let communication_hub = Arc::new(
            CommunicationHub::new(config.clone()).await
                .map_err(|e| CoordinatorError::Communication(e.to_string()))?
        );
        
        let memory_store = Arc::new(
            MemoryStore::new(config.clone()).await
                .map_err(|e| CoordinatorError::MemoryStore(e.to_string()))?
        );
        
        let feedback_loop = Arc::new(
            FeedbackLoop::new(config.clone(), memory_store.clone()).await
                .map_err(|e| CoordinatorError::MemoryStore(e.to_string()))?
        );
        
        let metrics = Arc::new(RwLock::new(SwarmMetrics::new()));
        let state = Arc::new(RwLock::new(SwarmState::Initializing));
        let active_tasks = Arc::new(DashMap::new());
        
        info!("✅ SwarmCoordinator initialized successfully");
        
        Ok(SwarmCoordinator {
            config,
            agent_registry,
            task_delegator,
            communication_hub,
            memory_store,
            feedback_loop,
            metrics,
            state,
            active_tasks,
            shutdown_tx: None,
        })
    }
    
    /// Uruchamia SwarmCoordinator
    #[instrument(skip(self))]
    pub async fn start(&mut self) -> Result<(), CoordinatorError> {
        info!("🚀 Starting SwarmCoordinator...");
        
        // Ustawienie stanu na aktywny
        {
            let mut state = self.state.write().await;
            *state = SwarmState::Active;
        }
        
        // Uruchomienie komponentów
        self.communication_hub.start().await
            .map_err(|e| CoordinatorError::Communication(e.to_string()))?;
        
        // Uruchomienie pętli heartbeat
        self.start_heartbeat_loop().await?;
        
        // Uruchomienie pętli przetwarzania zadań
        self.start_task_processing_loop().await?;
        
        // Uruchomienie pętli monitorowania wydajności
        self.start_performance_monitoring_loop().await?;
        
        info!("✅ SwarmCoordinator started successfully");
        Ok(())
    }
    
    /// Zatrzymuje SwarmCoordinator
    #[instrument(skip(self))]
    pub async fn shutdown(&mut self) -> Result<(), CoordinatorError> {
        info!("🛑 Shutting down SwarmCoordinator...");
        
        // Ustawienie stanu na shutdown
        {
            let mut state = self.state.write().await;
            *state = SwarmState::Shutdown;
        }
        
        // Wysłanie sygnału shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        // Zatrzymanie wszystkich aktywnych zadań
        self.cancel_all_tasks().await?;
        
        // Zatrzymanie komunikacji
        self.communication_hub.shutdown().await
            .map_err(|e| CoordinatorError::Communication(e.to_string()))?;
        
        info!("✅ SwarmCoordinator shutdown completed");
        Ok(())
    }
    
    /// Rejestruje nowego agenta w systemie
    #[instrument(skip(self))]
    pub async fn register_agent(&self, agent_info: AgentInfo) -> Result<(), CoordinatorError> {
        debug!("📝 Registering agent: {:?}", agent_info.agent_type);
        
        let mut registry = self.agent_registry.write().await;
        registry.register_agent(agent_info).await
            .map_err(|e| CoordinatorError::AgentRegistry(e.to_string()))?;
        
        // Aktualizacja metryk
        {
            let mut metrics = self.metrics.write().await;
            metrics.increment_agent_count(1);
        }
        
        info!("✅ Agent registered successfully");
        Ok(())
    }
    
    /// Deleguje zadanie do odpowiedniego agenta
    #[instrument(skip(self, task))]
    pub async fn delegate_task(&self, mut task: Task) -> Result<Uuid, CoordinatorError> {
        debug!("📋 Delegating task: {}", task.task_type);
        
        // Znajdź najlepszego agenta dla zadania
        let agent_id = {
            let registry = self.agent_registry.read().await;
            let delegator = self.task_delegator.read().await;
            
            delegator.find_best_agent(&task, &registry).await
                .map_err(|e| CoordinatorError::TaskDelegation(e.to_string()))?
        };
        
        // Przypisz agenta do zadania
        task.assigned_agent = Some(agent_id);
        task.status = TaskStatus::Assigned;
        
        // Dodaj do aktywnych zadań
        self.active_tasks.insert(task.id, task.clone());
        
        // Wyślij zadanie do agenta
        let message = AgentMessage::new_task_assignment(agent_id, task.clone());
        self.communication_hub.send_message(message).await
            .map_err(|e| CoordinatorError::Communication(e.to_string()))?;
        
        info!("✅ Task delegated to agent: {}", agent_id);
        Ok(task.id)
    }
    
    /// Otrzymuje wynik zadania od agenta
    #[instrument(skip(self, result))]
    pub async fn receive_task_result(&self, task_id: Uuid, result: TaskResult) -> Result<(), CoordinatorError> {
        debug!("📥 Receiving task result for: {}", task_id);
        
        // Usuń z aktywnych zadań
        if let Some((_, mut task)) = self.active_tasks.remove(&task_id) {
            task.status = TaskStatus::Completed;
            
            // Zapisz wynik w pamięci
            self.memory_store.store_task_result(task_id, &result).await
                .map_err(|e| CoordinatorError::MemoryStore(e.to_string()))?;
            
            // Przekaż do pętli uczenia się
            self.feedback_loop.process_task_result(task, result).await
                .map_err(|e| CoordinatorError::MemoryStore(e.to_string()))?;
            
            info!("✅ Task result processed: {}", task_id);
        } else {
            warn!("⚠️ Task not found in active tasks: {}", task_id);
        }
        
        Ok(())
    }
    
    /// Pobiera aktualny stan systemu
    pub async fn get_state(&self) -> SwarmState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Pobiera metryki wydajności
    pub async fn get_metrics(&self) -> SwarmMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Uruchamia pętlę heartbeat dla monitorowania agentów
    async fn start_heartbeat_loop(&self) -> Result<(), CoordinatorError> {
        let registry = self.agent_registry.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(config.swarm.heartbeat_interval_ms)
            );
            
            loop {
                interval.tick().await;
                
                let mut registry = registry.write().await;
                if let Err(e) = registry.check_agent_heartbeats().await {
                    error!("❌ Heartbeat check failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Uruchamia pętlę przetwarzania zadań
    async fn start_task_processing_loop(&self) -> Result<(), CoordinatorError> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
    
    /// Uruchamia pętlę monitorowania wydajności
    async fn start_performance_monitoring_loop(&self) -> Result<(), CoordinatorError> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
    
    /// Anuluje wszystkie aktywne zadania
    async fn cancel_all_tasks(&self) -> Result<(), CoordinatorError> {
        for entry in self.active_tasks.iter() {
            let task_id = *entry.key();
            debug!("🚫 Cancelling task: {}", task_id);
        }
        
        self.active_tasks.clear();
        Ok(())
    }
}
