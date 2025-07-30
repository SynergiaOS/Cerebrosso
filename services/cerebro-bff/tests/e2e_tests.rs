//! 🧪 End-to-End Tests for Cerberus Phoenix v2.0
//! 
//! Tests the complete flow from webhook to trading decision

use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

/// Test configuration for E2E tests
struct E2ETestConfig {
    cerebro_bff_url: String,
    hft_ninja_url: String,
    timeout: Duration,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            cerebro_bff_url: "http://localhost:3000".to_string(),
            hft_ninja_url: "http://localhost:8082".to_string(),
            timeout: Duration::from_secs(10),
        }
    }
}

/// Check if services are running
async fn check_services_health(config: &E2ETestConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Check Cerebro-BFF health
    let cerebro_response = client
        .get(&format!("{}/health", config.cerebro_bff_url))
        .timeout(config.timeout)
        .send()
        .await?;
    
    assert!(cerebro_response.status().is_success(), "Cerebro-BFF is not healthy");
    
    // Check HFT-Ninja health
    let ninja_response = client
        .get(&format!("{}/health", config.hft_ninja_url))
        .timeout(config.timeout)
        .send()
        .await?;
    
    assert!(ninja_response.status().is_success(), "HFT-Ninja is not healthy");
    
    Ok(())
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_services_health_check() {
    let config = E2ETestConfig::default();
    
    match check_services_health(&config).await {
        Ok(_) => println!("✅ All services are healthy"),
        Err(e) => {
            println!("⚠️ Services not running, skipping E2E test: {}", e);
            // Don't fail the test if services aren't running
            return;
        }
    }
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_full_webhook_to_decision_flow() {
    let config = E2ETestConfig::default();
    
    // Skip if services aren't running
    if check_services_health(&config).await.is_err() {
        println!("⚠️ Services not running, skipping E2E test");
        return;
    }
    
    let client = Client::new();
    
    // Step 1: Send webhook to HFT-Ninja (simulating Helius webhook)
    let webhook_payload = json!({
        "type": "TOKEN_TRANSFER",
        "data": {
            "mint": "E2ETestToken123456789",
            "amount": 1000000,
            "from": "11111111111111111111111111111112",
            "to": "22222222222222222222222222222222",
            "metadata": {
                "name": "E2E Test Memecoin",
                "symbol": "E2E",
                "decimals": 6,
                "supply": 1000000000,
                "volume_24h": 5000000,
                "holders": 1000
            }
        }
    });
    
    println!("🎣 Sending webhook to HFT-Ninja...");
    let webhook_response = client
        .post(&format!("{}/webhook/helius", config.hft_ninja_url))
        .json(&webhook_payload)
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send webhook");
    
    assert!(webhook_response.status().is_success(), "Webhook failed");
    
    let webhook_result: serde_json::Value = webhook_response
        .json()
        .await
        .expect("Failed to parse webhook response");
    
    println!("✅ Webhook processed: {:?}", webhook_result);
    
    // Step 2: Test direct analysis via Cerebro-BFF
    println!("🧠 Testing direct analysis...");
    let analysis_payload = json!({
        "mint": "E2ETestToken123456789",
        "metadata": {
            "name": "E2E Test Memecoin",
            "symbol": "E2E",
            "volume_24h": 5000000,
            "holders": 1000,
            "liquidity": 2000000
        }
    });
    
    let analysis_response = client
        .post(&format!("{}/api/analyze", config.cerebro_bff_url))
        .json(&analysis_payload)
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send analysis request");
    
    assert!(analysis_response.status().is_success(), "Analysis failed");
    
    let analysis_result: serde_json::Value = analysis_response
        .json()
        .await
        .expect("Failed to parse analysis response");
    
    println!("✅ Analysis completed: {:?}", analysis_result);
    
    // Verify analysis structure
    assert!(analysis_result.get("status").is_some());
    assert!(analysis_result.get("recommendation").is_some());
    assert!(analysis_result.get("confidence").is_some());
    
    // Step 3: Test trading decision
    println!("🤖 Testing trading decision...");
    let decision_payload = json!({
        "mint": "E2ETestToken123456789",
        "analysis": {
            "score": 0.85,
            "confidence": 0.9,
            "signals": ["high_volume", "growing_holders", "good_liquidity"]
        }
    });
    
    let decision_response = client
        .post(&format!("{}/api/decision", config.cerebro_bff_url))
        .json(&decision_payload)
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send decision request");
    
    assert!(decision_response.status().is_success(), "Decision failed");
    
    let decision_result: serde_json::Value = decision_response
        .json()
        .await
        .expect("Failed to parse decision response");
    
    println!("✅ Decision made: {:?}", decision_result);
    
    // Verify decision structure
    assert!(decision_result.get("decision").is_some());
    assert!(decision_result.get("amount").is_some());
    assert!(decision_result.get("confidence").is_some());
    assert!(decision_result.get("reasoning").is_some());
    
    println!("🎉 Full E2E flow completed successfully!");
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_high_volume_scenario() {
    let config = E2ETestConfig::default();
    
    // Skip if services aren't running
    if check_services_health(&config).await.is_err() {
        println!("⚠️ Services not running, skipping E2E test");
        return;
    }
    
    let client = Client::new();
    
    // Test high-volume scenario with multiple concurrent requests
    println!("🚀 Testing high-volume scenario...");
    
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let client_clone = client.clone();
        let config_clone = config.cerebro_bff_url.clone();
        
        let handle = tokio::spawn(async move {
            let payload = json!({
                "mint": format!("HighVolumeToken{}", i),
                "metadata": {
                    "name": format!("High Volume Token {}", i),
                    "symbol": "HVT",
                    "volume_24h": 10000000 + i * 1000000,
                    "holders": 2000 + i * 100
                }
            });
            
            let response = client_clone
                .post(&format!("{}/api/analyze", config_clone))
                .json(&payload)
                .timeout(Duration::from_secs(5))
                .send()
                .await
                .expect("Failed to send request");
            
            assert!(response.status().is_success());
            response.json::<serde_json::Value>().await.unwrap()
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.expect("Task failed");
        results.push(result);
    }
    
    assert_eq!(results.len(), 10);
    println!("✅ High-volume test completed: {} requests processed", results.len());
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_error_handling() {
    let config = E2ETestConfig::default();
    
    // Skip if services aren't running
    if check_services_health(&config).await.is_err() {
        println!("⚠️ Services not running, skipping E2E test");
        return;
    }
    
    let client = Client::new();
    
    // Test invalid request handling
    println!("🧪 Testing error handling...");
    
    // Test with invalid JSON
    let invalid_response = client
        .post(&format!("{}/api/analyze", config.cerebro_bff_url))
        .body("invalid json")
        .header("content-type", "application/json")
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send invalid request");
    
    assert!(invalid_response.status().is_client_error() || invalid_response.status().is_server_error());
    
    // Test with missing required fields
    let incomplete_payload = json!({
        "incomplete": "data"
    });
    
    let incomplete_response = client
        .post(&format!("{}/api/analyze", config.cerebro_bff_url))
        .json(&incomplete_payload)
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send incomplete request");
    
    // Should handle gracefully (either success with defaults or proper error)
    assert!(incomplete_response.status().is_success() || incomplete_response.status().is_client_error());
    
    println!("✅ Error handling test completed");
}

#[tokio::test]
#[ignore] // Requires running services
async fn test_performance_benchmarks() {
    let config = E2ETestConfig::default();
    
    // Skip if services aren't running
    if check_services_health(&config).await.is_err() {
        println!("⚠️ Services not running, skipping E2E test");
        return;
    }
    
    let client = Client::new();
    
    println!("⚡ Running performance benchmarks...");
    
    let test_payload = json!({
        "mint": "PerformanceTestToken",
        "metadata": {
            "name": "Performance Test Token",
            "symbol": "PERF",
            "volume_24h": 1000000,
            "holders": 500
        }
    });
    
    // Measure response time for analysis
    let start = std::time::Instant::now();
    
    let response = client
        .post(&format!("{}/api/analyze", config.cerebro_bff_url))
        .json(&test_payload)
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send performance test request");
    
    let duration = start.elapsed();
    
    assert!(response.status().is_success());
    
    println!("✅ Analysis response time: {:?}", duration);
    
    // Assert reasonable response time (adjust based on requirements)
    assert!(duration < Duration::from_millis(5000), "Response time too slow: {:?}", duration);
    
    // Test decision endpoint performance
    let decision_payload = json!({
        "mint": "PerformanceTestToken",
        "analysis": {
            "score": 0.8,
            "confidence": 0.85
        }
    });
    
    let start = std::time::Instant::now();
    
    let decision_response = client
        .post(&format!("{}/api/decision", config.cerebro_bff_url))
        .json(&decision_payload)
        .timeout(config.timeout)
        .send()
        .await
        .expect("Failed to send decision performance test");
    
    let decision_duration = start.elapsed();
    
    assert!(decision_response.status().is_success());
    
    println!("✅ Decision response time: {:?}", decision_duration);
    assert!(decision_duration < Duration::from_millis(3000), "Decision time too slow: {:?}", decision_duration);
}
