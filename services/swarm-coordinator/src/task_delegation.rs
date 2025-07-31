//! 📋 Task Delegation - Zarządzanie zadaniami w systemie
//! 
//! System delegacji zadań do agentów w architekturze Hive Mind

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, debug, warn, error, instrument};

use crate::{
    config::Config,
    agent_registry::{AgentRegistry, AgentInfo},
    agent_types::{AgentType, AgentCapability},
};

/// 🎯 Priorytet zadania
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// Krytyczne - natychmiastowe wykonanie
    Critical = 4,
    /// Wysokie - wykonanie w ciągu 1 sekundy
    High = 3,
    /// Średnie - wykonanie w ciągu 5 sekund
    Medium = 2,
    /// Niskie - wykonanie w ciągu 30 sekund
    Low = 1,
}

/// 📊 Status zadania
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Zadanie oczekuje na przydzielenie
    Pending,
    /// Zadanie zostało przydzielone agentowi
    Assigned,
    /// Zadanie jest w trakcie wykonywania
    InProgress,
    /// Zadanie zostało ukończone pomyślnie
    Completed,
    /// Zadanie nie powiodło się
    Failed,
    /// Zadanie zostało anulowane
    Cancelled,
    /// Zadanie przekroczyło limit czasu
    TimedOut,
}

/// 📋 Definicja zadania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unikalny identyfikator zadania
    pub id: Uuid,
    /// Typ zadania (np. "token_analysis", "risk_assessment")
    pub task_type: String,
    /// Priorytet zadania
    pub priority: TaskPriority,
    /// Aktualny status
    pub status: TaskStatus,
    /// ID przydzielonego agenta
    pub assigned_agent: Option<Uuid>,
    /// Czas utworzenia zadania
    pub created_at: DateTime<Utc>,
    /// Deadline wykonania
    pub deadline: DateTime<Utc>,
    /// Dane wejściowe zadania
    pub payload: Value,
    /// Wymagane możliwości agenta
    pub required_capabilities: Vec<AgentCapability>,
    /// Preferowany typ agenta
    pub preferred_agent_type: Option<AgentType>,
    /// Metadane zadania
    pub metadata: HashMap<String, String>,
    /// Liczba prób wykonania
    pub retry_count: u32,
    /// Maksymalna liczba prób
    pub max_retries: u32,
}

impl Task {
    /// Tworzy nowe zadanie
    pub fn new(
        task_type: String,
        priority: TaskPriority,
        payload: Value,
        required_capabilities: Vec<AgentCapability>,
    ) -> Self {
        let now = Utc::now();
        let timeout_seconds = match priority {
            TaskPriority::Critical => 5,
            TaskPriority::High => 30,
            TaskPriority::Medium => 120,
            TaskPriority::Low => 300,
        };
        
        Self {
            id: Uuid::new_v4(),
            task_type,
            priority,
            status: TaskStatus::Pending,
            assigned_agent: None,
            created_at: now,
            deadline: now + Duration::seconds(timeout_seconds),
            payload,
            required_capabilities,
            preferred_agent_type: None,
            metadata: HashMap::new(),
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    /// Sprawdza czy zadanie przekroczyło deadline
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.deadline
    }
    
    /// Sprawdza czy można ponowić zadanie
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    /// Zwiększa licznik prób
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// 📊 Wynik wykonania zadania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// ID zadania
    pub task_id: Uuid,
    /// ID agenta, który wykonał zadanie
    pub agent_id: Uuid,
    /// Czy zadanie zakończone sukcesem
    pub success: bool,
    /// Wynik zadania (jeśli sukces)
    pub result: Option<Value>,
    /// Błąd (jeśli niepowodzenie)
    pub error: Option<String>,
    /// Czas rozpoczęcia wykonywania
    pub started_at: DateTime<Utc>,
    /// Czas zakończenia wykonywania
    pub completed_at: DateTime<Utc>,
    /// Metryki wykonania
    pub metrics: TaskMetrics,
}

/// 📈 Metryki wykonania zadania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Czas wykonania w milisekundach
    pub execution_time_ms: u64,
    /// Zużycie pamięci w MB
    pub memory_usage_mb: f64,
    /// Zużycie CPU w procentach
    pub cpu_usage_percent: f64,
    /// Liczba wywołań API
    pub api_calls: u32,
    /// Rozmiar danych wejściowych w bajtach
    pub input_size_bytes: u64,
    /// Rozmiar danych wyjściowych w bajtach
    pub output_size_bytes: u64,
}

/// 🎯 Delegator zadań
pub struct TaskDelegator {
    /// Konfiguracja
    config: Arc<Config>,
    /// Kolejka zadań według priorytetu
    task_queues: HashMap<TaskPriority, VecDeque<Task>>,
    /// Aktywne zadania (ID -> Task)
    active_tasks: HashMap<Uuid, Task>,
    /// Historia zadań
    completed_tasks: VecDeque<TaskResult>,
    /// Statystyki delegacji
    stats: DelegationStats,
}

/// 📊 Statystyki delegacji zadań
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DelegationStats {
    pub total_tasks_created: u64,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub total_tasks_cancelled: u64,
    pub total_tasks_timed_out: u64,
    pub average_execution_time_ms: f64,
    pub success_rate: f64,
    pub current_queue_size: usize,
    pub current_active_tasks: usize,
}

impl TaskDelegator {
    /// Tworzy nowy delegator zadań
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("📋 Initializing TaskDelegator...");
        
        let mut task_queues = HashMap::new();
        task_queues.insert(TaskPriority::Critical, VecDeque::new());
        task_queues.insert(TaskPriority::High, VecDeque::new());
        task_queues.insert(TaskPriority::Medium, VecDeque::new());
        task_queues.insert(TaskPriority::Low, VecDeque::new());
        
        Ok(Self {
            config,
            task_queues,
            active_tasks: HashMap::new(),
            completed_tasks: VecDeque::new(),
            stats: DelegationStats::default(),
        })
    }
    
    /// Dodaje zadanie do kolejki
    #[instrument(skip(self, task))]
    pub async fn enqueue_task(&mut self, task: Task) -> Result<()> {
        let task_id = task.id;
        let priority = task.priority.clone();
        
        debug!("📥 Enqueuing task: {} (priority: {:?})", task_id, priority);
        
        // Dodaj do odpowiedniej kolejki według priorytetu
        if let Some(queue) = self.task_queues.get_mut(&priority) {
            queue.push_back(task);
            self.stats.total_tasks_created += 1;
            self.update_queue_stats();
            
            info!("✅ Task enqueued: {}", task_id);
        } else {
            return Err(anyhow!("Invalid task priority: {:?}", priority));
        }
        
        Ok(())
    }
    
    /// Pobiera następne zadanie z kolejki (według priorytetu)
    #[instrument(skip(self))]
    pub async fn dequeue_next_task(&mut self) -> Option<Task> {
        // Sprawdź kolejki według priorytetu (od najwyższego)
        let priorities = [
            TaskPriority::Critical,
            TaskPriority::High,
            TaskPriority::Medium,
            TaskPriority::Low,
        ];
        
        for priority in &priorities {
            if let Some(queue) = self.task_queues.get_mut(priority) {
                if let Some(task) = queue.pop_front() {
                    debug!("📤 Dequeued task: {} (priority: {:?})", task.id, priority);
                    self.update_queue_stats();
                    return Some(task);
                }
            }
        }
        
        None
    }
    
    /// Znajduje najlepszego agenta dla zadania
    #[instrument(skip(self, task, registry))]
    pub async fn find_best_agent(
        &self,
        task: &Task,
        registry: &AgentRegistry,
    ) -> Result<Uuid> {
        debug!("🔍 Finding best agent for task: {}", task.task_type);
        
        let agent_id = registry
            .find_best_agent(
                &task.task_type,
                &task.required_capabilities,
                task.preferred_agent_type.clone(),
            )
            .await?;
        
        debug!("✅ Best agent found: {}", agent_id);
        Ok(agent_id)
    }
    
    /// Przydziela zadanie do agenta
    #[instrument(skip(self, task))]
    pub async fn assign_task(&mut self, mut task: Task, agent_id: Uuid) -> Result<()> {
        let task_id = task.id;
        
        debug!("📌 Assigning task {} to agent {}", task_id, agent_id);
        
        task.assigned_agent = Some(agent_id);
        task.status = TaskStatus::Assigned;
        
        // Przenieś do aktywnych zadań
        self.active_tasks.insert(task_id, task);
        
        info!("✅ Task assigned: {} -> {}", task_id, agent_id);
        Ok(())
    }
    
    /// Rozpoczyna wykonywanie zadania
    #[instrument(skip(self))]
    pub async fn start_task_execution(&mut self, task_id: Uuid) -> Result<()> {
        if let Some(task) = self.active_tasks.get_mut(&task_id) {
            task.status = TaskStatus::InProgress;
            debug!("▶️ Task execution started: {}", task_id);
        } else {
            return Err(anyhow!("Task not found in active tasks: {}", task_id));
        }
        
        Ok(())
    }
    
    /// Kończy wykonywanie zadania
    #[instrument(skip(self, result))]
    pub async fn complete_task(&mut self, result: TaskResult) -> Result<()> {
        let task_id = result.task_id;
        
        debug!("✅ Completing task: {}", task_id);
        
        // Usuń z aktywnych zadań
        if let Some(mut task) = self.active_tasks.remove(&task_id) {
            task.status = if result.success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };
            
            // Dodaj do historii
            self.completed_tasks.push_back(result.clone());
            
            // Ogranicz rozmiar historii
            if self.completed_tasks.len() > 1000 {
                self.completed_tasks.pop_front();
            }
            
            // Aktualizuj statystyki
            if result.success {
                self.stats.total_tasks_completed += 1;
            } else {
                self.stats.total_tasks_failed += 1;
            }
            
            self.update_stats();
            
            info!("✅ Task completed: {} (success: {})", task_id, result.success);
        } else {
            warn!("⚠️ Task not found in active tasks: {}", task_id);
        }
        
        Ok(())
    }
    
    /// Anuluje zadanie
    #[instrument(skip(self))]
    pub async fn cancel_task(&mut self, task_id: Uuid) -> Result<()> {
        debug!("🚫 Cancelling task: {}", task_id);
        
        // Sprawdź w aktywnych zadaniach
        if let Some(mut task) = self.active_tasks.remove(&task_id) {
            task.status = TaskStatus::Cancelled;
            self.stats.total_tasks_cancelled += 1;
            self.update_stats();
            
            info!("✅ Task cancelled: {}", task_id);
            return Ok(());
        }
        
        // Sprawdź w kolejkach
        for queue in self.task_queues.values_mut() {
            if let Some(pos) = queue.iter().position(|t| t.id == task_id) {
                let mut task = queue.remove(pos).unwrap();
                task.status = TaskStatus::Cancelled;
                self.stats.total_tasks_cancelled += 1;
                self.update_queue_stats();
                self.update_stats();
                
                info!("✅ Task cancelled from queue: {}", task_id);
                return Ok(());
            }
        }
        
        Err(anyhow!("Task not found: {}", task_id))
    }
    
    /// Sprawdza zadania, które przekroczyły deadline
    #[instrument(skip(self))]
    pub async fn check_expired_tasks(&mut self) -> Result<Vec<Uuid>> {
        let mut expired_tasks = Vec::new();
        
        // Sprawdź aktywne zadania
        let mut to_remove = Vec::new();
        for (task_id, task) in &self.active_tasks {
            if task.is_expired() {
                expired_tasks.push(*task_id);
                to_remove.push(*task_id);
            }
        }
        
        // Usuń wygasłe zadania
        for task_id in to_remove {
            if let Some(mut task) = self.active_tasks.remove(&task_id) {
                task.status = TaskStatus::TimedOut;
                self.stats.total_tasks_timed_out += 1;
                
                debug!("⏰ Task timed out: {}", task_id);
            }
        }
        
        // Sprawdź kolejki
        for queue in self.task_queues.values_mut() {
            queue.retain(|task| {
                if task.is_expired() {
                    expired_tasks.push(task.id);
                    self.stats.total_tasks_timed_out += 1;
                    debug!("⏰ Task timed out in queue: {}", task.id);
                    false
                } else {
                    true
                }
            });
        }
        
        if !expired_tasks.is_empty() {
            self.update_queue_stats();
            self.update_stats();
            warn!("⚠️ Found {} expired tasks", expired_tasks.len());
        }
        
        Ok(expired_tasks)
    }
    
    /// Pobiera statystyki delegacji
    pub fn get_stats(&self) -> &DelegationStats {
        &self.stats
    }
    
    /// Pobiera aktywne zadania
    pub fn get_active_tasks(&self) -> &HashMap<Uuid, Task> {
        &self.active_tasks
    }
    
    /// Pobiera rozmiar kolejki dla priorytetu
    pub fn get_queue_size(&self, priority: &TaskPriority) -> usize {
        self.task_queues
            .get(priority)
            .map(|queue| queue.len())
            .unwrap_or(0)
    }
    
    /// Aktualizuje statystyki kolejek
    fn update_queue_stats(&mut self) {
        self.stats.current_queue_size = self.task_queues
            .values()
            .map(|queue| queue.len())
            .sum();
        
        self.stats.current_active_tasks = self.active_tasks.len();
    }
    
    /// Aktualizuje ogólne statystyki
    fn update_stats(&mut self) {
        let total_completed = self.stats.total_tasks_completed + self.stats.total_tasks_failed;
        
        if total_completed > 0 {
            self.stats.success_rate = self.stats.total_tasks_completed as f64 / total_completed as f64;
        }
        
        if !self.completed_tasks.is_empty() {
            self.stats.average_execution_time_ms = self.completed_tasks
                .iter()
                .map(|r| r.metrics.execution_time_ms as f64)
                .sum::<f64>() / self.completed_tasks.len() as f64;
        }
    }
}
