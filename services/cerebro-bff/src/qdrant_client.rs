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

    /// 🎯 Analyze token risk using TF-IDF and historical patterns
    pub async fn analyze_token_risk(&self, token_address: &str, metadata: &serde_json::Value) -> Result<TokenRiskAnalysis> {
        info!("🎯 Analyzing risk for token: {}", token_address);

        // Extract features for TF-IDF analysis
        let features = self.extract_token_features(metadata)?;

        // Search for similar tokens in Qdrant
        let similar_tokens = self.find_similar_tokens(&features, 50).await?;

        // Calculate risk scores
        let rug_pull_score = self.calculate_rug_pull_risk(&features, &similar_tokens)?;
        let liquidity_risk = self.calculate_liquidity_risk(&features)?;
        let team_risk = self.calculate_team_risk(&features)?;
        let contract_risk = self.calculate_contract_risk(&features)?;

        // Aggregate final risk score
        let overall_risk = (rug_pull_score * 0.4 + liquidity_risk * 0.3 + team_risk * 0.2 + contract_risk * 0.1).min(1.0);

        let analysis = TokenRiskAnalysis {
            token_address: token_address.to_string(),
            overall_risk_score: overall_risk,
            rug_pull_probability: rug_pull_score,
            liquidity_risk_score: liquidity_risk,
            team_risk_score: team_risk,
            contract_risk_score: contract_risk,
            risk_factors: self.identify_risk_factors(&features, overall_risk),
            confidence_level: self.calculate_confidence(&similar_tokens),
            analyzed_at: chrono::Utc::now(),
        };

        info!("✅ Risk analysis complete: {:.2}% risk", overall_risk * 100.0);
        Ok(analysis)
    }

    /// 📊 Extract TF-IDF features from token metadata
    fn extract_token_features(&self, metadata: &serde_json::Value) -> Result<TokenFeatures> {
        let name = metadata.get("name").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        let symbol = metadata.get("symbol").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        let description = metadata.get("description").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();

        // TF-IDF keywords extraction
        let high_risk_keywords = vec![
            "moon", "pump", "gem", "100x", "1000x", "rocket", "lambo", "diamond", "hands",
            "hodl", "ape", "yolo", "degen", "safe", "rug", "pull", "scam", "honeypot"
        ];

        let mut keyword_scores = std::collections::HashMap::new();
        let combined_text = format!("{} {} {}", name, symbol, description);

        for keyword in &high_risk_keywords {
            let count = combined_text.matches(keyword).count();
            if count > 0 {
                keyword_scores.insert(keyword.to_string(), count as f64);
            }
        }

        Ok(TokenFeatures {
            name: name.clone(),
            symbol: symbol.clone(),
            description: description.clone(),
            keyword_scores,
            holder_count: metadata.get("holder_count").and_then(|v| v.as_u64()).unwrap_or(0),
            liquidity_usd: metadata.get("liquidity_usd").and_then(|v| v.as_f64()).unwrap_or(0.0),
            volume_24h: metadata.get("volume_24h").and_then(|v| v.as_f64()).unwrap_or(0.0),
            market_cap: metadata.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0),
            is_verified: metadata.get("is_verified").and_then(|v| v.as_bool()).unwrap_or(false),
            team_doxxed: metadata.get("team_doxxed").and_then(|v| v.as_bool()).unwrap_or(false),
        })
    }

    /// 🔍 Find similar tokens using vector search
    async fn find_similar_tokens(&self, features: &TokenFeatures, limit: u32) -> Result<Vec<QdrantSearchResult>> {
        // Create feature vector from token characteristics
        let mut feature_vector = vec![0.0; 100]; // 100-dimensional feature space

        // Encode basic metrics
        feature_vector[0] = (features.holder_count as f32).ln_1p() / 10.0; // Log-normalized holder count
        feature_vector[1] = (features.liquidity_usd as f32).ln_1p() / 20.0; // Log-normalized liquidity
        feature_vector[2] = (features.volume_24h as f32).ln_1p() / 20.0; // Log-normalized volume
        feature_vector[3] = if features.is_verified { 1.0 } else { 0.0 };
        feature_vector[4] = if features.team_doxxed { 1.0 } else { 0.0 };

        // Encode keyword presence (TF-IDF style)
        let mut idx = 5;
        for (_keyword, score) in &features.keyword_scores {
            if idx < 50 {
                feature_vector[idx] = (*score as f32).min(5.0) / 5.0; // Normalized keyword score
                idx += 1;
            }
        }

        // Search for similar vectors
        self.search(feature_vector, limit).await
    }

    /// 🚨 Calculate rug pull probability
    fn calculate_rug_pull_risk(&self, features: &TokenFeatures, similar_tokens: &[QdrantSearchResult]) -> Result<f64> {
        let mut risk_score: f64 = 0.0;

        // High-risk keywords
        let high_risk_keywords = ["moon", "pump", "gem", "100x", "1000x", "safe"];
        for keyword in &high_risk_keywords {
            if features.keyword_scores.contains_key(*keyword) {
                risk_score += 0.15;
            }
        }

        // Low liquidity risk
        if features.liquidity_usd < 10000.0 {
            risk_score += 0.3;
        } else if features.liquidity_usd < 50000.0 {
            risk_score += 0.15;
        }

        // Low holder count
        if features.holder_count < 100 {
            risk_score += 0.25;
        } else if features.holder_count < 500 {
            risk_score += 0.1;
        }

        // Unverified team
        if !features.team_doxxed {
            risk_score += 0.2;
        }

        // Historical pattern analysis from similar tokens
        let mut rug_pull_count = 0;
        for similar in similar_tokens {
            if let Some(is_rug) = similar.payload.get("is_rug_pull").and_then(|v| v.as_bool()) {
                if is_rug {
                    rug_pull_count += 1;
                }
            }
        }

        if !similar_tokens.is_empty() {
            let historical_rug_rate = rug_pull_count as f64 / similar_tokens.len() as f64;
            risk_score += historical_rug_rate * 0.3;
        }

        Ok(risk_score.min(1.0))
    }

    /// 💧 Calculate liquidity risk
    fn calculate_liquidity_risk(&self, features: &TokenFeatures) -> Result<f64> {
        let mut risk_score: f64 = 0.0;

        // Very low liquidity
        if features.liquidity_usd < 5000.0 {
            risk_score += 0.8;
        } else if features.liquidity_usd < 25000.0 {
            risk_score += 0.5;
        } else if features.liquidity_usd < 100000.0 {
            risk_score += 0.2;
        }

        // Volume to liquidity ratio
        if features.liquidity_usd > 0.0 {
            let volume_ratio = features.volume_24h / features.liquidity_usd;
            if volume_ratio > 5.0 {
                risk_score += 0.3; // High volume relative to liquidity
            }
        }

        Ok(risk_score.min(1.0))
    }

    /// 👥 Calculate team risk
    fn calculate_team_risk(&self, features: &TokenFeatures) -> Result<f64> {
        let mut risk_score: f64 = 0.0;

        if !features.team_doxxed {
            risk_score += 0.6;
        }

        if !features.is_verified {
            risk_score += 0.4;
        }

        Ok(risk_score.min(1.0))
    }

    /// 📜 Calculate contract risk
    fn calculate_contract_risk(&self, features: &TokenFeatures) -> Result<f64> {
        let mut risk_score: f64 = 0.0;

        // Contract verification
        if !features.is_verified {
            risk_score += 0.5;
        }

        // Suspicious keywords in contract/description
        let suspicious_keywords = ["honeypot", "tax", "fee", "burn", "mint"];
        for keyword in &suspicious_keywords {
            if features.description.contains(keyword) {
                risk_score += 0.2;
            }
        }

        Ok(risk_score.min(1.0))
    }

    /// 🚩 Identify specific risk factors
    fn identify_risk_factors(&self, features: &TokenFeatures, overall_risk: f64) -> Vec<String> {
        let mut factors = Vec::new();

        if features.liquidity_usd < 10000.0 {
            factors.push("Low liquidity (<$10k)".to_string());
        }

        if features.holder_count < 100 {
            factors.push("Very few holders (<100)".to_string());
        }

        if !features.team_doxxed {
            factors.push("Anonymous team".to_string());
        }

        if !features.is_verified {
            factors.push("Unverified contract".to_string());
        }

        if features.keyword_scores.contains_key("moon") || features.keyword_scores.contains_key("pump") {
            factors.push("Pump-related keywords detected".to_string());
        }

        if overall_risk > 0.8 {
            factors.push("CRITICAL: Extremely high risk profile".to_string());
        } else if overall_risk > 0.6 {
            factors.push("HIGH: Multiple risk indicators".to_string());
        }

        factors
    }

    /// 📊 Calculate confidence level
    fn calculate_confidence(&self, similar_tokens: &[QdrantSearchResult]) -> f64 {
        if similar_tokens.is_empty() {
            return 0.3; // Low confidence with no historical data
        }

        let avg_score: f64 = similar_tokens.iter()
            .map(|t| t.score as f64)
            .sum::<f64>() / similar_tokens.len() as f64;

        // Higher similarity scores = higher confidence
        (avg_score * 2.0).min(0.95)
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

/// 🎯 Token risk analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRiskAnalysis {
    pub token_address: String,
    pub overall_risk_score: f64,
    pub rug_pull_probability: f64,
    pub liquidity_risk_score: f64,
    pub team_risk_score: f64,
    pub contract_risk_score: f64,
    pub risk_factors: Vec<String>,
    pub confidence_level: f64,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
}

/// 📊 Token features for TF-IDF analysis
#[derive(Debug, Clone)]
pub struct TokenFeatures {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub keyword_scores: std::collections::HashMap<String, f64>,
    pub holder_count: u64,
    pub liquidity_usd: f64,
    pub volume_24h: f64,
    pub market_cap: f64,
    pub is_verified: bool,
    pub team_doxxed: bool,
}
