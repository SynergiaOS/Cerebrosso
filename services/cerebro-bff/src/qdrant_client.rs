//! 🗄️ Qdrant Vector Database Client for Cerberus Phoenix v2.0
//!
//! High-performance vector database client for AI context storage and retrieval

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;
use tracing::{info, warn, debug};
use uuid::Uuid;

/// 🗄️ Qdrant client for vector operations
pub struct QdrantClient {
    client: Client,
    base_url: String,
    collection_name: String,
}

impl QdrantClient {
    /// 🚀 Create new Qdrant client
    pub async fn new(base_url: &str) -> Result<Self> {
        let client = Client::new();
        let collection_name = "cerberus_context".to_string();

        let qdrant_client = Self {
            client,
            base_url: base_url.to_string(),
            collection_name: collection_name.clone(),
        };

        // Initialize collection if it doesn't exist
        qdrant_client.ensure_collection_exists().await?;

        info!("✅ Qdrant client initialized for collection: {}", collection_name);
        Ok(qdrant_client)
    }

    /// 🏥 Health check
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/", self.base_url);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            debug!("✅ Qdrant health check passed");
            Ok(())
        } else {
            Err(anyhow!("Qdrant health check failed: {}", response.status()))
        }
    }

    /// 🗄️ Ensure collection exists
    async fn ensure_collection_exists(&self) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, self.collection_name);
        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            debug!("✅ Collection {} already exists", self.collection_name);
            return Ok(());
        }

        // Create collection
        self.create_collection(1536).await?;
        Ok(())
    }

    /// 🏗️ Create collection with specified vector size
    pub async fn create_collection(&self, vector_size: u64) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, self.collection_name);

        let payload = json!({
            "vectors": {
                "size": vector_size,
                "distance": "Cosine"
            },
            "optimizers_config": {
                "default_segment_number": 2
            },
            "replication_factor": 1
        });

        let response = self.client
            .put(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("🗄️ Created collection {} with vector size {}", self.collection_name, vector_size);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Failed to create collection: {}", error_text))
        }
    }

    /// 📝 Upsert points to collection
    pub async fn upsert_points(&self, points: Vec<QdrantPoint>) -> Result<()> {
        if points.is_empty() {
            return Ok(());
        }

        let url = format!("{}/collections/{}/points", self.base_url, self.collection_name);

        let payload = json!({
            "points": points
        });

        let response = self.client
            .put(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("📝 Upserted {} points to collection", points.len());
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Failed to upsert points: {}", error_text))
        }
    }

    /// 🔍 Search for similar vectors
    pub async fn search(&self, vector: Vec<f32>, limit: u32) -> Result<Vec<QdrantSearchResult>> {
        let url = format!("{}/collections/{}/points/search", self.base_url, self.collection_name);

        let payload = json!({
            "vector": vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let search_response: QdrantSearchResponse = response.json().await?;
            debug!("🔍 Found {} similar vectors", search_response.result.len());
            Ok(search_response.result)
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Search failed: {}", error_text))
        }
    }

    /// 🗑️ Delete points by filter
    pub async fn delete_points(&self, filter: serde_json::Value) -> Result<()> {
        let url = format!("{}/collections/{}/points/delete", self.base_url, self.collection_name);

        let payload = json!({
            "filter": filter
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("🗑️ Deleted points with filter");
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Failed to delete points: {}", error_text))
        }
    }
}

/// 📍 Qdrant point for vector storage
#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: serde_json::Value,
}

/// 🔍 Qdrant search result
#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantSearchResult {
    pub id: String,
    pub score: f32,
    pub payload: serde_json::Value,
}

/// 📊 Qdrant search response wrapper
#[derive(Debug, Deserialize)]
struct QdrantSearchResponse {
    result: Vec<QdrantSearchResult>,
}

impl QdrantPoint {
    /// 🆕 Create new point with auto-generated ID
    pub fn new(vector: Vec<f32>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            vector,
            payload,
        }
    }

    /// 🆔 Create new point with specific ID
    pub fn with_id(id: String, vector: Vec<f32>, payload: serde_json::Value) -> Self {
        Self {
            id,
            vector,
            payload,
        }
    }
}
