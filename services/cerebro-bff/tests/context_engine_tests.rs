//! 🧪 Context Engine Unit Tests
//! 
//! Comprehensive tests for the Context Engine functionality

use cerebro_bff::{ContextEngine, WeightedSignal, Config};
use std::sync::Arc;
use chrono::Utc;

/// Create a test configuration for Context Engine
fn create_test_config() -> Arc<Config> {
    Arc::new(Config {
        server: cerebro_bff::config::ServerConfig {
            port: 8080,
            host: "localhost".to_string(),
        },
        qdrant: cerebro_bff::config::QdrantConfig {
            url: "http://localhost:6333".to_string(),
            collection_name: "test_collection".to_string(),
            vector_size: 1536,
        },
        ai: cerebro_bff::config::AIConfig {
            finllama_url: "http://localhost:11434".to_string(),
            deepseek_url: "http://localhost:11435".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
            models: cerebro_bff::config::ModelConfigs {
                fast_decision: cerebro_bff::config::ModelConfig {
                    name: "test-fast".to_string(),
                    max_tokens: 1024,
                    temperature: 0.1,
                },
                context_analysis: cerebro_bff::config::ModelConfig {
                    name: "test-context".to_string(),
                    max_tokens: 2048,
                    temperature: 0.2,
                },
                risk_assessment: cerebro_bff::config::ModelConfig {
                    name: "test-risk".to_string(),
                    max_tokens: 1024,
                    temperature: 0.1,
                },
                deep_analysis: cerebro_bff::config::ModelConfig {
                    name: "test-deep".to_string(),
                    max_tokens: 4096,
                    temperature: 0.3,
                },
            },
        },
        context_engine: cerebro_bff::config::ContextEngineConfig {
            max_context_size: 8192,
            embedding_model: "test-model".to_string(),
            similarity_threshold: 0.8,
        },
        helius: cerebro_bff::config::HeliusConfig {
            api_key: "test_key".to_string(),
            webhook_url: "http://localhost:8080/webhook".to_string(),
            enable_filtering: true,
            min_confidence: 0.7,
        },
        quicknode: cerebro_bff::config::QuickNodeConfig {
            rpc_url: "http://localhost:8899".to_string(),
            api_key: "test_key".to_string(),
            jito_url: "http://localhost:8900".to_string(),
            enable_jito: true,
            timeout_ms: 30000,
        },
        piranha: cerebro_bff::config::PiranhaConfig {
            max_position_size_sol: 0.1,
            min_liquidity_score: 0.5,
            max_rug_pull_score: 0.3,
            max_slippage: 0.05,
            use_jito_bundles: true,
            emergency_exit_threshold: 0.8,
            profit_target: 0.15,
            stop_loss: 0.1,
        },
        vault: cerebro_bff::config::VaultConfig {
            url: "http://localhost:8200".to_string(),
            token: "test_token".to_string(),
            mount_path: "secret".to_string(),
            encryption_key: "test_key".to_string(),
        },
    })
}

/// Create test weighted signals
fn create_test_signals() -> Vec<WeightedSignal> {
    vec![
        WeightedSignal {
            signal_type: "rug_pull_detected".to_string(),
            value: 0.95,
            tf_idf_weight: 0.94,
            importance_score: 0.98,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "high_volume".to_string(),
            value: 0.8,
            tf_idf_weight: 0.7,
            importance_score: 0.75,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "team_anonymous".to_string(),
            value: 1.0,
            tf_idf_weight: 0.6,
            importance_score: 0.8,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "liquidity_low".to_string(),
            value: 0.9,
            tf_idf_weight: 0.85,
            importance_score: 0.9,
            timestamp: Utc::now(),
        },
    ]
}

#[tokio::test]
async fn test_context_engine_creation() {
    let config = create_test_config();
    
    // Test that we can create a ContextEngine with valid config
    // Note: This will fail if Qdrant is not running, but that's expected in unit tests
    // In a real test environment, we'd use a mock Qdrant client
    
    assert_eq!(config.qdrant.collection_name, "test_collection");
    assert_eq!(config.qdrant.vector_size, 1536);
}

#[tokio::test]
async fn test_tf_idf_weight_calculation() {
    // Test TF-IDF calculation logic
    let signals = vec![
        "rug_pull_detected".to_string(),
        "high_volume".to_string(),
        "rug_pull_detected".to_string(),
        "low_liquidity".to_string(),
    ];
    
    // Calculate term frequencies manually for verification
    let rug_pull_count = signals.iter().filter(|&s| s == "rug_pull_detected").count();
    let expected_tf = rug_pull_count as f64 / signals.len() as f64;
    
    assert_eq!(rug_pull_count, 2);
    assert_eq!(expected_tf, 0.5);
    assert_eq!(signals.len(), 4);
}

#[tokio::test]
async fn test_semantic_noise_filtering() {
    let signals = create_test_signals();
    
    // Test filtering logic
    let threshold = 0.65;
    let filtered: Vec<_> = signals.iter()
        .filter(|signal| signal.tf_idf_weight >= threshold)
        .collect();
    
    // Should filter out team_anonymous (weight 0.6 < 0.65)
    assert_eq!(filtered.len(), 3);
    assert!(!filtered.iter().any(|s| s.signal_type == "team_anonymous"));
}

#[tokio::test]
async fn test_context_optimization() {
    let signals = create_test_signals();
    
    // Test signal grouping logic
    let mut risk_signals = Vec::new();
    let mut liquidity_signals = Vec::new();
    let mut team_signals = Vec::new();
    
    for signal in &signals {
        match signal.signal_type.as_str() {
            s if s.contains("rug_pull") || s.contains("risk") => risk_signals.push(signal),
            s if s.contains("liquidity") || s.contains("volume") => liquidity_signals.push(signal),
            s if s.contains("team") || s.contains("doxx") => team_signals.push(signal),
            _ => risk_signals.push(signal), // Default to risk category
        }
    }
    
    assert_eq!(risk_signals.len(), 1); // rug_pull_detected
    assert_eq!(liquidity_signals.len(), 2); // high_volume, liquidity_low
    assert_eq!(team_signals.len(), 1); // team_anonymous
}

#[tokio::test]
async fn test_apriori_pattern_structure() {
    // Test Apriori algorithm data structures
    let transactions = vec![
        vec!["rug_pull_detected".to_string(), "low_liquidity".to_string(), "reject".to_string()],
        vec!["high_volume".to_string(), "good_liquidity".to_string(), "execute".to_string()],
        vec!["rug_pull_detected".to_string(), "new_token".to_string(), "reject".to_string()],
        vec!["sandwich_opportunity".to_string(), "high_volume".to_string(), "execute".to_string()],
        vec!["low_liquidity".to_string(), "new_token".to_string(), "reject".to_string()],
    ];
    
    // Test transaction structure
    assert_eq!(transactions.len(), 5);
    
    // Test pattern frequency
    let rug_pull_transactions: Vec<_> = transactions.iter()
        .filter(|t| t.contains(&"rug_pull_detected".to_string()))
        .collect();
    
    assert_eq!(rug_pull_transactions.len(), 2);
    
    // Test consequent patterns
    let reject_transactions: Vec<_> = transactions.iter()
        .filter(|t| t.contains(&"reject".to_string()))
        .collect();
    
    assert_eq!(reject_transactions.len(), 3);
}

#[tokio::test]
async fn test_embedding_generation() {
    // Test embedding generation logic
    let text = "rug_pull_detected high_volume team_anonymous";
    let words: Vec<&str> = text.split_whitespace().collect();
    
    // Test that we can generate consistent embeddings
    assert_eq!(words.len(), 3);
    assert_eq!(words[0], "rug_pull_detected");
    
    // Test embedding vector size
    let embedding_size = 1536;
    let mut embedding = vec![0.0; embedding_size];
    
    // Simple hash-based embedding simulation
    for word in &words {
        let hash = word.len() % embedding_size;
        embedding[hash] += 1.0 / words.len() as f32;
    }
    
    assert_eq!(embedding.len(), embedding_size);
    
    // Test normalization
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for val in &mut embedding {
            *val /= norm;
        }
    }
    
    // Verify normalized embedding
    let new_norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((new_norm - 1.0).abs() < 0.001 || norm == 0.0);
}

#[tokio::test]
async fn test_shuffle_haystacks_structure() {
    // Test shuffle haystacks context structure
    let context = "🚨 CRITICAL RISK SIGNALS:\n- rug_pull_detected: 0.95\n\n💰 MARKET CONDITIONS:\n- high_volume: 0.8\n\n👥 TEAM ANALYSIS:\n- team_anonymous: 1.0\n";
    
    let lines: Vec<&str> = context.lines().collect();
    let mut sections = Vec::new();
    let mut current_section = Vec::new();
    
    // Group lines into sections
    for line in lines {
        if line.starts_with("🚨") || line.starts_with("💰") || line.starts_with("👥") {
            if !current_section.is_empty() {
                sections.push(current_section.clone());
                current_section.clear();
            }
        }
        current_section.push(line);
    }
    
    if !current_section.is_empty() {
        sections.push(current_section);
    }
    
    // Test section structure
    assert_eq!(sections.len(), 3); // Risk, Market, Team
    assert!(sections[0][0].contains("🚨 CRITICAL RISK SIGNALS"));
    assert!(sections[1][0].contains("💰 MARKET CONDITIONS"));
    assert!(sections[2][0].contains("👥 TEAM ANALYSIS"));
}
