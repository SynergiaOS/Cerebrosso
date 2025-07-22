//! 🔧 Konfiguracja Cerebro-BFF

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub qdrant: QdrantConfig,
    pub ai: AIConfig,
    pub context_engine: ContextEngineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub vector_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub finllama_url: String,
    pub deepseek_url: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEngineConfig {
    pub max_context_size: u32,
    pub embedding_model: String,
    pub similarity_threshold: f32,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Config {
            server: ServerConfig {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            },
            qdrant: QdrantConfig {
                url: env::var("QDRANT_URL")
                    .unwrap_or_else(|_| "http://qdrant:6333".to_string()),
                collection_name: env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_| "cerberus_context".to_string()),
                vector_size: 1536, // OpenAI embedding size
            },
            ai: AIConfig {
                finllama_url: env::var("FINLLAMA_API_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string()),
                deepseek_url: env::var("DEEPSEEK_API_URL")
                    .unwrap_or_else(|_| "http://localhost:11435".to_string()),
                max_tokens: 4096,
                temperature: 0.1,
            },
            context_engine: ContextEngineConfig {
                max_context_size: 8192,
                embedding_model: "text-embedding-ada-002".to_string(),
                similarity_threshold: 0.8,
            },
        })
    }
}
