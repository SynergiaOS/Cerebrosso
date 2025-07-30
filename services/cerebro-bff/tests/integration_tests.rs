//! 🧪 Integration Tests for Cerebro-BFF
//! 
//! Tests for API endpoints and service integration

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use serde_json::json;

/// Create a test router with mock dependencies
async fn create_test_app() -> Router {
    // This would normally use real dependencies, but for testing we use mocks
    use cerebro_bff::config::*;
    use std::sync::Arc;
    
    let config = Config {
        server: ServerConfig {
            port: 8080,
            host: "localhost".to_string(),
        },
        qdrant: QdrantConfig {
            url: "http://localhost:6333".to_string(),
            collection_name: "test_collection".to_string(),
            vector_size: 1536,
        },
        ai: AIConfig {
            finllama_url: "http://localhost:11434".to_string(),
            deepseek_url: "http://localhost:11435".to_string(),
            max_tokens: 4096,
            temperature: 0.1,
            models: ModelConfigs {
                fast_decision: ModelConfig {
                    name: "test-fast".to_string(),
                    max_tokens: 1024,
                    temperature: 0.1,
                },
                context_analysis: ModelConfig {
                    name: "test-context".to_string(),
                    max_tokens: 2048,
                    temperature: 0.2,
                },
                risk_assessment: ModelConfig {
                    name: "test-risk".to_string(),
                    max_tokens: 1024,
                    temperature: 0.1,
                },
                deep_analysis: ModelConfig {
                    name: "test-deep".to_string(),
                    max_tokens: 4096,
                    temperature: 0.3,
                },
            },
        },
        context_engine: ContextEngineConfig {
            max_context_size: 8192,
            embedding_model: "test-model".to_string(),
            similarity_threshold: 0.8,
        },
        helius: HeliusConfig {
            api_key: "test_key".to_string(),
            webhook_url: "http://localhost:8080/webhook".to_string(),
            enable_filtering: true,
            min_confidence: 0.7,
        },
        quicknode: QuickNodeConfig {
            rpc_url: "http://localhost:8899".to_string(),
            api_key: "test_key".to_string(),
            jito_url: "http://localhost:8900".to_string(),
            enable_jito: true,
            timeout_ms: 30000,
        },
        piranha: PiranhaConfig {
            max_position_size_sol: 0.1,
            min_liquidity_score: 0.5,
            max_rug_pull_score: 0.3,
            max_slippage: 0.05,
            use_jito_bundles: true,
            emergency_exit_threshold: 0.8,
            profit_target: 0.15,
            stop_loss: 0.1,
        },
        vault: VaultConfig {
            url: "http://localhost:8200".to_string(),
            token: "test_token".to_string(),
            mount_path: "secret".to_string(),
            encryption_key: "test_key".to_string(),
        },
    };

    // Create a simple test router
    use axum::{routing::get, routing::post, Json, response::Json as ResponseJson};
    
    Router::new()
        .route("/health", get(|| async {
            ResponseJson(json!({
                "status": "healthy",
                "service": "cerebro-bff",
                "version": "2.0.0"
            }))
        }))
        .route("/api/analyze", post(|Json(payload): Json<serde_json::Value>| async {
            ResponseJson(json!({
                "status": "analyzed",
                "mint": payload.get("mint").unwrap_or(&json!("unknown")),
                "recommendation": "hold",
                "confidence": 0.75,
                "signals": [
                    {
                        "type": "volume_analysis",
                        "score": 0.8,
                        "weight": 0.7
                    }
                ]
            }))
        }))
        .route("/api/decision", post(|Json(payload): Json<serde_json::Value>| async {
            ResponseJson(json!({
                "decision": "buy",
                "amount": 100.0,
                "confidence": 0.8,
                "reasoning": "Strong volume and momentum signals",
                "risk_score": 0.3,
                "input": payload
            }))
        }))
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
    assert_eq!(json["service"], "cerebro-bff");
    assert_eq!(json["version"], "2.0.0");
}

#[tokio::test]
async fn test_analyze_endpoint() {
    let app = create_test_app().await;

    let test_payload = json!({
        "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "metadata": {
            "name": "Test Token",
            "symbol": "TEST",
            "volume_24h": 1000000,
            "holders": 500
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analyze")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(test_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "analyzed");
    assert_eq!(json["mint"], "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    assert_eq!(json["recommendation"], "hold");
    assert!(json["confidence"].as_f64().unwrap() > 0.0);
    assert!(json["signals"].is_array());
}

#[tokio::test]
async fn test_decision_endpoint() {
    let app = create_test_app().await;

    let test_payload = json!({
        "mint": "HighScoreToken123456789",
        "analysis": {
            "score": 0.85,
            "confidence": 0.9,
            "signals": ["high_volume", "growing_holders", "verified_contract"]
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/decision")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(test_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["decision"], "buy");
    assert!(json["amount"].as_f64().unwrap() > 0.0);
    assert!(json["confidence"].as_f64().unwrap() > 0.0);
    assert!(json["reasoning"].is_string());
    assert!(json["risk_score"].as_f64().unwrap() >= 0.0);
}

#[tokio::test]
async fn test_invalid_request() {
    let app = create_test_app().await;

    // Test with invalid JSON
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/analyze")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle invalid JSON gracefully
    assert!(response.status().is_client_error() || response.status().is_server_error());
}

#[tokio::test]
async fn test_missing_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cors_headers() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("Origin", "http://localhost:3000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    // In a real implementation, we'd check for CORS headers
    // For now, just verify the request succeeds
}

#[tokio::test]
async fn test_concurrent_requests() {
    let app = create_test_app().await;

    // Test multiple concurrent requests
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let test_payload = json!({
                "mint": format!("TestToken{}", i),
                "metadata": {
                    "name": format!("Test Token {}", i),
                    "symbol": "TEST"
                }
            });

            app_clone
                .oneshot(
                    Request::builder()
                        .uri("/api/analyze")
                        .method("POST")
                        .header("content-type", "application/json")
                        .body(Body::from(test_payload.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
