//! 🚦 Request Router - Intelligent Request Routing

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{config::Config, PerformanceError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    FastestResponse,
    LeastLoad,
    Geographic,
    ContentBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMetrics {
    pub total_requests: u64,
    pub avg_routing_time_ms: f64,
    pub routing_accuracy: f64,
}

pub struct RequestRouter {
    config: Arc<Config>,
}

impl RequestRouter {
    pub async fn new(config: Arc<Config>) -> Result<Self, PerformanceError> {
        Ok(Self { config })
    }
}
