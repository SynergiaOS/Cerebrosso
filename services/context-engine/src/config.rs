//! 🔧 Context Engine Configuration
//! 
//! Centralna konfiguracja dla Context Engine (CEM)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub memory: MemoryConfig,
    pub embeddings: EmbeddingConfig,
    pub optimization: OptimizationConfig,
    pub feedback: FeedbackConfig,
    pub knowledge_graph: KnowledgeGraphConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// URL Redis dla cache
    pub redis_url: String,
    /// URL Qdrant dla vector storage
    pub qdrant_url: String,
    /// Nazwa kolekcji w Qdrant
    pub collection_name: String,
    /// Rozmiar wektorów embeddings
    pub vector_size: usize,
    /// Maksymalna liczba wpisów w pamięci
    pub max_memory_entries: usize,
    /// TTL dla wpisów w sekundach
    pub ttl_seconds: u64,
    /// Rozmiar batch dla operacji
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Nazwa modelu embeddings
    pub model_name: String,
    /// API key dla OpenAI
    pub api_key: String,
    /// Rozmiar batch dla embeddings
    pub batch_size: usize,
    /// Maksymalna liczba tokenów
    pub max_tokens: usize,
    /// Timeout dla API calls
    pub timeout_seconds: u64,
    /// Czy używać lokalnych modeli
    pub use_local_models: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Czy włączyć TF-IDF weighting
    pub enable_tf_idf: bool,
    /// Czy włączyć clustering
    pub enable_clustering: bool,
    /// Czy włączyć deduplikację
    pub enable_deduplication: bool,
    /// Próg jakości kontekstu
    pub quality_threshold: f64,
    /// Próg relevance
    pub relevance_threshold: f64,
    /// Maksymalny rozmiar kontekstu
    pub max_context_size: usize,
    /// Czy shufflować haystack
    pub shuffle_haystack: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// Czy włączyć uczenie się
    pub enable_learning: bool,
    /// Learning rate
    pub learning_rate: f64,
    /// Okno czasowe dla feedback (godziny)
    pub feedback_window_hours: u64,
    /// Minimalna liczba próbek feedback
    pub min_feedback_samples: usize,
    /// Czy zapisywać feedback do pliku
    pub persist_feedback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphConfig {
    /// Czy włączyć knowledge graph
    pub enable_knowledge_graph: bool,
    /// Maksymalna liczba węzłów
    pub max_nodes: usize,
    /// Maksymalna liczba relacji
    pub max_relationships: usize,
    /// Próg pewności dla relacji
    pub relationship_confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Czy włączyć metryki
    pub metrics_enabled: bool,
    /// Port Prometheus
    pub prometheus_port: u16,
    /// Poziom logowania
    pub log_level: String,
    /// Interwał raportowania metryk
    pub metrics_interval_seconds: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            server: ServerConfig {
                port: env::var("CONTEXT_ENGINE_PORT")
                    .unwrap_or_else(|_| "8200".to_string())
                    .parse()?,
                host: env::var("CONTEXT_ENGINE_HOST")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                workers: env::var("CONTEXT_ENGINE_WORKERS")
                    .unwrap_or_else(|_| "4".to_string())
                    .parse()?,
            },
            memory: MemoryConfig {
                redis_url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                qdrant_url: env::var("QDRANT_URL")
                    .unwrap_or_else(|_| "http://localhost:6333".to_string()),
                collection_name: env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_| "context_memory".to_string()),
                vector_size: env::var("EMBEDDING_VECTOR_SIZE")
                    .unwrap_or_else(|_| "1536".to_string())
                    .parse()?,
                max_memory_entries: env::var("MAX_MEMORY_ENTRIES")
                    .unwrap_or_else(|_| "100000".to_string())
                    .parse()?,
                ttl_seconds: env::var("MEMORY_TTL_SECONDS")
                    .unwrap_or_else(|_| "86400".to_string())
                    .parse()?,
                batch_size: env::var("MEMORY_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
            },
            embeddings: EmbeddingConfig {
                model_name: env::var("EMBEDDING_MODEL")
                    .unwrap_or_else(|_| "text-embedding-ada-002".to_string()),
                api_key: env::var("OPENAI_API_KEY")
                    .unwrap_or_else(|_| "demo_key".to_string()),
                batch_size: env::var("EMBEDDING_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                max_tokens: env::var("EMBEDDING_MAX_TOKENS")
                    .unwrap_or_else(|_| "8192".to_string())
                    .parse()?,
                timeout_seconds: env::var("EMBEDDING_TIMEOUT_SECONDS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                use_local_models: env::var("USE_LOCAL_EMBEDDINGS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()?,
            },
            optimization: OptimizationConfig {
                enable_tf_idf: env::var("ENABLE_TF_IDF")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                enable_clustering: env::var("ENABLE_CLUSTERING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                enable_deduplication: env::var("ENABLE_DEDUPLICATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                quality_threshold: env::var("CONTEXT_QUALITY_THRESHOLD")
                    .unwrap_or_else(|_| "0.7".to_string())
                    .parse()?,
                relevance_threshold: env::var("RELEVANCE_THRESHOLD")
                    .unwrap_or_else(|_| "0.5".to_string())
                    .parse()?,
                max_context_size: env::var("MAX_CONTEXT_SIZE")
                    .unwrap_or_else(|_| "32768".to_string())
                    .parse()?,
                shuffle_haystack: env::var("SHUFFLE_HAYSTACK")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
            feedback: FeedbackConfig {
                enable_learning: env::var("ENABLE_LEARNING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                learning_rate: env::var("LEARNING_RATE")
                    .unwrap_or_else(|_| "0.01".to_string())
                    .parse()?,
                feedback_window_hours: env::var("FEEDBACK_WINDOW_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
                min_feedback_samples: env::var("MIN_FEEDBACK_SAMPLES")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                persist_feedback: env::var("PERSIST_FEEDBACK")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
            knowledge_graph: KnowledgeGraphConfig {
                enable_knowledge_graph: env::var("ENABLE_KNOWLEDGE_GRAPH")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                max_nodes: env::var("MAX_GRAPH_NODES")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()?,
                max_relationships: env::var("MAX_GRAPH_RELATIONSHIPS")
                    .unwrap_or_else(|_| "50000".to_string())
                    .parse()?,
                relationship_confidence_threshold: env::var("RELATIONSHIP_CONFIDENCE_THRESHOLD")
                    .unwrap_or_else(|_| "0.8".to_string())
                    .parse()?,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: env::var("CONTEXT_METRICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                prometheus_port: env::var("CONTEXT_PROMETHEUS_PORT")
                    .unwrap_or_else(|_| "9200".to_string())
                    .parse()?,
                log_level: env::var("CONTEXT_LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                metrics_interval_seconds: env::var("CONTEXT_METRICS_INTERVAL_S")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()?,
            },
        })
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate vector size
        if self.memory.vector_size == 0 {
            return Err(anyhow::anyhow!("Vector size must be positive"));
        }
        
        // Validate thresholds
        if self.optimization.quality_threshold < 0.0 || self.optimization.quality_threshold > 1.0 {
            return Err(anyhow::anyhow!("Quality threshold must be between 0.0 and 1.0"));
        }
        
        if self.optimization.relevance_threshold < 0.0 || self.optimization.relevance_threshold > 1.0 {
            return Err(anyhow::anyhow!("Relevance threshold must be between 0.0 and 1.0"));
        }
        
        // Validate learning rate
        if self.feedback.learning_rate <= 0.0 || self.feedback.learning_rate > 1.0 {
            return Err(anyhow::anyhow!("Learning rate must be between 0.0 and 1.0"));
        }
        
        // Validate batch sizes
        if self.memory.batch_size == 0 || self.embeddings.batch_size == 0 {
            return Err(anyhow::anyhow!("Batch sizes must be positive"));
        }
        
        Ok(())
    }
}
