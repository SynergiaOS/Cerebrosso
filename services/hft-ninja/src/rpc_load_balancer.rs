//! 🔄 RPC Load Balancer - Smart RPC Endpoint Management
//! 
//! Intelligent load balancing between premium (Helius, QuickNode) and free RPC endpoints
//! with automatic failover, rate limiting, and cost optimization.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use reqwest::Client;

/// 🔄 RPC endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoint {
    pub name: String,
    pub url: String,
    pub tier: RpcTier,
    pub max_requests_per_minute: u32,
    pub timeout_ms: u64,
    pub cost_per_request: f64,
    pub priority: u8, // 1-10, higher = better
}

/// 🏆 RPC endpoint tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RpcTier {
    Premium,    // Helius, QuickNode
    Standard,   // Alchemy, Infura
    Free,       // Public endpoints
}

/// 📊 RPC endpoint statistics
#[derive(Debug, Clone, Default)]
pub struct RpcStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub last_request_time: Option<Instant>,
    pub requests_this_minute: u32,
    pub minute_window_start: Instant,
}

/// 🎯 Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    CostOptimized,      // Prefer cheaper endpoints
    PerformanceFirst,   // Prefer fastest endpoints
    Balanced,           // Balance cost and performance
    RoundRobin,         // Simple round-robin
}

/// 🔄 RPC Load Balancer
pub struct RpcLoadBalancer {
    endpoints: Arc<RwLock<Vec<RpcEndpoint>>>,
    stats: Arc<RwLock<HashMap<String, RpcStats>>>,
    strategy: LoadBalanceStrategy,
    client: Client,
    current_index: Arc<Mutex<usize>>,
}

impl RpcLoadBalancer {
    /// 🚀 Initialize RPC load balancer
    pub fn new(strategy: LoadBalanceStrategy) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoints: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            client,
            current_index: Arc::new(Mutex::new(0)),
        }
    }

    /// ➕ Add RPC endpoint
    pub async fn add_endpoint(&self, endpoint: RpcEndpoint) {
        let mut endpoints = self.endpoints.write().await;
        let mut stats = self.stats.write().await;
        
        info!("➕ Adding RPC endpoint: {} ({})", endpoint.name, endpoint.url);
        
        stats.insert(endpoint.name.clone(), RpcStats::default());
        endpoints.push(endpoint);
        
        // Sort by priority (highest first)
        endpoints.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 🎯 Get best available endpoint
    pub async fn get_best_endpoint(&self) -> Result<RpcEndpoint> {
        let endpoints = self.endpoints.read().await;
        
        if endpoints.is_empty() {
            return Err(anyhow!("No RPC endpoints available"));
        }

        match self.strategy {
            LoadBalanceStrategy::CostOptimized => self.get_cheapest_available(&endpoints).await,
            LoadBalanceStrategy::PerformanceFirst => self.get_fastest_available(&endpoints).await,
            LoadBalanceStrategy::Balanced => self.get_balanced_endpoint(&endpoints).await,
            LoadBalanceStrategy::RoundRobin => self.get_round_robin_endpoint(&endpoints).await,
        }
    }

    /// 💰 Get cheapest available endpoint
    async fn get_cheapest_available(&self, endpoints: &[RpcEndpoint]) -> Result<RpcEndpoint> {
        let stats = self.stats.read().await;
        
        let mut available_endpoints: Vec<_> = endpoints.iter()
            .filter(|ep| self.is_endpoint_available(ep, &stats))
            .collect();
        
        if available_endpoints.is_empty() {
            return Err(anyhow!("No available endpoints"));
        }
        
        // Sort by cost (cheapest first)
        available_endpoints.sort_by(|a, b| a.cost_per_request.partial_cmp(&b.cost_per_request).unwrap());
        
        Ok(available_endpoints[0].clone())
    }

    /// ⚡ Get fastest available endpoint
    async fn get_fastest_available(&self, endpoints: &[RpcEndpoint]) -> Result<RpcEndpoint> {
        let stats = self.stats.read().await;
        
        let mut available_endpoints: Vec<_> = endpoints.iter()
            .filter(|ep| self.is_endpoint_available(ep, &stats))
            .collect();
        
        if available_endpoints.is_empty() {
            return Err(anyhow!("No available endpoints"));
        }
        
        // Sort by average response time (fastest first)
        available_endpoints.sort_by(|a, b| {
            let a_stats = stats.get(&a.name).unwrap();
            let b_stats = stats.get(&b.name).unwrap();
            a_stats.avg_response_time_ms.partial_cmp(&b_stats.avg_response_time_ms).unwrap()
        });
        
        Ok(available_endpoints[0].clone())
    }

    /// ⚖️ Get balanced endpoint (cost + performance)
    async fn get_balanced_endpoint(&self, endpoints: &[RpcEndpoint]) -> Result<RpcEndpoint> {
        let stats = self.stats.read().await;
        
        let available_endpoints: Vec<_> = endpoints.iter()
            .filter(|ep| self.is_endpoint_available(ep, &stats))
            .collect();
        
        if available_endpoints.is_empty() {
            return Err(anyhow!("No available endpoints"));
        }
        
        // Calculate balanced score (lower is better)
        let mut scored_endpoints: Vec<_> = available_endpoints.iter()
            .map(|ep| {
                let ep_stats = stats.get(&ep.name).unwrap();
                let cost_score = ep.cost_per_request * 1000.0; // Normalize cost
                let perf_score = ep_stats.avg_response_time_ms.max(100.0); // Min 100ms
                let reliability_score = if ep_stats.total_requests > 0 {
                    (ep_stats.failed_requests as f64 / ep_stats.total_requests as f64) * 1000.0
                } else {
                    0.0
                };
                
                let total_score = cost_score * 0.4 + perf_score * 0.4 + reliability_score * 0.2;
                (ep, total_score)
            })
            .collect();
        
        // Sort by score (lowest first)
        scored_endpoints.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        Ok(scored_endpoints[0].0.clone())
    }

    /// 🔄 Get round-robin endpoint
    async fn get_round_robin_endpoint(&self, endpoints: &[RpcEndpoint]) -> Result<RpcEndpoint> {
        let stats = self.stats.read().await;
        let mut current_index = self.current_index.lock().await;
        
        let available_endpoints: Vec<_> = endpoints.iter()
            .filter(|ep| self.is_endpoint_available(ep, &stats))
            .collect();
        
        if available_endpoints.is_empty() {
            return Err(anyhow!("No available endpoints"));
        }
        
        let endpoint = available_endpoints[*current_index % available_endpoints.len()].clone();
        *current_index = (*current_index + 1) % available_endpoints.len();
        
        Ok(endpoint)
    }

    /// ✅ Check if endpoint is available
    fn is_endpoint_available(&self, endpoint: &RpcEndpoint, stats: &HashMap<String, RpcStats>) -> bool {
        if let Some(ep_stats) = stats.get(&endpoint.name) {
            // Check rate limiting
            let now = Instant::now();
            if now.duration_since(ep_stats.minute_window_start) < Duration::from_secs(60) {
                if ep_stats.requests_this_minute >= endpoint.max_requests_per_minute {
                    return false;
                }
            }
            
            // Check if endpoint is healthy (success rate > 80%)
            if ep_stats.total_requests > 10 {
                let success_rate = ep_stats.successful_requests as f64 / ep_stats.total_requests as f64;
                if success_rate < 0.8 {
                    return false;
                }
            }
        }
        
        true
    }

    /// 📡 Make RPC request with automatic failover
    pub async fn make_request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            let endpoint = self.get_best_endpoint().await?;
            let start_time = Instant::now();
            
            debug!("📡 Making RPC request to {} (attempt {})", endpoint.name, attempts + 1);
            
            let request_body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            });
            
            match self.client
                .post(&endpoint.url)
                .json(&request_body)
                .timeout(Duration::from_millis(endpoint.timeout_ms))
                .send()
                .await
            {
                Ok(response) => {
                    let response_time = start_time.elapsed();
                    
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json_response) => {
                                self.update_stats(&endpoint.name, true, response_time).await;
                                
                                if let Some(error) = json_response.get("error") {
                                    warn!("🚨 RPC error from {}: {}", endpoint.name, error);
                                    attempts += 1;
                                    continue;
                                }
                                
                                return Ok(json_response);
                            }
                            Err(e) => {
                                warn!("📡 Failed to parse response from {}: {}", endpoint.name, e);
                                self.update_stats(&endpoint.name, false, response_time).await;
                            }
                        }
                    } else {
                        warn!("📡 HTTP error from {}: {}", endpoint.name, response.status());
                        self.update_stats(&endpoint.name, false, response_time).await;
                    }
                }
                Err(e) => {
                    warn!("📡 Request failed to {}: {}", endpoint.name, e);
                    self.update_stats(&endpoint.name, false, start_time.elapsed()).await;
                }
            }
            
            attempts += 1;
        }
        
        Err(anyhow!("All RPC endpoints failed after {} attempts", max_attempts))
    }

    /// 📊 Update endpoint statistics
    async fn update_stats(&self, endpoint_name: &str, success: bool, response_time: Duration) {
        let mut stats = self.stats.write().await;
        let ep_stats = stats.entry(endpoint_name.to_string()).or_insert_with(RpcStats::default);
        
        let now = Instant::now();
        
        // Reset minute window if needed
        if now.duration_since(ep_stats.minute_window_start) >= Duration::from_secs(60) {
            ep_stats.requests_this_minute = 0;
            ep_stats.minute_window_start = now;
        }
        
        ep_stats.total_requests += 1;
        ep_stats.requests_this_minute += 1;
        ep_stats.last_request_time = Some(now);
        
        if success {
            ep_stats.successful_requests += 1;
        } else {
            ep_stats.failed_requests += 1;
        }
        
        // Update average response time (exponential moving average)
        let response_time_ms = response_time.as_millis() as f64;
        if ep_stats.avg_response_time_ms == 0.0 {
            ep_stats.avg_response_time_ms = response_time_ms;
        } else {
            ep_stats.avg_response_time_ms = ep_stats.avg_response_time_ms * 0.9 + response_time_ms * 0.1;
        }
    }

    /// 📊 Get load balancer statistics
    pub async fn get_stats(&self) -> HashMap<String, RpcStats> {
        self.stats.read().await.clone()
    }

    /// 🔧 Initialize default endpoints
    pub async fn init_default_endpoints(&self) {
        // Premium endpoints
        self.add_endpoint(RpcEndpoint {
            name: "Helius".to_string(),
            url: std::env::var("HELIUS_RPC_URL").unwrap_or_else(|_| "https://api.helius.xyz/v0/rpc".to_string()),
            tier: RpcTier::Premium,
            max_requests_per_minute: 1000,
            timeout_ms: 5000,
            cost_per_request: 0.001,
            priority: 10,
        }).await;
        
        self.add_endpoint(RpcEndpoint {
            name: "QuickNode".to_string(),
            url: std::env::var("QUICKNODE_RPC_URL").unwrap_or_else(|_| "https://api.quicknode.com".to_string()),
            tier: RpcTier::Premium,
            max_requests_per_minute: 500,
            timeout_ms: 5000,
            cost_per_request: 0.0015,
            priority: 9,
        }).await;
        
        // Free endpoints (backup)
        self.add_endpoint(RpcEndpoint {
            name: "Solana Public".to_string(),
            url: "https://api.mainnet-beta.solana.com".to_string(),
            tier: RpcTier::Free,
            max_requests_per_minute: 100,
            timeout_ms: 10000,
            cost_per_request: 0.0,
            priority: 5,
        }).await;
        
        info!("🔄 RPC Load Balancer initialized with default endpoints");
    }
}
