//! 🛡️ Chainguardia Integration - Security Monitoring Integration
//! 
//! Integration module for Chainguardia security monitoring with Context Engine

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error, instrument};

use crate::{
    config::Config,
    memory_store::{MemoryStore, MemoryEntry, MemoryType, MemoryLevel},
    embedding_service::EmbeddingService,
    context_engine::ContextError,
};

/// 🛡️ Chainguardia Integration State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardiaIntegrationState {
    /// Integration initializing
    Initializing,
    /// Connected to Chainguardia service
    Connected,
    /// Monitoring security events
    Monitoring,
    /// Integration paused
    Paused,
    /// Connection lost
    Disconnected,
    /// Integration failed
    Failed(String),
}

/// 🚨 Security Alert Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlertData {
    /// Alert ID
    pub id: Uuid,
    /// Alert type
    pub alert_type: String,
    /// Severity level
    pub severity: String,
    /// Description
    pub description: String,
    /// Affected resources
    pub affected_resources: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Status (active, resolved, investigating)
    pub status: String,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 📊 Security Status Data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatusData {
    /// Overall security level
    pub security_level: String,
    /// Active threats count
    pub active_threats: u32,
    /// Recent anomalies count
    pub recent_anomalies: u32,
    /// Security score (0.0 - 1.0)
    pub security_score: f64,
    /// Compliance status
    pub compliance_status: String,
    /// Last security scan
    pub last_scan: DateTime<Utc>,
    /// Monitored wallets count
    pub monitored_wallets: u32,
}

/// 🛡️ Chainguardia Integration Manager
pub struct GuardiaIntegration {
    /// Configuration
    config: Arc<Config>,
    /// Memory store for security data
    memory_store: Arc<MemoryStore>,
    /// Embedding service
    embedding_service: Arc<EmbeddingService>,
    /// Current integration state
    state: Arc<RwLock<GuardiaIntegrationState>>,
    /// Security alerts cache
    security_alerts_cache: Arc<RwLock<HashMap<Uuid, SecurityAlertData>>>,
    /// Security status cache
    security_status_cache: Arc<RwLock<Option<SecurityStatusData>>>,
    /// Chainguardia service URL
    guardia_url: String,
    /// HTTP client for Chainguardia API
    http_client: reqwest::Client,
}

impl GuardiaIntegration {
    /// Creates new Chainguardia integration
    #[instrument(skip(config, memory_store, embedding_service))]
    pub async fn new(
        config: Arc<Config>,
        memory_store: Arc<MemoryStore>,
        embedding_service: Arc<EmbeddingService>,
    ) -> Result<Self, ContextError> {
        info!("🛡️ Initializing Chainguardia Integration...");
        
        let guardia_url = std::env::var("CHAINGUARDIA_URL")
            .unwrap_or_else(|_| "http://localhost:8400".to_string());
        
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        let integration = Self {
            config,
            memory_store,
            embedding_service,
            state: Arc::new(RwLock::new(GuardiaIntegrationState::Initializing)),
            security_alerts_cache: Arc::new(RwLock::new(HashMap::new())),
            security_status_cache: Arc::new(RwLock::new(None)),
            guardia_url,
            http_client,
        };
        
        info!("✅ Chainguardia Integration initialized");
        Ok(integration)
    }
    
    /// Starts the Chainguardia integration
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<(), ContextError> {
        info!("🚀 Starting Chainguardia Integration...");
        
        // Test connection to Chainguardia service
        if self.test_guardia_connection().await? {
            let mut state = self.state.write().await;
            *state = GuardiaIntegrationState::Connected;
            info!("✅ Connected to Chainguardia service at {}", self.guardia_url);
        } else {
            let mut state = self.state.write().await;
            *state = GuardiaIntegrationState::Failed("Cannot connect to Chainguardia service".to_string());
            return Err(ContextError::Configuration("Chainguardia service unavailable".to_string()));
        }
        
        // Start security monitoring
        self.start_security_monitoring().await?;
        
        info!("✅ Chainguardia Integration started successfully");
        Ok(())
    }
    
    /// Gets current security status
    #[instrument(skip(self))]
    pub async fn get_security_status(&self) -> Result<SecurityStatusData, ContextError> {
        debug!("🛡️ Fetching current security status...");
        
        // Try cache first
        {
            let cache = self.security_status_cache.read().await;
            if let Some(status) = cache.as_ref() {
                if status.last_scan > Utc::now() - chrono::Duration::minutes(5) {
                    debug!("📋 Returning cached security status");
                    return Ok(status.clone());
                }
            }
        }
        
        // Fetch from Chainguardia service
        let security_status = self.fetch_security_status_from_guardia().await?;
        
        // Update cache
        {
            let mut cache = self.security_status_cache.write().await;
            *cache = Some(security_status.clone());
        }
        
        // Store in memory for context
        self.store_security_status_in_memory(&security_status).await?;
        
        debug!("✅ Security status fetched: level={}, threats={}", 
               security_status.security_level, security_status.active_threats);
        
        Ok(security_status)
    }
    
    /// Gets active security alerts
    #[instrument(skip(self))]
    pub async fn get_active_alerts(&self) -> Result<Vec<SecurityAlertData>, ContextError> {
        debug!("🚨 Fetching active security alerts...");
        
        let url = format!("{}/api/v1/alerts/active", self.guardia_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        if response.status().is_success() {
            let alerts: Vec<SecurityAlertData> = response
                .json()
                .await
                .map_err(|e| ContextError::Configuration(e.to_string()))?;
            
            // Update cache
            {
                let mut cache = self.security_alerts_cache.write().await;
                for alert in &alerts {
                    cache.insert(alert.id, alert.clone());
                }
            }
            
            // Store critical alerts in memory
            for alert in &alerts {
                if alert.severity == "Critical" || alert.severity == "Emergency" {
                    self.store_security_alert_in_memory(alert).await?;
                }
            }
            
            debug!("✅ Active alerts fetched: {} alerts", alerts.len());
            Ok(alerts)
        } else {
            Err(ContextError::Configuration("Failed to fetch active alerts".to_string()))
        }
    }
    
    /// Reports a security event to Chainguardia
    #[instrument(skip(self, event_data))]
    pub async fn report_security_event(&self, event_data: serde_json::Value) -> Result<(), ContextError> {
        debug!("📝 Reporting security event to Chainguardia...");
        
        let url = format!("{}/api/v1/events", self.guardia_url);
        
        let response = self.http_client
            .post(&url)
            .json(&event_data)
            .send()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        if response.status().is_success() {
            debug!("✅ Security event reported successfully");
            Ok(())
        } else {
            error!("❌ Failed to report security event: {}", response.status());
            Err(ContextError::Configuration("Failed to report security event".to_string()))
        }
    }
    
    /// Gets current integration state
    pub async fn get_state(&self) -> GuardiaIntegrationState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Tests connection to Chainguardia service
    async fn test_guardia_connection(&self) -> Result<bool, ContextError> {
        let health_url = format!("{}/health", self.guardia_url);
        
        match self.http_client.get(&health_url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    /// Fetches security status from Chainguardia service
    async fn fetch_security_status_from_guardia(&self) -> Result<SecurityStatusData, ContextError> {
        let url = format!("{}/api/v1/security/status", self.guardia_url);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        if response.status().is_success() {
            let security_status: SecurityStatusData = response
                .json()
                .await
                .map_err(|e| ContextError::Configuration(e.to_string()))?;
            
            Ok(security_status)
        } else {
            Err(ContextError::Configuration("Failed to fetch security status".to_string()))
        }
    }
    
    /// Stores security status in memory for context
    async fn store_security_status_in_memory(&self, status: &SecurityStatusData) -> Result<(), ContextError> {
        let content = format!(
            "Security Status - Level: {}, Active Threats: {}, Security Score: {:.2}, Compliance: {}",
            status.security_level, status.active_threats, status.security_score, status.compliance_status
        );
        
        // Generate embedding for security status
        let embedding = self.embedding_service
            .generate_embedding(&content)
            .await
            .map_err(|e| ContextError::EmbeddingService(e.to_string()))?;
        
        // Create memory entry
        let memory_entry = MemoryEntry::new(
            content,
            embedding,
            MemoryType::ShortTerm,
            MemoryLevel::Strategic,
        );
        
        // Store in memory
        self.memory_store
            .store(memory_entry)
            .await
            .map_err(|e| ContextError::MemoryStore(e.to_string()))?;
        
        Ok(())
    }
    
    /// Stores security alert in memory for context
    async fn store_security_alert_in_memory(&self, alert: &SecurityAlertData) -> Result<(), ContextError> {
        let content = format!(
            "Security Alert - Type: {}, Severity: {}, Description: {}, Confidence: {:.2}",
            alert.alert_type, alert.severity, alert.description, alert.confidence
        );
        
        // Generate embedding for security alert
        let embedding = self.embedding_service
            .generate_embedding(&content)
            .await
            .map_err(|e| ContextError::EmbeddingService(e.to_string()))?;
        
        // Create memory entry with high priority for critical alerts
        let memory_level = match alert.severity.as_str() {
            "Critical" | "Emergency" => MemoryLevel::Strategic,
            "High" => MemoryLevel::Operational,
            _ => MemoryLevel::Working,
        };
        
        let memory_entry = MemoryEntry::new(
            content,
            embedding,
            MemoryType::ShortTerm,
            memory_level,
        );
        
        // Store in memory
        self.memory_store
            .store(memory_entry)
            .await
            .map_err(|e| ContextError::MemoryStore(e.to_string()))?;
        
        Ok(())
    }
    
    /// Starts security monitoring loop
    async fn start_security_monitoring(&self) -> Result<(), ContextError> {
        let state = self.state.clone();
        let security_status_cache = self.security_status_cache.clone();
        let security_alerts_cache = self.security_alerts_cache.clone();
        let guardia_url = self.guardia_url.clone();
        let http_client = self.http_client.clone();
        let memory_store = self.memory_store.clone();
        let embedding_service = self.embedding_service.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Check if we should continue monitoring
                {
                    let current_state = state.read().await;
                    match *current_state {
                        GuardiaIntegrationState::Connected | GuardiaIntegrationState::Monitoring => {},
                        _ => continue,
                    }
                }
                
                // Update state to monitoring
                {
                    let mut current_state = state.write().await;
                    *current_state = GuardiaIntegrationState::Monitoring;
                }
                
                // Fetch and cache security status
                if let Ok(security_status) = fetch_security_status(&http_client, &guardia_url).await {
                    // Update cache
                    {
                        let mut cache = security_status_cache.write().await;
                        *cache = Some(security_status.clone());
                    }
                    
                    // Store in memory
                    let _ = store_security_status_in_memory_sync(
                        &security_status,
                        &memory_store,
                        &embedding_service,
                    ).await;
                    
                    debug!("🔄 Security status synchronized: level={}", security_status.security_level);
                } else {
                    warn!("⚠️ Failed to synchronize security status");
                }
                
                // Fetch active alerts
                if let Ok(alerts) = fetch_active_alerts(&http_client, &guardia_url).await {
                    // Update cache
                    {
                        let mut cache = security_alerts_cache.write().await;
                        cache.clear();
                        for alert in &alerts {
                            cache.insert(alert.id, alert.clone());
                        }
                    }
                    
                    // Store critical alerts in memory
                    for alert in &alerts {
                        if alert.severity == "Critical" || alert.severity == "Emergency" {
                            let _ = store_security_alert_in_memory_sync(
                                alert,
                                &memory_store,
                                &embedding_service,
                            ).await;
                        }
                    }
                    
                    debug!("🔄 Security alerts synchronized: {} alerts", alerts.len());
                } else {
                    warn!("⚠️ Failed to synchronize security alerts");
                }
                
                // Update state back to connected
                {
                    let mut current_state = state.write().await;
                    *current_state = GuardiaIntegrationState::Connected;
                }
            }
        });
        
        Ok(())
    }
}

/// Helper function for fetching security status
async fn fetch_security_status(
    client: &reqwest::Client,
    guardia_url: &str,
) -> Result<SecurityStatusData, ContextError> {
    let url = format!("{}/api/v1/security/status", guardia_url);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ContextError::Configuration(e.to_string()))?;
    
    if response.status().is_success() {
        let security_status: SecurityStatusData = response
            .json()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        Ok(security_status)
    } else {
        Err(ContextError::Configuration("Failed to fetch security status".to_string()))
    }
}

/// Helper function for fetching active alerts
async fn fetch_active_alerts(
    client: &reqwest::Client,
    guardia_url: &str,
) -> Result<Vec<SecurityAlertData>, ContextError> {
    let url = format!("{}/api/v1/alerts/active", guardia_url);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ContextError::Configuration(e.to_string()))?;
    
    if response.status().is_success() {
        let alerts: Vec<SecurityAlertData> = response
            .json()
            .await
            .map_err(|e| ContextError::Configuration(e.to_string()))?;
        
        Ok(alerts)
    } else {
        Err(ContextError::Configuration("Failed to fetch active alerts".to_string()))
    }
}

/// Helper function for storing security status in memory
async fn store_security_status_in_memory_sync(
    status: &SecurityStatusData,
    memory_store: &MemoryStore,
    embedding_service: &EmbeddingService,
) -> Result<(), ContextError> {
    let content = format!(
        "Security Status - Level: {}, Active Threats: {}, Security Score: {:.2}, Compliance: {}",
        status.security_level, status.active_threats, status.security_score, status.compliance_status
    );
    
    // Generate embedding for security status
    let embedding = embedding_service
        .generate_embedding(&content)
        .await
        .map_err(|e| ContextError::EmbeddingService(e.to_string()))?;
    
    // Create memory entry
    let memory_entry = MemoryEntry::new(
        content,
        embedding,
        MemoryType::ShortTerm,
        MemoryLevel::Strategic,
    );
    
    // Store in memory
    memory_store
        .store(memory_entry)
        .await
        .map_err(|e| ContextError::MemoryStore(e.to_string()))?;
    
    Ok(())
}

/// Helper function for storing security alert in memory
async fn store_security_alert_in_memory_sync(
    alert: &SecurityAlertData,
    memory_store: &MemoryStore,
    embedding_service: &EmbeddingService,
) -> Result<(), ContextError> {
    let content = format!(
        "Security Alert - Type: {}, Severity: {}, Description: {}, Confidence: {:.2}",
        alert.alert_type, alert.severity, alert.description, alert.confidence
    );
    
    // Generate embedding for security alert
    let embedding = embedding_service
        .generate_embedding(&content)
        .await
        .map_err(|e| ContextError::EmbeddingService(e.to_string()))?;
    
    // Create memory entry with high priority for critical alerts
    let memory_level = match alert.severity.as_str() {
        "Critical" | "Emergency" => MemoryLevel::Strategic,
        "High" => MemoryLevel::Operational,
        _ => MemoryLevel::Working,
    };
    
    let memory_entry = MemoryEntry::new(
        content,
        embedding,
        MemoryType::ShortTerm,
        memory_level,
    );
    
    // Store in memory
    memory_store
        .store(memory_entry)
        .await
        .map_err(|e| ContextError::MemoryStore(e.to_string()))?;
    
    Ok(())
}
