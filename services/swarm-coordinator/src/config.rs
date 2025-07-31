//! 🔧 SwarmCoordinator Configuration
//! 
//! Centralna konfiguracja dla wszystkich komponentów Hive Mind

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub qdrant: QdrantConfig,
    pub swarm: SwarmConfig,
    pub communication: CommunicationConfig,
    pub agents: AgentsConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout_ms: u64,
    pub streams: RedisStreamsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisStreamsConfig {
    pub agent_commands: String,
    pub agent_responses: String,
    pub task_queue: String,
    pub metrics_stream: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub vector_size: u64,
    pub distance_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub max_agents: usize,
    pub min_agents: usize,
    pub task_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub decision_threshold: f64,
    pub auto_scaling: AutoScalingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    pub enabled: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub cooldown_period_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationConfig {
    pub websocket_port: u16,
    pub message_buffer_size: usize,
    pub broadcast_timeout_ms: u64,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    pub strateg: AgentConfig,
    pub analityk: AgentConfig,
    pub quant: AgentConfig,
    pub nadzorca: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_instances: usize,
    pub resource_limits: ResourceLimits,
    pub specialization: SpecializationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_concurrent_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationConfig {
    pub ai_models: Vec<String>,
    pub data_sources: Vec<String>,
    pub decision_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub agent_auth_required: bool,
    pub rate_limiting: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub window_size_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_port: u16,
    pub tracing_level: String,
    pub performance_tracking: PerformanceTrackingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrackingConfig {
    pub latency_percentiles: Vec<f64>,
    pub accuracy_window_size: usize,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_latency_ms: u64,
    pub min_accuracy: f64,
    pub max_error_rate: f64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            server: ServerConfig {
                port: env::var("SWARM_PORT")
                    .unwrap_or_else(|_| "8090".to_string())
                    .parse()?,
                host: env::var("SWARM_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                workers: env::var("SWARM_WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                pool_size: env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                timeout_ms: env::var("REDIS_TIMEOUT_MS")
                    .unwrap_or_else(|_| "5000".to_string())
                    .parse()?,
                streams: RedisStreamsConfig {
                    agent_commands: "swarm:commands".to_string(),
                    agent_responses: "swarm:responses".to_string(),
                    task_queue: "swarm:tasks".to_string(),
                    metrics_stream: "swarm:metrics".to_string(),
                },
            },
            qdrant: QdrantConfig {
                url: env::var("QDRANT_URL")
                    .unwrap_or_else(|_| "http://localhost:6333".to_string()),
                collection_name: env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_| "swarm_memory".to_string()),
                vector_size: env::var("QDRANT_VECTOR_SIZE")
                    .unwrap_or_else(|_| "1536".to_string())
                    .parse()?,
                distance_metric: "Cosine".to_string(),
            },
            swarm: SwarmConfig {
                max_agents: env::var("SWARM_MAX_AGENTS")
                    .unwrap_or_else(|_| "40".to_string())
                    .parse()?,
                min_agents: env::var("SWARM_MIN_AGENTS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()?,
                task_timeout_ms: env::var("SWARM_TASK_TIMEOUT_MS")
                    .unwrap_or_else(|_| "30000".to_string())
                    .parse()?,
                heartbeat_interval_ms: env::var("SWARM_HEARTBEAT_MS")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()?,
                decision_threshold: env::var("SWARM_DECISION_THRESHOLD")
                    .unwrap_or_else(|_| "0.848".to_string())
                    .parse()?,
                auto_scaling: AutoScalingConfig {
                    enabled: env::var("SWARM_AUTO_SCALING")
                        .unwrap_or_else(|_| "true".to_string())
                        .parse()?,
                    scale_up_threshold: 0.8,
                    scale_down_threshold: 0.3,
                    cooldown_period_ms: 60000,
                },
            },
            communication: CommunicationConfig {
                websocket_port: env::var("SWARM_WS_PORT")
                    .unwrap_or_else(|_| "8091".to_string())
                    .parse()?,
                message_buffer_size: 1000,
                broadcast_timeout_ms: 100,
                compression_enabled: true,
            },
            agents: AgentsConfig {
                strateg: AgentConfig {
                    max_instances: 1,
                    resource_limits: ResourceLimits {
                        max_memory_mb: 512,
                        max_cpu_percent: 50.0,
                        max_concurrent_tasks: 10,
                    },
                    specialization: SpecializationConfig {
                        ai_models: vec!["gpt-4".to_string(), "claude-3".to_string()],
                        data_sources: vec!["market_data".to_string(), "sentiment".to_string()],
                        decision_weight: 0.4,
                    },
                },
                analityk: AgentConfig {
                    max_instances: 2,
                    resource_limits: ResourceLimits {
                        max_memory_mb: 256,
                        max_cpu_percent: 30.0,
                        max_concurrent_tasks: 5,
                    },
                    specialization: SpecializationConfig {
                        ai_models: vec!["phi-3".to_string()],
                        data_sources: vec!["whitepaper".to_string(), "team_analysis".to_string()],
                        decision_weight: 0.25,
                    },
                },
                quant: AgentConfig {
                    max_instances: 3,
                    resource_limits: ResourceLimits {
                        max_memory_mb: 1024,
                        max_cpu_percent: 70.0,
                        max_concurrent_tasks: 8,
                    },
                    specialization: SpecializationConfig {
                        ai_models: vec!["llama3".to_string()],
                        data_sources: vec!["price_data".to_string(), "volume_data".to_string()],
                        decision_weight: 0.3,
                    },
                },
                nadzorca: AgentConfig {
                    max_instances: 1,
                    resource_limits: ResourceLimits {
                        max_memory_mb: 128,
                        max_cpu_percent: 20.0,
                        max_concurrent_tasks: 3,
                    },
                    specialization: SpecializationConfig {
                        ai_models: vec!["mistral".to_string()],
                        data_sources: vec!["security_logs".to_string(), "anomaly_detection".to_string()],
                        decision_weight: 0.05,
                    },
                },
            },
            security: SecurityConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "swarm_secret_key".to_string()),
                agent_auth_required: true,
                rate_limiting: RateLimitConfig {
                    requests_per_second: 1000,
                    burst_size: 100,
                    window_size_ms: 1000,
                },
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: env::var("PROMETHEUS_PORT")
                    .unwrap_or_else(|_| "9090".to_string())
                    .parse()?,
                tracing_level: env::var("TRACING_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                performance_tracking: PerformanceTrackingConfig {
                    latency_percentiles: vec![0.5, 0.9, 0.95, 0.99],
                    accuracy_window_size: 1000,
                    alert_thresholds: AlertThresholds {
                        max_latency_ms: 100,
                        min_accuracy: 0.848,
                        max_error_rate: 0.05,
                    },
                },
            },
        })
    }
}
