//! 🎯 Context Optimizer - Advanced Context Optimization
//! 
//! System for optimizing context quality using TF-IDF, clustering, and deduplication

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    config::Config,
    context_engine::{ContextRequest, ContextType},
    semantic_search::SearchResult,
    pattern_recognition::Pattern,
};

/// 🎯 Strategia optymalizacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// TF-IDF weighting
    TfIdf,
    /// Clustering podobnych kontekstów
    Clustering,
    /// Deduplikacja
    Deduplication,
    /// Shuffling haystack
    ShuffleHaystack,
    /// Quality filtering
    QualityFiltering,
}

/// 📊 Jakość kontekstu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQuality {
    /// Ogólna jakość (0.0 - 1.0)
    pub overall_score: f64,
    /// Relevance score
    pub relevance_score: f64,
    /// Diversity score
    pub diversity_score: f64,
    /// Coherence score
    pub coherence_score: f64,
}

/// 🎯 Zoptymalizowany kontekst
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedContext {
    /// Zawartość kontekstu
    pub content: String,
    /// Użyte strategie
    pub strategies_used: Vec<OptimizationStrategy>,
    /// Jakość kontekstu
    pub quality: ContextQuality,
    /// Rozmiar w tokenach
    pub token_count: usize,
    /// Źródła kontekstu
    pub source_count: usize,
}

/// 🎯 Optymalizator kontekstu
pub struct ContextOptimizer {
    config: Arc<Config>,
}

impl ContextOptimizer {
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { config })
    }
    
    /// Optymalizuje kontekst
    #[instrument(skip(self, request, search_results, patterns))]
    pub async fn optimize(
        &self,
        request: &ContextRequest,
        search_results: &[SearchResult],
        patterns: &[Pattern],
    ) -> Result<OptimizedContext> {
        debug!("🎯 Optimizing context for request: {}", request.id);
        
        let mut content_parts = Vec::new();
        let mut strategies_used = Vec::new();
        
        // Apply TF-IDF weighting if enabled
        if self.config.optimization.enable_tf_idf {
            let weighted_results = self.apply_tf_idf_weighting(search_results).await?;
            content_parts.extend(weighted_results);
            strategies_used.push(OptimizationStrategy::TfIdf);
        } else {
            content_parts.extend(search_results.iter().map(|r| r.content.clone()));
        }
        
        // Apply clustering if enabled
        if self.config.optimization.enable_clustering {
            content_parts = self.apply_clustering(&content_parts).await?;
            strategies_used.push(OptimizationStrategy::Clustering);
        }
        
        // Apply deduplication if enabled
        if self.config.optimization.enable_deduplication {
            content_parts = self.apply_deduplication(&content_parts).await?;
            strategies_used.push(OptimizationStrategy::Deduplication);
        }
        
        // Shuffle haystack if enabled
        if self.config.optimization.shuffle_haystack {
            content_parts = self.shuffle_haystack(&content_parts).await?;
            strategies_used.push(OptimizationStrategy::ShuffleHaystack);
        }
        
        // Combine content
        let content = content_parts.join("\n\n");
        
        // Calculate quality
        let quality = self.calculate_quality(&content, search_results).await?;
        
        // Apply quality filtering
        if quality.overall_score >= self.config.optimization.quality_threshold {
            strategies_used.push(OptimizationStrategy::QualityFiltering);
        }
        
        let optimized_context = OptimizedContext {
            content,
            strategies_used,
            quality,
            token_count: content_parts.iter().map(|c| c.len() / 4).sum(), // Rough estimate
            source_count: search_results.len(),
        };
        
        debug!("✅ Context optimized: quality={:.2}, sources={}", 
               optimized_context.quality.overall_score, 
               optimized_context.source_count);
        
        Ok(optimized_context)
    }
    
    /// Aplikuje TF-IDF weighting
    async fn apply_tf_idf_weighting(&self, search_results: &[SearchResult]) -> Result<Vec<String>> {
        // Simplified TF-IDF implementation
        let mut weighted_results = Vec::new();
        
        for result in search_results {
            // In a real implementation, this would calculate actual TF-IDF scores
            let weight = result.score;
            if weight >= self.config.optimization.relevance_threshold {
                weighted_results.push(result.content.clone());
            }
        }
        
        Ok(weighted_results)
    }
    
    /// Aplikuje clustering
    async fn apply_clustering(&self, content_parts: &[String]) -> Result<Vec<String>> {
        // Simplified clustering - in reality would use proper clustering algorithms
        let mut clustered = content_parts.to_vec();
        clustered.sort_by(|a, b| a.len().cmp(&b.len()));
        Ok(clustered)
    }
    
    /// Aplikuje deduplikację
    async fn apply_deduplication(&self, content_parts: &[String]) -> Result<Vec<String>> {
        let mut seen = std::collections::HashSet::new();
        let mut deduplicated = Vec::new();
        
        for content in content_parts {
            let hash = self.calculate_content_hash(content);
            if seen.insert(hash) {
                deduplicated.push(content.clone());
            }
        }
        
        Ok(deduplicated)
    }
    
    /// Shuffluje haystack
    async fn apply_shuffle_haystack(&self, content_parts: &[String]) -> Result<Vec<String>> {
        use rand::seq::SliceRandom;
        let mut shuffled = content_parts.to_vec();
        let mut rng = rand::thread_rng();
        shuffled.shuffle(&mut rng);
        Ok(shuffled)
    }
    
    /// Shuffluje haystack (alias)
    async fn shuffle_haystack(&self, content_parts: &[String]) -> Result<Vec<String>> {
        self.apply_shuffle_haystack(content_parts).await
    }
    
    /// Oblicza jakość kontekstu
    async fn calculate_quality(&self, content: &str, search_results: &[SearchResult]) -> Result<ContextQuality> {
        // Simplified quality calculation
        let relevance_score = search_results.iter().map(|r| r.score).sum::<f64>() / search_results.len() as f64;
        let diversity_score = self.calculate_diversity_score(search_results);
        let coherence_score = self.calculate_coherence_score(content);
        
        let overall_score = (relevance_score + diversity_score + coherence_score) / 3.0;
        
        Ok(ContextQuality {
            overall_score,
            relevance_score,
            diversity_score,
            coherence_score,
        })
    }
    
    /// Oblicza diversity score
    fn calculate_diversity_score(&self, search_results: &[SearchResult]) -> f64 {
        // Simplified diversity calculation
        let unique_sources = search_results.iter()
            .map(|r| &r.source_id)
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        unique_sources as f64 / search_results.len() as f64
    }
    
    /// Oblicza coherence score
    fn calculate_coherence_score(&self, content: &str) -> f64 {
        // Simplified coherence calculation based on content length and structure
        let word_count = content.split_whitespace().count();
        if word_count > 100 && word_count < 2000 {
            0.8
        } else {
            0.6
        }
    }
    
    /// Oblicza hash zawartości
    fn calculate_content_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}
