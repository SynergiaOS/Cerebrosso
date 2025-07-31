//! 📊 Context Engine Metrics

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextMetrics {
    pub performance: PerformanceMetrics,
    pub quality: QualityMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub average_response_time_ms: f64,
    pub cache_hit_rate: f64,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub average_quality_score: f64,
    pub average_relevance_score: f64,
    pub optimization_success_rate: f64,
}

impl ContextMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_context_request(&mut self, response_time_ms: u64, quality_score: f64, relevance_score: f64) {
        self.performance.total_requests += 1;
        
        let total = self.performance.total_requests as f64;
        self.performance.average_response_time_ms = 
            (self.performance.average_response_time_ms * (total - 1.0) + response_time_ms as f64) / total;
        
        self.quality.average_quality_score = 
            (self.quality.average_quality_score * (total - 1.0) + quality_score) / total;
        
        self.quality.average_relevance_score = 
            (self.quality.average_relevance_score * (total - 1.0) + relevance_score) / total;
        
        self.performance.last_updated = Some(Utc::now());
    }
}
