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
    pub helius: HeliusConfig,
    pub quicknode: QuickNodeConfig,
    pub piranha: PiranhaConfig,
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
    pub models: ModelConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigs {
    pub fast_decision: ModelConfig,
    pub context_analysis: ModelConfig,
    pub risk_assessment: ModelConfig,
    pub deep_analysis: ModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub url: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub target_latency_ms: u32,
    pub enable_kv_cache: bool,
    pub quantization: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEngineConfig {
    pub max_context_size: u32,
    pub embedding_model: String,
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeliusConfig {
    pub api_key: String,
    pub base_url: String,
    pub enable_filtering: bool,
    pub min_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickNodeConfig {
    pub rpc_url: String,
    pub api_key: String,
    pub jito_url: String,
    pub enable_jito: bool,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiranhaConfig {
    pub max_position_size_sol: f64,
    pub min_liquidity_score: f64,
    pub max_rug_pull_score: f64,
    pub max_slippage: f64,
    pub use_jito_bundles: bool,
    pub emergency_exit_threshold: f64,
    pub profit_target: f64,
    pub stop_loss: f64,
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
                    .unwrap_or_else(|_| "http://finllama:11434".to_string()),
                deepseek_url: env::var("DEEPSEEK_API_URL")
                    .unwrap_or_else(|_| "http://deepseek:11434".to_string()),
                max_tokens: 4096,
                temperature: 0.1,
                models: ModelConfigs {
                    fast_decision: ModelConfig {
                        name: "phi3".to_string(),
                        url: env::var("FINLLAMA_API_URL")
                            .unwrap_or_else(|_| "http://finllama:11434".to_string()),
                        max_tokens: 1024,
                        temperature: 0.3,
                        target_latency_ms: 20,
                        enable_kv_cache: true,
                        quantization: "4bit".to_string(),
                    },
                    context_analysis: ModelConfig {
                        name: "llama3:8b-instruct".to_string(),
                        url: env::var("FINLLAMA_API_URL")
                            .unwrap_or_else(|_| "http://finllama:11434".to_string()),
                        max_tokens: 2048,
                        temperature: 0.5,
                        target_latency_ms: 50,
                        enable_kv_cache: true,
                        quantization: "4bit".to_string(),
                    },
                    risk_assessment: ModelConfig {
                        name: "mistral:small".to_string(),
                        url: env::var("DEEPSEEK_API_URL")
                            .unwrap_or_else(|_| "http://deepseek:11434".to_string()),
                        max_tokens: 1536,
                        temperature: 0.4,
                        target_latency_ms: 30,
                        enable_kv_cache: true,
                        quantization: "4bit".to_string(),
                    },
                    deep_analysis: ModelConfig {
                        name: "llama3:70b-instruct".to_string(),
                        url: env::var("FINLLAMA_API_URL")
                            .unwrap_or_else(|_| "http://finllama:11434".to_string()),
                        max_tokens: 4096,
                        temperature: 0.6,
                        target_latency_ms: 200,
                        enable_kv_cache: true,
                        quantization: "4bit".to_string(),
                    },
                },
            },
            context_engine: ContextEngineConfig {
                max_context_size: 8192,
                embedding_model: "text-embedding-ada-002".to_string(),
                similarity_threshold: 0.8,
            },
            helius: HeliusConfig {
                api_key: env::var("HELIUS_API_KEY")
                    .expect("HELIUS_API_KEY environment variable is required"),
                base_url: env::var("HELIUS_BASE_URL")
                    .unwrap_or_else(|_| "https://api.helius.xyz".to_string()),
                enable_filtering: true,
                min_confidence: 0.7,
            },
            quicknode: QuickNodeConfig {
                rpc_url: env::var("QUICKNODE_RPC_URL")
                    .expect("QUICKNODE_RPC_URL environment variable is required"),
                api_key: env::var("QUICKNODE_API_KEY")
                    .expect("QUICKNODE_API_KEY environment variable is required"),
                jito_url: env::var("JITO_URL")
                    .unwrap_or_else(|_| "https://mainnet.block-engine.jito.wtf".to_string()),
                enable_jito: env::var("ENABLE_JITO")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                timeout_ms: 30000,
            },
            piranha: PiranhaConfig {
                max_position_size_sol: env::var("MAX_POSITION_SIZE_SOL")
                    .unwrap_or_else(|_| "0.1".to_string())
                    .parse()
                    .unwrap_or(0.1),
                min_liquidity_score: 0.5,
                max_rug_pull_score: 0.3,
                max_slippage: 0.05,
                use_jito_bundles: true,
                emergency_exit_threshold: 0.8,
                profit_target: 0.15,
                stop_loss: 0.1,
            },
        })
    }
}
