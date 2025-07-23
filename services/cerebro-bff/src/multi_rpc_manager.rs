//! 🔄 Multi-RPC Provider Manager
//! 
//! Advanced RPC provider management with intelligent load balancing,
//! cost optimization, and automatic failover between multiple providers.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use reqwest::Client;

/// 🌐 Solana Network Types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SolanaNetwork {
    MainnetBeta,
    Devnet,
    Testnet,
}

impl SolanaNetwork {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mainnet-beta" | "mainnet" => Self::MainnetBeta,
            "devnet" => Self::Devnet,
            "testnet" => Self::Testnet,
            _ => Self::MainnetBeta, // Default to mainnet
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Self::MainnetBeta => "mainnet-beta",
            Self::Devnet => "devnet",
            Self::Testnet => "testnet",
        }
    }
}

/// 🏢 RPC Provider configuration with network support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcProvider {
    pub name: String,
    pub mainnet_url: String,
    pub devnet_url: String,
    pub api_key: Option<String>,
    pub monthly_limit: u32,
    pub rpm_limit: Option<u32>,
    pub cost_per_request: f64,
    pub has_enhanced_data: bool,
    pub supports_webhooks: bool,
    pub priority: u8, // 1-10, higher = better
    pub is_free: bool, // TRUE for free providers only
}

impl RpcProvider {
    /// Get the appropriate URL for the specified network
    pub fn get_url_for_network(&self, network: &SolanaNetwork) -> &str {
        match network {
            SolanaNetwork::MainnetBeta => &self.mainnet_url,
            SolanaNetwork::Devnet => &self.devnet_url,
            SolanaNetwork::Testnet => &self.devnet_url, // Use devnet for testnet
        }
    }
}

/// 📊 Provider usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub requests_this_hour: u32,
    pub requests_today: u32,
    pub requests_this_month: u32,
    #[serde(skip)]
    pub last_request_time: Option<Instant>,
    pub success_rate: f64,
    pub avg_response_time_ms: f64,
    pub is_healthy: bool,
    #[serde(skip)]
    pub last_health_check: Option<Instant>,
}

impl Default for ProviderStats {
    fn default() -> Self {
        Self {
            requests_this_hour: 0,
            requests_today: 0,
            requests_this_month: 0,
            last_request_time: None,
            success_rate: 100.0,
            avg_response_time_ms: 0.0,
            is_healthy: true,
            last_health_check: None,
        }
    }
}

/// 🎯 Request routing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    CostOptimized,      // Prefer cheapest available provider
    PerformanceFirst,   // Prefer fastest/most reliable provider
    RoundRobin,         // Distribute evenly across providers
    WeightedRoundRobin, // Distribute based on provider priority
    EnhancedDataFirst,  // Prefer providers with rich metadata
}

/// 🔄 Multi-RPC Manager
pub struct MultiRpcManager {
    providers: Arc<RwLock<HashMap<String, RpcProvider>>>,
    stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
    client: Client,
    routing_strategy: RoutingStrategy,
    current_provider_index: Arc<Mutex<usize>>,
    health_check_interval: Duration,
}

impl MultiRpcManager {
    /// 🚀 Initialize multi-RPC manager
    pub fn new(routing_strategy: RoutingStrategy) -> Self {
        let mut providers = HashMap::new();
        
        // Add default providers
        providers.insert("helius".to_string(), RpcProvider {
            name: "Helius API Pro".to_string(),
            url: "https://mainnet.helius-rpc.com/v1/".to_string(),
            api_key: std::env::var("HELIUS_API_KEY").ok(),
            monthly_limit: 1_000_000,
            rpm_limit: Some(10),
            cost_per_request: 0.001,
            has_enhanced_data: true,
            supports_webhooks: true,
            priority: 10,
        });
        
        providers.insert("quicknode".to_string(), RpcProvider {
            name: "QuickNode Free".to_string(),
            url: "https://solana-mainnet.g.alchemy.com/v2/".to_string(),
            api_key: std::env::var("QUICKNODE_API_KEY").ok(),
            monthly_limit: 100_000,
            rpm_limit: None,
            cost_per_request: 0.0015,
            has_enhanced_data: false,
            supports_webhooks: false,
            priority: 8,
        });
        
        providers.insert("alchemy".to_string(), RpcProvider {
            name: "Alchemy Free".to_string(),
            url: "https://solana-mainnet.g.alchemy.com/v2/".to_string(),
            api_key: std::env::var("ALCHEMY_API_KEY").ok(),
            monthly_limit: 100_000,
            rpm_limit: None,
            cost_per_request: 0.0012,
            has_enhanced_data: false,
            supports_webhooks: false,
            priority: 7,
        });
        
        providers.insert("genesys".to_string(), RpcProvider {
            name: "Genesys Free".to_string(),
            url: "https://mainnet-beta.genesys.network/v1/rpc".to_string(),
            api_key: std::env::var("GENESYS_API_KEY").ok(),
            monthly_limit: 1_000_000,
            rpm_limit: None,
            cost_per_request: 0.0008,
            has_enhanced_data: true,
            supports_webhooks: false,
            priority: 9,
        });
        
        providers.insert("public_solana".to_string(), RpcProvider {
            name: "Public Solana RPC".to_string(),
            url: "https://api.mainnet-beta.solana.com".to_string(),
            api_key: None,
            monthly_limit: u32::MAX, // No official limit
            rpm_limit: None,
            cost_per_request: 0.0,
            has_enhanced_data: false,
            supports_webhooks: false,
            priority: 5,
        });
        
        let stats: HashMap<String, ProviderStats> = providers.keys()
            .map(|k| (k.clone(), ProviderStats::default()))
            .collect();
        
        Self {
            providers: Arc::new(RwLock::new(providers)),
            stats: Arc::new(RwLock::new(stats)),
            client: Client::new(),
            routing_strategy,
            current_provider_index: Arc::new(Mutex::new(0)),
            health_check_interval: Duration::from_secs(300), // 5 minutes
        }
    }

    /// 🎯 Select best provider based on routing strategy
    pub async fn select_provider(&self, requires_enhanced_data: bool) -> Result<String> {
        let providers = self.providers.read().await;
        let stats = self.stats.read().await;
        
        let available_providers: Vec<_> = providers.iter()
            .filter(|(_, provider)| {
                if requires_enhanced_data && !provider.has_enhanced_data {
                    return false;
                }
                
                // Check if provider has available quota
                if let Some(provider_stats) = stats.get(provider.name.as_str()) {
                    if provider_stats.requests_this_month >= provider.monthly_limit {
                        return false;
                    }
                    
                    // Check RPM limit
                    if let Some(rpm_limit) = provider.rpm_limit {
                        if provider_stats.requests_this_hour >= rpm_limit * 60 {
                            return false;
                        }
                    }
                    
                    // Check health
                    if !provider_stats.is_healthy {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        if available_providers.is_empty() {
            return Err(anyhow!("No available RPC providers"));
        }
        
        let selected = match self.routing_strategy {
            RoutingStrategy::CostOptimized => {
                available_providers.iter()
                    .min_by(|a, b| a.1.cost_per_request.partial_cmp(&b.1.cost_per_request).unwrap())
                    .map(|(name, _)| name.as_str())
            },
            RoutingStrategy::PerformanceFirst => {
                available_providers.iter()
                    .max_by(|a, b| {
                        let a_score = a.1.priority as f64 + 
                            stats.get(a.0.as_str()).map_or(0.0, |s| s.success_rate / 10.0);
                        let b_score = b.1.priority as f64 + 
                            stats.get(b.0.as_str()).map_or(0.0, |s| s.success_rate / 10.0);
                        a_score.partial_cmp(&b_score).unwrap()
                    })
                    .map(|(name, _)| name.as_str())
            },
            RoutingStrategy::RoundRobin => {
                let mut index = self.current_provider_index.lock().await;
                let selected = &available_providers[*index % available_providers.len()];
                *index += 1;
                Some(selected.0.as_str())
            },
            RoutingStrategy::WeightedRoundRobin => {
                // Select based on priority weights
                let total_priority: u32 = available_providers.iter()
                    .map(|(_, provider)| provider.priority as u32)
                    .sum();
                
                let mut index = self.current_provider_index.lock().await;
                let target = (*index % total_priority as usize) as u32;
                *index += 1;
                
                let mut current_weight = 0;
                available_providers.iter()
                    .find(|(_, provider)| {
                        current_weight += provider.priority as u32;
                        current_weight > target
                    })
                    .map(|(name, _)| name.as_str())
            },
            RoutingStrategy::EnhancedDataFirst => {
                available_providers.iter()
                    .filter(|(_, provider)| provider.has_enhanced_data)
                    .max_by_key(|(_, provider)| provider.priority)
                    .or_else(|| available_providers.iter().max_by_key(|(_, provider)| provider.priority))
                    .map(|(name, _)| name.as_str())
            },
        };
        
        match selected {
            Some(provider_name) => {
                debug!("🎯 Selected RPC provider: {}", provider_name);
                Ok(provider_name.to_string())
            },
            None => Err(anyhow!("Failed to select RPC provider")),
        }
    }

    /// 📡 Make RPC request with automatic provider selection
    pub async fn make_request(
        &self,
        method: &str,
        params: serde_json::Value,
        requires_enhanced_data: bool,
    ) -> Result<serde_json::Value> {
        let provider_name = self.select_provider(requires_enhanced_data).await?;
        
        let start_time = Instant::now();
        let result = self.make_request_to_provider(&provider_name, method, params.clone()).await;
        let duration = start_time.elapsed();
        
        // Update statistics
        self.update_provider_stats(&provider_name, result.is_ok(), duration).await;
        
        match result {
            Ok(response) => Ok(response),
            Err(e) => {
                warn!("❌ Request failed for provider {}: {}", provider_name, e);
                
                // Try fallback provider
                if let Ok(fallback_provider) = self.select_provider(false).await {
                    if fallback_provider != provider_name {
                        info!("🔄 Trying fallback provider: {}", fallback_provider);
                        return self.make_request_to_provider(&fallback_provider, method, params).await;
                    }
                }
                
                Err(e)
            }
        }
    }

    /// 📡 Make request to specific provider
    async fn make_request_to_provider(
        &self,
        provider_name: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let providers = self.providers.read().await;
        let provider = providers.get(provider_name)
            .ok_or_else(|| anyhow!("Provider not found: {}", provider_name))?;
        
        let mut url = provider.url.clone();
        if let Some(api_key) = &provider.api_key {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str(api_key);
        }
        
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        
        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("HTTP error: {}", response.status()));
        }
        
        let json_response: serde_json::Value = response.json().await?;
        
        if let Some(error) = json_response.get("error") {
            return Err(anyhow!("RPC error: {}", error));
        }
        
        json_response.get("result")
            .cloned()
            .ok_or_else(|| anyhow!("No result in response"))
    }

    /// 📊 Update provider statistics
    async fn update_provider_stats(
        &self,
        provider_name: &str,
        success: bool,
        duration: Duration,
    ) {
        let mut stats = self.stats.write().await;
        if let Some(provider_stats) = stats.get_mut(provider_name) {
            provider_stats.requests_this_hour += 1;
            provider_stats.requests_today += 1;
            provider_stats.requests_this_month += 1;
            provider_stats.last_request_time = Some(Instant::now());
            
            // Update success rate (exponential moving average)
            let new_success_rate = if success { 100.0 } else { 0.0 };
            provider_stats.success_rate = provider_stats.success_rate * 0.9 + new_success_rate * 0.1;
            
            // Update average response time
            let duration_ms = duration.as_millis() as f64;
            if provider_stats.avg_response_time_ms == 0.0 {
                provider_stats.avg_response_time_ms = duration_ms;
            } else {
                provider_stats.avg_response_time_ms = provider_stats.avg_response_time_ms * 0.9 + duration_ms * 0.1;
            }
            
            // Update health status
            provider_stats.is_healthy = provider_stats.success_rate > 50.0;
        }
    }

    /// 🏥 Perform health checks on all providers
    pub async fn health_check_all(&self) -> Result<()> {
        let providers = self.providers.read().await;
        
        for (name, provider) in providers.iter() {
            let health_result = self.check_provider_health(provider).await;
            
            let mut stats = self.stats.write().await;
            if let Some(provider_stats) = stats.get_mut(name) {
                provider_stats.is_healthy = health_result.is_ok();
                provider_stats.last_health_check = Some(Instant::now());
                
                if health_result.is_err() {
                    warn!("🏥 Health check failed for {}: {:?}", name, health_result);
                } else {
                    debug!("✅ Health check passed for {}", name);
                }
            }
        }
        
        Ok(())
    }

    /// 🏥 Check individual provider health
    async fn check_provider_health(&self, provider: &RpcProvider) -> Result<()> {
        let mut url = provider.url.clone();
        if let Some(api_key) = &provider.api_key {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str(api_key);
        }
        
        let health_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getHealth"
        });
        
        let response = tokio::time::timeout(
            Duration::from_secs(10),
            self.client.post(&url).json(&health_request).send()
        ).await??;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Health check failed: {}", response.status()))
        }
    }

    /// 📊 Get comprehensive provider statistics
    pub async fn get_provider_stats(&self) -> HashMap<String, ProviderStats> {
        self.stats.read().await.clone()
    }

    /// 🎯 Get current routing strategy
    pub fn get_routing_strategy(&self) -> &RoutingStrategy {
        &self.routing_strategy
    }

    /// 🔄 Change routing strategy
    pub fn set_routing_strategy(&mut self, strategy: RoutingStrategy) {
        info!("🔄 Routing strategy changed to: {:?}", strategy);
        self.routing_strategy = strategy;
    }

    /// 💰 Calculate total cost savings
    pub async fn calculate_cost_savings(&self) -> f64 {
        let stats = self.stats.read().await;
        let providers = self.providers.read().await;
        
        let mut total_savings = 0.0;
        let helius_cost_per_request = providers.get("helius")
            .map(|p| p.cost_per_request)
            .unwrap_or(0.001);
        
        for (name, provider_stats) in stats.iter() {
            if let Some(provider) = providers.get(name) {
                if name != "helius" {
                    let requests = provider_stats.requests_this_month;
                    let cost_difference = helius_cost_per_request - provider.cost_per_request;
                    total_savings += requests as f64 * cost_difference;
                }
            }
        }
        
        total_savings
    }

    /// 📈 Generate provider performance report
    pub async fn generate_performance_report(&self) -> serde_json::Value {
        let stats = self.stats.read().await;
        let providers = self.providers.read().await;
        
        let mut provider_reports = serde_json::Map::new();
        
        for (name, provider_stats) in stats.iter() {
            if let Some(provider) = providers.get(name) {
                let report = serde_json::json!({
                    "provider_info": {
                        "name": provider.name,
                        "monthly_limit": provider.monthly_limit,
                        "cost_per_request": provider.cost_per_request,
                        "has_enhanced_data": provider.has_enhanced_data,
                        "priority": provider.priority
                    },
                    "usage_stats": {
                        "requests_this_month": provider_stats.requests_this_month,
                        "usage_percentage": (provider_stats.requests_this_month as f64 / provider.monthly_limit as f64) * 100.0,
                        "success_rate": provider_stats.success_rate,
                        "avg_response_time_ms": provider_stats.avg_response_time_ms,
                        "is_healthy": provider_stats.is_healthy
                    },
                    "cost_analysis": {
                        "total_cost": provider_stats.requests_this_month as f64 * provider.cost_per_request,
                        "cost_per_successful_request": if provider_stats.success_rate > 0.0 {
                            provider.cost_per_request / (provider_stats.success_rate / 100.0)
                        } else { 0.0 }
                    }
                });
                
                provider_reports.insert(name.clone(), report);
            }
        }
        
        serde_json::json!({
            "routing_strategy": self.routing_strategy,
            "total_cost_savings": self.calculate_cost_savings().await,
            "providers": provider_reports,
            "summary": {
                "total_providers": providers.len(),
                "healthy_providers": stats.values().filter(|s| s.is_healthy).count(),
                "total_requests": stats.values().map(|s| s.requests_this_month).sum::<u32>()
            }
        })
    }
}
