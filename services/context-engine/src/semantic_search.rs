//! 🔍 Semantic Search Engine

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::{config::Config, memory_store::MemoryStore, embedding_service::EmbeddingVector, context_engine::ContextType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub embedding: EmbeddingVector,
    pub limit: usize,
    pub threshold: f64,
    pub context_type: Option<ContextType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub content: String,
    pub score: f64,
    pub confidence: f64,
    pub source_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relevance {
    pub score: f64,
    pub explanation: String,
}

pub struct SemanticSearchEngine {
    config: Arc<Config>,
    memory_store: Arc<MemoryStore>,
}

impl SemanticSearchEngine {
    pub async fn new(config: Arc<Config>, memory_store: Arc<MemoryStore>) -> Result<Self> {
        Ok(Self { config, memory_store })
    }
    
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let memory_entries = self.memory_store
            .search_similar(query.embedding, query.limit, query.threshold)
            .await?;
        
        let results = memory_entries.into_iter().map(|entry| SearchResult {
            id: entry.id,
            content: entry.content,
            score: entry.relevance_score,
            confidence: entry.quality_score,
            source_id: entry.id.to_string(),
        }).collect();
        
        Ok(results)
    }
}
