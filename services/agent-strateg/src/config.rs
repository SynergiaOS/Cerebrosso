//! 🔧 Agent-Strateg Configuration
//! 
//! Centralna konfiguracja dla Agent-Strateg (CEO)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub swarm: SwarmConfig,
    pub ai: AIConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
    pub communication: CommunicationConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    /// URL SwarmCoordinator
    pub coordinator_url: String,
    /// Unikalny ID agenta
    pub agent_id: String,
    /// Interwał heartbeat w ms
    pub heartbeat_interval_ms: u64,
    /// Timeout zadań w ms
    pub task_timeout_ms: u64,
    /// Maksymalna liczba równoczesnych zadań
    pub max_concurrent_tasks: usize,
    /// Typ agenta
    pub agent_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// Główny model AI (np. GPT-4)
    pub primary_model: String,
    /// Model zapasowy
    pub backup_model: String,
    /// Maksymalna liczba tokenów
    pub max_tokens: usize,
    /// Temperatura dla kreatywności
    pub temperature: f64,
    /// Próg pewności decyzji
    pub decision_threshold: f64,
    /// Konfiguracje modeli
    pub models: AIModelsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelsConfig {
    /// GPT-4 dla strategicznych decyzji
    pub gpt4: ModelConfig,
    /// Claude-3 jako backup
    pub claude3: ModelConfig,
    /// Llama3 dla lokalnych operacji
    pub llama3: ModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub endpoint: String,
    pub api_key: String,
    pub max_tokens: usize,
    pub temperature: f64,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Maksymalna liczba równoczesnych celów
    pub max_concurrent_goals: usize,
    /// Timeout celu w minutach
    pub goal_timeout_minutes: i64,
    /// Waga decyzyjna agenta (0.0 - 1.0)
    pub decision_weight: f64,
    /// Tolerancja ryzyka (0.0 - 1.0)
    pub risk_tolerance: f64,
    /// Horyzont planowania strategicznego (godziny)
    pub planning_horizon_hours: i64,
    /// Próg pewności dla delegacji zadań
    pub delegation_confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maksymalny rozmiar pozycji w SOL
    pub max_position_size: f64,
    /// Maksymalna dzienna strata w SOL
    pub max_daily_loss: f64,
    /// Procent stop-loss
    pub stop_loss_percentage: f64,
    /// Procent take-profit
    pub take_profit_percentage: f64,
    /// Maksymalna liczba równoczesnych pozycji
    pub max_concurrent_positions: usize,
    /// Minimalna pewność dla otwarcia pozycji
    pub min_confidence_for_trade: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    /// URL Redis dla komunikacji
    pub redis_url: String,
    /// Rozmiar bufora wiadomości
    pub message_buffer_size: usize,
    /// Timeout komunikacji w ms
    pub communication_timeout_ms: u64,
    /// Interwał retry w ms
    pub retry_interval_ms: u64,
    /// Maksymalna liczba prób
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Czy włączyć metryki
    pub metrics_enabled: bool,
    /// Port Prometheus
    pub prometheus_port: u16,
    /// Poziom logowania
    pub log_level: String,
    /// Interwał raportowania metryk w sekundach
    pub metrics_interval_seconds: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            server: ServerConfig {
                port: env::var("STRATEG_PORT")
                    .unwrap_or_else(|_| "8100".to_string())
                    .parse()?,
                host: env::var("STRATEG_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                workers: env::var("STRATEG_WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()?,
            },
            swarm: SwarmConfig {
                coordinator_url: env::var("SWARM_COORDINATOR_URL")
                    .unwrap_or_else(|_| "http://localhost:8090".to_string()),
                agent_id: env::var("STRATEG_AGENT_ID")
                    .unwrap_or_else(|_| "strateg_1".to_string()),
                heartbeat_interval_ms: env::var("STRATEG_HEARTBEAT_MS")
                    .unwrap_or_else(|_| "5000".to_string())
                    .parse()?,
                task_timeout_ms: env::var("STRATEG_TASK_TIMEOUT_MS")
                    .unwrap_or_else(|_| "30000".to_string())
                    .parse()?,
                max_concurrent_tasks: env::var("STRATEG_MAX_TASKS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                agent_type: "Strateg".to_string(),
            },
            ai: AIConfig {
                primary_model: env::var("STRATEG_PRIMARY_MODEL")
                    .unwrap_or_else(|_| "gpt-4".to_string()),
                backup_model: env::var("STRATEG_BACKUP_MODEL")
                    .unwrap_or_else(|_| "claude-3".to_string()),
                max_tokens: env::var("STRATEG_MAX_TOKENS")
                    .unwrap_or_else(|_| "4096".to_string())
                    .parse()?,
                temperature: env::var("STRATEG_TEMPERATURE")
                    .unwrap_or_else(|_| "0.1".to_string())
                    .parse()?,
                decision_threshold: env::var("STRATEG_DECISION_THRESHOLD")
                    .unwrap_or_else(|_| "0.8".to_string())
                    .parse()?,
                models: AIModelsConfig {
                    gpt4: ModelConfig {
                        endpoint: env::var("GPT4_ENDPOINT")
                            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                        api_key: env::var("OPENAI_API_KEY")
                            .unwrap_or_else(|_| "demo_key".to_string()),
                        max_tokens: 4096,
                        temperature: 0.1,
                        timeout_seconds: 30,
                    },
                    claude3: ModelConfig {
                        endpoint: env::var("CLAUDE3_ENDPOINT")
                            .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string()),
                        api_key: env::var("ANTHROPIC_API_KEY")
                            .unwrap_or_else(|_| "demo_key".to_string()),
                        max_tokens: 4096,
                        temperature: 0.1,
                        timeout_seconds: 30,
                    },
                    llama3: ModelConfig {
                        endpoint: env::var("LLAMA3_ENDPOINT")
                            .unwrap_or_else(|_| "http://localhost:11434".to_string()),
                        api_key: env::var("LLAMA3_API_KEY")
                            .unwrap_or_else(|_| "local".to_string()),
                        max_tokens: 4096,
                        temperature: 0.1,
                        timeout_seconds: 60,
                    },
                },
            },
            strategy: StrategyConfig {
                max_concurrent_goals: env::var("STRATEG_MAX_GOALS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()?,
                goal_timeout_minutes: env::var("STRATEG_GOAL_TIMEOUT_MIN")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                decision_weight: env::var("STRATEG_DECISION_WEIGHT")
                    .unwrap_or_else(|_| "0.4".to_string())
                    .parse()?,
                risk_tolerance: env::var("STRATEG_RISK_TOLERANCE")
                    .unwrap_or_else(|_| "0.3".to_string())
                    .parse()?,
                planning_horizon_hours: env::var("STRATEG_PLANNING_HORIZON_H")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
                delegation_confidence_threshold: env::var("STRATEG_DELEGATION_THRESHOLD")
                    .unwrap_or_else(|_| "0.7".to_string())
                    .parse()?,
            },
            risk: RiskConfig {
                max_position_size: env::var("STRATEG_MAX_POSITION_SOL")
                    .unwrap_or_else(|_| "10.0".to_string())
                    .parse()?,
                max_daily_loss: env::var("STRATEG_MAX_DAILY_LOSS_SOL")
                    .unwrap_or_else(|_| "1.0".to_string())
                    .parse()?,
                stop_loss_percentage: env::var("STRATEG_STOP_LOSS_PCT")
                    .unwrap_or_else(|_| "0.05".to_string())
                    .parse()?,
                take_profit_percentage: env::var("STRATEG_TAKE_PROFIT_PCT")
                    .unwrap_or_else(|_| "0.15".to_string())
                    .parse()?,
                max_concurrent_positions: env::var("STRATEG_MAX_POSITIONS")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
                min_confidence_for_trade: env::var("STRATEG_MIN_TRADE_CONFIDENCE")
                    .unwrap_or_else(|_| "0.85".to_string())
                    .parse()?,
            },
            communication: CommunicationConfig {
                redis_url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                message_buffer_size: env::var("STRATEG_MSG_BUFFER_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
                communication_timeout_ms: env::var("STRATEG_COMM_TIMEOUT_MS")
                    .unwrap_or_else(|_| "5000".to_string())
                    .parse()?,
                retry_interval_ms: env::var("STRATEG_RETRY_INTERVAL_MS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
                max_retries: env::var("STRATEG_MAX_RETRIES")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()?,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: env::var("STRATEG_METRICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                prometheus_port: env::var("STRATEG_PROMETHEUS_PORT")
                    .unwrap_or_else(|_| "9100".to_string())
                    .parse()?,
                log_level: env::var("STRATEG_LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                metrics_interval_seconds: env::var("STRATEG_METRICS_INTERVAL_S")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()?,
            },
        })
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate decision weight
        if self.strategy.decision_weight < 0.0 || self.strategy.decision_weight > 1.0 {
            return Err(anyhow::anyhow!("Decision weight must be between 0.0 and 1.0"));
        }
        
        // Validate risk tolerance
        if self.risk.risk_tolerance < 0.0 || self.risk.risk_tolerance > 1.0 {
            return Err(anyhow::anyhow!("Risk tolerance must be between 0.0 and 1.0"));
        }
        
        // Validate position size
        if self.risk.max_position_size <= 0.0 {
            return Err(anyhow::anyhow!("Max position size must be positive"));
        }
        
        // Validate timeouts
        if self.swarm.task_timeout_ms == 0 {
            return Err(anyhow::anyhow!("Task timeout must be positive"));
        }
        
        Ok(())
    }
}
