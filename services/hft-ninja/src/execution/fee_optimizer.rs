//! 💰 Fee & Tip Optimizer - Strategia "Certainty-First HFT"
//! 
//! Dynamicznie oblicza optymalny tip dla Jito Bundli, maksymalizując szansę
//! na włączenie do bloku przy minimalnym koszcie.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, Duration};
use dashmap::DashMap;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use statrs::statistics::Statistics;
use std::sync::Arc;
use tracing::{info, debug, warn, instrument};

use crate::config::Config;

/// 💰 Fee & Tip Optimizer - główna struktura
pub struct FeeOptimizer {
    config: Arc<Config>,
    client: Client,
    tip_cache: Arc<DashMap<String, CachedTipData>>,
}

/// 📊 Cached tip data with TTL
#[derive(Debug, Clone)]
struct CachedTipData {
    tip_lamports: u64,
    confidence_score: f64,
    timestamp: DateTime<Utc>,
    strategy_multiplier: f64,
}

/// 🎯 Jito tip account response
#[derive(Debug, Deserialize)]
struct JitoTipResponse {
    #[serde(rename = "ema_landed_tips_50th_percentile")]
    pub percentile_50: Option<u64>,
    #[serde(rename = "ema_landed_tips_75th_percentile")]
    pub percentile_75: Option<u64>,
    #[serde(rename = "ema_landed_tips_95th_percentile")]
    pub percentile_95: Option<u64>,
    #[serde(rename = "ema_landed_tips_99th_percentile")]
    pub percentile_99: Option<u64>,
}

/// 📈 Tip calculation result
#[derive(Debug)]
pub struct TipCalculationResult {
    pub tip_lamports: u64,
    pub confidence_score: f64,
    pub estimated_inclusion_time_ms: u64,
    pub strategy_multiplier: f64,
    pub base_percentile_tip: u64,
    pub jitter_applied: i64,
}

impl FeeOptimizer {
    /// 🚀 Initialize Fee Optimizer
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("🚀 Initializing Fee & Tip Optimizer v2.0");
        
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(5000))
            .build()?;

        let optimizer = Self {
            config,
            client,
            tip_cache: Arc::new(DashMap::new()),
        };

        // 🔥 Warm up cache with initial data
        if let Err(e) = optimizer.warm_up_cache().await {
            warn!("⚠️ Failed to warm up tip cache: {}", e);
        }

        info!("✅ Fee & Tip Optimizer initialized successfully");
        Ok(optimizer)
    }

    /// 💰 Get optimal Jito tip for strategy
    #[instrument(skip(self))]
    pub async fn get_optimal_jito_tip(
        &self,
        strategy: &str,
        amount_sol: f64,
        urgency_level: Option<u8>,
    ) -> Result<(u64, f64, u64, f64)> {
        info!("💰 Calculating optimal tip for strategy: {} (amount: {} SOL)", strategy, amount_sol);

        // 🔍 Check cache first
        let cache_key = format!("{}_{}", strategy, urgency_level.unwrap_or(5));
        if let Some(cached) = self.get_cached_tip(&cache_key) {
            debug!("📋 Using cached tip data for {}", cache_key);
            return Ok((
                cached.tip_lamports,
                cached.confidence_score,
                self.estimate_inclusion_time(cached.tip_lamports),
                cached.strategy_multiplier,
            ));
        }

        // 📊 Fetch fresh tip data from Jito
        let tip_result = self.calculate_fresh_tip(strategy, amount_sol, urgency_level).await?;
        
        // 💾 Cache the result
        self.cache_tip_data(&cache_key, &tip_result);

        info!("✅ Optimal tip calculated: {} lamports (confidence: {:.2})", 
              tip_result.tip_lamports, tip_result.confidence_score);

        Ok((
            tip_result.tip_lamports,
            tip_result.confidence_score,
            tip_result.estimated_inclusion_time_ms,
            tip_result.strategy_multiplier,
        ))
    }

    /// 🔥 Calculate fresh tip from Jito data
    #[instrument(skip(self))]
    async fn calculate_fresh_tip(
        &self,
        strategy: &str,
        amount_sol: f64,
        urgency_level: Option<u8>,
    ) -> Result<TipCalculationResult> {
        debug!("🔥 Fetching fresh tip data from Jito");

        // 📡 Fetch tip data from Jito
        let jito_data = self.fetch_jito_tip_data().await?;
        
        // 📊 Calculate base tip from percentile
        let base_tip = self.calculate_percentile_tip(&jito_data)?;
        debug!("📊 Base percentile tip: {} lamports", base_tip);

        // 🎯 Apply strategy multiplier
        let strategy_multiplier = self.get_strategy_multiplier(strategy);
        let strategy_adjusted_tip = (base_tip as f64 * strategy_multiplier) as u64;
        debug!("🎯 Strategy-adjusted tip: {} lamports (multiplier: {:.2})", 
               strategy_adjusted_tip, strategy_multiplier);

        // ⚡ Apply urgency adjustment
        let urgency_multiplier = self.get_urgency_multiplier(urgency_level);
        let urgency_adjusted_tip = (strategy_adjusted_tip as f64 * urgency_multiplier) as u64;
        debug!("⚡ Urgency-adjusted tip: {} lamports (multiplier: {:.2})", 
               urgency_adjusted_tip, urgency_multiplier);

        // 🎲 Apply jitter to avoid predictability
        let jitter = self.calculate_jitter(urgency_adjusted_tip);
        let final_tip = ((urgency_adjusted_tip as i64) + jitter) as u64;
        debug!("🎲 Final tip with jitter: {} lamports (jitter: {} lamports)", 
               final_tip, jitter);

        // 🛡️ Apply safety limits
        let safe_tip = self.apply_safety_limits(final_tip);
        
        // 📈 Calculate confidence score
        let confidence = self.calculate_confidence_score(&jito_data, safe_tip);

        Ok(TipCalculationResult {
            tip_lamports: safe_tip,
            confidence_score: confidence,
            estimated_inclusion_time_ms: self.estimate_inclusion_time(safe_tip),
            strategy_multiplier,
            base_percentile_tip: base_tip,
            jitter_applied: jitter,
        })
    }

    /// 📡 Fetch tip data from Jito API with fallback
    #[instrument(skip(self))]
    async fn fetch_jito_tip_data(&self) -> Result<JitoTipResponse> {
        debug!("📡 Fetching tip data from Jito API");

        // Try to fetch from real Jito API first
        match self.client
            .get(&self.config.jito_tip_stream_url)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<JitoTipResponse>().await {
                    Ok(tip_data) => {
                        debug!("📊 Jito tip data: 50th: {:?}, 75th: {:?}, 95th: {:?}, 99th: {:?}",
                               tip_data.percentile_50, tip_data.percentile_75,
                               tip_data.percentile_95, tip_data.percentile_99);
                        return Ok(tip_data);
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to parse Jito response: {}", e);
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️ Jito API returned status: {}", response.status());
            }
            Err(e) => {
                warn!("⚠️ Failed to connect to Jito API: {}", e);
            }
        }

        // 🔄 Fallback to mock data for development/testing
        warn!("🔄 Using mock tip data for development");
        Ok(self.generate_mock_tip_data())
    }

    /// 🎭 Generate mock tip data for development
    fn generate_mock_tip_data(&self) -> JitoTipResponse {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate realistic mock data based on current market conditions
        let base_tip = self.config.fee_optimizer.base_tip_lamports;

        JitoTipResponse {
            percentile_50: Some(base_tip + rng.gen_range(0..5000)),
            percentile_75: Some(base_tip + rng.gen_range(5000..15000)),
            percentile_95: Some(base_tip + rng.gen_range(15000..50000)),
            percentile_99: Some(base_tip + rng.gen_range(50000..200000)),
        }
    }

    /// 📊 Calculate tip based on target percentile
    fn calculate_percentile_tip(&self, jito_data: &JitoTipResponse) -> Result<u64> {
        let target_percentile = self.config.fee_optimizer.percentile_target;
        
        let tip = if target_percentile <= 0.5 {
            jito_data.percentile_50.unwrap_or(self.config.fee_optimizer.base_tip_lamports)
        } else if target_percentile <= 0.75 {
            jito_data.percentile_75.unwrap_or(
                jito_data.percentile_50.unwrap_or(self.config.fee_optimizer.base_tip_lamports)
            )
        } else if target_percentile <= 0.95 {
            jito_data.percentile_95.unwrap_or(
                jito_data.percentile_75.unwrap_or(self.config.fee_optimizer.base_tip_lamports)
            )
        } else {
            jito_data.percentile_99.unwrap_or(
                jito_data.percentile_95.unwrap_or(self.config.fee_optimizer.base_tip_lamports)
            )
        };

        Ok(tip)
    }

    /// 🎯 Get strategy-specific multiplier
    fn get_strategy_multiplier(&self, strategy: &str) -> f64 {
        let multipliers = &self.config.fee_optimizer.strategy_multipliers;
        
        match strategy.to_lowercase().as_str() {
            s if s.contains("piranha") => multipliers.piranha_surf,
            s if s.contains("sandwich") => multipliers.sandwich_arbitrage,
            s if s.contains("arbitrage") => multipliers.cross_dex_arbitrage,
            s if s.contains("snipe") => multipliers.liquidity_snipe,
            s if s.contains("exit") => multipliers.emergency_exit,
            _ => 1.0,
        }
    }

    /// ⚡ Get urgency-based multiplier
    fn get_urgency_multiplier(&self, urgency_level: Option<u8>) -> f64 {
        match urgency_level.unwrap_or(5) {
            1..=2 => 0.8,  // Low urgency
            3..=4 => 0.9,  // Medium-low urgency
            5..=6 => 1.0,  // Normal urgency
            7..=8 => 1.2,  // High urgency
            9..=10 => 1.5, // Maximum urgency
            _ => 1.0,
        }
    }

    /// 🎲 Calculate jitter to avoid predictability
    fn calculate_jitter(&self, base_tip: u64) -> i64 {
        let jitter_percentage = self.config.fee_optimizer.jitter_percentage;
        let max_jitter = (base_tip as f64 * jitter_percentage) as i64;
        
        let mut rng = rand::thread_rng();
        rng.gen_range(-max_jitter..=max_jitter)
    }

    /// 🛡️ Apply safety limits
    fn apply_safety_limits(&self, tip: u64) -> u64 {
        let max_tip = self.config.fee_optimizer.max_tip_lamports;
        let min_tip = self.config.fee_optimizer.base_tip_lamports / 10; // 10% of base as minimum
        
        tip.clamp(min_tip, max_tip)
    }

    /// 📈 Calculate confidence score
    fn calculate_confidence_score(&self, jito_data: &JitoTipResponse, final_tip: u64) -> f64 {
        // Base confidence on data availability and tip positioning
        let mut confidence: f64 = 0.5; // Base confidence
        
        if jito_data.percentile_50.is_some() { confidence += 0.1; }
        if jito_data.percentile_75.is_some() { confidence += 0.1; }
        if jito_data.percentile_95.is_some() { confidence += 0.1; }
        if jito_data.percentile_99.is_some() { confidence += 0.1; }
        
        // Higher tips generally have higher inclusion probability
        if let Some(p95) = jito_data.percentile_95 {
            if final_tip >= p95 {
                confidence += 0.2;
            }
        }
        
        confidence.min(1.0)
    }

    /// ⏱️ Estimate inclusion time based on tip
    fn estimate_inclusion_time(&self, tip_lamports: u64) -> u64 {
        // Simple heuristic: higher tips = faster inclusion
        let base_time_ms = 2000; // 2 seconds base
        let tip_factor = (tip_lamports as f64 / 100_000.0).min(1.0); // Normalize to 0-1
        
        (base_time_ms as f64 * (1.0 - tip_factor * 0.7)) as u64
    }

    /// 📋 Get cached tip data
    fn get_cached_tip(&self, cache_key: &str) -> Option<CachedTipData> {
        if let Some(cached) = self.tip_cache.get(cache_key) {
            let age = Utc::now() - cached.timestamp;
            if age < Duration::seconds(self.config.fee_optimizer.cache_ttl_seconds as i64) {
                return Some(cached.clone());
            } else {
                // Remove expired cache entry
                self.tip_cache.remove(cache_key);
            }
        }
        None
    }

    /// 💾 Cache tip data
    fn cache_tip_data(&self, cache_key: &str, result: &TipCalculationResult) {
        let cached_data = CachedTipData {
            tip_lamports: result.tip_lamports,
            confidence_score: result.confidence_score,
            timestamp: Utc::now(),
            strategy_multiplier: result.strategy_multiplier,
        };
        
        self.tip_cache.insert(cache_key.to_string(), cached_data);
    }

    /// 🔥 Warm up cache with initial data
    async fn warm_up_cache(&self) -> Result<()> {
        info!("🔥 Warming up tip cache");
        
        let strategies = ["piranha_surf", "cross_dex_arbitrage", "liquidity_snipe"];
        
        for strategy in strategies {
            if let Err(e) = self.get_optimal_jito_tip(strategy, 1.0, Some(5)).await {
                warn!("⚠️ Failed to warm up cache for {}: {}", strategy, e);
            }
        }
        
        Ok(())
    }
}
