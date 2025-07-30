//! 🧠 Context Engine - Advanced AI Context Management System
//!
//! Implements TF-IDF weighting, Apriori rule mining, and context rot prevention
//! for the Cerberus Phoenix v2.0 trading system.

use crate::config::Config;
use crate::qdrant_client::{QdrantClient, QdrantPoint};
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

        // Try to search for existing TF-IDF weights
        match self.qdrant_client.search(vec![0.0; 1536], 1000).await {
            Ok(results) => {
                let mut weights = self.tf_idf_weights.write().await;
                for result in results {
                    if let (Some(signal_id), Some(weight)) = (
                        result.payload.get("signal_id").and_then(|v| v.as_str()),
                        result.payload.get("weight").and_then(|v| v.as_f64())
                    ) {
                        weights.insert(signal_id.to_string(), weight);
                    }
                }
                info!("📊 Loaded {} TF-IDF weights from storage", weights.len());
            }
            Err(e) => {
                warn!("⚠️ Could not load TF-IDF weights: {}", e);
                // Initialize with default weights
                let mut weights = self.tf_idf_weights.write().await;
                weights.insert("dev_allocation_high".to_string(), 2.5);
                weights.insert("freeze_function_present".to_string(), 2.0);
                weights.insert("suspicious_metadata".to_string(), 3.0);
                weights.insert("low_holder_count".to_string(), 1.8);
                weights.insert("team_doxxed".to_string(), 1.5);
                info!("📊 Initialized default TF-IDF weights");
            }
        }
        Ok(())
    }

    /// 💾 Load Apriori rules from storage
    async fn load_apriori_rules(&self) -> Result<()> {
        debug!("🔍 Loading Apriori rules from storage");

        // Try to search for existing Apriori rules
        match self.qdrant_client.search(vec![0.0; 1536], 1000).await {
            Ok(results) => {
                let mut rules = self.apriori_rules.write().await;
                for result in results {
                    if let Ok(rule) = serde_json::from_value::<AprioriRule>(result.payload.clone()) {
                        rules.push(rule);
                    }
                }
                info!("🔍 Loaded {} Apriori rules from storage", rules.len());
            }
            Err(e) => {
                warn!("⚠️ Could not load Apriori rules: {}", e);
                // Initialize with default high-confidence rules
                let mut rules = self.apriori_rules.write().await;

                // Rule 1: High dev allocation + suspicious metadata → AVOID
                rules.push(AprioriRule {
                    antecedent: vec!["dev_allocation_high".to_string(), "suspicious_metadata".to_string()],
                    consequent: "avoid_token".to_string(),
                    confidence: 0.95,
                    support: 0.15,
                    lift: 3.2,
                    created_at: Utc::now(),
                    last_validated: Utc::now(),
                    success_count: 47,
                    total_count: 50,
                });

                // Rule 2: Team doxxed + verified contract → CONSIDER
                rules.push(AprioriRule {
                    antecedent: vec!["team_doxxed".to_string(), "contract_verified".to_string()],
                    consequent: "consider_token".to_string(),
                    confidence: 0.78,
                    support: 0.08,
                    lift: 2.1,
                    created_at: Utc::now(),
                    last_validated: Utc::now(),
                    success_count: 23,
                    total_count: 30,
                });

                info!("🔍 Initialized {} default Apriori rules", rules.len());
            }
        }
        Ok(())
    }

    /// 💾 Save TF-IDF weights to storage
    pub async fn save_tf_idf_weights(&self) -> Result<()> {
        debug!("💾 Saving TF-IDF weights to storage");

        let weights = self.tf_idf_weights.read().await;
        let mut points = Vec::new();

        for (signal_id, weight) in weights.iter() {
            let mut payload = serde_json::Map::new();
            payload.insert("signal_id".to_string(), serde_json::Value::String(signal_id.clone()));
            payload.insert("weight".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(*weight).unwrap()));
            payload.insert("updated_at".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));

            points.push(QdrantPoint {
                id: signal_id.clone(),
                vector: vec![*weight as f32; 1536], // Simple embedding based on weight
                payload: serde_json::Value::Object(payload),
            });
        }

        if !points.is_empty() {
            match self.qdrant_client.upsert_points(points).await {
                Ok(_) => info!("💾 Saved {} TF-IDF weights to storage", weights.len()),
                Err(e) => warn!("⚠️ Failed to save TF-IDF weights: {}", e),
            }
        }

        Ok(())
    }

    /// 💾 Save Apriori rules to storage
    pub async fn save_apriori_rules(&self) -> Result<()> {
        debug!("💾 Saving Apriori rules to storage");

        let rules = self.apriori_rules.read().await;
        let mut points = Vec::new();

        for (idx, rule) in rules.iter().enumerate() {
            let rule_json = serde_json::to_value(rule)?;

            // Create embedding based on rule characteristics
            let mut embedding = vec![0.0f32; 1536];
            embedding[0] = rule.confidence as f32;
            embedding[1] = rule.support as f32;
            embedding[2] = rule.lift as f32;

            points.push(QdrantPoint {
                id: format!("rule_{}", idx),
                vector: embedding,
                payload: rule_json,
            });
        }

        if !points.is_empty() {
            match self.qdrant_client.upsert_points(points).await {
                Ok(_) => info!("💾 Saved {} Apriori rules to storage", rules.len()),
                Err(e) => warn!("⚠️ Failed to save Apriori rules: {}", e),
            }
        }

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

    /// 🔍 Apply Apriori rules for decision making
    pub async fn apply_apriori_rules(&self, signals: &[String]) -> Result<Vec<String>> {
        let rules = self.apriori_rules.read().await;
        let mut recommendations = Vec::new();

        for rule in rules.iter() {
            // Check if all antecedents are present in the signals
            let antecedents_present = rule.antecedent.iter()
                .all(|antecedent| signals.contains(antecedent));

            if antecedents_present && rule.confidence > 0.7 {
                recommendations.push(format!(
                    "{} (confidence: {:.2}, support: {:.2}, lift: {:.2})",
                    rule.consequent, rule.confidence, rule.support, rule.lift
                ));

                info!("🔍 Applied rule: {:?} → {} (confidence: {:.2})",
                      rule.antecedent, rule.consequent, rule.confidence);
            }
        }

        Ok(recommendations)
    }

    /// 📊 Update TF-IDF weights based on signal performance
    pub async fn update_tf_idf_weights(&self, signal_id: &str, performance_delta: f64) -> Result<()> {
        let mut weights = self.tf_idf_weights.write().await;

        let current_weight = *weights.get(signal_id).unwrap_or(&1.0);
        let new_weight = (current_weight + performance_delta * 0.1).max(0.1).min(5.0); // Bounded between 0.1 and 5.0

        weights.insert(signal_id.to_string(), new_weight);

        debug!("📊 Updated TF-IDF weight for {}: {:.3} → {:.3} (Δ: {:.3})",
               signal_id, current_weight, new_weight, performance_delta);

        Ok(())
    }

    /// 📈 Update Apriori rule performance
    pub async fn update_rule_performance(&self, antecedents: &[String], consequent: &str, success: bool) -> Result<()> {
        let mut rules = self.apriori_rules.write().await;

        for rule in rules.iter_mut() {
            if rule.antecedent == antecedents && rule.consequent == consequent {
                rule.total_count += 1;
                if success {
                    rule.success_count += 1;
                }
                rule.last_validated = Utc::now();

                // Recalculate confidence based on actual performance
                rule.confidence = rule.success_count as f64 / rule.total_count as f64;

                info!("📈 Updated rule performance: {:?} → {} (success: {}, confidence: {:.2})",
                      antecedents, consequent, success, rule.confidence);
                break;
            }
        }

        Ok(())
    }

    /// 🔍 Search for similar contexts
    async fn search_similar_contexts(&self, query_embeddings: &[f32], limit: u32) -> Result<Vec<ContextMemory>> {
        let memory = self.context_memory.read().await;
        let contexts: Vec<ContextMemory> = memory.values().take(limit as usize).cloned().collect();
        Ok(contexts)
    }

    /// 🧠 Create embeddings for text
    pub async fn create_embeddings(&self, text: &str) -> Result<Vec<f32>> {
        // Simple TF-IDF based embeddings for now
        // In production, this would call FinLlama API
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut embedding = vec![0.0; 1536];

        // Generate simple hash-based embeddings
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        for (i, word) in words.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            word.hash(&mut hasher);
            let hash = hasher.finish();

            let idx = (hash as usize) % 1536;
            embedding[idx] += 1.0 / (words.len() as f32);
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        Ok(embedding)
    }

    /// 🔍 Search for similar contexts (simplified for webhook integration)
    pub async fn search_similar(&self, query: &str, limit: usize) -> Result<Vec<ContextMemory>> {
        let memory = self.context_memory.read().await;
        let contexts: Vec<ContextMemory> = memory.values()
            .filter(|context| {
                // Simple text matching for now - can be enhanced with embeddings
                context.content.contains(query) ||
                context.tags.iter().any(|tag| tag.contains(query))
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(contexts)
    }

    /// 💾 Store context data (simplified for webhook integration)
    pub async fn store_context(&self, key: &str, data: &serde_json::Value) -> Result<()> {
        let context = ContextMemory {
            id: key.to_string(),
            content: data.to_string(),
            embeddings: vec![0.1; 1536], // Placeholder embeddings
            importance_score: 0.5,
            access_count: 0,
            last_accessed: Utc::now(),
            created_at: Utc::now(),
            tags: vec!["webhook_data".to_string(), "token_discovery".to_string()],
            performance_impact: 0.0,
        };

        let mut memory = self.context_memory.write().await;
        memory.insert(key.to_string(), context);

        info!("💾 Stored context: {}", key);
        Ok(())
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
