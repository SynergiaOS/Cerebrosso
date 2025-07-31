//! 🔄 Feedback Loop - Learning from Context Usage
//! 
//! System for collecting feedback and improving context quality

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, instrument};

use crate::config::Config;

/// 📊 Dane feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackData {
    /// ID kontekstu
    pub context_id: Uuid,
    /// Czy kontekst był użyteczny
    pub was_useful: bool,
    /// Ocena jakości (0.0 - 1.0)
    pub quality_rating: f64,
    /// Ocena relevance (0.0 - 1.0)
    pub relevance_rating: f64,
    /// Komentarz użytkownika
    pub user_comment: Option<String>,
    /// Czas feedback
    pub timestamp: DateTime<Utc>,
    /// Metadane
    pub metadata: HashMap<String, String>,
}

/// 📈 Metryki uczenia się
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub total_feedback_samples: u64,
    pub average_quality_rating: f64,
    pub average_relevance_rating: f64,
    pub improvement_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// 🔄 Pętla feedback
pub struct FeedbackLoop {
    config: Arc<Config>,
    feedback_history: Arc<tokio::sync::RwLock<Vec<FeedbackData>>>,
    learning_metrics: Arc<tokio::sync::RwLock<LearningMetrics>>,
}

impl FeedbackLoop {
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🔄 Initializing FeedbackLoop...");
        
        Ok(Self {
            config,
            feedback_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            learning_metrics: Arc::new(tokio::sync::RwLock::new(LearningMetrics::default())),
        })
    }
    
    pub async fn record_feedback(&mut self, feedback: FeedbackData) -> Result<()> {
        debug!("📝 Recording feedback for context: {}", feedback.context_id);
        
        {
            let mut history = self.feedback_history.write().await;
            history.push(feedback.clone());
            
            // Limit history size
            if history.len() > 10000 {
                history.remove(0);
            }
        }
        
        // Update learning metrics
        self.update_learning_metrics(&feedback).await;
        
        Ok(())
    }
    
    pub async fn get_learning_metrics(&self) -> LearningMetrics {
        let metrics = self.learning_metrics.read().await;
        metrics.clone()
    }
    
    async fn update_learning_metrics(&self, feedback: &FeedbackData) {
        let mut metrics = self.learning_metrics.write().await;
        
        metrics.total_feedback_samples += 1;
        
        // Update averages
        let total = metrics.total_feedback_samples as f64;
        metrics.average_quality_rating = 
            (metrics.average_quality_rating * (total - 1.0) + feedback.quality_rating) / total;
        metrics.average_relevance_rating = 
            (metrics.average_relevance_rating * (total - 1.0) + feedback.relevance_rating) / total;
        
        metrics.last_updated = Utc::now();
    }
}
