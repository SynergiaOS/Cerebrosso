//! 🧠 Embedding Service - AI Embeddings Generation
//! 
//! Service for generating embeddings using various AI models

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_openai::{Client as OpenAIClient, types::*};
use tracing::{info, debug, instrument};

use crate::config::Config;

/// 🤖 Model embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// OpenAI text-embedding-ada-002
    OpenAIAda002,
    /// OpenAI text-embedding-3-small
    OpenAI3Small,
    /// OpenAI text-embedding-3-large
    OpenAI3Large,
    /// Local model
    Local(String),
}

/// 📊 Wektor embeddings
pub type EmbeddingVector = Vec<f32>;

/// 📊 Wynik embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResult {
    /// Wektor embeddings
    pub vector: EmbeddingVector,
    /// Model użyty do generacji
    pub model: EmbeddingModel,
    /// Liczba tokenów
    pub token_count: usize,
    /// Czas generacji
    pub generation_time_ms: u64,
}

/// 🧠 Serwis embeddings
pub struct EmbeddingService {
    /// Konfiguracja
    config: Arc<Config>,
    /// Klient OpenAI
    openai_client: Option<OpenAIClient<async_openai::config::OpenAIConfig>>,
    /// Cache embeddings
    embedding_cache: Arc<tokio::sync::RwLock<HashMap<String, EmbeddingResult>>>,
}

impl EmbeddingService {
    /// Tworzy nowy serwis embeddings
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🧠 Initializing EmbeddingService...");
        
        let openai_client = if !config.embeddings.use_local_models {
            let openai_config = async_openai::config::OpenAIConfig::new()
                .with_api_key(&config.embeddings.api_key);
            Some(OpenAIClient::with_config(openai_config))
        } else {
            None
        };
        
        info!("✅ EmbeddingService initialized");
        
        Ok(Self {
            config,
            openai_client,
            embedding_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }
    
    /// Generuje embedding dla tekstu
    #[instrument(skip(self))]
    pub async fn generate_embedding(&self, text: &str) -> Result<EmbeddingVector> {
        debug!("🧠 Generating embedding for text: {} chars", text.len());
        
        // Sprawdź cache
        let cache_key = self.generate_cache_key(text);
        if let Some(cached_result) = self.get_cached_embedding(&cache_key).await {
            debug!("📋 Returning cached embedding");
            return Ok(cached_result.vector);
        }
        
        let start_time = std::time::Instant::now();
        
        // Generuj embedding
        let result = if let Some(client) = &self.openai_client {
            self.generate_openai_embedding(client, text).await?
        } else {
            self.generate_local_embedding(text).await?
        };
        
        let generation_time = start_time.elapsed().as_millis() as u64;
        
        let embedding_result = EmbeddingResult {
            vector: result.clone(),
            model: EmbeddingModel::OpenAIAda002,
            token_count: text.len() / 4, // Rough estimate
            generation_time_ms: generation_time,
        };
        
        // Zapisz w cache
        self.cache_embedding(&cache_key, &embedding_result).await;
        
        debug!("✅ Embedding generated: {} dimensions", result.len());
        Ok(result)
    }
    
    /// Generuje embeddings dla wielu tekstów
    #[instrument(skip(self, texts))]
    pub async fn generate_batch_embeddings(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        debug!("🧠 Generating batch embeddings for {} texts", texts.len());
        
        let mut results = Vec::new();
        
        // Process in batches
        for chunk in texts.chunks(self.config.embeddings.batch_size) {
            for text in chunk {
                let embedding = self.generate_embedding(text).await?;
                results.push(embedding);
            }
        }
        
        debug!("✅ Batch embeddings generated: {} results", results.len());
        Ok(results)
    }
    
    /// Oblicza podobieństwo między wektorami
    pub fn calculate_similarity(&self, vec1: &EmbeddingVector, vec2: &EmbeddingVector) -> f64 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }
        
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }
        
        (dot_product / (norm1 * norm2)) as f64
    }
    
    /// Generuje embedding przez OpenAI
    async fn generate_openai_embedding(
        &self,
        client: &OpenAIClient<async_openai::config::OpenAIConfig>,
        text: &str,
    ) -> Result<EmbeddingVector> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.config.embeddings.model_name)
            .input(text)
            .build()?;
        
        let response = client.embeddings().create(request).await?;
        
        if let Some(embedding_data) = response.data.first() {
            Ok(embedding_data.embedding.clone())
        } else {
            Err(anyhow::anyhow!("No embedding data received"))
        }
    }
    
    /// Generuje embedding lokalnie
    async fn generate_local_embedding(&self, _text: &str) -> Result<EmbeddingVector> {
        // Placeholder for local embedding generation
        // In a real implementation, this would use a local model
        Ok(vec![0.0; self.config.memory.vector_size])
    }
    
    /// Generuje klucz cache
    fn generate_cache_key(&self, text: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Pobiera embedding z cache
    async fn get_cached_embedding(&self, cache_key: &str) -> Option<EmbeddingResult> {
        let cache = self.embedding_cache.read().await;
        cache.get(cache_key).cloned()
    }
    
    /// Zapisuje embedding w cache
    async fn cache_embedding(&self, cache_key: &str, result: &EmbeddingResult) {
        let mut cache = self.embedding_cache.write().await;
        cache.insert(cache_key.to_string(), result.clone());
        
        // Limit cache size
        if cache.len() > 10000 {
            let oldest_key = cache.keys().next().unwrap().clone();
            cache.remove(&oldest_key);
        }
    }
}
