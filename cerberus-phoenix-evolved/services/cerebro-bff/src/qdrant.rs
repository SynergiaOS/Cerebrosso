//! 🔍 Qdrant vector database client for signal storage and retrieval

use crate::config::Config;
use anyhow::Result;
// Temporarily disabled for MVP
// use cerberus_core_types::Signal;
// Temporarily using HTTP client instead of gRPC
// use qdrant_client::{
//     prelude::*,
//     qdrant::{
//         point_id::PointIdOptions, vectors::VectorsOptions, CreateCollection, Distance, PointStruct, SearchPoints, UpsertPoints,
//         VectorParams, VectorsConfig,
//     },
// };
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

// Temporary Signal struct for MVP (replace with cerberus-core-types later)
#[derive(Debug, Clone)]
pub struct Signal {
    pub id: Uuid,
    pub source: SignalSource,
    pub signal_type: SignalType,
    pub token: String,
    pub pool_address: String,
    pub confidence: f32,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum SignalSource {
    Mempool,
    Dex,
    Social,
    Technical,
    Arbitrage,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    NewListing,
    LiquidityAdd,
    LargeTransaction,
    PriceMovement,
    VolumeSpike,
    ArbitrageOpportunity,
}

pub struct QdrantClient {
    http_client: reqwest::Client,
    base_url: String,
    config: Arc<Config>,
}

impl QdrantClient {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🔍 Connecting to Qdrant at {} (HTTP mode)", config.qdrant.url);

        let http_client = reqwest::Client::new();
        let base_url = config.qdrant.url.clone();

        let qdrant_client = Self {
            http_client,
            base_url,
            config
        };

        // Initialize collection if it doesn't exist
        qdrant_client.ensure_collection().await?;

        Ok(qdrant_client)
    }

    async fn ensure_collection(&self) -> Result<()> {
        let collection_name = &self.config.qdrant.collection_name;

        // Check if collection exists via HTTP
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        match self.http_client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("✅ Collection '{}' already exists", collection_name);
                Ok(())
            }
            _ => {
                info!("📦 Creating collection '{}'", collection_name);
                self.create_collection().await
            }
        }
    }

    async fn create_collection(&self) -> Result<()> {
        let collection_name = &self.config.qdrant.collection_name;

        let create_payload = serde_json::json!({
            "vectors": {
                "size": self.config.qdrant.vector_size,
                "distance": "Cosine"
            }
        });

        let url = format!("{}/collections/{}", self.base_url, collection_name);
        let response = self.http_client
            .put(&url)
            .json(&create_payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("✅ Collection '{}' created successfully", collection_name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create collection: {}", error_text);
        }
    }

    pub async fn store_signal(&self, signal: &Signal) -> Result<()> {
        info!("💾 Storing signal {} in Qdrant (HTTP)", signal.id);

        // Convert signal to vector (simplified embedding)
        let vector = self.signal_to_vector(signal);

        // Create point payload for HTTP API
        let point_payload = serde_json::json!({
            "points": [{
                "id": signal.id.to_string(),
                "vector": vector,
                "payload": {
                    "signal_id": signal.id.to_string(),
                    "source": format!("{:?}", signal.source),
                    "signal_type": format!("{:?}", signal.signal_type),
                    "token": signal.token.to_string(),
                    "pool_address": signal.pool_address.to_string(),
                    "confidence": signal.confidence,
                    "timestamp": signal.timestamp.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default().as_secs(),
                }
            }]
        });

        let url = format!("{}/collections/{}/points", self.base_url, self.config.qdrant.collection_name);
        let response = self.http_client
            .put(&url)
            .json(&point_payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("✅ Signal {} stored successfully", signal.id);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            warn!("⚠️ Failed to store signal: {}", error_text);
            Ok(()) // Don't fail the whole system for storage issues
        }
    }

    pub async fn search_similar_signals(&self, signal: &Signal, limit: u64) -> Result<Vec<String>> {
        info!("🔍 Searching for similar signals to {} (simplified)", signal.id);

        // For MVP, return empty results - search functionality can be added later
        info!("✅ Found 0 similar signals (search not implemented in HTTP mode)");
        Ok(vec![])
    }

    fn signal_to_vector(&self, signal: &Signal) -> Vec<f32> {
        // Simplified signal embedding - in production, use proper ML embeddings
        let mut vector = vec![0.0; self.config.qdrant.vector_size];

        // Encode signal type
        let signal_type_idx = match signal.signal_type {
            SignalType::NewListing => 0,
            SignalType::LiquidityAdd => 1,
            SignalType::LargeTransaction => 2,
            SignalType::PriceMovement => 3,
            SignalType::VolumeSpike => 4,
            SignalType::ArbitrageOpportunity => 5,
        };
        if signal_type_idx < vector.len() {
            vector[signal_type_idx] = 1.0;
        }

        // Encode source
        let source_idx = match signal.source {
            SignalSource::Mempool => 10,
            SignalSource::Dex => 11,
            SignalSource::Social => 12,
            SignalSource::Technical => 13,
            SignalSource::Arbitrage => 14,
        };
        if source_idx < vector.len() {
            vector[source_idx] = 1.0;
        }

        // Encode confidence
        if vector.len() > 20 {
            vector[20] = signal.confidence;
        }

        // Add some randomness for diversity (in production, use proper features)
        for i in 21..vector.len().min(50) {
            vector[i] = (signal.id.as_u128() as f32 * (i as f32 + 1.0)).sin() * 0.1;
        }

        vector
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/collections", self.base_url);
        match self.http_client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                info!("✅ Qdrant health check passed (HTTP)");
                Ok(true)
            }
            Ok(response) => {
                error!("❌ Qdrant health check failed: HTTP {}", response.status());
                Ok(false)
            }
            Err(e) => {
                error!("❌ Qdrant health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
