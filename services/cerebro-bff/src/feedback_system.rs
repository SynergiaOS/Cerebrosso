//! 📊 Feedback System - AI Trading Performance Analytics
//! 
//! System zbierania i analizy feedback z decyzji AI agentów,
//! umożliwiający continuous learning i optymalizację parametrów.

use crate::config::Config;
use crate::ai_agent::{AIDecision, AgentType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, debug, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeFeedback {
    pub id: Uuid,
    pub decision_id: Uuid,
    pub agent_type: AgentType,
    pub market_conditions: MarketSnapshot,
    pub execution_result: TradeResult,
    pub performance_metrics: PerformanceMetrics,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketSnapshot {
    pub token_address: String,
    pub price_usd: f64,
    pub volume_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub volatility: Option<f64>,
    pub market_cap: Option<f64>,
    pub holder_count: Option<i32>,
    pub dex_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TradeResult {
    pub executed: bool,
    pub transaction_hash: Option<String>,
    pub amount_sol: f64,
    pub amount_tokens: f64,
    pub execution_price: f64,
    pub gas_fee: f64,
    pub slippage: f64,
    pub market_impact: f64,
    pub execution_time: Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetrics {
    pub pnl: f64,
    pub roi_percentage: f64,
    pub execution_latency: Duration,
    pub slippage: f64,
    pub market_impact: f64,
    pub confidence_accuracy: f64,
    pub risk_adjusted_return: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentPerformance {
    pub agent_type: AgentType,
    pub success_rate: f64,
    pub avg_roi: f64,
    pub total_trades: i32,
    pub profitable_trades: i32,
    pub avg_latency_ms: f64,
    pub confidence_calibration: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: f64,
    pub optimal_parameters: HashMap<String, f64>,
    pub last_updated: DateTime<Utc>,
}

pub struct FeedbackSystem {
    config: Arc<Config>,
    db_pool: PgPool,
    performance_cache: Arc<tokio::sync::RwLock<HashMap<AgentType, AgentPerformance>>>,
}

impl FeedbackSystem {
    /// 🚀 Initialize feedback system with database connection
    pub async fn new(config: Arc<Config>, database_url: &str) -> Result<Self> {
        info!("📊 Initializing Feedback System v2.0");
        
        // Connect to TimescaleDB
        let db_pool = PgPool::connect(database_url).await?;
        
        // Run migrations if needed
        sqlx::migrate!("./migrations").run(&db_pool).await?;
        
        let performance_cache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        
        let system = FeedbackSystem {
            config,
            db_pool,
            performance_cache,
        };
        
        // Load existing performance data
        system.load_performance_cache().await?;
        
        info!("✅ Feedback System initialized successfully");
        Ok(system)
    }
    
    /// 📝 Log AI decision to database
    pub async fn log_decision(&self, decision: &AIDecision, market_conditions: &MarketSnapshot) -> Result<Uuid> {
        let decision_id = Uuid::new_v4();
        
        let agent_type_str = match decision.agent_type {
            AgentType::FastDecision => "FastDecision",
            AgentType::ContextAnalysis => "ContextAnalysis", 
            AgentType::RiskAssessment => "RiskAssessment",
            AgentType::DeepAnalysis => "DeepAnalysis",
        };
        
        let decision_data = serde_json::json!({
            "action": decision.action,
            "confidence": decision.confidence,
            "reasoning": decision.reasoning,
            "risk_assessment": decision.risk_assessment,
            "model_used": decision.model_used,
            "latency_ms": decision.latency_ms
        });
        
        let market_data = serde_json::to_value(market_conditions)?;
        
        sqlx::query(
            r#"
            INSERT INTO trade_decisions (id, agent_type, decision_data, market_conditions, confidence, reasoning)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(decision_id)
        .bind(agent_type_str)
        .bind(decision_data)
        .bind(market_data)
        .bind(decision.confidence as f64)
        .bind(decision.reasoning.clone())
        .execute(&self.db_pool)
        .await?;
        
        debug!("📝 Decision logged: {} by {}", decision_id, agent_type_str);
        Ok(decision_id)
    }
    
    /// 📈 Log trade execution result and calculate performance metrics
    pub async fn log_trade_result(&self, decision_id: Uuid, trade_result: &TradeResult) -> Result<()> {
        let execution_data = serde_json::to_value(trade_result)?;
        
        // Calculate performance metrics
        let performance_metrics = self.calculate_performance_metrics(decision_id, trade_result).await?;
        let metrics_data = serde_json::to_value(&performance_metrics)?;
        
        sqlx::query(
            r#"
            INSERT INTO trade_results (
                decision_id, execution_data, performance_metrics, pnl, roi_percentage,
                execution_latency_ms, slippage, market_impact, confidence_accuracy
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(decision_id)
        .bind(execution_data)
        .bind(metrics_data)
        .bind(performance_metrics.pnl)
        .bind(performance_metrics.roi_percentage)
        .bind(performance_metrics.execution_latency.as_millis() as i32)
        .bind(performance_metrics.slippage)
        .bind(performance_metrics.market_impact)
        .bind(performance_metrics.confidence_accuracy)
        .execute(&self.db_pool)
        .await?;
        
        info!("📈 Trade result logged for decision: {} (P&L: {:.4} SOL)", decision_id, performance_metrics.pnl);
        
        // Trigger performance update
        self.update_agent_performance_async(decision_id).await?;
        
        Ok(())
    }
    
    /// 🧮 Calculate performance metrics for a trade
    async fn calculate_performance_metrics(&self, decision_id: Uuid, trade_result: &TradeResult) -> Result<PerformanceMetrics> {
        // Get original decision for confidence comparison
        let decision_row = sqlx::query(
            "SELECT confidence, decision_data FROM trade_decisions WHERE id = $1"
        )
        .bind(decision_id)
        .fetch_one(&self.db_pool)
        .await?;
        
        let original_confidence: f64 = decision_row.get("confidence");
        
        // Calculate P&L (simplified - in real system would be more complex)
        let pnl = if trade_result.executed {
            // Simplified P&L calculation
            trade_result.amount_sol * 0.05 // Assume 5% gain for demo
        } else {
            0.0
        };
        
        let roi_percentage = if trade_result.amount_sol > 0.0 {
            (pnl / trade_result.amount_sol) * 100.0
        } else {
            0.0
        };
        
        // Calculate confidence accuracy (how well confidence predicted success)
        let confidence_accuracy = if trade_result.executed && pnl > 0.0 {
            original_confidence // Good prediction
        } else if !trade_result.executed || pnl <= 0.0 {
            1.0 - original_confidence // Poor prediction
        } else {
            0.5 // Neutral
        };
        
        // Risk-adjusted return (Sharpe-like metric)
        let risk_adjusted_return = roi_percentage / (trade_result.slippage + 0.01); // Avoid division by zero
        
        Ok(PerformanceMetrics {
            pnl,
            roi_percentage,
            execution_latency: trade_result.execution_time,
            slippage: trade_result.slippage,
            market_impact: trade_result.market_impact,
            confidence_accuracy,
            risk_adjusted_return,
        })
    }
    
    /// 📊 Get performance metrics for specific agent
    pub async fn get_agent_performance(&self, agent_type: AgentType) -> Result<Option<AgentPerformance>> {
        let cache = self.performance_cache.read().await;
        Ok(cache.get(&agent_type).cloned())
    }
    
    /// 📈 Get all agents performance summary
    pub async fn get_all_performance(&self) -> Result<HashMap<AgentType, AgentPerformance>> {
        let cache = self.performance_cache.read().await;
        Ok(cache.clone())
    }
    
    /// 🔄 Update agent performance metrics asynchronously
    async fn update_agent_performance_async(&self, decision_id: Uuid) -> Result<()> {
        // Get agent type from decision
        let agent_type_row = sqlx::query(
            "SELECT agent_type FROM trade_decisions WHERE id = $1"
        )
        .bind(decision_id)
        .fetch_one(&self.db_pool)
        .await?;

        let agent_type_str: String = agent_type_row.get("agent_type");
        
        // Call stored procedure to update performance
        sqlx::query(
            "SELECT update_agent_performance($1)"
        )
        .bind(&agent_type_str)
        .execute(&self.db_pool)
        .await?;
        
        // Refresh cache
        self.refresh_performance_cache(&agent_type_str).await?;
        
        Ok(())
    }
    
    /// 🔄 Load performance data into cache
    async fn load_performance_cache(&self) -> Result<()> {
        let rows = sqlx::query(
            r#"
            SELECT agent_type, success_rate, avg_roi, total_trades, profitable_trades,
                   avg_latency_ms, confidence_calibration, optimal_parameters, last_updated
            FROM agent_performance
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;
        
        let mut cache = self.performance_cache.write().await;
        
        for row in rows {
            let agent_type_str: String = row.get("agent_type");
            let agent_type = match agent_type_str.as_str() {
                "FastDecision" => AgentType::FastDecision,
                "ContextAnalysis" => AgentType::ContextAnalysis,
                "RiskAssessment" => AgentType::RiskAssessment,
                "DeepAnalysis" => AgentType::DeepAnalysis,
                _ => continue,
            };

            let optimal_parameters_json: Option<serde_json::Value> = row.get("optimal_parameters");
            let optimal_parameters: HashMap<String, f64> = optimal_parameters_json
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let performance = AgentPerformance {
                agent_type: agent_type.clone(),
                success_rate: row.get("success_rate"),
                avg_roi: row.get("avg_roi"),
                total_trades: row.get("total_trades"),
                profitable_trades: row.get("profitable_trades"),
                avg_latency_ms: row.get("avg_latency_ms"),
                confidence_calibration: row.get("confidence_calibration"),
                sharpe_ratio: None, // Will be calculated separately
                max_drawdown: 0.0, // Will be calculated separately
                optimal_parameters,
                last_updated: row.get("last_updated"),
            };

            cache.insert(agent_type, performance);
        }
        
        info!("📊 Loaded {} agent performance records into cache", cache.len());
        Ok(())
    }
    
    /// 🔄 Refresh specific agent performance in cache
    async fn refresh_performance_cache(&self, agent_type_str: &str) -> Result<()> {
        let row = sqlx::query(
            r#"
            SELECT success_rate, avg_roi, total_trades, profitable_trades,
                   avg_latency_ms, confidence_calibration, optimal_parameters, last_updated
            FROM agent_performance WHERE agent_type = $1
            "#
        )
        .bind(agent_type_str)
        .fetch_one(&self.db_pool)
        .await?;
        
        let agent_type = match agent_type_str {
            "FastDecision" => AgentType::FastDecision,
            "ContextAnalysis" => AgentType::ContextAnalysis,
            "RiskAssessment" => AgentType::RiskAssessment,
            "DeepAnalysis" => AgentType::DeepAnalysis,
            _ => return Ok(()),
        };
        
        let optimal_parameters_json: Option<serde_json::Value> = row.get("optimal_parameters");
        let optimal_parameters: HashMap<String, f64> = optimal_parameters_json
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        let performance = AgentPerformance {
            agent_type: agent_type.clone(),
            success_rate: row.get("success_rate"),
            avg_roi: row.get("avg_roi"),
            total_trades: row.get("total_trades"),
            profitable_trades: row.get("profitable_trades"),
            avg_latency_ms: row.get("avg_latency_ms"),
            confidence_calibration: row.get("confidence_calibration"),
            sharpe_ratio: None,
            max_drawdown: 0.0,
            optimal_parameters,
            last_updated: row.get("last_updated"),
        };
        
        let mut cache = self.performance_cache.write().await;
        cache.insert(agent_type, performance);
        
        debug!("🔄 Refreshed performance cache for {}", agent_type_str);
        Ok(())
    }
}
