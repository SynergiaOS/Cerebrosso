//! 💾 Memory Store - Advanced Memory Management
//! 
//! Multi-level memory system with Redis cache and Qdrant vector storage

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use redis::{Client as RedisClient, AsyncCommands};
use qdrant_client::{client::QdrantClient, qdrant::*};
use tracing::{info, debug, instrument};

use crate::config::Config;

/// 💾 Typ pamięci
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Pamięć robocza (RAM) - milisekundy
    Working,
    /// Pamięć krótkoterminowa (Redis) - minuty/godziny
    ShortTerm,
    /// Pamięć długoterminowa (Qdrant) - dni/tygodnie
    LongTerm,
}

/// 📊 Poziom pamięci
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryLevel {
    /// Poziom roboczy
    Working,
    /// Poziom operacyjny
    Operational,
    /// Poziom strategiczny
    Strategic,
    /// Poziom historyczny
    Historical,
}

/// 📝 Wpis w pamięci
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unikalny identyfikator
    pub id: Uuid,
    /// Zawartość
    pub content: String,
    /// Wektor embeddings
    pub embedding: Vec<f32>,
    /// Typ pamięci
    pub memory_type: MemoryType,
    /// Poziom pamięci
    pub memory_level: MemoryLevel,
    /// Czas utworzenia
    pub created_at: DateTime<Utc>,
    /// Ostatni dostęp
    pub accessed_at: DateTime<Utc>,
    /// Liczba dostępów
    pub access_count: u32,
    /// Wynik relevance
    pub relevance_score: f64,
    /// Wynik jakości
    pub quality_score: f64,
    /// Metadane
    pub metadata: HashMap<String, String>,
}

impl MemoryEntry {
    /// Tworzy nowy wpis
    pub fn new(
        content: String,
        embedding: Vec<f32>,
        memory_type: MemoryType,
        memory_level: MemoryLevel,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            content,
            embedding,
            memory_type,
            memory_level,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            relevance_score: 0.5,
            quality_score: 0.5,
            metadata: HashMap::new(),
        }
    }
    
    /// Aktualizuje dostęp
    pub fn update_access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Utc::now();
    }
}

/// 💾 Magazyn pamięci
pub struct MemoryStore {
    /// Konfiguracja
    config: Arc<Config>,
    /// Klient Redis
    redis_client: RedisClient,
    /// Klient Qdrant
    qdrant_client: QdrantClient,
    /// Pamięć robocza
    working_memory: Arc<tokio::sync::RwLock<HashMap<Uuid, MemoryEntry>>>,
}

impl MemoryStore {
    /// Tworzy nowy magazyn pamięci
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("💾 Initializing MemoryStore...");
        
        // Redis connection
        let redis_client = RedisClient::open(config.memory.redis_url.as_str())?;
        let mut conn = redis_client.get_async_connection().await?;
        let _: String = conn.ping().await?;
        
        // Qdrant connection
        let qdrant_client = QdrantClient::from_url(&config.memory.qdrant_url).build()?;
        
        // Create collection if not exists
        let collection_name = &config.memory.collection_name;
        if !qdrant_client.collection_exists(collection_name).await? {
            info!("🗄️ Creating Qdrant collection: {}", collection_name);
            
            let create_collection = CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(VectorParams {
                    size: config.memory.vector_size as u64,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                }.into()),
                ..Default::default()
            };
            
            qdrant_client.create_collection(&create_collection).await?;
        }
        
        info!("✅ MemoryStore initialized");
        
        Ok(Self {
            config,
            redis_client,
            qdrant_client,
            working_memory: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }
    
    /// Zapisuje wpis w pamięci
    #[instrument(skip(self, entry))]
    pub async fn store(&self, entry: MemoryEntry) -> Result<()> {
        debug!("💾 Storing memory entry: {} (type: {:?})", entry.id, entry.memory_type);
        
        match entry.memory_type {
            MemoryType::Working => {
                self.store_in_working_memory(entry).await?;
            }
            MemoryType::ShortTerm => {
                self.store_in_redis(entry).await?;
            }
            MemoryType::LongTerm => {
                self.store_in_qdrant(entry).await?;
            }
        }
        
        Ok(())
    }
    
    /// Pobiera wpis z pamięci
    #[instrument(skip(self))]
    pub async fn retrieve(&self, id: Uuid) -> Result<Option<MemoryEntry>> {
        debug!("🔍 Retrieving memory entry: {}", id);
        
        // Check working memory first
        if let Some(entry) = self.retrieve_from_working_memory(id).await? {
            return Ok(Some(entry));
        }
        
        // Check Redis
        if let Some(entry) = self.retrieve_from_redis(id).await? {
            return Ok(Some(entry));
        }
        
        // Check Qdrant
        if let Some(entry) = self.retrieve_from_qdrant(id).await? {
            return Ok(Some(entry));
        }
        
        Ok(None)
    }
    
    /// Wyszukuje podobne wpisy
    #[instrument(skip(self, query_embedding))]
    pub async fn search_similar(
        &self,
        query_embedding: Vec<f32>,
        limit: usize,
        threshold: f64,
    ) -> Result<Vec<MemoryEntry>> {
        debug!("🔍 Searching similar entries: limit={}, threshold={}", limit, threshold);
        
        let collection_name = &self.config.memory.collection_name;
        
        let search_points = SearchPoints {
            collection_name: collection_name.clone(),
            vector: query_embedding,
            limit: limit as u64,
            score_threshold: Some(threshold as f32),
            with_payload: Some(true.into()),
            ..Default::default()
        };
        
        let search_result = self.qdrant_client.search_points(&search_points).await?;
        
        let mut entries = Vec::new();
        for scored_point in search_result.result {
            if let Some(payload) = scored_point.payload {
                if let Some(entry_json) = payload.get("entry") {
                    if let Ok(entry) = serde_json::from_str::<MemoryEntry>(&entry_json.to_string()) {
                        entries.push(entry);
                    }
                }
            }
        }
        
        debug!("✅ Found {} similar entries", entries.len());
        Ok(entries)
    }
    
    /// Zapisuje w pamięci roboczej
    async fn store_in_working_memory(&self, entry: MemoryEntry) -> Result<()> {
        let mut working_memory = self.working_memory.write().await;
        working_memory.insert(entry.id, entry);
        Ok(())
    }
    
    /// Zapisuje w Redis
    async fn store_in_redis(&self, entry: MemoryEntry) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let entry_json = serde_json::to_string(&entry)?;
        
        let _: () = conn.setex(
            entry.id.to_string(),
            self.config.memory.ttl_seconds as usize,
            entry_json
        ).await?;
        
        Ok(())
    }
    
    /// Zapisuje w Qdrant
    async fn store_in_qdrant(&self, entry: MemoryEntry) -> Result<()> {
        let collection_name = &self.config.memory.collection_name;
        
        let mut payload = HashMap::new();
        payload.insert("entry".to_string(), serde_json::to_value(&entry)?.into());
        payload.insert("content".to_string(), entry.content.clone().into());
        payload.insert("memory_level".to_string(), format!("{:?}", entry.memory_level).into());
        
        let point = PointStruct::new(
            entry.id.to_string(),
            entry.embedding.clone(),
            payload,
        );
        
        let upsert_points = UpsertPoints {
            collection_name: collection_name.clone(),
            points: vec![point],
            ..Default::default()
        };
        
        self.qdrant_client.upsert_points(upsert_points).await?;
        Ok(())
    }
    
    /// Pobiera z pamięci roboczej
    async fn retrieve_from_working_memory(&self, id: Uuid) -> Result<Option<MemoryEntry>> {
        let working_memory = self.working_memory.read().await;
        Ok(working_memory.get(&id).cloned())
    }
    
    /// Pobiera z Redis
    async fn retrieve_from_redis(&self, id: Uuid) -> Result<Option<MemoryEntry>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let entry_json: Option<String> = conn.get(id.to_string()).await?;
        
        if let Some(json) = entry_json {
            let entry: MemoryEntry = serde_json::from_str(&json)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
    
    /// Pobiera z Qdrant
    async fn retrieve_from_qdrant(&self, id: Uuid) -> Result<Option<MemoryEntry>> {
        let collection_name = &self.config.memory.collection_name;
        
        let get_points = GetPoints {
            collection_name: collection_name.clone(),
            ids: vec![id.to_string().into()],
            with_payload: Some(true.into()),
            with_vectors: Some(true.into()),
        };
        
        let get_result = self.qdrant_client.get_points(get_points).await?;
        
        if let Some(point) = get_result.result.first() {
            if let Some(payload) = &point.payload {
                if let Some(entry_json) = payload.get("entry") {
                    if let Ok(entry) = serde_json::from_str::<MemoryEntry>(&entry_json.to_string()) {
                        return Ok(Some(entry));
                    }
                }
            }
        }
        
        Ok(None)
    }
}
