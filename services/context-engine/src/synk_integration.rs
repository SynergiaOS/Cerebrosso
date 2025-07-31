//! 🔗 Synk Integration - Network State Synchronization Integration
//! 
//! Integration module for Synk network state synchronization with Context Engine

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug, warn, instrument};

use crate::{
    config::Config,
    memory_store::{MemoryStore, MemoryEntry, MemoryType, MemoryLevel},
    embedding_service::EmbeddingService,
    context_engine::ContextError,
};

/// 🔗 Synk Integration State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SynkIntegrationState {
    /// Integration initializing
    Initializing,
    /// Connected to Synk service
    Connected,
    /// Synchronizing network state
    Synchronizing,
    /// Integration paused
    Paused,
    /// Connection lost
    Disconnected,
    /// Integration failed
    Failed(String),
}

/// 📊 Network State Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStateData {
    /// Current slot
    pub current_slot: u64,
    /// Block height
    pub block_height: u64,
    /// Network congestion level (0.0 - 1.0)
    pub congestion_level: f64,
    /// Average transaction fee
    pub avg_transaction_fee: u64,
    /// TPS (Transactions Per Second)
    pub tps: f64,
    /// Active validator count
    pub validator_count: u32,
    /// Network health score (0.0 - 1.0)
    pub health_score: f64,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// 🔗 Synk Integration Manager
pub struct SynkIntegration {
    /// Configuration
    config: Arc<Config>,
    /// Memory store for network state
    memory_store: Arc<MemoryStore>,
    /// Embedding service
    embedding_service: Arc<EmbeddingService>,
    /// Current integration state
    state: Arc<RwLock<SynkIntegrationState>>,
    /// Network state cache
    network_state_cache: Arc<RwLock<HashMap<String, NetworkStateData>>>,
    /// Synk service URL
    synk_url: String,
    /// HTTP client for Synk API
    http_client: reqwest::Client,
}

impl SynkIntegration {
    /// Creates new Synk integration
    #[instrument(skip(config, memory_store, embedding_service))]
    pub async fn new(
        config: Arc<Config>,
        memory_store: Arc<MemoryStore>,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self, ContextError> {
        info!("🔗 Initializing Synk Integration...");
        
        let synk_url = std::env::var("SYNK_URL")
            .unwrap_or_else(|_| "http://localhost:8300".to_string());
        
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        let integration = Self {
            config,
            memory_store,
            embedding_service,
            state: Arc::new(RwLock::new(SynkIntegrationState::Initializing)),
            network_state_cache: Arc::new(RwLock::new(HashMap::new())),
            synk_url,
            http_client,
        };
        
        info!("✅ Synk Integration initialized");
        Ok(integration)
    }
    
    /// Starts the Synk integration
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<(), ContextError> {
        info!("🚀 Starting Synk Integration...");
        
        // Test connection to Synk service
        if self.test_synk_connection().await? {
            let mut state = self.state.write().await;
            *state = SynkIntegrationState::Connected;
            info!("✅ Connected to Synk service at {}", self.synk_url);
        } else {
            let mut state = self.state.write().await;
            *state = SynkIntegrationState::Failed("Cannot connect to Synk service".to_string());
            return Err(ContextError::Configuration("Synk service unavailable".to_string()));
        }
        
        // Start network state synchronization
        self.start_network_state_sync().await?;
        
        info!("✅ Synk Integration started successfully");
        Ok(())
    }
    
    /// Gets current network state
    #[instrument(skip(self))]
    pub async fn get_network_state(&self) -> Result<NetworkStateData, ContextError> {
        debug!("📊 Fetching current network state...");
        
        // Try cache first
        {
            let cache = self.network_state_cache.read().await;
            if let Some(state) = cache.get("current") {
                if state.last_updated > Utc::now() - chrono::Duration::minutes(1) {
                    debug!("📋 Returning cached network state");
                    return Ok(state.clone());
                }
            }
        }
        
        // Fetch from Synk service
        let network_state = self.fetch_network_state_from_synk().await?;
        
        // Update cache
        {
            let mut cache = self.network_state_cache.write().await;
            cache.insert("current".to_string(), network_state.clone());
        }
        
        // Store in memory for context
        self.store_network_state_in_memory(&network_state).await?;
        
        debug!("✅ Network state fetched: slot={}, congestion={:.2}", 
               network_state.current_slot, network_state.congestion_level);
        
        Ok(network_state)
    }
    
    /// Gets network congestion forecast
    #[instrument(skip(self))]
    pub async fn get_congestion_forecast(&self, hours_ahead: u32) -> Result<Vec<f64>, ContextError> {
        debug!("🔮 Fetching congestion forecast for {} hours", hours_ahead);
        
        let url = format!("{}/api/v1/forecast/congestion?hours={}", self.synk_url, hours_ahead);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        if response.status().is_success() {
            let forecast: Vec<f64> = response
                .json()
                .await
                .map_err(|e| ContextError::Configuration(e.to_string()))?;
            
            debug!("✅ Congestion forecast fetched: {} data points", forecast.len());
            Ok(forecast)
        } else {
            Err(ContextError::Configuration("Failed to fetch congestion forecast".to_string()))
        }
    }
    
    /// Gets current integration state
    pub async fn get_state(&self) -> SynkIntegrationState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Tests connection to Synk service
    async fn test_synk_connection(&self) -> Result<bool, ContextError> {
        let health_url = format!("{}/health", self.synk_url);
        
        match self.http_client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    /// Fetches network state from Synk service
    async fn fetch_network_state_from_synk(&self) -> Result<NetworkStateData, ContextError> {
        let url = format!("{}/api/v1/network/state", self.synk_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        if response.status().is_success() {
            let network_state: NetworkStateData = response
                .json()
                .await
                .map_err(|e| ContextError::Configuration(e.to_string()))?;
            
            Ok(network_state)
        } else {
            Err(ContextError::Configuration("Failed to fetch network state".to_string()))
        }
    }
    
    /// Stores network state in memory for context
    async fn store_network_state_in_memory(&self, state: &NetworkStateData) -> Result<(), ContextError> {
        let content = format!(
            "Network State - Slot: {}, Congestion: {:.2}, TPS: {:.1}, Health: {:.2}",
            state.current_slot, state.congestion_level, state.tps, state.health_score
        );
        
        // Generate embedding for network state
        let embedding = self.embedding_service
            .generate_embedding(&content)
            .await
            .map_err(|e| ContextError::EmbeddingService(e.to_string()))?;
        
        // Create memory entry
        let memory_entry = MemoryEntry::new(
            content,
            embedding,
            MemoryType::ShortTerm,
            MemoryLevel::Operational,
        );
        
        // Store in memory
        self.memory_store
            .store(memory_entry)
            .await
            .map_err(|e| ContextError::MemoryStore(e.to_string()))?;
        
        Ok(())
    }
    
    /// Starts network state synchronization loop
    async fn start_network_state_sync(&self) -> Result<(), ContextError> {
        let state = self.state.clone();
        let network_state_cache = self.network_state_cache.clone();
        let synk_url = self.synk_url.clone();
        let http_client = self.http_client.clone();
        let memory_store = self.memory_store.clone();
        let embedding_service = self.embedding_service.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check if we should continue syncing
                {
                    let current_state = state.read().await;
                    match *current_state {
                        SynkIntegrationState::Connected | SynkIntegrationState::Synchronizing => {},
                        _ => continue,
                    }
                }
                
                // Update state to synchronizing
                {
                    let mut current_state = state.write().await;
                    *current_state = SynkIntegrationState::Synchronizing;
                }
                
                // Fetch and cache network state
                if let Ok(network_state) = fetch_network_state(&http_client, &synk_url).await {
                    // Update cache
                    {
                        let mut cache = network_state_cache.write().await;
                        cache.insert("current".to_string(), network_state.clone());
                    }
                    
                    // Store in memory
                    let _ = store_network_state_in_memory_sync(
                        &network_state,
                        &memory_store,
                        &embedding_service,
                    ).await;
                    
                    debug!("🔄 Network state synchronized: slot={}", network_state.current_slot);
                } else {
                    warn!("⚠️ Failed to synchronize network state");
                }
                
                // Update state back to connected
                {
                    let mut current_state = state.write().await;
                    *current_state = SynkIntegrationState::Connected;
                }
            }
        });
        
        Ok(())
    }
}

/// Helper function for fetching network state
async fn fetch_network_state(
    client: &reqwest::Client,
    synk_url: &str,
) -> Result<NetworkStateData, ContextError> {
    let url = format!("{}/api/v1/network/state", synk_url);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ContextError::Configuration(e.to_string()))?;
    
    if response.status().is_success() {
        let network_state: NetworkStateData = response
            .json()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        Ok(network_state)
    } else {
        Err(ContextError::Configuration("Failed to fetch network state".to_string()))
    }
}

/// Helper function for storing network state in memory
async fn store_network_state_in_memory_sync(
    state: &NetworkStateData,
    memory_store: &MemoryStore,
    embedding_service: &EmbeddingService,
) -> Result<(), ContextError> {
    let content = format!(
        "Network State - Slot: {}, Congestion: {:.2}, TPS: {:.1}, Health: {:.2}",
        state.current_slot, state.congestion_level, state.tps, state.health_score
    );
    
    // Generate embedding for network state
    let embedding = embedding_service
        .generate_embedding(&content)
        .await
        .map_err(|e| ContextError::EmbeddingService(e.to_string()))?;
    
    // Create memory entry
    let memory_entry = MemoryEntry::new(
        content,
        embedding,
        MemoryType::ShortTerm,
        MemoryLevel::Operational,
    );
    
    // Store in memory
    memory_store
        .store(memory_entry)
        .await
        .map_err(|e| ContextError::MemoryStore(e.to_string()))?;
    
    Ok(())
}
