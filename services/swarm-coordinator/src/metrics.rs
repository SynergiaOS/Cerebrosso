//! 📊 Metrics - System wydajności i monitoringu
//! 
//! Zbieranie i analiza metryk w architekturze Hive Mind

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug, warn, instrument};

use crate::agent_types::AgentType;

/// 📊 Główne metryki systemu Swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    /// Metryki agentów
    pub agents: AgentMetrics,
    /// Metryki wydajności
    pub performance: PerformanceMetrics,
    /// Metryki zadań
    pub tasks: TaskMetrics,
    /// Metryki komunikacji
    pub communication: CommunicationMetrics,
    /// Metryki systemu
    pub system: SystemMetrics,
    /// Czas ostatniej aktualizacji
    pub last_updated: DateTime<Utc>,
}

impl SwarmMetrics {
    /// Tworzy nowe metryki
    pub fn new() -> Self {
        Self {
            agents: AgentMetrics::default(),
            performance: PerformanceMetrics::default(),
            tasks: TaskMetrics::default(),
            communication: CommunicationMetrics::default(),
            system: SystemMetrics::default(),
            last_updated: Utc::now(),
        }
    }
    
    /// Aktualizuje timestamp
    pub fn update_timestamp(&mut self) {
        self.last_updated = Utc::now();
    }
    
    /// Zwiększa licznik agentów
    pub fn increment_agent_count(&mut self, count: i32) {
        if count > 0 {
            self.agents.total_agents += count as u32;
        } else {
            self.agents.total_agents = self.agents.total_agents.saturating_sub((-count) as u32);
        }
        self.update_timestamp();
    }
}

/// 🤖 Metryki agentów
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Całkowita liczba agentów
    pub total_agents: u32,
    /// Aktywni agenci
    pub active_agents: u32,
    /// Zajęci agenci
    pub busy_agents: u32,
    /// Nieaktywni agenci
    pub inactive_agents: u32,
    /// Metryki według typu agenta
    pub by_type: HashMap<AgentType, AgentTypeMetrics>,
    /// Średni wynik wydajności
    pub average_performance_score: f64,
    /// Całkowita liczba zadań przydzielonych
    pub total_tasks_assigned: u64,
    /// Całkowita liczba zadań ukończonych
    pub total_tasks_completed: u64,
}

/// 📈 Metryki dla konkretnego typu agenta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentTypeMetrics {
    /// Liczba instancji
    pub instance_count: u32,
    /// Średni wynik wydajności
    pub average_performance: f64,
    /// Całkowity czas pracy (w sekundach)
    pub total_uptime_seconds: u64,
    /// Liczba ukończonych zadań
    pub completed_tasks: u64,
    /// Liczba nieudanych zadań
    pub failed_tasks: u64,
    /// Średni czas wykonania zadania (ms)
    pub average_task_duration_ms: f64,
}

/// ⚡ Metryki wydajności
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Średnie opóźnienie (ms)
    pub average_latency_ms: f64,
    /// Percentyle opóźnień
    pub latency_percentiles: LatencyPercentiles,
    /// Dokładność decyzji (0.0 - 1.0)
    pub decision_accuracy: f64,
    /// Przepustowość (zadania/sekundę)
    pub throughput_tasks_per_second: f64,
    /// Wykorzystanie zasobów
    pub resource_utilization: ResourceUtilization,
    /// Wskaźnik sukcesu (0.0 - 1.0)
    pub success_rate: f64,
    /// Czas odpowiedzi systemu (ms)
    pub system_response_time_ms: f64,
}

/// 📊 Percentyle opóźnień
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub p99_9: f64,
}

/// 💻 Wykorzystanie zasobów
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// Wykorzystanie CPU (%)
    pub cpu_percent: f64,
    /// Wykorzystanie pamięci (%)
    pub memory_percent: f64,
    /// Wykorzystanie sieci (MB/s)
    pub network_mbps: f64,
    /// Wykorzystanie dysku (%)
    pub disk_percent: f64,
}

/// 📋 Metryki zadań
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Całkowita liczba zadań
    pub total_tasks: u64,
    /// Zadania oczekujące
    pub pending_tasks: u32,
    /// Zadania w trakcie wykonywania
    pub in_progress_tasks: u32,
    /// Zadania ukończone
    pub completed_tasks: u64,
    /// Zadania nieudane
    pub failed_tasks: u64,
    /// Zadania anulowane
    pub cancelled_tasks: u64,
    /// Zadania przekraczające deadline
    pub timed_out_tasks: u64,
    /// Średni czas wykonania (ms)
    pub average_execution_time_ms: f64,
    /// Rozkład według priorytetu
    pub by_priority: HashMap<String, u64>,
    /// Rozkład według typu
    pub by_type: HashMap<String, u64>,
}

/// 📡 Metryki komunikacji
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationMetrics {
    /// Całkowita liczba wiadomości wysłanych
    pub total_messages_sent: u64,
    /// Całkowita liczba wiadomości otrzymanych
    pub total_messages_received: u64,
    /// Liczba broadcastów
    pub total_broadcasts: u64,
    /// Nieudane dostarczenia
    pub failed_deliveries: u64,
    /// Średni czas dostarczenia (ms)
    pub average_delivery_time_ms: f64,
    /// Aktywne połączenia
    pub active_connections: u32,
    /// Rozmiar kolejki wiadomości
    pub message_queue_size: u32,
    /// Przepustowość (wiadomości/sekundę)
    pub throughput_messages_per_second: f64,
}

/// 🖥️ Metryki systemu
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Czas działania systemu (sekundy)
    pub uptime_seconds: u64,
    /// Wersja systemu
    pub version: String,
    /// Stan systemu
    pub status: String,
    /// Liczba restartów
    pub restart_count: u32,
    /// Ostatni restart
    pub last_restart: Option<DateTime<Utc>>,
    /// Wykorzystanie pamięci (MB)
    pub memory_usage_mb: f64,
    /// Liczba wątków
    pub thread_count: u32,
    /// Liczba otwartych plików
    pub open_file_descriptors: u32,
}

/// 📊 Kolektor metryk
pub struct MetricsCollector {
    /// Główne metryki
    metrics: Arc<RwLock<SwarmMetrics>>,
    /// Historia metryk (ostatnie 24h)
    history: Arc<RwLock<Vec<SwarmMetrics>>>,
    /// Czas rozpoczęcia systemu
    start_time: DateTime<Utc>,
}

impl MetricsCollector {
    /// Tworzy nowy kolektor metryk
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(SwarmMetrics::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            start_time: Utc::now(),
        }
    }
    
    /// Aktualizuje metryki agentów
    #[instrument(skip(self))]
    pub async fn update_agent_metrics(
        &self,
        total: u32,
        active: u32,
        busy: u32,
        inactive: u32,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.agents.total_agents = total;
        metrics.agents.active_agents = active;
        metrics.agents.busy_agents = busy;
        metrics.agents.inactive_agents = inactive;
        metrics.update_timestamp();
        
        debug!("📊 Agent metrics updated: total={}, active={}, busy={}, inactive={}", 
               total, active, busy, inactive);
    }
    
    /// Aktualizuje metryki wydajności
    #[instrument(skip(self))]
    pub async fn update_performance_metrics(
        &self,
        latency_ms: f64,
        accuracy: f64,
        throughput: f64,
        success_rate: f64,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.performance.average_latency_ms = latency_ms;
        metrics.performance.decision_accuracy = accuracy;
        metrics.performance.throughput_tasks_per_second = throughput;
        metrics.performance.success_rate = success_rate;
        metrics.update_timestamp();
        
        debug!("📊 Performance metrics updated: latency={:.2}ms, accuracy={:.1}%, throughput={:.1}/s", 
               latency_ms, accuracy * 100.0, throughput);
    }
    
    /// Aktualizuje metryki zadań
    #[instrument(skip(self))]
    pub async fn update_task_metrics(
        &self,
        pending: u32,
        in_progress: u32,
        completed: u64,
        failed: u64,
        avg_execution_ms: f64,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.tasks.pending_tasks = pending;
        metrics.tasks.in_progress_tasks = in_progress;
        metrics.tasks.completed_tasks = completed;
        metrics.tasks.failed_tasks = failed;
        metrics.tasks.average_execution_time_ms = avg_execution_ms;
        metrics.tasks.total_tasks = completed + failed;
        metrics.update_timestamp();
        
        debug!("📊 Task metrics updated: pending={}, in_progress={}, completed={}, failed={}", 
               pending, in_progress, completed, failed);
    }
    
    /// Aktualizuje metryki komunikacji
    #[instrument(skip(self))]
    pub async fn update_communication_metrics(
        &self,
        sent: u64,
        received: u64,
        broadcasts: u64,
        failed: u64,
        active_connections: u32,
    ) {
        let mut metrics = self.metrics.write().await;
        metrics.communication.total_messages_sent = sent;
        metrics.communication.total_messages_received = received;
        metrics.communication.total_broadcasts = broadcasts;
        metrics.communication.failed_deliveries = failed;
        metrics.communication.active_connections = active_connections;
        metrics.update_timestamp();
        
        debug!("📊 Communication metrics updated: sent={}, received={}, broadcasts={}, failed={}", 
               sent, received, broadcasts, failed);
    }
    
    /// Rejestruje wykonanie zadania
    #[instrument(skip(self))]
    pub async fn record_task_execution(
        &self,
        task_type: &str,
        duration_ms: u64,
        success: bool,
        agent_type: AgentType,
    ) {
        let mut metrics = self.metrics.write().await;
        
        // Aktualizuj metryki zadań
        if success {
            metrics.tasks.completed_tasks += 1;
        } else {
            metrics.tasks.failed_tasks += 1;
        }
        
        // Aktualizuj rozkład według typu
        *metrics.tasks.by_type.entry(task_type.to_string()).or_insert(0) += 1;
        
        // Aktualizuj metryki typu agenta
        let agent_metrics = metrics.agents.by_type.entry(agent_type).or_insert_with(Default::default);
        if success {
            agent_metrics.completed_tasks += 1;
        } else {
            agent_metrics.failed_tasks += 1;
        }
        
        // Aktualizuj średni czas wykonania
        let total_tasks = agent_metrics.completed_tasks + agent_metrics.failed_tasks;
        if total_tasks > 0 {
            agent_metrics.average_task_duration_ms = 
                (agent_metrics.average_task_duration_ms * (total_tasks - 1) as f64 + duration_ms as f64) / total_tasks as f64;
        }
        
        metrics.update_timestamp();
        
        debug!("📊 Task execution recorded: type={}, duration={}ms, success={}, agent_type={:?}", 
               task_type, duration_ms, success, agent_type);
    }
    
    /// Rejestruje latencję operacji
    #[instrument(skip(self))]
    pub async fn record_latency(&self, operation: &str, latency_ms: f64) {
        // Tutaj można dodać bardziej szczegółowe śledzenie latencji
        debug!("📊 Latency recorded: operation={}, latency={:.2}ms", operation, latency_ms);
        
        // Aktualizuj średnią latencję
        let mut metrics = self.metrics.write().await;
        metrics.performance.average_latency_ms = 
            (metrics.performance.average_latency_ms * 0.9) + (latency_ms * 0.1);
        metrics.update_timestamp();
    }
    
    /// Pobiera aktualne metryki
    pub async fn get_metrics(&self) -> SwarmMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
    
    /// Pobiera historię metryk
    pub async fn get_metrics_history(&self) -> Vec<SwarmMetrics> {
        let history = self.history.read().await;
        history.clone()
    }
    
    /// Zapisuje snapshot metryk do historii
    #[instrument(skip(self))]
    pub async fn snapshot_metrics(&self) {
        let current_metrics = {
            let metrics = self.metrics.read().await;
            metrics.clone()
        };
        
        let mut history = self.history.write().await;
        history.push(current_metrics);
        
        // Ogranicz historię do ostatnich 24h (1440 minut przy snapshot co minutę)
        if history.len() > 1440 {
            history.remove(0);
        }
        
        debug!("📊 Metrics snapshot saved to history");
    }
    
    /// Oblicza czas działania systemu
    pub fn get_uptime_seconds(&self) -> u64 {
        (Utc::now() - self.start_time).num_seconds() as u64
    }
    
    /// Sprawdza czy system spełnia cele wydajnościowe
    pub async fn check_performance_targets(&self) -> PerformanceStatus {
        let metrics = self.metrics.read().await;
        
        let latency_ok = metrics.performance.average_latency_ms < 100.0; // <100ms
        let accuracy_ok = metrics.performance.decision_accuracy > 0.848; // >84.8%
        let success_rate_ok = metrics.performance.success_rate > 0.95; // >95%
        
        if latency_ok && accuracy_ok && success_rate_ok {
            PerformanceStatus::Excellent
        } else if latency_ok && accuracy_ok {
            PerformanceStatus::Good
        } else if latency_ok || accuracy_ok {
            PerformanceStatus::Degraded
        } else {
            PerformanceStatus::Poor
        }
    }
}

/// 🎯 Status wydajności systemu
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceStatus {
    /// Doskonała wydajność - wszystkie cele spełnione
    Excellent,
    /// Dobra wydajność - większość celów spełniona
    Good,
    /// Obniżona wydajność - niektóre cele niespełnione
    Degraded,
    /// Słaba wydajność - większość celów niespełniona
    Poor,
}
