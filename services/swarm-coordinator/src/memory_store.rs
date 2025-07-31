//! 💾 Memory Store - Zarządzanie pamięcią systemu
//! 
//! System pamięci krótkoterminowej i długoterminowej dla Hive Mind

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use redis::{Client as RedisClient, AsyncCommands};
use qdrant_client::{
    client::QdrantClient,
    qdrant::{
        CreateCollection, Distance, VectorParams, CollectionOperationResponse,
        PointStruct, UpsertPoints, SearchPoints, Filter, Condition, FieldCondition,
        Range, MatchValue,
    },
};
use tracing::{info, debug, warn, error, instrument};

use crate::{
    config::Config,
    task_delegation::TaskResult,
    constants::MEMORY_RETENTION_DAYS,
};

/// 🧠 Typ pamięci
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Pamięć krótkoterminowa (Redis) - sekundy/minuty
    ShortTerm,
    /// Pamięć długoterminowa (Qdrant) - dni/tygodnie
    LongTerm,
    /// Pamięć robocza (RAM) - milisekundy/sekundy
    Working,
}

/// 📝 Wpis w pamięci
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unikalny identyfikator
    pub id: Uuid,
    /// Typ pamięci
    pub memory_type: MemoryType,
    /// Kategoria wpisu
    pub category: String,
    /// Klucz wpisu
    pub key: String,
    /// Zawartość
    pub content: Value,
    /// Wektor embeddings (dla Qdrant)
    pub embedding: Option<Vec<f32>>,
    /// Metadane
    pub metadata: HashMap<String, String>,
    /// Czas utworzenia
    pub created_at: DateTime<Utc>,
    /// Czas wygaśnięcia
    pub expires_at: Option<DateTime<Utc>>,
    /// Liczba dostępów
    pub access_count: u32,
    /// Ostatni dostęp
    pub last_accessed: DateTime<Utc>,
    /// Waga ważności (0.0 - 1.0)
    pub importance_weight: f64,
}

impl MemoryEntry {
    /// Tworzy nowy wpis w pamięci
    pub fn new(
        memory_type: MemoryType,
        category: String,
        key: String,
        content: Value,
    ) -> Self {
        let now = Utc::now();
        let expires_at = match memory_type {
            MemoryType::Working => Some(now + Duration::minutes(5)),
            MemoryType::ShortTerm => Some(now + Duration::hours(24)),
            MemoryType::LongTerm => Some(now + Duration::days(MEMORY_RETENTION_DAYS)),
        };
        
        Self {
            id: Uuid::new_v4(),
            memory_type,
            category,
            key,
            content,
            embedding: None,
            metadata: HashMap::new(),
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            importance_weight: 0.5,
        }
    }
    
    /// Sprawdza czy wpis wygasł
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Aktualizuje dostęp do wpisu
    pub fn update_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
        
        // Zwiększ wagę ważności na podstawie częstotliwości dostępu
        let frequency_boost = (self.access_count as f64).ln() * 0.1;
        self.importance_weight = (self.importance_weight + frequency_boost).min(1.0);
    }
}

/// 💾 Magazyn pamięci
pub struct MemoryStore {
    /// Konfiguracja
    config: Arc<Config>,
    /// Klient Redis (pamięć krótkoterminowa)
    redis_client: RedisClient,
    /// Klient Qdrant (pamięć długoterminowa)
    qdrant_client: QdrantClient,
    /// Pamięć robocza (RAM)
    working_memory: Arc<tokio::sync::RwLock<HashMap<String, MemoryEntry>>>,
    /// Statystyki pamięci
    stats: Arc<tokio::sync::RwLock<MemoryStats>>,
}

/// 📊 Statystyki pamięci
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: u64,
    pub working_memory_entries: u32,
    pub short_term_entries: u64,
    pub long_term_entries: u64,
    pub total_access_count: u64,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: f64,
    pub average_retrieval_time_ms: f64,
}

impl MemoryStore {
    /// Tworzy nowy magazyn pamięci
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("💾 Initializing MemoryStore...");
        
        // Połączenie z Redis
        let redis_client = RedisClient::open(config.redis.url.as_str())?;
        let mut conn = redis_client.get_async_connection().await?;
        let _: String = conn.ping().await?;
        
        // Połączenie z Qdrant
        let qdrant_client = QdrantClient::from_url(&config.qdrant.url).build()?;
        
        // Sprawdź/utwórz kolekcję w Qdrant
        let collection_name = &config.qdrant.collection_name;
        if !qdrant_client.collection_exists(collection_name).await? {
            info!("🗄️ Creating Qdrant collection: {}", collection_name);
            
            let create_collection = CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(VectorParams {
                    size: config.qdrant.vector_size,
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
            stats: Arc::new(tokio::sync::RwLock::new(MemoryStats::default())),
        })
    }
    
    /// Zapisuje wpis w pamięci
    #[instrument(skip(self, entry))]
    pub async fn store(&self, mut entry: MemoryEntry) -> Result<()> {
        let entry_id = entry.id;
        let memory_type = entry.memory_type.clone();
        
        debug!("💾 Storing memory entry: {} (type: {:?})", entry_id, memory_type);
        
        match memory_type {
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
        
        // Aktualizuj statystyki
        {
            let mut stats = self.stats.write().await;
            stats.total_entries += 1;
            match memory_type {
                MemoryType::Working => stats.working_memory_entries += 1,
                MemoryType::ShortTerm => stats.short_term_entries += 1,
                MemoryType::LongTerm => stats.long_term_entries += 1,
            }
        }
        
        debug!("✅ Memory entry stored: {}", entry_id);
        Ok(())
    }
    
    /// Pobiera wpis z pamięci
    #[instrument(skip(self))]
    pub async fn retrieve(&self, key: &str, category: Option<&str>) -> Result<Option<MemoryEntry>> {
        debug!("🔍 Retrieving memory entry: key={}, category={:?}", key, category);
        
        let start_time = std::time::Instant::now();
        
        // Sprawdź pamięć roboczą
        if let Some(mut entry) = self.retrieve_from_working_memory(key).await? {
            entry.update_access();
            self.update_access_stats(start_time.elapsed().as_millis() as f64, true).await;
            return Ok(Some(entry));
        }
        
        // Sprawdź Redis (pamięć krótkoterminowa)
        if let Some(mut entry) = self.retrieve_from_redis(key).await? {
            entry.update_access();
            
            // Przenieś do pamięci roboczej dla szybszego dostępu
            self.store_in_working_memory(entry.clone()).await?;
            
            self.update_access_stats(start_time.elapsed().as_millis() as f64, true).await;
            return Ok(Some(entry));
        }
        
        // Sprawdź Qdrant (pamięć długoterminowa)
        if let Some(mut entry) = self.retrieve_from_qdrant(key, category).await? {
            entry.update_access();
            
            // Przenieś do Redis dla szybszego dostępu
            self.store_in_redis(entry.clone()).await?;
            
            self.update_access_stats(start_time.elapsed().as_millis() as f64, true).await;
            return Ok(Some(entry));
        }
        
        self.update_access_stats(start_time.elapsed().as_millis() as f64, false).await;
        debug!("❌ Memory entry not found: {}", key);
        Ok(None)
    }
    
    /// Wyszukuje podobne wpisy w pamięci długoterminowej
    #[instrument(skip(self, query_embedding))]
    pub async fn search_similar(
        &self,
        query_embedding: Vec<f32>,
        limit: u32,
        category: Option<&str>,
    ) -> Result<Vec<MemoryEntry>> {
        debug!("🔍 Searching similar entries: limit={}, category={:?}", limit, category);
        
        let collection_name = &self.config.qdrant.collection_name;
        
        let mut search_points = SearchPoints {
            collection_name: collection_name.clone(),
            vector: query_embedding,
            limit: limit as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        };
        
        // Dodaj filtr kategorii jeśli podano
        if let Some(cat) = category {
            search_points.filter = Some(Filter {
                must: vec![Condition {
                    condition_one_of: Some(
                        FieldCondition {
                            key: "category".to_string(),
                            match_: Some(MatchValue {
                                match_value: Some(cat.into()),
                            }.into()),
                            ..Default::default()
                        }.into()
                    ),
                }],
                ..Default::default()
            });
        }
        
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
    
    /// Zapisuje wynik zadania w pamięci
    #[instrument(skip(self, result))]
    pub async fn store_task_result(&self, task_id: Uuid, result: &TaskResult) -> Result<()> {
        debug!("💾 Storing task result: {}", task_id);
        
        let entry = MemoryEntry::new(
            MemoryType::LongTerm,
            "task_results".to_string(),
            task_id.to_string(),
            serde_json::to_value(result)?,
        );
        
        self.store(entry).await?;
        
        debug!("✅ Task result stored: {}", task_id);
        Ok(())
    }
    
    /// Czyści wygasłe wpisy
    #[instrument(skip(self))]
    pub async fn cleanup_expired_entries(&self) -> Result<u32> {
        debug!("🧹 Cleaning up expired entries...");
        
        let mut cleaned_count = 0;
        
        // Wyczyść pamięć roboczą
        {
            let mut working_memory = self.working_memory.write().await;
            let initial_count = working_memory.len();
            working_memory.retain(|_, entry| !entry.is_expired());
            cleaned_count += (initial_count - working_memory.len()) as u32;
        }
        
        // Redis automatycznie usuwa wygasłe klucze (TTL)
        // Qdrant wymaga manualnego czyszczenia - można dodać w przyszłości
        
        if cleaned_count > 0 {
            info!("🧹 Cleaned up {} expired entries", cleaned_count);
            
            // Aktualizuj statystyki
            let mut stats = self.stats.write().await;
            stats.working_memory_entries = stats.working_memory_entries.saturating_sub(cleaned_count);
        }
        
        Ok(cleaned_count)
    }
    
    /// Pobiera statystyki pamięci
    pub async fn get_stats(&self) -> MemoryStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Zapisuje w pamięci roboczej
    async fn store_in_working_memory(&self, entry: MemoryEntry) -> Result<()> {
        let mut working_memory = self.working_memory.write().await;
        working_memory.insert(entry.key.clone(), entry);
        Ok(())
    }
    
    /// Zapisuje w Redis
    async fn store_in_redis(&self, entry: MemoryEntry) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let entry_json = serde_json::to_string(&entry)?;
        
        let ttl_seconds = if let Some(expires_at) = entry.expires_at {
            (expires_at - Utc::now()).num_seconds().max(1)
        } else {
            86400 // 24 hours default
        };
        
        let _: () = conn.setex(&entry.key, ttl_seconds as usize, entry_json).await?;
        Ok(())
    }
    
    /// Zapisuje w Qdrant
    async fn store_in_qdrant(&self, entry: MemoryEntry) -> Result<()> {
        let collection_name = &self.config.qdrant.collection_name;
        
        // Jeśli nie ma embeddingu, utwórz dummy vector
        let vector = entry.embedding.unwrap_or_else(|| {
            vec![0.0; self.config.qdrant.vector_size as usize]
        });
        
        let mut payload = HashMap::new();
        payload.insert("entry".to_string(), serde_json::to_value(&entry)?.into());
        payload.insert("category".to_string(), entry.category.clone().into());
        payload.insert("created_at".to_string(), entry.created_at.to_rfc3339().into());
        
        let point = PointStruct::new(
            entry.id.to_string(),
            vector,
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
    async fn retrieve_from_working_memory(&self, key: &str) -> Result<Option<MemoryEntry>> {
        let working_memory = self.working_memory.read().await;
        Ok(working_memory.get(key).cloned())
    }
    
    /// Pobiera z Redis
    async fn retrieve_from_redis(&self, key: &str) -> Result<Option<MemoryEntry>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let entry_json: Option<String> = conn.get(key).await?;
        
        if let Some(json) = entry_json {
            let entry: MemoryEntry = serde_json::from_str(&json)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }
    
    /// Pobiera z Qdrant
    async fn retrieve_from_qdrant(&self, key: &str, category: Option<&str>) -> Result<Option<MemoryEntry>> {
        // Implementacja wyszukiwania po kluczu w Qdrant
        // Na razie zwracamy None - można rozszerzyć w przyszłości
        Ok(None)
    }
    
    /// Aktualizuje statystyki dostępu
    async fn update_access_stats(&self, retrieval_time_ms: f64, cache_hit: bool) {
        let mut stats = self.stats.write().await;
        stats.total_access_count += 1;
        
        // Aktualizuj cache hit rate
        let hit_rate = if cache_hit { 1.0 } else { 0.0 };
        stats.cache_hit_rate = (stats.cache_hit_rate * 0.9) + (hit_rate * 0.1);
        
        // Aktualizuj średni czas pobierania
        stats.average_retrieval_time_ms = (stats.average_retrieval_time_ms * 0.9) + (retrieval_time_ms * 0.1);
    }
}
