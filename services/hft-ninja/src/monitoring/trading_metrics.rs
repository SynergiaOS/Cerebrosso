//! 📊 Trading Metrics & Performance Monitoring
//! 
//! Kompletny system monitoringu transakcji, profitowości i wydajności

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};
use chrono::{DateTime, Utc, Duration};

/// 📊 Trading Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingMetrics {
    // 💰 Profitability Metrics
    pub total_profit_loss_sol: f64,
    pub total_profit_loss_usd: f64,
    pub win_rate_percentage: f64,
    pub average_profit_per_trade: f64,
    pub max_drawdown_percentage: f64,
    pub sharpe_ratio: f64,
    
    // 📈 Trade Statistics
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub largest_win_sol: f64,
    pub largest_loss_sol: f64,
    
    // ⚡ Performance Metrics
    pub average_execution_time_ms: f64,
    pub fastest_execution_ms: u64,
    pub slowest_execution_ms: u64,
    pub failed_transactions: u64,
    pub success_rate_percentage: f64,
    
    // 💸 Cost Analysis
    pub total_fees_paid_sol: f64,
    pub average_fee_per_trade: f64,
    pub jito_tips_paid_sol: f64,
    pub gas_efficiency_score: f64,
    
    // 🎯 Strategy Performance
    pub strategy_performance: HashMap<String, StrategyMetrics>,
    
    // 📅 Time-based Metrics
    pub daily_pnl: Vec<DailyPnL>,
    pub hourly_performance: HashMap<u32, f64>, // Hour -> PnL
    
    // 🔄 Real-time Metrics
    pub current_session_pnl: f64,
    pub trades_last_hour: u64,
    pub last_updated: DateTime<Utc>,
}

/// 🎯 Strategy-specific Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub strategy_name: String,
    pub total_trades: u64,
    pub profit_loss_sol: f64,
    pub win_rate: f64,
    pub average_hold_time_minutes: f64,
    pub best_trade_sol: f64,
    pub worst_trade_sol: f64,
    pub active_positions: u32,
}

/// 📅 Daily P&L Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPnL {
    pub date: DateTime<Utc>,
    pub profit_loss_sol: f64,
    pub profit_loss_usd: f64,
    pub trades_count: u64,
    pub win_rate: f64,
    pub fees_paid: f64,
}

/// 💹 Individual Trade Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub trade_id: String,
    pub token_address: String,
    pub token_symbol: String,
    pub strategy: String,
    pub action: TradeAction,
    pub amount_sol: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub profit_loss_sol: f64,
    pub profit_loss_percentage: f64,
    pub fees_paid_sol: f64,
    pub jito_tip_sol: f64,
    pub execution_time_ms: u64,
    pub slippage_percentage: f64,
    pub timestamp: DateTime<Utc>,
    pub exit_timestamp: Option<DateTime<Utc>>,
    pub ai_confidence: f64,
    pub risk_score: f64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 🎯 Trade Actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
    StopLoss,
    TakeProfit,
}

/// 📊 Trading Metrics Manager
pub struct TradingMetricsManager {
    metrics: Arc<RwLock<TradingMetrics>>,
    trade_history: Arc<RwLock<Vec<TradeRecord>>>,
    prometheus_metrics: PrometheusMetrics,
}

/// 📈 Prometheus Metrics Integration
pub struct PrometheusMetrics {
    // Counters
    pub total_trades_counter: prometheus::Counter,
    pub winning_trades_counter: prometheus::Counter,
    pub failed_trades_counter: prometheus::Counter,
    
    // Gauges
    pub current_pnl_gauge: prometheus::Gauge,
    pub win_rate_gauge: prometheus::Gauge,
    pub active_positions_gauge: prometheus::Gauge,
    
    // Histograms
    pub execution_time_histogram: prometheus::Histogram,
    pub trade_size_histogram: prometheus::Histogram,
    pub profit_loss_histogram: prometheus::Histogram,
}

impl TradingMetricsManager {
    /// 🚀 Initialize Trading Metrics Manager
    pub fn new() -> Result<Self> {
        info!("📊 Initializing Trading Metrics Manager");

        let metrics = TradingMetrics {
            total_profit_loss_sol: 0.0,
            total_profit_loss_usd: 0.0,
            win_rate_percentage: 0.0,
            average_profit_per_trade: 0.0,
            max_drawdown_percentage: 0.0,
            sharpe_ratio: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            largest_win_sol: 0.0,
            largest_loss_sol: 0.0,
            average_execution_time_ms: 0.0,
            fastest_execution_ms: u64::MAX,
            slowest_execution_ms: 0,
            failed_transactions: 0,
            success_rate_percentage: 0.0,
            total_fees_paid_sol: 0.0,
            average_fee_per_trade: 0.0,
            jito_tips_paid_sol: 0.0,
            gas_efficiency_score: 0.0,
            strategy_performance: HashMap::new(),
            daily_pnl: Vec::new(),
            hourly_performance: HashMap::new(),
            current_session_pnl: 0.0,
            trades_last_hour: 0,
            last_updated: Utc::now(),
        };

        let prometheus_metrics = Self::init_prometheus_metrics()?;

        Ok(TradingMetricsManager {
            metrics: Arc::new(RwLock::new(metrics)),
            trade_history: Arc::new(RwLock::new(Vec::new())),
            prometheus_metrics,
        })
    }

    /// 📈 Initialize Prometheus Metrics
    fn init_prometheus_metrics() -> Result<PrometheusMetrics> {
        use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts};

        let total_trades_counter = Counter::with_opts(
            Opts::new("cerberus_total_trades", "Total number of trades executed")
        )?;

        let winning_trades_counter = Counter::with_opts(
            Opts::new("cerberus_winning_trades", "Number of profitable trades")
        )?;

        let failed_trades_counter = Counter::with_opts(
            Opts::new("cerberus_failed_trades", "Number of failed trades")
        )?;

        let current_pnl_gauge = Gauge::with_opts(
            Opts::new("cerberus_current_pnl_sol", "Current P&L in SOL")
        )?;

        let win_rate_gauge = Gauge::with_opts(
            Opts::new("cerberus_win_rate_percentage", "Win rate percentage")
        )?;

        let active_positions_gauge = Gauge::with_opts(
            Opts::new("cerberus_active_positions", "Number of active positions")
        )?;

        let execution_time_histogram = Histogram::with_opts(
            HistogramOpts::new("cerberus_execution_time_ms", "Trade execution time in milliseconds")
                .buckets(vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0])
        )?;

        let trade_size_histogram = Histogram::with_opts(
            HistogramOpts::new("cerberus_trade_size_sol", "Trade size in SOL")
                .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0])
        )?;

        let profit_loss_histogram = Histogram::with_opts(
            HistogramOpts::new("cerberus_profit_loss_sol", "Profit/Loss per trade in SOL")
                .buckets(vec![-10.0, -1.0, -0.1, 0.0, 0.1, 1.0, 10.0])
        )?;

        // Register metrics with Prometheus
        prometheus::register(Box::new(total_trades_counter.clone()))?;
        prometheus::register(Box::new(winning_trades_counter.clone()))?;
        prometheus::register(Box::new(failed_trades_counter.clone()))?;
        prometheus::register(Box::new(current_pnl_gauge.clone()))?;
        prometheus::register(Box::new(win_rate_gauge.clone()))?;
        prometheus::register(Box::new(active_positions_gauge.clone()))?;
        prometheus::register(Box::new(execution_time_histogram.clone()))?;
        prometheus::register(Box::new(trade_size_histogram.clone()))?;
        prometheus::register(Box::new(profit_loss_histogram.clone()))?;

        Ok(PrometheusMetrics {
            total_trades_counter,
            winning_trades_counter,
            failed_trades_counter,
            current_pnl_gauge,
            win_rate_gauge,
            active_positions_gauge,
            execution_time_histogram,
            trade_size_histogram,
            profit_loss_histogram,
        })
    }

    /// 📊 Record New Trade
    #[instrument(skip(self))]
    pub async fn record_trade(&self, trade: TradeRecord) -> Result<()> {
        info!("📊 Recording trade: {:?} {} {} SOL",
              trade.action, trade.token_symbol, trade.amount_sol);

        // Update Prometheus metrics
        self.prometheus_metrics.total_trades_counter.inc();
        self.prometheus_metrics.execution_time_histogram.observe(trade.execution_time_ms as f64);
        self.prometheus_metrics.trade_size_histogram.observe(trade.amount_sol);
        self.prometheus_metrics.profit_loss_histogram.observe(trade.profit_loss_sol);

        if trade.profit_loss_sol > 0.0 {
            self.prometheus_metrics.winning_trades_counter.inc();
        }

        if !trade.success {
            self.prometheus_metrics.failed_trades_counter.inc();
        }

        // Update internal metrics
        {
            let mut metrics = self.metrics.write().await;
            let mut history = self.trade_history.write().await;

            // Add to history
            history.push(trade.clone());

            // Update aggregate metrics
            metrics.total_trades += 1;
            metrics.total_profit_loss_sol += trade.profit_loss_sol;
            metrics.total_fees_paid_sol += trade.fees_paid_sol;
            metrics.jito_tips_paid_sol += trade.jito_tip_sol;
            metrics.current_session_pnl += trade.profit_loss_sol;

            if trade.profit_loss_sol > 0.0 {
                metrics.winning_trades += 1;
                if trade.profit_loss_sol > metrics.largest_win_sol {
                    metrics.largest_win_sol = trade.profit_loss_sol;
                }
            } else {
                metrics.losing_trades += 1;
                if trade.profit_loss_sol < metrics.largest_loss_sol {
                    metrics.largest_loss_sol = trade.profit_loss_sol;
                }
            }

            // Update execution time metrics
            if trade.execution_time_ms < metrics.fastest_execution_ms {
                metrics.fastest_execution_ms = trade.execution_time_ms;
            }
            if trade.execution_time_ms > metrics.slowest_execution_ms {
                metrics.slowest_execution_ms = trade.execution_time_ms;
            }

            // Calculate derived metrics
            metrics.win_rate_percentage = if metrics.total_trades > 0 {
                (metrics.winning_trades as f64 / metrics.total_trades as f64) * 100.0
            } else {
                0.0
            };

            metrics.average_profit_per_trade = if metrics.total_trades > 0 {
                metrics.total_profit_loss_sol / metrics.total_trades as f64
            } else {
                0.0
            };

            metrics.success_rate_percentage = if metrics.total_trades > 0 {
                ((metrics.total_trades - metrics.failed_transactions) as f64 / metrics.total_trades as f64) * 100.0
            } else {
                0.0
            };

            metrics.last_updated = Utc::now();

            // Update Prometheus gauges
            self.prometheus_metrics.current_pnl_gauge.set(metrics.total_profit_loss_sol);
            self.prometheus_metrics.win_rate_gauge.set(metrics.win_rate_percentage);
        }

        info!("✅ Trade recorded successfully");
        Ok(())
    }

    /// 📊 Get Current Metrics
    pub async fn get_metrics(&self) -> TradingMetrics {
        self.metrics.read().await.clone()
    }

    /// 📈 Get Performance Summary
    pub async fn get_performance_summary(&self) -> Result<serde_json::Value> {
        let metrics = self.metrics.read().await;
        let history = self.trade_history.read().await;

        Ok(serde_json::json!({
            "summary": {
                "total_pnl_sol": metrics.total_profit_loss_sol,
                "win_rate": format!("{:.1}%", metrics.win_rate_percentage),
                "total_trades": metrics.total_trades,
                "success_rate": format!("{:.1}%", metrics.success_rate_percentage),
                "avg_execution_time": format!("{:.1}ms", metrics.average_execution_time_ms)
            },
            "recent_trades": history.iter().rev().take(10).collect::<Vec<_>>(),
            "strategy_breakdown": metrics.strategy_performance,
            "cost_analysis": {
                "total_fees_sol": metrics.total_fees_paid_sol,
                "jito_tips_sol": metrics.jito_tips_paid_sol,
                "avg_fee_per_trade": metrics.average_fee_per_trade
            }
        }))
    }

    /// 🎯 Export Metrics for Prometheus
    pub fn export_prometheus_metrics(&self) -> String {
        use prometheus::{Encoder, TextEncoder};
        
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        encoder.encode_to_string(&metric_families).unwrap_or_default()
    }
}
