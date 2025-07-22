//! 🔧 Configuration for Cerebro BFF

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub hft_ninja: HftNinjaConfig,
    pub qdrant: QdrantConfig,
    pub ai: AIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HftNinjaConfig {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection_name: String,
    pub vector_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub confidence_threshold: f32,
    pub max_risk_score: f32,
    pub learning_enabled: bool,
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config = Config {
            server: ServerConfig {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()?,
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            },
            hft_ninja: HftNinjaConfig {
                url: env::var("HFT_NINJA_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
                timeout_ms: env::var("HFT_NINJA_TIMEOUT_MS")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()?,
            },
            qdrant: QdrantConfig {
                url: env::var("QDRANT_URL")
                    .unwrap_or_else(|_| "http://localhost:6333".to_string()),
                collection_name: env::var("QDRANT_COLLECTION")
                    .unwrap_or_else(|_| "trading_signals".to_string()),
                vector_size: env::var("QDRANT_VECTOR_SIZE")
                    .unwrap_or_else(|_| "384".to_string())
                    .parse()?,
            },
            ai: AIConfig {
                confidence_threshold: env::var("AI_CONFIDENCE_THRESHOLD")
                    .unwrap_or_else(|_| "0.7".to_string())
                    .parse()?,
                max_risk_score: env::var("AI_MAX_RISK_SCORE")
                    .unwrap_or_else(|_| "0.3".to_string())
                    .parse()?,
                learning_enabled: env::var("AI_LEARNING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
        };

        Ok(config)
    }
}
