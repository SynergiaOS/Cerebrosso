//! 🐝 SwarmCoordinator Library
//! 
//! Centralny orkiestrator Hive Mind Architecture dla Cerberus Phoenix v3.0
//! Zarządza komunikacją między agentami, deleguje zadania i monitoruje wydajność swarm

pub mod config;
pub mod swarm_coordinator;
pub mod agent_registry;
pub mod task_delegation;
pub mod communication;
pub mod metrics;
pub mod agent_types;
pub mod memory_store;
pub mod feedback_loop;

// Core exports
pub use config::Config;
pub use swarm_coordinator::{SwarmCoordinator, SwarmState, CoordinatorError};
pub use agent_registry::{AgentRegistry, AgentInfo, AgentStatus, AgentCapability};
pub use task_delegation::{TaskDelegator, Task, TaskPriority, TaskStatus, TaskResult};
pub use communication::{CommunicationHub, AgentMessage, MessageType, MessagePriority};
pub use metrics::{SwarmMetrics, AgentMetrics, PerformanceMetrics};
pub use agent_types::{AgentType, AgentRole, SpecializedAgent};
pub use memory_store::{MemoryStore, MemoryEntry, MemoryType};
pub use feedback_loop::{FeedbackLoop, FeedbackData, LearningMetrics};

/// 🎯 Core SwarmCoordinator Result Type
pub type SwarmResult<T> = Result<T, CoordinatorError>;

/// 🧪 Test utilities for SwarmCoordinator
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8090,
                host: "localhost".to_string(),
            },
            redis: config::RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                timeout_ms: 5000,
            },
            qdrant: config::QdrantConfig {
                url: "http://localhost:6333".to_string(),
                collection_name: "swarm_memory".to_string(),
                vector_size: 1536,
            },
            swarm: config::SwarmConfig {
                max_agents: 40,
                min_agents: 4,
                task_timeout_ms: 30000,
                heartbeat_interval_ms: 1000,
                decision_threshold: 0.848, // 84.8% accuracy target
            },
            communication: config::CommunicationConfig {
                websocket_port: 8091,
                message_buffer_size: 1000,
                broadcast_timeout_ms: 100,
            },
        })
    }
    
    /// Create a mock agent for testing
    pub fn create_mock_agent(agent_type: AgentType) -> AgentInfo {
        AgentInfo {
            id: uuid::Uuid::new_v4(),
            agent_type,
            status: AgentStatus::Active,
            capabilities: vec![
                AgentCapability::Analysis,
                AgentCapability::DecisionMaking,
            ],
            last_heartbeat: chrono::Utc::now(),
            performance_score: 0.85,
            current_tasks: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_swarm_coordinator_initialization() {
        let config = create_test_config();
        
        // Test that config is created correctly
        assert_eq!(config.server.port, 8090);
        assert_eq!(config.swarm.max_agents, 40);
        assert_eq!(config.swarm.decision_threshold, 0.848);
    }
    
    #[tokio::test]
    async fn test_agent_registry_operations() {
        let config = create_test_config();
        
        // Test agent creation
        let agent = create_mock_agent(AgentType::Strateg);
        assert_eq!(agent.agent_type, AgentType::Strateg);
        assert_eq!(agent.status, AgentStatus::Active);
        assert_eq!(agent.capabilities.len(), 2);
    }
    
    #[tokio::test]
    async fn test_task_delegation_structure() {
        let config = create_test_config();
        
        // Test task structure
        let task = Task {
            id: uuid::Uuid::new_v4(),
            task_type: "token_analysis".to_string(),
            priority: TaskPriority::High,
            status: TaskStatus::Pending,
            assigned_agent: None,
            created_at: chrono::Utc::now(),
            deadline: chrono::Utc::now() + chrono::Duration::seconds(30),
            payload: serde_json::json!({
                "token_address": "So11111111111111111111111111111111111111112",
                "analysis_type": "risk_assessment"
            }),
        };
        
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.assigned_agent.is_none());
    }
}

/// 🎯 SwarmCoordinator Constants
pub mod constants {
    /// Maximum latency target for swarm operations (100ms)
    pub const MAX_LATENCY_MS: u64 = 100;
    
    /// Target decision accuracy (84.8%)
    pub const TARGET_ACCURACY: f64 = 0.848;
    
    /// Maximum number of concurrent tasks per agent
    pub const MAX_TASKS_PER_AGENT: usize = 5;
    
    /// Heartbeat timeout before marking agent as inactive
    pub const AGENT_TIMEOUT_MS: u64 = 5000;
    
    /// Memory retention period for feedback data (7 days)
    pub const MEMORY_RETENTION_DAYS: i64 = 7;
}
