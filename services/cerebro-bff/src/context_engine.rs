//! 🧠 Context Engine - Advanced AI Context Management System
//!
//! Implements TF-IDF weighting, Apriori rule mining, and context rot prevention
//! for the Cerberus Phoenix v2.0 trading system.

use crate::config::Config;
use crate::qdrant_client::QdrantClient;
use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// 🧠 Context Engine with advanced AI capabilities
pub struct ContextEngine {
    config: Arc<Config>,
    qdrant_client: Arc<QdrantClient>,
    tf_idf_weights: Arc<RwLock<HashMap<String, f64>>>,
    apriori_rules: Arc<RwLock<Vec<AprioriRule>>>,
    context_memory: Arc<RwLock<HashMap<String, ContextMemory>>>,
}

/// 📊 TF-IDF weighted signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedSignal {
    pub signal_type: String,
    pub value: f64,
    pub tf_idf_weight: f64,
    pub importance_score: f64,
    pub timestamp: DateTime<Utc>,
}

/// 🔍 Apriori rule for pattern discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AprioriRule {
    pub antecedent: Vec<String>,
    pub consequent: String,
    pub confidence: f64,
    pub support: f64,
    pub lift: f64,
    pub created_at: DateTime<Utc>,
    pub last_validated: DateTime<Utc>,
    pub success_count: u32,
    pub total_count: u32,
}

/// 🧠 Context memory with decay mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMemory {
    pub id: String,
    pub content: String,
    pub embeddings: Vec<f32>,
    pub importance_score: f64,
    pub access_count: u32,
    pub last_accessed: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub performance_impact: f64,
}

impl ContextEngine {
    /// 🚀 Initialize the advanced Context Engine
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🧠 Initializing Advanced Context Engine v2.0");

        let qdrant_client = Arc::new(QdrantClient::new(&config.qdrant.url).await?);

        // Initialize collections
        let tf_idf_weights = Arc::new(RwLock::new(HashMap::new()));
        let apriori_rules = Arc::new(RwLock::new(Vec::new()));
        let context_memory = Arc::new(RwLock::new(HashMap::new()));

        let engine = ContextEngine {
            config,
            qdrant_client,
            tf_idf_weights,
            apriori_rules,
            context_memory,
        };

        // Load existing TF-IDF weights and rules
        engine.load_tf_idf_weights().await?;
        engine.load_apriori_rules().await?;

        info!("✅ Context Engine initialized with TF-IDF and Apriori mining");
        Ok(engine)
    }

    /// 🔍 Check Qdrant connection health
    pub async fn check_qdrant_connection(&self) -> bool {
        match self.qdrant_client.health_check().await {
            Ok(_) => true,
            Err(e) => {
                warn!("❌ Qdrant connection failed: {}", e);
                false
            }
        }
    }

    /// 📊 Get total context count
    pub async fn get_context_count(&self) -> u64 {
        self.context_memory.read().await.len() as u64
    }

    /// 🧮 Calculate TF-IDF weights for signals
    pub async fn calculate_tf_idf_weights(&self, signals: &[String]) -> Result<HashMap<String, f64>> {
        let mut weights = HashMap::new();
        let total_docs = self.get_context_count().await as f64;

        if total_docs == 0.0 {
            return Ok(weights);
        }

        for signal in signals {
            // Calculate Term Frequency (TF)
            let tf = signals.iter().filter(|&s| s == signal).count() as f64 / signals.len() as f64;

            // Calculate Inverse Document Frequency (IDF)
            let doc_freq = self.get_signal_document_frequency(signal).await?;
            let idf = (total_docs / (1.0 + doc_freq)).ln();

            // TF-IDF score
            let tf_idf = tf * idf;
            weights.insert(signal.clone(), tf_idf);

            debug!("📊 TF-IDF for '{}': TF={:.4}, IDF={:.4}, Score={:.4}", signal, tf, idf, tf_idf);
        }

        Ok(weights)
    }

    /// 🔍 Discover patterns using Apriori algorithm
    pub async fn discover_apriori_patterns(&self, transactions: &[Vec<String>]) -> Result<Vec<AprioriRule>> {
        let min_support = 0.1; // 10% minimum support
        let min_confidence = 0.7; // 70% minimum confidence

        info!("🔍 Starting Apriori pattern discovery with {} transactions", transactions.len());

        // Generate frequent itemsets
        let frequent_itemsets = self.generate_frequent_itemsets(transactions, min_support).await?;

        // Generate association rules
        let mut rules = Vec::new();
        for itemset in frequent_itemsets.iter() {
            if itemset.len() >= 2 {
                let generated_rules = self.generate_association_rules(itemset, transactions, min_confidence).await?;
                rules.extend(generated_rules);
            }
        }

        info!("✅ Discovered {} Apriori rules", rules.len());

        // Update stored rules
        let mut stored_rules = self.apriori_rules.write().await;
        stored_rules.extend(rules.clone());

        Ok(rules)
    }

    /// 🧹 Context rot prevention - remove stale contexts
    pub async fn prevent_context_rot(&self) -> Result<u32> {
        let mut removed_count = 0;
        let current_time = Utc::now();
        let max_age_days = 30;
        let min_importance_threshold = 0.1;

        info!("🧹 Starting context rot prevention");

        let mut memory = self.context_memory.write().await;
        let mut to_remove = Vec::new();

        for (id, context) in memory.iter() {
            let age_days = (current_time - context.created_at).num_days();
            let is_stale = age_days > max_age_days;
            let is_low_importance = context.importance_score < min_importance_threshold;
            let is_rarely_accessed = context.access_count < 3;

            if is_stale && (is_low_importance || is_rarely_accessed) {
                to_remove.push(id.clone());
                debug!("🗑️ Marking context {} for removal (age: {} days, importance: {:.3}, access: {})",
                       id, age_days, context.importance_score, context.access_count);
            }
        }

        for id in to_remove {
            memory.remove(&id);
            removed_count += 1;
        }

        info!("✅ Context rot prevention complete: removed {} stale contexts", removed_count);
        Ok(removed_count)
    }

    /// 📊 Get signal document frequency for IDF calculation
    async fn get_signal_document_frequency(&self, signal: &str) -> Result<f64> {
        let memory = self.context_memory.read().await;
        let count = memory.values()
            .filter(|context| context.content.contains(signal))
            .count() as f64;
        Ok(count)
    }

    /// 🔍 Generate frequent itemsets using Apriori algorithm
    async fn generate_frequent_itemsets(&self, transactions: &[Vec<String>], min_support: f64) -> Result<Vec<Vec<String>>> {
        let mut frequent_itemsets = Vec::new();

        // Generate 1-itemsets
        let mut candidates: HashSet<Vec<String>> = HashSet::new();
        for transaction in transactions {
            for item in transaction {
                candidates.insert(vec![item.clone()]);
            }
        }

        let mut k = 1;
        while !candidates.is_empty() {
            let mut frequent_k_itemsets = Vec::new();

            for candidate in &candidates {
                let support = self.calculate_support(candidate, transactions);
                if support >= min_support {
                    frequent_k_itemsets.push(candidate.clone());
                }
            }

            if frequent_k_itemsets.is_empty() {
                break;
            }

            frequent_itemsets.extend(frequent_k_itemsets.clone());

            // Generate (k+1)-itemsets
            candidates = self.generate_candidates(&frequent_k_itemsets);
            k += 1;
        }

        Ok(frequent_itemsets)
    }

    /// 📏 Calculate support for an itemset
    fn calculate_support(&self, itemset: &[String], transactions: &[Vec<String>]) -> f64 {
        let count = transactions.iter()
            .filter(|transaction| itemset.iter().all(|item| transaction.contains(item)))
            .count() as f64;
        count / transactions.len() as f64
    }

    /// 🔗 Generate candidate itemsets
    fn generate_candidates(&self, frequent_itemsets: &[Vec<String>]) -> HashSet<Vec<String>> {
        let mut candidates = HashSet::new();

        for i in 0..frequent_itemsets.len() {
            for j in i + 1..frequent_itemsets.len() {
                let mut candidate = frequent_itemsets[i].clone();
                for item in &frequent_itemsets[j] {
                    if !candidate.contains(item) {
                        candidate.push(item.clone());
                    }
                }
                candidate.sort();
                if candidate.len() == frequent_itemsets[i].len() + 1 {
                    candidates.insert(candidate);
                }
            }
        }

        candidates
    }

    /// 🔗 Generate association rules from frequent itemsets
    async fn generate_association_rules(&self, itemset: &[String], transactions: &[Vec<String>], min_confidence: f64) -> Result<Vec<AprioriRule>> {
        let mut rules = Vec::new();

        for i in 1..itemset.len() {
            let antecedent: Vec<String> = itemset[..i].to_vec();
            let consequent = itemset[i].clone();

            let antecedent_support = self.calculate_support(&antecedent, transactions);
            let itemset_support = self.calculate_support(itemset, transactions);

            if antecedent_support > 0.0 {
                let confidence = itemset_support / antecedent_support;

                if confidence >= min_confidence {
                    let lift = confidence / self.calculate_support(&[consequent.clone()], transactions);

                    let rule = AprioriRule {
                        antecedent,
                        consequent,
                        confidence,
                        support: itemset_support,
                        lift,
                        created_at: Utc::now(),
                        last_validated: Utc::now(),
                        success_count: 0,
                        total_count: 0,
                    };

                    rules.push(rule);
                }
            }
        }

        Ok(rules)
    }

    /// 💾 Load TF-IDF weights from storage
    async fn load_tf_idf_weights(&self) -> Result<()> {
        debug!("📊 Loading TF-IDF weights from storage");
        Ok(())
    }

    /// 💾 Load Apriori rules from storage
    async fn load_apriori_rules(&self) -> Result<()> {
        debug!("🔍 Loading Apriori rules from storage");
        Ok(())
    }

    /// 🎯 Get weighted context for decision making
    pub async fn get_weighted_context(&self, query: &str, limit: u32) -> Result<Vec<WeightedSignal>> {
        let query_embeddings = self.create_embeddings(query).await?;
        let similar_contexts = self.search_similar_contexts(&query_embeddings, limit).await?;

        let mut weighted_signals = Vec::new();
        let tf_idf_weights = self.tf_idf_weights.read().await;

        for context in similar_contexts {
            let importance = tf_idf_weights.get(&context.id).unwrap_or(&1.0);

            let signal = WeightedSignal {
                signal_type: context.tags.join(","),
                value: context.performance_impact,
                tf_idf_weight: *importance,
                importance_score: context.importance_score,
                timestamp: context.last_accessed,
            };

            weighted_signals.push(signal);
        }

        // Sort by importance score
        weighted_signals.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());

        Ok(weighted_signals)
    }

    /// 🔍 Search for similar contexts
    async fn search_similar_contexts(&self, query_embeddings: &[f32], limit: u32) -> Result<Vec<ContextMemory>> {
        let memory = self.context_memory.read().await;
        let contexts: Vec<ContextMemory> = memory.values().take(limit as usize).cloned().collect();
        Ok(contexts)
    }

    /// 🧠 Create embeddings for text
    pub async fn create_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        // TODO: Implement actual embedding generation using FinLlama
        Ok(vec![0.1; 1536])
    }

    /// 🎯 Advanced Context Optimization - Shuffle Haystacks Strategy
    pub async fn optimize_context_for_llm(&self, signals: &[WeightedSignal]) -> Result<String> {
        info!("🎯 Optimizing context using shuffle haystacks strategy");

        // 1. Group signals by type to avoid semantic blending
        let mut risk_signals = Vec::new();
        let mut liquidity_signals = Vec::new();
        let mut team_signals = Vec::new();
        let mut timing_signals = Vec::new();

        for signal in signals {
            match signal.signal_type.as_str() {
                s if s.contains("rug_pull") || s.contains("risk") => risk_signals.push(signal),
                s if s.contains("liquidity") || s.contains("volume") => liquidity_signals.push(signal),
                s if s.contains("team") || s.contains("doxx") => team_signals.push(signal),
                s if s.contains("time") || s.contains("launch") => timing_signals.push(signal),
                _ => risk_signals.push(signal), // Default to risk category
            }
        }

        // 2. Sort each group by TF-IDF weight (highest first)
        risk_signals.sort_by(|a, b| b.tf_idf_weight.partial_cmp(&a.tf_idf_weight).unwrap());
        liquidity_signals.sort_by(|a, b| b.tf_idf_weight.partial_cmp(&a.tf_idf_weight).unwrap());
        team_signals.sort_by(|a, b| b.tf_idf_weight.partial_cmp(&a.tf_idf_weight).unwrap());
        timing_signals.sort_by(|a, b| b.tf_idf_weight.partial_cmp(&a.tf_idf_weight).unwrap());

        // 3. Build optimized context with clear separation
        let mut context = String::new();

        // High-priority risk signals first (anti-distractor strategy)
        if !risk_signals.is_empty() {
            context.push_str("🚨 CRITICAL RISK SIGNALS:\n");
            for signal in risk_signals.iter().take(3) { // Top 3 risk signals
                context.push_str(&format!(
                    "- {}: {} (confidence: {:.2}, weight: {:.3})\n",
                    signal.signal_type, signal.value, signal.importance_score, signal.tf_idf_weight
                ));
            }
            context.push_str("\n");
        }

        // Market conditions (liquidity/volume)
        if !liquidity_signals.is_empty() {
            context.push_str("💰 MARKET CONDITIONS:\n");
            for signal in liquidity_signals.iter().take(2) {
                context.push_str(&format!(
                    "- {}: {} (weight: {:.3})\n",
                    signal.signal_type, signal.value, signal.tf_idf_weight
                ));
            }
            context.push_str("\n");
        }

        // Team information
        if !team_signals.is_empty() {
            context.push_str("👥 TEAM ANALYSIS:\n");
            for signal in team_signals.iter().take(2) {
                context.push_str(&format!(
                    "- {}: {} (weight: {:.3})\n",
                    signal.signal_type, signal.value, signal.tf_idf_weight
                ));
            }
            context.push_str("\n");
        }

        // Timing signals
        if !timing_signals.is_empty() {
            context.push_str("⏰ TIMING ANALYSIS:\n");
            for signal in timing_signals.iter().take(2) {
                context.push_str(&format!(
                    "- {}: {} (weight: {:.3})\n",
                    signal.signal_type, signal.value, signal.tf_idf_weight
                ));
            }
        }

        debug!("📊 Context optimized: {} risk, {} liquidity, {} team, {} timing signals",
               risk_signals.len(), liquidity_signals.len(), team_signals.len(), timing_signals.len());

        Ok(context)
    }

    /// 🔍 Semantic Noise Filtering
    pub async fn filter_semantic_noise(&self, signals: &[WeightedSignal], threshold: f64) -> Vec<WeightedSignal> {
        info!("🔍 Filtering semantic noise with threshold: {}", threshold);

        let mut filtered = Vec::new();

        for signal in signals {
            // Filter out low-weight signals that could be distractors
            if signal.tf_idf_weight >= threshold {
                // Additional semantic filtering
                let is_meaningful = !signal.signal_type.contains("unknown")
                    && !signal.signal_type.contains("null")
                    && signal.importance_score > 0.1;

                if is_meaningful {
                    filtered.push(signal.clone());
                }
            }
        }

        debug!("🔍 Filtered {} signals from {} (removed {} noise)",
               filtered.len(), signals.len(), signals.len() - filtered.len());

        filtered
    }

    /// 🎲 Randomize Context Structure (Anti-Narrative Strategy)
    pub fn randomize_context_structure(&self, context: &str) -> String {
        let lines: Vec<&str> = context.lines().collect();
        let mut sections = Vec::new();
        let mut current_section = Vec::new();

        // Group lines into sections
        for line in lines {
            if line.starts_with("🚨") || line.starts_with("💰") ||
               line.starts_with("👥") || line.starts_with("⏰") {
                if !current_section.is_empty() {
                    sections.push(current_section.clone());
                    current_section.clear();
                }
            }
            current_section.push(line);
        }

        if !current_section.is_empty() {
            sections.push(current_section);
        }

        // Randomize section order (except keep risk signals first)
        if sections.len() > 1 {
            let risk_section = sections.remove(0); // Keep risk first
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            // Simple deterministic shuffle based on content hash
            let mut hasher = DefaultHasher::new();
            context.hash(&mut hasher);
            let seed = hasher.finish() as usize;

            let remaining_sections = sections.len();
            if seed % 2 == 0 && remaining_sections > 1 {
                sections.swap(0, remaining_sections - 1);
            }

            sections.insert(0, risk_section); // Risk always first
        }

        // Rebuild context
        sections.into_iter()
            .flatten()
            .collect::<Vec<&str>>()
            .join("\n")
    }
}
