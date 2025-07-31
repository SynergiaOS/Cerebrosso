//! 👑 Agent-Strateg Library
//! 
//! CEO agenta w architekturze Hive Mind - główny decydent i koordynator strategiczny
//! Odpowiedzialny za goal decomposition, task delegation i decision synthesis

pub mod config;
pub mod agent_strateg;
pub mod goal_decomposition;
pub mod task_delegation;
pub mod decision_synthesis;
pub mod strategy_planning;
pub mod swarm_communication;
pub mod ai_models;
pub mod metrics;
pub mod risk_management;

// Core exports
pub use config::Config;
pub use agent_strateg::{AgentStrateg, StrategState, StrategError};
pub use goal_decomposition::{GoalDecomposer, Goal, SubGoal, GoalPriority};
pub use task_delegation::{TaskDelegator, DelegationStrategy, TaskAssignment};
pub use decision_synthesis::{DecisionSynthesizer, Decision, DecisionConfidence, DecisionRationale};
pub use strategy_planning::{StrategyPlanner, TradingStrategy, StrategyType, StrategyMetrics};
pub use swarm_communication::{SwarmClient, SwarmMessage, MessageType};
pub use ai_models::{AIModelManager, ModelType, AIResponse};
pub use metrics::{StrategMetrics, PerformanceMetrics, DecisionMetrics};
pub use risk_management::{RiskManager, RiskAssessment, RiskLevel};

/// 🎯 Core Agent-Strateg Result Type
pub type StrategResult<T> = Result<T, StrategError>;

/// 🧪 Test utilities for Agent-Strateg
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8100,
                host: "localhost".to_string(),
            },
            swarm: config::SwarmConfig {
                coordinator_url: "http://localhost:8090".to_string(),
                agent_id: "test_strateg_1".to_string(),
                heartbeat_interval_ms: 1000,
                task_timeout_ms: 30000,
            },
            ai: config::AIConfig {
                primary_model: "gpt-4".to_string(),
                backup_model: "claude-3".to_string(),
                max_tokens: 4096,
                temperature: 0.1,
                decision_threshold: 0.8,
            },
            strategy: config::StrategyConfig {
                max_concurrent_goals: 5,
                goal_timeout_minutes: 30,
                decision_weight: 0.4, // 40% wagi w końcowej decyzji
                risk_tolerance: 0.3,
            },
            risk: config::RiskConfig {
                max_position_size: 1000.0,
                max_daily_loss: 100.0,
                stop_loss_percentage: 0.05,
                take_profit_percentage: 0.15,
            },
        })
    }
    
    /// Create a mock goal for testing
    pub fn create_mock_goal() -> Goal {
        Goal {
            id: uuid::Uuid::new_v4(),
            title: "Test Market Analysis".to_string(),
            description: "Analyze market conditions for token XYZ".to_string(),
            priority: GoalPriority::High,
            deadline: chrono::Utc::now() + chrono::Duration::hours(1),
            context: serde_json::json!({
                "token_address": "So11111111111111111111111111111111111111112",
                "market_cap": 1000000,
                "volume_24h": 50000
            }),
            sub_goals: vec![],
            status: goal_decomposition::GoalStatus::Pending,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_agent_strateg_initialization() {
        let config = create_test_config();
        
        // Test that config is created correctly
        assert_eq!(config.server.port, 8100);
        assert_eq!(config.strategy.decision_weight, 0.4);
        assert_eq!(config.ai.primary_model, "gpt-4");
    }
    
    #[tokio::test]
    async fn test_goal_decomposition() {
        let config = create_test_config();
        let goal = create_mock_goal();
        
        // Test goal structure
        assert_eq!(goal.priority, GoalPriority::High);
        assert_eq!(goal.title, "Test Market Analysis");
        assert!(goal.sub_goals.is_empty());
    }
    
    #[tokio::test]
    async fn test_decision_synthesis_structure() {
        let config = create_test_config();
        
        // Test decision synthesis configuration
        assert!(config.strategy.decision_weight > 0.0);
        assert!(config.ai.decision_threshold > 0.0);
        assert!(config.risk.max_position_size > 0.0);
    }
}

/// 🎯 Agent-Strateg Constants
pub mod constants {
    /// Maximum number of concurrent goals
    pub const MAX_CONCURRENT_GOALS: usize = 5;
    
    /// Default decision weight (40% of final decision)
    pub const DEFAULT_DECISION_WEIGHT: f64 = 0.4;
    
    /// Maximum task delegation depth
    pub const MAX_DELEGATION_DEPTH: usize = 3;
    
    /// Goal timeout in minutes
    pub const GOAL_TIMEOUT_MINUTES: i64 = 30;
    
    /// Decision confidence threshold
    pub const DECISION_CONFIDENCE_THRESHOLD: f64 = 0.8;
    
    /// Maximum number of sub-goals per goal
    pub const MAX_SUB_GOALS: usize = 10;
    
    /// Strategy planning horizon in hours
    pub const STRATEGY_HORIZON_HOURS: i64 = 24;
}
