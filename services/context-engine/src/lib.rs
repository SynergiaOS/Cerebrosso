//! 🧠 Context Engine (CEM) Library
//! 
//! Advanced Context Engine with dynamic memory system, Qdrant integration, and AI-powered learning

pub mod config;
pub mod context_engine;
pub mod memory_store;
pub mod embedding_service;
pub mod feedback_loop;
pub mod context_optimizer;
pub mod knowledge_graph;
pub mod pattern_recognition;
pub mod semantic_search;
pub mod metrics;
pub mod synk_integration;
pub mod chainguardia_integration;

// Core exports
pub use config::Config;
pub use context_engine::{ContextEngine, ContextState, ContextError};
pub use memory_store::{MemoryStore, MemoryEntry, MemoryType, MemoryLevel};
pub use embedding_service::{EmbeddingService, EmbeddingModel, EmbeddingVector};
pub use feedback_loop::{FeedbackLoop, FeedbackData, LearningMetrics};
pub use context_optimizer::{ContextOptimizer, OptimizationStrategy, ContextQuality};
pub use knowledge_graph::{KnowledgeGraph, Entity, Relationship, GraphQuery};
pub use pattern_recognition::{PatternRecognizer, Pattern, PatternType, PatternConfidence};
pub use semantic_search::{SemanticSearchEngine, SearchQuery, SearchResult, Relevance};
pub use metrics::{ContextMetrics, PerformanceMetrics, QualityMetrics};
pub use synk_integration::{SynkIntegration, SynkIntegrationState, NetworkStateData};
pub use chainguardia_integration::{GuardiaIntegration, GuardiaIntegrationState, SecurityAlertData, SecurityStatusData};

/// 🎯 Core Context Engine Result Type
pub type ContextResult<T> = Result<T, ContextError>;

/// 🧪 Test utilities for Context Engine
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8200,
                host: "localhost".to_string(),
            },
            memory: config::MemoryConfig {
                redis_url: "redis://localhost:6379".to_string(),
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "test_context".to_string(),
                vector_size: 1536,
                max_memory_entries: 10000,
                ttl_seconds: 3600,
            },
            embeddings: config::EmbeddingConfig {
                model_name: "text-embedding-ada-002".to_string(),
                api_key: "test_key".to_string(),
                batch_size: 100,
                max_tokens: 8192,
            },
            optimization: config::OptimizationConfig {
                enable_tf_idf: true,
                enable_clustering: true,
                enable_deduplication: true,
                quality_threshold: 0.7,
                relevance_threshold: 0.5,
            },
            feedback: config::FeedbackConfig {
                enable_learning: true,
                learning_rate: 0.01,
                feedback_window_hours: 24,
                min_feedback_samples: 10,
            },
        })
    }
    
    /// Create a mock memory entry
    pub fn create_mock_memory_entry() -> MemoryEntry {
        MemoryEntry {
            id: uuid::Uuid::new_v4(),
            content: "Test memory content".to_string(),
            embedding: vec![0.1; 1536],
            memory_type: MemoryType::ShortTerm,
            memory_level: MemoryLevel::Working,
            created_at: chrono::Utc::now(),
            accessed_at: chrono::Utc::now(),
            access_count: 1,
            relevance_score: 0.8,
            quality_score: 0.9,
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_context_engine_initialization() {
        let config = create_test_config();
        
        // Test that config is created correctly
        assert_eq!(config.server.port, 8200);
        assert_eq!(config.memory.vector_size, 1536);
        assert_eq!(config.embeddings.model_name, "text-embedding-ada-002");
    }
    
    #[tokio::test]
    async fn test_memory_entry_creation() {
        let entry = create_mock_memory_entry();
        
        // Test memory entry structure
        assert_eq!(entry.content, "Test memory content");
        assert_eq!(entry.embedding.len(), 1536);
        assert_eq!(entry.memory_type, MemoryType::ShortTerm);
        assert_eq!(entry.access_count, 1);
    }
    
    #[tokio::test]
    async fn test_context_optimization() {
        let config = create_test_config();
        
        // Test optimization configuration
        assert!(config.optimization.enable_tf_idf);
        assert!(config.optimization.enable_clustering);
        assert_eq!(config.optimization.quality_threshold, 0.7);
    }
}

/// 🎯 Context Engine Constants
pub mod constants {
    /// Maximum context window size
    pub const MAX_CONTEXT_WINDOW: usize = 32768;
    
    /// Default embedding dimension
    pub const DEFAULT_EMBEDDING_DIM: usize = 1536;
    
    /// Maximum memory entries per level
    pub const MAX_MEMORY_ENTRIES: usize = 100000;
    
    /// Context quality threshold
    pub const CONTEXT_QUALITY_THRESHOLD: f64 = 0.7;
    
    /// Relevance score threshold
    pub const RELEVANCE_THRESHOLD: f64 = 0.5;
    
    /// Learning rate for feedback loop
    pub const DEFAULT_LEARNING_RATE: f64 = 0.01;
    
    /// Memory retention periods (in hours)
    pub const WORKING_MEMORY_TTL: i64 = 1;
    pub const SHORT_TERM_MEMORY_TTL: i64 = 24;
    pub const LONG_TERM_MEMORY_TTL: i64 = 720; // 30 days
    
    /// Pattern recognition thresholds
    pub const PATTERN_CONFIDENCE_THRESHOLD: f64 = 0.8;
    pub const MIN_PATTERN_OCCURRENCES: usize = 5;
    
    /// Semantic search parameters
    pub const DEFAULT_SEARCH_LIMIT: usize = 10;
    pub const SEMANTIC_SIMILARITY_THRESHOLD: f64 = 0.6;
}
