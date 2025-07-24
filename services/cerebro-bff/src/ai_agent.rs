//! 🤖 AI Agent - Agent sztucznej inteligencji

use crate::config::{Config, ModelConfig};
use crate::metrics::MetricsCollector;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct AIDecision {
    pub action: String,
    pub confidence: f64,
    pub reasoning: String,
    pub risk_assessment: f64,
    pub agent_type: AgentType,
    pub latency_ms: u64,
    pub model_used: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AgentType {
    FastDecision,
    ContextAnalysis,
    RiskAssessment,
    DeepAnalysis,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::FastDecision => write!(f, "FastDecision"),
            AgentType::ContextAnalysis => write!(f, "ContextAnalysis"),
            AgentType::RiskAssessment => write!(f, "RiskAssessment"),
            AgentType::DeepAnalysis => write!(f, "DeepAnalysis"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FastDecisionResult {
    pub should_execute: bool,
    pub urgency: f64,
    pub quick_reasoning: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextAnalysisResult {
    pub market_sentiment: f64,
    pub trend_direction: String,
    pub key_factors: Vec<String>,
    pub context_confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskAssessmentResult {
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub recommended_position_size: f64,
    pub stop_loss_level: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepAnalysisResult {
    pub strategy_optimization: Vec<String>,
    pub long_term_outlook: String,
    pub confidence_intervals: HashMap<String, f64>,
    pub recommended_adjustments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkMetrics {
    pub model_name: String,
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub memory_usage_mb: f64,
    pub requests_per_second: f64,
    pub last_updated: String,
}

pub struct AIAgent {
    config: Arc<Config>,
    metrics: Arc<MetricsCollector>,
    benchmarks: Arc<tokio::sync::RwLock<HashMap<String, BenchmarkMetrics>>>,
    kv_cache: Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
}

impl AIAgent {
    pub async fn new(config: Arc<Config>, metrics: Arc<MetricsCollector>) -> Result<Self> {
        info!("🤖 Inicjalizacja Multi-Model AI Agent v2.0");

        let benchmarks = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let kv_cache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        // Inicjalizacja benchmarków dla każdego modelu
        let mut initial_benchmarks = HashMap::new();

        initial_benchmarks.insert("phi3".to_string(), BenchmarkMetrics {
            model_name: "phi3".to_string(),
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            memory_usage_mb: 0.0,
            requests_per_second: 0.0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        });

        initial_benchmarks.insert("llama3:8b-instruct".to_string(), BenchmarkMetrics {
            model_name: "llama3:8b-instruct".to_string(),
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            memory_usage_mb: 0.0,
            requests_per_second: 0.0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        });

        initial_benchmarks.insert("mistral:small".to_string(), BenchmarkMetrics {
            model_name: "mistral:small".to_string(),
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            memory_usage_mb: 0.0,
            requests_per_second: 0.0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        });

        initial_benchmarks.insert("llama3:70b-instruct".to_string(), BenchmarkMetrics {
            model_name: "llama3:70b-instruct".to_string(),
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            memory_usage_mb: 0.0,
            requests_per_second: 0.0,
            last_updated: chrono::Utc::now().to_rfc3339(),
        });

        *benchmarks.write().await = initial_benchmarks;

        info!("✅ Multi-Model AI Agent zainicjalizowany z 4 agentami");

        Ok(AIAgent {
            config,
            metrics,
            benchmarks,
            kv_cache,
        })
    }

    pub async fn check_llm_connection(&self) -> bool {
        // Sprawdzenie połączenia z FinLlama (port 11434)
        let finllama_check = self.check_ollama_connection(&self.config.ai.finllama_url).await;
        if !finllama_check {
            warn!("❌ Nie udało się połączyć z FinLlama API pod adresem: {}", self.config.ai.finllama_url);
        } else {
            info!("✅ Połączono z FinLlama API");
        }

        // Sprawdzenie połączenia z DeepSeek (port 11435)
        let deepseek_check = self.check_ollama_connection(&self.config.ai.deepseek_url).await;
        if !deepseek_check {
            warn!("❌ Nie udało się połączyć z DeepSeek API pod adresem: {}", self.config.ai.deepseek_url);
        } else {
            info!("✅ Połączono z DeepSeek API");
        }

        finllama_check && deepseek_check
    }

    /// Sprawdza połączenie z serwerem Ollama
    async fn check_ollama_connection(&self, base_url: &str) -> bool {
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", base_url);
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Połączono z Ollama API pod adresem: {}", base_url);
                    true
                } else {
                    warn!("❌ Błąd połączenia z Ollama API pod adresem {}: Status {}", base_url, response.status());
                    false
                }
            }
            Err(e) => {
                warn!("❌ Nie udało się połączyć z Ollama API pod adresem {}: {}", base_url, e);
                false
            }
        }
    }

    /// 🚀 Fast Decision Agent - Szybka egzekucja transakcji (<20ms)
    pub async fn make_fast_decision(&self, context: &str, signals: &[serde_json::Value]) -> Result<FastDecisionResult> {
        let start_time = Instant::now();
        let model_config = &self.config.ai.models.fast_decision;

        let prompt = format!(
            "FAST TRADING DECISION REQUIRED:\nContext: {}\nSignals: {}\n\nRespond with JSON: {{\"should_execute\": bool, \"urgency\": 0.0-1.0, \"quick_reasoning\": \"brief explanation\"}}",
            context,
            serde_json::to_string(signals)?
        );

        let response = self.call_ollama_api(model_config, &prompt).await?;
        let latency = start_time.elapsed().as_millis() as u64;

        // Update benchmarks
        self.update_benchmark_metrics(&model_config.name, latency, true).await;

        // Parse response
        let result: FastDecisionResult = serde_json::from_str(&response)
            .unwrap_or(FastDecisionResult {
                should_execute: false,
                urgency: 0.0,
                quick_reasoning: "Failed to parse AI response".to_string(),
            });

        debug!("🚀 Fast decision completed in {}ms", latency);
        Ok(result)
    }

    /// 🧠 Context Analysis Agent - Analiza kontekstu rynkowego (<50ms)
    pub async fn analyze_context(&self, market_data: &serde_json::Value) -> Result<ContextAnalysisResult> {
        let start_time = Instant::now();
        let model_config = &self.config.ai.models.context_analysis;

        let prompt = format!(
            "MARKET CONTEXT ANALYSIS:\nData: {}\n\nAnalyze and respond with JSON: {{\"market_sentiment\": -1.0 to 1.0, \"trend_direction\": \"up/down/sideways\", \"key_factors\": [\"factor1\", \"factor2\"], \"context_confidence\": 0.0-1.0}}",
            serde_json::to_string(market_data)?
        );

        let response = self.call_ollama_api(model_config, &prompt).await?;
        let latency = start_time.elapsed().as_millis() as u64;

        self.update_benchmark_metrics(&model_config.name, latency, true).await;

        let result: ContextAnalysisResult = serde_json::from_str(&response)
            .unwrap_or(ContextAnalysisResult {
                market_sentiment: 0.0,
                trend_direction: "unknown".to_string(),
                key_factors: vec!["Analysis failed".to_string()],
                context_confidence: 0.0,
            });

        debug!("🧠 Context analysis completed in {}ms", latency);
        Ok(result)
    }

    /// ⚠️ Risk Assessment Agent - Ocena ryzyka transakcji (<30ms)
    pub async fn assess_risk(&self, transaction_data: &serde_json::Value) -> Result<RiskAssessmentResult> {
        let start_time = Instant::now();
        let model_config = &self.config.ai.models.risk_assessment;

        let prompt = format!(
            "RISK ASSESSMENT:\nTransaction: {}\n\nEvaluate and respond with JSON: {{\"risk_score\": 0.0-1.0, \"risk_factors\": [\"factor1\"], \"recommended_position_size\": 0.0-1.0, \"stop_loss_level\": number_or_null}}",
            serde_json::to_string(transaction_data)?
        );

        let response = self.call_ollama_api(model_config, &prompt).await?;
        let latency = start_time.elapsed().as_millis() as u64;

        self.update_benchmark_metrics(&model_config.name, latency, true).await;

        let result: RiskAssessmentResult = serde_json::from_str(&response)
            .unwrap_or(RiskAssessmentResult {
                risk_score: 1.0, // High risk as fallback
                risk_factors: vec!["Assessment failed".to_string()],
                recommended_position_size: 0.1, // Conservative fallback
                stop_loss_level: None,
            });

        debug!("⚠️ Risk assessment completed in {}ms", latency);
        Ok(result)
    }

    /// 🔬 Deep Analysis Agent - Kompleksowa analiza strategii (<200ms)
    pub async fn deep_analysis(&self, strategy_data: &serde_json::Value) -> Result<DeepAnalysisResult> {
        let start_time = Instant::now();
        let model_config = &self.config.ai.models.deep_analysis;

        let prompt = format!(
            "DEEP STRATEGY ANALYSIS:\nData: {}\n\nProvide comprehensive analysis in JSON: {{\"strategy_optimization\": [\"opt1\"], \"long_term_outlook\": \"outlook\", \"confidence_intervals\": {{\"metric1\": 0.95}}, \"recommended_adjustments\": [\"adj1\"]}}",
            serde_json::to_string(strategy_data)?
        );

        let response = self.call_ollama_api(model_config, &prompt).await?;
        let latency = start_time.elapsed().as_millis() as u64;

        self.update_benchmark_metrics(&model_config.name, latency, true).await;

        let result: DeepAnalysisResult = serde_json::from_str(&response)
            .unwrap_or(DeepAnalysisResult {
                strategy_optimization: vec!["Analysis failed".to_string()],
                long_term_outlook: "Unknown".to_string(),
                confidence_intervals: HashMap::new(),
                recommended_adjustments: vec!["Retry analysis".to_string()],
            });

        debug!("🔬 Deep analysis completed in {}ms", latency);
        Ok(result)
    }

    /// 🔧 Ollama API Communication
    async fn call_ollama_api(&self, model_config: &ModelConfig, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", model_config.url);

        // Check KV cache first
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        let cache_key = format!("{}:{:x}", model_config.name, hasher.finalize());
        if model_config.enable_kv_cache {
            if let Some(cached_response) = self.kv_cache.read().await.get(&cache_key) {
                debug!("📦 Cache hit for model {}", model_config.name);
                return Ok(cached_response.as_str().unwrap_or("").to_string());
            }
        }

        let request_body = serde_json::json!({
            "model": model_config.name,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": model_config.temperature,
                "num_predict": model_config.max_tokens,
                "top_k": 40,
                "top_p": 0.9,
                "repeat_penalty": 1.1
            }
        });

        let response = client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_millis(model_config.target_latency_ms as u64 * 10)) // 10x target as timeout
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Ollama API error: {}", response.status()));
        }

        let response_json: serde_json::Value = response.json().await?;
        let generated_text = response_json["response"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Cache the response
        if model_config.enable_kv_cache && !generated_text.is_empty() {
            self.kv_cache.write().await.insert(cache_key, serde_json::Value::String(generated_text.clone()));
        }

        Ok(generated_text)
    }

    /// 📊 Update benchmark metrics
    async fn update_benchmark_metrics(&self, model_name: &str, latency_ms: u64, success: bool) {
        let mut benchmarks = self.benchmarks.write().await;
        if let Some(metrics) = benchmarks.get_mut(model_name) {
            // Update moving average latency
            metrics.avg_latency_ms = (metrics.avg_latency_ms * 0.9) + (latency_ms as f64 * 0.1);

            // Update success rate
            metrics.success_rate = (metrics.success_rate * 0.9) + (if success { 1.0 } else { 0.0 } * 0.1);

            // Update requests per second (simplified)
            metrics.requests_per_second += 1.0;

            metrics.last_updated = chrono::Utc::now().to_rfc3339();
        }
    }

    /// 📈 Get benchmark metrics
    pub async fn get_benchmark_metrics(&self) -> HashMap<String, BenchmarkMetrics> {
        self.benchmarks.read().await.clone()
    }

    /// 🧹 Clear KV cache
    pub async fn clear_cache(&self) {
        self.kv_cache.write().await.clear();
        info!("🧹 KV Cache cleared");
    }

    /// 🔄 Fallback mechanism - use simpler model if primary fails
    async fn fallback_decision(&self, context: &str) -> AIDecision {
        warn!("🔄 Using fallback decision mechanism");
        AIDecision {
            action: "hold".to_string(),
            confidence: 0.3,
            reasoning: "Fallback decision due to AI failure".to_string(),
            risk_assessment: 0.8, // High risk when AI fails
            agent_type: AgentType::FastDecision,
            latency_ms: 1,
            model_used: "fallback".to_string(),
        }
    }

    /// 🎯 Legacy compatibility method
    pub async fn make_decision(&self, context: &str, signals: &[serde_json::Value]) -> Result<AIDecision> {
        let start_time = Instant::now();

        // Use fast decision agent for legacy compatibility
        let fast_result = self.make_fast_decision(context, signals).await?;

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let action = if fast_result.should_execute { "execute" } else { "hold" };
        let model_name = self.config.ai.models.fast_decision.name.clone();

        // Record metrics
        self.metrics.record_ai_decision(
            "FastDecision",
            action,
            fast_result.urgency,
            latency_ms,
            &model_name
        );

        Ok(AIDecision {
            action: action.to_string(),
            confidence: fast_result.urgency,
            reasoning: fast_result.quick_reasoning,
            risk_assessment: 1.0 - fast_result.urgency, // Inverse relationship
            agent_type: AgentType::FastDecision,
            latency_ms,
            model_used: model_name,
        })
    }

    pub async fn analyze_patterns(&self, data: &serde_json::Value) -> Result<serde_json::Value> {
        let context_result = self.analyze_context(data).await?;
        Ok(serde_json::json!({
            "patterns_found": context_result.key_factors.len(),
            "confidence": context_result.context_confidence,
            "market_sentiment": context_result.market_sentiment,
            "trend_direction": context_result.trend_direction
        }))
    }

    pub async fn generate_optimizations(&self, improvements: &[serde_json::Value]) -> Result<serde_json::Value> {
        let strategy_data = serde_json::json!({ "improvements": improvements });
        let deep_result = self.deep_analysis(&strategy_data).await?;
        Ok(serde_json::json!({
            "optimizations": deep_result.strategy_optimization,
            "adjustments": deep_result.recommended_adjustments,
            "outlook": deep_result.long_term_outlook
        }))
    }
}
