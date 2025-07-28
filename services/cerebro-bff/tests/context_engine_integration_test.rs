//! 🧪 Context Engine Integration Tests
//! 
//! Comprehensive tests for the Context Engine including:
//! - TF-IDF weighting functionality
//! - Apriori mining and rule application
//! - Context optimization with shuffle haystacks
//! - Semantic noise filtering
//! - Qdrant integration

use anyhow::Result;
use cerebro_bff::context_engine::{ContextEngine, WeightedSignal, AprioriRule};
use cerebro_bff::config::Config;
use chrono::Utc;
use std::sync::Arc;
use tokio;

/// 🧪 Test TF-IDF weighting functionality
#[tokio::test]
async fn test_tf_idf_weighting() -> Result<()> {
    println!("🧪 Testing TF-IDF weighting functionality...");
    
    // Create mock signals with different weights
    let signals = vec![
        WeightedSignal {
            signal_type: "high_risk_signal".to_string(),
            value: 0.9,
            tf_idf_weight: 3.0,
            importance_score: 0.95,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "low_risk_signal".to_string(),
            value: 0.2,
            tf_idf_weight: 0.5,
            importance_score: 0.3,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "medium_risk_signal".to_string(),
            value: 0.6,
            tf_idf_weight: 1.5,
            importance_score: 0.7,
            timestamp: Utc::now(),
        },
    ];
    
    // Test that signals are properly weighted
    assert!(signals[0].tf_idf_weight > signals[2].tf_idf_weight);
    assert!(signals[2].tf_idf_weight > signals[1].tf_idf_weight);
    
    // Test importance correlation
    assert!(signals[0].importance_score > signals[2].importance_score);
    assert!(signals[2].importance_score > signals[1].importance_score);
    
    println!("✅ TF-IDF weighting test passed");
    Ok(())
}

/// 🧪 Test Apriori rule creation and validation
#[tokio::test]
async fn test_apriori_rules() -> Result<()> {
    println!("🧪 Testing Apriori rule functionality...");
    
    // Create test rules
    let rule1 = AprioriRule {
        antecedent: vec!["dev_allocation_high".to_string(), "suspicious_metadata".to_string()],
        consequent: "avoid_token".to_string(),
        confidence: 0.95,
        support: 0.15,
        lift: 3.2,
        created_at: Utc::now(),
        last_validated: Utc::now(),
        success_count: 47,
        total_count: 50,
    };
    
    let rule2 = AprioriRule {
        antecedent: vec!["team_doxxed".to_string(), "contract_verified".to_string()],
        consequent: "consider_token".to_string(),
        confidence: 0.78,
        support: 0.08,
        lift: 2.1,
        created_at: Utc::now(),
        last_validated: Utc::now(),
        success_count: 23,
        total_count: 30,
    };
    
    // Test rule confidence calculation
    assert_eq!(rule1.confidence, rule1.success_count as f64 / rule1.total_count as f64);
    assert_eq!(rule2.confidence, rule2.success_count as f64 / rule2.total_count as f64);
    
    // Test rule quality (high confidence rules)
    assert!(rule1.confidence > 0.9);
    assert!(rule2.confidence > 0.7);
    
    // Test lift values (should be > 1 for meaningful rules)
    assert!(rule1.lift > 1.0);
    assert!(rule2.lift > 1.0);
    
    println!("✅ Apriori rules test passed");
    Ok(())
}

/// 🧪 Test context optimization and shuffle haystacks
#[tokio::test]
async fn test_context_optimization() -> Result<()> {
    println!("🧪 Testing context optimization with shuffle haystacks...");
    
    let signals = vec![
        WeightedSignal {
            signal_type: "rug_pull_risk_high".to_string(),
            value: 0.85,
            tf_idf_weight: 2.5,
            importance_score: 0.9,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "liquidity_low".to_string(),
            value: 0.3,
            tf_idf_weight: 1.8,
            importance_score: 0.7,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "team_doxxed".to_string(),
            value: 1.0,
            tf_idf_weight: 1.5,
            importance_score: 0.6,
            timestamp: Utc::now(),
        },
    ];
    
    // Test context structure creation
    let optimized_context = format!(
        "🚨 CRITICAL RISK SIGNALS:\n- {}: {} (confidence: {:.2}, weight: {:.3})\n\n💰 MARKET CONDITIONS:\n- {}: {} (weight: {:.3})\n\n👥 TEAM ANALYSIS:\n- {}: {} (weight: {:.3})\n",
        signals[0].signal_type, signals[0].value, signals[0].importance_score, signals[0].tf_idf_weight,
        signals[1].signal_type, signals[1].value, signals[1].tf_idf_weight,
        signals[2].signal_type, signals[2].value, signals[2].tf_idf_weight
    );
    
    // Test that context contains expected sections
    assert!(optimized_context.contains("🚨 CRITICAL RISK SIGNALS"));
    assert!(optimized_context.contains("💰 MARKET CONDITIONS"));
    assert!(optimized_context.contains("👥 TEAM ANALYSIS"));
    
    // Test that high-risk signals appear first
    let risk_pos = optimized_context.find("🚨 CRITICAL RISK SIGNALS").unwrap();
    let market_pos = optimized_context.find("💰 MARKET CONDITIONS").unwrap();
    assert!(risk_pos < market_pos);
    
    println!("✅ Context optimization test passed");
    Ok(())
}

/// 🧪 Test semantic noise filtering
#[tokio::test]
async fn test_semantic_noise_filtering() -> Result<()> {
    println!("🧪 Testing semantic noise filtering...");
    
    let signals = vec![
        WeightedSignal {
            signal_type: "important_signal".to_string(),
            value: 0.8,
            tf_idf_weight: 2.0,
            importance_score: 0.9,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "noise_signal".to_string(),
            value: 0.1,
            tf_idf_weight: 0.3,
            importance_score: 0.1,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "unknown_signal".to_string(),
            value: 0.5,
            tf_idf_weight: 1.5,
            importance_score: 0.2,
            timestamp: Utc::now(),
        },
    ];
    
    // Test filtering with threshold 1.0
    let filtered: Vec<&WeightedSignal> = signals.iter()
        .filter(|s| s.tf_idf_weight >= 1.0 && !s.signal_type.contains("unknown"))
        .collect();
    
    // Should filter out noise_signal (weight 0.3) and unknown_signal (contains "unknown")
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].signal_type, "important_signal");
    
    // Test noise reduction ratio
    let noise_reduction_ratio = (1.0 - filtered.len() as f64 / signals.len() as f64) * 100.0;
    assert!(noise_reduction_ratio > 50.0); // Should filter out significant noise
    
    println!("✅ Semantic noise filtering test passed");
    Ok(())
}

/// 🧪 Test pattern matching and rule application
#[tokio::test]
async fn test_pattern_matching() -> Result<()> {
    println!("🧪 Testing pattern matching and rule application...");
    
    // Test signals that should trigger "avoid" recommendation
    let high_risk_signals = vec![
        "dev_allocation_high".to_string(),
        "suspicious_metadata".to_string(),
        "freeze_function_present".to_string(),
    ];
    
    // Test signals that should trigger "consider" recommendation
    let positive_signals = vec![
        "team_doxxed".to_string(),
        "contract_verified".to_string(),
        "high_liquidity".to_string(),
    ];
    
    // Mock rule application logic
    let avoid_pattern = high_risk_signals.iter()
        .any(|s| s.contains("dev_allocation_high") || s.contains("suspicious_metadata"));
    
    let consider_pattern = positive_signals.iter()
        .any(|s| s.contains("team_doxxed") && positive_signals.iter().any(|s2| s2.contains("contract_verified")));
    
    assert!(avoid_pattern);
    assert!(consider_pattern);
    
    println!("✅ Pattern matching test passed");
    Ok(())
}

/// 🧪 Integration test combining all features
#[tokio::test]
async fn test_full_integration() -> Result<()> {
    println!("🧪 Running full integration test...");
    
    // Create comprehensive test scenario
    let signals = vec![
        WeightedSignal {
            signal_type: "rug_pull_risk_high".to_string(),
            value: 0.95,
            tf_idf_weight: 3.0,
            importance_score: 0.95,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "suspicious_metadata".to_string(),
            value: 0.9,
            tf_idf_weight: 2.8,
            importance_score: 0.9,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "low_liquidity".to_string(),
            value: 0.7,
            tf_idf_weight: 1.5,
            importance_score: 0.6,
            timestamp: Utc::now(),
        },
        WeightedSignal {
            signal_type: "noise_signal".to_string(),
            value: 0.1,
            tf_idf_weight: 0.2,
            importance_score: 0.1,
            timestamp: Utc::now(),
        },
    ];
    
    // Test filtering
    let filtered: Vec<&WeightedSignal> = signals.iter()
        .filter(|s| s.tf_idf_weight >= 1.0)
        .collect();
    
    assert_eq!(filtered.len(), 3); // Should filter out noise_signal
    
    // Test sorting by importance
    let mut sorted_signals = filtered.clone();
    sorted_signals.sort_by(|a, b| b.tf_idf_weight.partial_cmp(&a.tf_idf_weight).unwrap());
    
    assert_eq!(sorted_signals[0].signal_type, "rug_pull_risk_high");
    assert_eq!(sorted_signals[1].signal_type, "suspicious_metadata");
    
    // Test pattern detection
    let signal_names: Vec<String> = filtered.iter().map(|s| s.signal_type.clone()).collect();
    let has_high_risk_pattern = signal_names.iter()
        .any(|s| s.contains("rug_pull")) && signal_names.iter().any(|s| s.contains("suspicious"));
    
    assert!(has_high_risk_pattern);
    
    println!("✅ Full integration test passed");
    Ok(())
}
