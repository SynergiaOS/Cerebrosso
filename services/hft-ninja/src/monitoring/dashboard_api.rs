//! 📊 Dashboard API - Real-time Monitoring Interface
//! 
//! REST API dla real-time monitoringu wydajności bota

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, error, instrument};

use super::{TradingMetricsManager, PerformanceAnalyzer};

/// 📊 Dashboard API State
#[derive(Clone)]
pub struct DashboardState {
    pub metrics_manager: Arc<TradingMetricsManager>,
    pub performance_analyzer: Arc<PerformanceAnalyzer>,
}

/// 🔍 Query Parameters for Filtering
#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    pub hours: Option<u32>,
    pub days: Option<u32>,
    pub strategy: Option<String>,
}

/// 📊 Dashboard Summary Response
#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub current_pnl_sol: f64,
    pub current_pnl_usd: f64,
    pub win_rate_percentage: f64,
    pub total_trades: u64,
    pub trades_last_hour: u64,
    pub success_rate_percentage: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_percentage: f64,
    pub active_strategies: u32,
    pub last_trade_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
}

/// 📈 Real-time Performance Data
#[derive(Debug, Serialize)]
pub struct RealTimeData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub current_pnl: f64,
    pub trades_count: u64,
    pub win_rate: f64,
    pub execution_time_avg: f64,
    pub fees_paid: f64,
    pub active_positions: u32,
}

/// 🎯 Strategy Performance Breakdown
#[derive(Debug, Serialize)]
pub struct StrategyBreakdown {
    pub strategy_name: String,
    pub pnl_sol: f64,
    pub trades_count: u64,
    pub win_rate: f64,
    pub avg_execution_time: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
    pub risk_score: f64,
}

/// 📊 Dashboard API Implementation
pub struct DashboardApi;

impl DashboardApi {
    /// 🚀 Create Dashboard Router
    pub fn create_router(state: DashboardState) -> Router {
        Router::new()
            .route("/api/dashboard/summary", get(get_dashboard_summary))
            .route("/api/dashboard/realtime", get(get_realtime_data))
            .route("/api/dashboard/performance", get(get_performance_analysis))
            .route("/api/dashboard/strategies", get(get_strategy_breakdown))
            .route("/api/dashboard/trades", get(get_recent_trades))
            .route("/api/dashboard/metrics/prometheus", get(get_prometheus_metrics))
            .route("/api/dashboard/health", get(get_health_status))
            .with_state(state)
    }
}

/// 📊 Get Dashboard Summary
#[instrument(skip(state))]
async fn get_dashboard_summary(
    State(state): State<DashboardState>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<DashboardSummary>, StatusCode> {
    info!("📊 Dashboard summary requested");

    match state.metrics_manager.get_metrics().await {
        metrics => {
            // Calculate USD value (simplified - would need real SOL price)
            let sol_price_usd = 100.0; // Mock price
            let current_pnl_usd = metrics.total_profit_loss_sol * sol_price_usd;

            let summary = DashboardSummary {
                current_pnl_sol: metrics.total_profit_loss_sol,
                current_pnl_usd,
                win_rate_percentage: metrics.win_rate_percentage,
                total_trades: metrics.total_trades,
                trades_last_hour: metrics.trades_last_hour,
                success_rate_percentage: metrics.success_rate_percentage,
                sharpe_ratio: metrics.sharpe_ratio,
                max_drawdown_percentage: metrics.max_drawdown_percentage,
                active_strategies: metrics.strategy_performance.len() as u32,
                last_trade_timestamp: Some(metrics.last_updated),
                status: if metrics.total_trades > 0 { "Active".to_string() } else { "Idle".to_string() },
            };

            Ok(Json(summary))
        }
    }
}

/// 📈 Get Real-time Data
#[instrument(skip(state))]
async fn get_realtime_data(
    State(state): State<DashboardState>,
) -> Result<Json<RealTimeData>, StatusCode> {
    let metrics = state.metrics_manager.get_metrics().await;

    let realtime_data = RealTimeData {
        timestamp: chrono::Utc::now(),
        current_pnl: metrics.total_profit_loss_sol,
        trades_count: metrics.total_trades,
        win_rate: metrics.win_rate_percentage,
        execution_time_avg: metrics.average_execution_time_ms,
        fees_paid: metrics.total_fees_paid_sol,
        active_positions: 0, // Would need position tracking
    };

    Ok(Json(realtime_data))
}

/// 📊 Get Performance Analysis
#[instrument(skip(state))]
async fn get_performance_analysis(
    State(state): State<DashboardState>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("📊 Performance analysis requested");

    // This would need access to trade history
    // For now, return basic metrics
    let metrics = state.metrics_manager.get_metrics().await;
    
    let analysis = serde_json::json!({
        "sharpe_ratio": metrics.sharpe_ratio,
        "max_drawdown": metrics.max_drawdown_percentage,
        "win_rate": metrics.win_rate_percentage,
        "total_pnl": metrics.total_profit_loss_sol,
        "total_trades": metrics.total_trades,
        "average_profit_per_trade": metrics.average_profit_per_trade,
        "risk_warnings": [],
        "recommendations": [
            "Continue monitoring performance",
            "Consider position size optimization"
        ]
    });

    Ok(Json(analysis))
}

/// 🎯 Get Strategy Breakdown
#[instrument(skip(state))]
async fn get_strategy_breakdown(
    State(state): State<DashboardState>,
) -> Result<Json<Vec<StrategyBreakdown>>, StatusCode> {
    let metrics = state.metrics_manager.get_metrics().await;

    let breakdown: Vec<StrategyBreakdown> = metrics.strategy_performance
        .into_iter()
        .map(|(name, strategy_metrics)| {
            StrategyBreakdown {
                strategy_name: name,
                pnl_sol: strategy_metrics.profit_loss_sol,
                trades_count: strategy_metrics.total_trades,
                win_rate: strategy_metrics.win_rate,
                avg_execution_time: 0.0, // Would need detailed tracking
                best_trade: strategy_metrics.best_trade_sol,
                worst_trade: strategy_metrics.worst_trade_sol,
                risk_score: 0.5, // Would calculate based on volatility
            }
        })
        .collect();

    Ok(Json(breakdown))
}

/// 📋 Get Recent Trades
#[instrument(skip(state))]
async fn get_recent_trades(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit: usize = params.get("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(20);

    // This would need access to trade history
    // For now, return mock data
    let recent_trades = serde_json::json!({
        "trades": [],
        "total_count": 0,
        "message": "Trade history tracking not yet implemented"
    });

    Ok(Json(recent_trades))
}

/// 📊 Get Prometheus Metrics
#[instrument(skip(state))]
async fn get_prometheus_metrics(
    State(state): State<DashboardState>,
) -> Result<String, StatusCode> {
    let metrics_text = state.metrics_manager.export_prometheus_metrics();
    Ok(metrics_text)
}

/// 🏥 Get Health Status
#[instrument(skip(state))]
async fn get_health_status(
    State(state): State<DashboardState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metrics = state.metrics_manager.get_metrics().await;
    
    let health = serde_json::json!({
        "status": "healthy",
        "service": "cerberus-monitoring",
        "version": "2.0.0",
        "uptime_seconds": 0, // Would track actual uptime
        "total_trades": metrics.total_trades,
        "current_pnl": metrics.total_profit_loss_sol,
        "last_updated": metrics.last_updated,
        "components": {
            "metrics_manager": "healthy",
            "performance_analyzer": "healthy",
            "prometheus": "healthy"
        }
    });

    Ok(Json(health))
}
