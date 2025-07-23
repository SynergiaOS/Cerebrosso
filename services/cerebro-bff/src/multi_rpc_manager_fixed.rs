//! 🔄 Multi-RPC Provider Manager - DEVNET/MAINNET Support
//! 
//! Advanced RPC provider management with intelligent load balancing,
//! cost optimization, and automatic failover between FREE providers only.
//! Supports both DEVNET and MAINNET with real production data.

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

/// 🏢 RPC Provider configuration with network support (FREE PROVIDERS ONLY)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcProvider {
    pub name: String,
    pub mainnet_url: String,
    pub devnet_url: String,
    pub api_key: Option<String>,
    pub monthly_limit: u32,
    pub rpm_limit: Option<u32>,
    pub cost_per_request: f64, // Should be 0.0 for free providers
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
    
    /// Build complete RPC URL with API key if needed
    pub fn build_rpc_url(&self, network: &SolanaNetwork) -> String {
        let base_url = self.get_url_for_network(network);
        
        // For Alchemy, API key is in the URL
        if self.name.contains("Alchemy") {
            if let Some(api_key) = &self.api_key {
                match network {
                    SolanaNetwork::MainnetBeta => {
                        format!("https://solana-mainnet.g.alchemy.com/v2/{}", api_key)
                    },
                    SolanaNetwork::Devnet => {
                        format!("https://solana-devnet.g.alchemy.com/v2/{}", api_key)
                    },
                    SolanaNetwork::Testnet => {
                        format!("https://solana-devnet.g.alchemy.com/v2/{}", api_key)
                    },
                }
            } else {
                base_url.to_string()
            }
        } else {
            base_url.to_string()
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

/// 🎯 Routing strategies for provider selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    CostOptimized,     // Prefer free providers
    PerformanceFirst,  // Prefer fastest providers
    RoundRobin,        // Distribute evenly
    WeightedRoundRobin, // Distribute by priority
    EnhancedDataFirst, // Prefer providers with enhanced data
}

/// 🔄 Multi-RPC Manager with FREE providers only
#[derive(Debug)]
pub struct MultiRpcManager {
    providers: Arc<RwLock<HashMap<String, RpcProvider>>>,
    stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
    routing_strategy: RoutingStrategy,
    current_network: SolanaNetwork,
    client: Client,
    round_robin_index: Arc<Mutex<usize>>,
    health_check_interval: Duration,
}

impl MultiRpcManager {
    /// 🚀 Initialize multi-RPC manager with FREE providers only
    pub fn new(routing_strategy: RoutingStrategy, network: SolanaNetwork) -> Self {
        let mut providers = HashMap::new();
        
        // 🌟 Helius API Pro - FREE TIER (100k requests/month)
        // REAL LIMITS: Free tier is 100k, NOT 1M!
        providers.insert("helius".to_string(), RpcProvider {
            name: "Helius API Pro (FREE)".to_string(),
            mainnet_url: "https://api.helius.xyz/v1/rpc".to_string(),
            devnet_url: "https://api.helius.xyz/v1/rpc?cluster=devnet".to_string(),
            api_key: std::env::var("HELIUS_API_KEY").ok(),
            monthly_limit: 100_000, // FREE tier limit (NOT 1M)
            rpm_limit: Some(10),
            cost_per_request: 0.0, // FREE
            has_enhanced_data: true,
            supports_webhooks: true,
            priority: 10,
            is_free: true,
        });
        
        // 🔮 Alchemy - FREE TIER (100k requests/month)
        providers.insert("alchemy".to_string(), RpcProvider {
            name: "Alchemy (FREE)".to_string(),
            mainnet_url: "https://solana-mainnet.g.alchemy.com/v2/".to_string(),
            devnet_url: "https://solana-devnet.g.alchemy.com/v2/".to_string(),
            api_key: std::env::var("ALCHEMY_API_KEY").ok(),
            monthly_limit: 100_000, // FREE tier
            rpm_limit: None, // No RPM limit
            cost_per_request: 0.0, // FREE
            has_enhanced_data: false,
            supports_webhooks: false,
            priority: 8,
            is_free: true,
        });
        
        // 🌐 Public Solana RPC - FREE (Unlimited but rate limited)
        providers.insert("public".to_string(), RpcProvider {
            name: "Public Solana RPC (FREE)".to_string(),
            mainnet_url: "https://api.mainnet-beta.solana.com".to_string(),
            devnet_url: "https://api.devnet.solana.com".to_string(),
            api_key: None,
            monthly_limit: u32::MAX, // Unlimited
            rpm_limit: Some(100), // Rate limited but free
            cost_per_request: 0.0, // FREE
            has_enhanced_data: false,
            supports_webhooks: false,
            priority: 6,
            is_free: true,
        });
        
        // NOTE: QuickNode REMOVED - it's PAID, not free!
        // NOTE: Genesys REMOVED - unclear if truly free
        
        let stats = providers.keys()
            .map(|k| (k.clone(), ProviderStats::default()))
            .collect();
        
        info!("🔄 Initialized Multi-RPC Manager with {} FREE providers for network: {}", 
              providers.len(), network.to_string());
        info!("📊 Total FREE requests available: 200k+/month");
        
        Self {
            providers: Arc::new(RwLock::new(providers)),
            stats: Arc::new(RwLock::new(stats)),
            routing_strategy,
            current_network: network,
            client: Client::new(),
            round_robin_index: Arc::new(Mutex::new(0)),
            health_check_interval: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// 🌐 Switch network (DEVNET/MAINNET)
    pub async fn switch_network(&mut self, network: SolanaNetwork) {
        self.current_network = network.clone();
        info!("🌐 Switched to network: {}", network.to_string());
    }
    
    /// 🎯 Get the best provider based on routing strategy
    pub async fn get_best_provider(&self) -> Result<(String, String)> {
        let providers = self.providers.read().await;
        let stats = self.stats.read().await;
        
        if providers.is_empty() {
            return Err(anyhow!("No providers available"));
        }
        
        let provider_name = match self.routing_strategy {
            RoutingStrategy::CostOptimized => {
                // Always prefer free providers (all our providers are free)
                self.select_by_cost(&providers, &stats).await?
            },
            RoutingStrategy::PerformanceFirst => {
                self.select_by_performance(&providers, &stats).await?
            },
            RoutingStrategy::RoundRobin => {
                self.select_round_robin(&providers).await?
            },
            RoutingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&providers, &stats).await?
            },
            RoutingStrategy::EnhancedDataFirst => {
                self.select_by_enhanced_data(&providers, &stats).await?
            },
        };
        
        let provider = providers.get(&provider_name)
            .ok_or_else(|| anyhow!("Provider not found: {}", provider_name))?;
        
        let url = provider.build_rpc_url(&self.current_network);
        
        Ok((provider_name, url))
    }
    
    /// 💰 Select provider by cost (prefer free)
    async fn select_by_cost(&self, providers: &HashMap<String, RpcProvider>, 
                           stats: &HashMap<String, ProviderStats>) -> Result<String> {
        // All our providers are free, so select by health and usage
        let mut best_provider = None;
        let mut best_score = f64::MIN;
        
        for (name, provider) in providers {
            if let Some(provider_stats) = stats.get(name) {
                if !provider_stats.is_healthy {
                    continue;
                }
                
                // Score based on remaining capacity and success rate
                let usage_ratio = provider_stats.requests_this_month as f64 / provider.monthly_limit as f64;
                let remaining_capacity = 1.0 - usage_ratio;
                let score = remaining_capacity * provider_stats.success_rate / 100.0;
                
                if score > best_score {
                    best_score = score;
                    best_provider = Some(name.clone());
                }
            }
        }
        
        best_provider.ok_or_else(|| anyhow!("No healthy providers available"))
    }
    
    /// ⚡ Select provider by performance
    async fn select_by_performance(&self, providers: &HashMap<String, RpcProvider>, 
                                  stats: &HashMap<String, ProviderStats>) -> Result<String> {
        let mut best_provider = None;
        let mut best_response_time = f64::MAX;
        
        for (name, _provider) in providers {
            if let Some(provider_stats) = stats.get(name) {
                if provider_stats.is_healthy && 
                   provider_stats.avg_response_time_ms > 0.0 &&
                   provider_stats.avg_response_time_ms < best_response_time {
                    best_response_time = provider_stats.avg_response_time_ms;
                    best_provider = Some(name.clone());
                }
            }
        }
        
        best_provider.or_else(|| providers.keys().next().cloned())
            .ok_or_else(|| anyhow!("No providers available"))
    }
    
    /// 🔄 Round robin selection
    async fn select_round_robin(&self, providers: &HashMap<String, RpcProvider>) -> Result<String> {
        let provider_names: Vec<String> = providers.keys().cloned().collect();
        if provider_names.is_empty() {
            return Err(anyhow!("No providers available"));
        }
        
        let mut index = self.round_robin_index.lock().await;
        let selected = provider_names[*index % provider_names.len()].clone();
        *index += 1;
        
        Ok(selected)
    }
    
    /// ⚖️ Weighted round robin selection
    async fn select_weighted_round_robin(&self, providers: &HashMap<String, RpcProvider>, 
                                       stats: &HashMap<String, ProviderStats>) -> Result<String> {
        // Select based on priority and health
        let mut weighted_providers = Vec::new();
        
        for (name, provider) in providers {
            if let Some(provider_stats) = stats.get(name) {
                if provider_stats.is_healthy {
                    // Add provider multiple times based on priority
                    for _ in 0..provider.priority {
                        weighted_providers.push(name.clone());
                    }
                }
            }
        }
        
        if weighted_providers.is_empty() {
            return Err(anyhow!("No healthy providers available"));
        }
        
        let mut index = self.round_robin_index.lock().await;
        let selected = weighted_providers[*index % weighted_providers.len()].clone();
        *index += 1;
        
        Ok(selected)
    }
    
    /// 📊 Select provider with enhanced data first
    async fn select_by_enhanced_data(&self, providers: &HashMap<String, RpcProvider>, 
                                   stats: &HashMap<String, ProviderStats>) -> Result<String> {
        // Prefer providers with enhanced data (like Helius)
        for (name, provider) in providers {
            if provider.has_enhanced_data {
                if let Some(provider_stats) = stats.get(name) {
                    if provider_stats.is_healthy {
                        return Ok(name.clone());
                    }
                }
            }
        }
        
        // Fallback to any healthy provider
        self.select_by_cost(providers, stats).await
    }
}
