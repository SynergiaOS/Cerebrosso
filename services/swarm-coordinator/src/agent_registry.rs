//! 🤖 Agent Registry - Zarządzanie agentami w systemie
//! 
//! Rejestr wszystkich agentów w architekturze Hive Mind

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error, instrument};

use crate::{
    config::Config,
    agent_types::{AgentType, AgentCapability, SpecializedAgent},
    constants::AGENT_TIMEOUT_MS,
};

/// 📊 Status agenta w systemie
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent jest aktywny i gotowy do pracy
    Active,
    /// Agent jest zajęty wykonywaniem zadania
    Busy,
    /// Agent jest niedostępny (timeout heartbeat)
    Inactive,
    /// Agent jest w trybie maintenance
    Maintenance,
    /// Agent został wyłączony
    Shutdown,
}

/// 🤖 Informacje o agencie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unikalny identyfikator agenta
    pub id: Uuid,
    /// Typ agenta
    pub agent_type: AgentType,
    /// Aktualny status
    pub status: AgentStatus,
    /// Możliwości agenta
    pub capabilities: Vec<AgentCapability>,
    /// Ostatni heartbeat
    pub last_heartbeat: DateTime<Utc>,
    /// Wynik wydajności (0.0 - 1.0)
    pub performance_score: f64,
    /// Lista aktualnie wykonywanych zadań
    pub current_tasks: Vec<Uuid>,
    /// Adres endpoint agenta
    pub endpoint: Option<String>,
    /// Metadane agenta
    pub metadata: HashMap<String, String>,
}

impl AgentInfo {
    /// Tworzy nowego agenta
    pub fn new(
        agent_type: AgentType,
        capabilities: Vec<AgentCapability>,
        endpoint: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_type,
            status: AgentStatus::Active,
            capabilities,
            last_heartbeat: Utc::now(),
            performance_score: 0.5, // Początkowy wynik
            current_tasks: Vec::new(),
            endpoint,
            metadata: HashMap::new(),
        }
    }
    
    /// Sprawdza czy agent może obsłużyć zadanie
    pub fn can_handle_task(&self, task_type: &str, required_capabilities: &[AgentCapability]) -> bool {
        // Sprawdź status
        if self.status != AgentStatus::Active {
            return false;
        }
        
        // Sprawdź czy agent ma wymagane możliwości
        for capability in required_capabilities {
            if !self.capabilities.contains(capability) {
                return false;
            }
        }
        
        // Sprawdź czy agent może obsłużyć więcej zadań
        let max_tasks = match self.agent_type {
            AgentType::Strateg => 10,
            AgentType::Analityk => 5,
            AgentType::Quant => 8,
            AgentType::Nadzorca => 3,
        };
        
        if self.current_tasks.len() >= max_tasks {
            return false;
        }
        
        true
    }
    
    /// Aktualizuje heartbeat agenta
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        if self.status == AgentStatus::Inactive {
            self.status = AgentStatus::Active;
        }
    }
    
    /// Sprawdza czy agent jest aktywny (heartbeat w ostatnich 5 sekundach)
    pub fn is_active(&self) -> bool {
        let timeout = Duration::milliseconds(AGENT_TIMEOUT_MS as i64);
        Utc::now() - self.last_heartbeat < timeout
    }
    
    /// Dodaje zadanie do agenta
    pub fn assign_task(&mut self, task_id: Uuid) -> Result<()> {
        if self.status != AgentStatus::Active {
            return Err(anyhow!("Agent is not active"));
        }
        
        self.current_tasks.push(task_id);
        if self.current_tasks.len() == 1 {
            self.status = AgentStatus::Busy;
        }
        
        Ok(())
    }
    
    /// Usuwa zadanie z agenta
    pub fn complete_task(&mut self, task_id: Uuid) -> Result<()> {
        self.current_tasks.retain(|&id| id != task_id);
        
        if self.current_tasks.is_empty() {
            self.status = AgentStatus::Active;
        }
        
        Ok(())
    }
    
    /// Aktualizuje wynik wydajności agenta
    pub fn update_performance_score(&mut self, score: f64) {
        // Exponential moving average
        let alpha = 0.1;
        self.performance_score = alpha * score + (1.0 - alpha) * self.performance_score;
    }
}

/// 📋 Rejestr agentów
pub struct AgentRegistry {
    /// Konfiguracja
    config: Arc<Config>,
    /// Mapa agentów (ID -> AgentInfo)
    agents: HashMap<Uuid, AgentInfo>,
    /// Indeks według typu agenta
    agents_by_type: HashMap<AgentType, Vec<Uuid>>,
    /// Statystyki rejestru
    stats: RegistryStats,
}

/// 📊 Statystyki rejestru agentów
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub active_agents: usize,
    pub busy_agents: usize,
    pub inactive_agents: usize,
    pub average_performance: f64,
    pub total_tasks_assigned: u64,
    pub total_tasks_completed: u64,
}

impl AgentRegistry {
    /// Tworzy nowy rejestr agentów
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("📋 Initializing AgentRegistry...");
        
        Ok(Self {
            config,
            agents: HashMap::new(),
            agents_by_type: HashMap::new(),
            stats: RegistryStats::default(),
        })
    }
    
    /// Rejestruje nowego agenta
    #[instrument(skip(self))]
    pub async fn register_agent(&mut self, agent_info: AgentInfo) -> Result<()> {
        let agent_id = agent_info.id;
        let agent_type = agent_info.agent_type.clone();
        
        debug!("📝 Registering agent: {} ({})", agent_id, agent_type);
        
        // Sprawdź limity dla typu agenta
        self.check_agent_limits(&agent_type)?;
        
        // Dodaj do mapy głównej
        self.agents.insert(agent_id, agent_info);
        
        // Dodaj do indeksu według typu
        self.agents_by_type
            .entry(agent_type)
            .or_insert_with(Vec::new)
            .push(agent_id);
        
        // Aktualizuj statystyki
        self.update_stats();
        
        info!("✅ Agent registered: {}", agent_id);
        Ok(())
    }
    
    /// Wyrejestrowuje agenta
    #[instrument(skip(self))]
    pub async fn unregister_agent(&mut self, agent_id: Uuid) -> Result<()> {
        debug!("🗑️ Unregistering agent: {}", agent_id);
        
        if let Some(agent) = self.agents.remove(&agent_id) {
            // Usuń z indeksu według typu
            if let Some(agents) = self.agents_by_type.get_mut(&agent.agent_type) {
                agents.retain(|&id| id != agent_id);
            }
            
            // Aktualizuj statystyki
            self.update_stats();
            
            info!("✅ Agent unregistered: {}", agent_id);
        } else {
            warn!("⚠️ Agent not found for unregistration: {}", agent_id);
        }
        
        Ok(())
    }
    
    /// Znajduje najlepszego agenta dla zadania
    #[instrument(skip(self))]
    pub async fn find_best_agent(
        &self,
        task_type: &str,
        required_capabilities: &[AgentCapability],
        preferred_agent_type: Option<AgentType>,
    ) -> Result<Uuid> {
        debug!("🔍 Finding best agent for task: {}", task_type);
        
        let mut candidates = Vec::new();
        
        // Jeśli określono preferowany typ agenta, szukaj tylko w tej grupie
        if let Some(agent_type) = preferred_agent_type {
            if let Some(agent_ids) = self.agents_by_type.get(&agent_type) {
                for &agent_id in agent_ids {
                    if let Some(agent) = self.agents.get(&agent_id) {
                        if agent.can_handle_task(task_type, required_capabilities) {
                            candidates.push((agent_id, agent.performance_score, agent.current_tasks.len()));
                        }
                    }
                }
            }
        } else {
            // Szukaj we wszystkich agentach
            for (agent_id, agent) in &self.agents {
                if agent.can_handle_task(task_type, required_capabilities) {
                    candidates.push((*agent_id, agent.performance_score, agent.current_tasks.len()));
                }
            }
        }
        
        if candidates.is_empty() {
            return Err(anyhow!("No suitable agent found for task: {}", task_type));
        }
        
        // Sortuj według wydajności i obciążenia
        candidates.sort_by(|a, b| {
            // Najpierw według liczby zadań (mniej = lepiej)
            let task_cmp = a.2.cmp(&b.2);
            if task_cmp != std::cmp::Ordering::Equal {
                return task_cmp;
            }
            // Potem według wydajności (więcej = lepiej)
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let best_agent_id = candidates[0].0;
        debug!("✅ Best agent found: {}", best_agent_id);
        
        Ok(best_agent_id)
    }
    
    /// Aktualizuje heartbeat agenta
    #[instrument(skip(self))]
    pub async fn update_agent_heartbeat(&mut self, agent_id: Uuid) -> Result<()> {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.update_heartbeat();
            debug!("💓 Heartbeat updated for agent: {}", agent_id);
        } else {
            warn!("⚠️ Agent not found for heartbeat update: {}", agent_id);
        }
        
        Ok(())
    }
    
    /// Sprawdza heartbeaty wszystkich agentów
    #[instrument(skip(self))]
    pub async fn check_agent_heartbeats(&mut self) -> Result<()> {
        let mut inactive_agents = Vec::new();
        
        for (agent_id, agent) in &mut self.agents {
            if !agent.is_active() && agent.status != AgentStatus::Inactive {
                agent.status = AgentStatus::Inactive;
                inactive_agents.push(*agent_id);
            }
        }
        
        if !inactive_agents.is_empty() {
            warn!("⚠️ Found {} inactive agents", inactive_agents.len());
            for agent_id in inactive_agents {
                debug!("💀 Agent marked as inactive: {}", agent_id);
            }
        }
        
        // Aktualizuj statystyki
        self.update_stats();
        
        Ok(())
    }
    
    /// Pobiera informacje o agencie
    pub fn get_agent(&self, agent_id: Uuid) -> Option<&AgentInfo> {
        self.agents.get(&agent_id)
    }
    
    /// Pobiera wszystkich agentów określonego typu
    pub fn get_agents_by_type(&self, agent_type: &AgentType) -> Vec<&AgentInfo> {
        self.agents_by_type
            .get(agent_type)
            .map(|agent_ids| {
                agent_ids
                    .iter()
                    .filter_map(|id| self.agents.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Pobiera statystyki rejestru
    pub fn get_stats(&self) -> &RegistryStats {
        &self.stats
    }
    
    /// Sprawdza limity dla typu agenta
    fn check_agent_limits(&self, agent_type: &AgentType) -> Result<()> {
        let current_count = self.agents_by_type
            .get(agent_type)
            .map(|agents| agents.len())
            .unwrap_or(0);
        
        let max_instances = match agent_type {
            AgentType::Strateg => self.config.agents.strateg.max_instances,
            AgentType::Analityk => self.config.agents.analityk.max_instances,
            AgentType::Quant => self.config.agents.quant.max_instances,
            AgentType::Nadzorca => self.config.agents.nadzorca.max_instances,
        };
        
        if current_count >= max_instances {
            return Err(anyhow!(
                "Maximum number of {} agents reached: {}/{}",
                agent_type,
                current_count,
                max_instances
            ));
        }
        
        Ok(())
    }
    
    /// Aktualizuje statystyki rejestru
    fn update_stats(&mut self) {
        self.stats.total_agents = self.agents.len();
        self.stats.active_agents = self.agents.values()
            .filter(|a| a.status == AgentStatus::Active)
            .count();
        self.stats.busy_agents = self.agents.values()
            .filter(|a| a.status == AgentStatus::Busy)
            .count();
        self.stats.inactive_agents = self.agents.values()
            .filter(|a| a.status == AgentStatus::Inactive)
            .count();
        
        if !self.agents.is_empty() {
            self.stats.average_performance = self.agents.values()
                .map(|a| a.performance_score)
                .sum::<f64>() / self.agents.len() as f64;
        }
    }
}
