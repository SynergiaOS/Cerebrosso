//! 📊 Comprehensive Metrics & Monitoring for Cerebro Phoenix v2.0
//!
//! Advanced metrics collection for AI trading system including:
//! - AI agent performance metrics
//! - Paper trading metrics
//! - Adaptive learning metrics
//! - System performance metrics

use axum::{http::StatusCode, response::Response};
use prometheus::{TextEncoder, Counter, Histogram, HistogramOpts, Gauge, Registry, CounterVec, HistogramVec, GaugeVec, Opts};
use once_cell::sync::Lazy;
use std::collections::HashMap;

static REGISTRY: Lazy<Registry> = Lazy::new(|| Registry::new());

// 🤖 AI Agent Metrics
static AI_DECISIONS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let counter = CounterVec::new(
        Opts::new("cerebro_ai_decisions_total", "Total number of AI decisions"),
        &["agent_type", "action", "confidence_level"]
    ).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static AI_DECISION_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new("cerebro_ai_decision_latency_seconds", "AI decision latency")
        .buckets(vec![0.001, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0]);
    let histogram = HistogramVec::new(opts, &["agent_type", "model"]).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static AI_CONFIDENCE_ACCURACY: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new("cerebro_ai_confidence_accuracy", "AI confidence vs actual accuracy")
        .buckets(vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]);
    let histogram = HistogramVec::new(opts, &["agent_type"]).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

// 📈 Paper Trading Metrics
static PAPER_TRADES_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let counter = CounterVec::new(
        Opts::new("cerebro_paper_trades_total", "Total number of paper trades"),
        &["portfolio_id", "trade_type", "result"]
    ).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static PAPER_TRADING_PNL: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new("cerebro_paper_trading_pnl_sol", "Paper trading P&L in SOL")
        .buckets(vec![-10.0, -5.0, -1.0, -0.1, 0.0, 0.1, 1.0, 5.0, 10.0]);
    let histogram = HistogramVec::new(opts, &["portfolio_id", "agent_type"]).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static PORTFOLIO_VALUE: Lazy<GaugeVec> = Lazy::new(|| {
    let gauge = GaugeVec::new(
        Opts::new("cerebro_portfolio_value_usd", "Current portfolio value in USD"),
        &["portfolio_id"]
    ).unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

static PORTFOLIO_ROI: Lazy<GaugeVec> = Lazy::new(|| {
    let gauge = GaugeVec::new(
        Opts::new("cerebro_portfolio_roi_percentage", "Portfolio ROI percentage"),
        &["portfolio_id"]
    ).unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

// 🧠 Adaptive Learning Metrics
static PARAMETER_OPTIMIZATIONS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    let counter = CounterVec::new(
        Opts::new("cerebro_parameter_optimizations_total", "Total parameter optimizations"),
        &["agent_type", "optimization_method"]
    ).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static OPTIMIZATION_IMPROVEMENT: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new("cerebro_optimization_improvement_percentage", "Expected improvement from optimization")
        .buckets(vec![0.0, 0.01, 0.02, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5]);
    let histogram = HistogramVec::new(opts, &["agent_type"]).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static AGENT_PERFORMANCE_SCORE: Lazy<GaugeVec> = Lazy::new(|| {
    let gauge = GaugeVec::new(
        Opts::new("cerebro_agent_performance_score", "Agent performance score (0-1)"),
        &["agent_type", "metric_type"]
    ).unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

// 📊 Market Data Metrics
static MARKET_DATA_UPDATES: Lazy<CounterVec> = Lazy::new(|| {
    let counter = CounterVec::new(
        Opts::new("cerebro_market_data_updates_total", "Total market data updates"),
        &["source", "token_address"]
    ).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static MARKET_DATA_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new("cerebro_market_data_latency_seconds", "Market data fetch latency")
        .buckets(vec![0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0]);
    let histogram = HistogramVec::new(opts, &["source"]).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

// 🔄 System Metrics
static CONTEXT_PROCESSING_DURATION: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new("cerebro_context_processing_duration_seconds", "Context processing duration");
    let histogram = Histogram::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static ACTIVE_CONTEXTS: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::new("cerebro_active_contexts", "Number of active contexts").unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

static FEEDBACK_PROCESSING_RATE: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::new("cerebro_feedback_processing_total", "Total feedback processed").unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

pub struct MetricsCollector {
    custom_metrics: HashMap<String, f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            custom_metrics: HashMap::new(),
        }
    }

    // 🤖 AI Agent Metrics
    pub fn record_ai_decision(&self, agent_type: &str, action: &str, confidence: f64, latency_ms: u64, model: &str) {
        let confidence_level = match confidence {
            c if c >= 0.8 => "high",
            c if c >= 0.6 => "medium",
            c if c >= 0.4 => "low",
            _ => "very_low",
        };

        AI_DECISIONS_TOTAL
            .with_label_values(&[agent_type, action, confidence_level])
            .inc();

        AI_DECISION_LATENCY
            .with_label_values(&[agent_type, model])
            .observe(latency_ms as f64 / 1000.0);
    }

    pub fn record_confidence_accuracy(&self, agent_type: &str, predicted_confidence: f64, actual_success: bool) {
        let accuracy = if actual_success { predicted_confidence } else { 1.0 - predicted_confidence };
        AI_CONFIDENCE_ACCURACY
            .with_label_values(&[agent_type])
            .observe(accuracy);
    }

    // 📈 Paper Trading Metrics
    pub fn record_paper_trade(&self, portfolio_id: &str, trade_type: &str, pnl: f64, agent_type: &str) {
        let result = if pnl > 0.0 { "profit" } else if pnl < 0.0 { "loss" } else { "neutral" };

        PAPER_TRADES_TOTAL
            .with_label_values(&[portfolio_id, trade_type, result])
            .inc();

        PAPER_TRADING_PNL
            .with_label_values(&[portfolio_id, agent_type])
            .observe(pnl);
    }

    pub fn update_portfolio_metrics(&self, portfolio_id: &str, value_usd: f64, roi_percentage: f64) {
        PORTFOLIO_VALUE
            .with_label_values(&[portfolio_id])
            .set(value_usd);

        PORTFOLIO_ROI
            .with_label_values(&[portfolio_id])
            .set(roi_percentage);
    }

    // 🧠 Adaptive Learning Metrics
    pub fn record_parameter_optimization(&self, agent_type: &str, method: &str, improvement: f64) {
        PARAMETER_OPTIMIZATIONS_TOTAL
            .with_label_values(&[agent_type, method])
            .inc();

        OPTIMIZATION_IMPROVEMENT
            .with_label_values(&[agent_type])
            .observe(improvement);
    }

    pub fn update_agent_performance(&self, agent_type: &str, win_rate: f64, avg_roi: f64, sharpe_ratio: f64, confidence_calibration: f64) {
        AGENT_PERFORMANCE_SCORE
            .with_label_values(&[agent_type, "win_rate"])
            .set(win_rate);

        AGENT_PERFORMANCE_SCORE
            .with_label_values(&[agent_type, "avg_roi"])
            .set(avg_roi);

        AGENT_PERFORMANCE_SCORE
            .with_label_values(&[agent_type, "sharpe_ratio"])
            .set(sharpe_ratio);

        AGENT_PERFORMANCE_SCORE
            .with_label_values(&[agent_type, "confidence_calibration"])
            .set(confidence_calibration);
    }

    // 📊 Market Data Metrics
    pub fn record_market_data_update(&self, source: &str, token_address: &str, latency_ms: u64) {
        MARKET_DATA_UPDATES
            .with_label_values(&[source, token_address])
            .inc();

        MARKET_DATA_LATENCY
            .with_label_values(&[source])
            .observe(latency_ms as f64 / 1000.0);
    }

    // 🔄 System Metrics (Legacy compatibility)
    pub fn increment_ai_decisions(&self) {
        // Legacy method - use record_ai_decision instead
        AI_DECISIONS_TOTAL
            .with_label_values(&["unknown", "unknown", "unknown"])
            .inc();
    }

    pub fn record_context_processing_time(&self, duration: f64) {
        CONTEXT_PROCESSING_DURATION.observe(duration);
    }

    pub fn set_active_contexts(&self, count: f64) {
        ACTIVE_CONTEXTS.set(count);
    }

    pub fn increment_feedback_processing(&self) {
        FEEDBACK_PROCESSING_RATE.inc();
    }

    // 📊 Custom Metrics
    pub fn set_custom_metric(&mut self, name: String, value: f64) {
        self.custom_metrics.insert(name, value);
    }

    pub fn get_custom_metrics(&self) -> &HashMap<String, f64> {
        &self.custom_metrics
    }

    // 📈 Aggregated Metrics for Dashboard
    pub fn get_system_health_score(&self) -> f64 {
        // Calculate overall system health based on various metrics
        // This would be used in Grafana dashboards
        let mut score = 1.0;

        // Factor in AI performance, trading success, etc.
        // Simplified calculation for demo
        score
    }

    pub fn get_trading_performance_summary(&self) -> TradingPerformanceSummary {
        // This would aggregate metrics for dashboard display
        TradingPerformanceSummary {
            total_trades: 0, // Would be calculated from metrics
            win_rate: 0.0,
            total_pnl: 0.0,
            best_performing_agent: "FastDecision".to_string(),
            avg_decision_latency_ms: 0.0,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct TradingPerformanceSummary {
    pub total_trades: u64,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub best_performing_agent: String,
    pub avg_decision_latency_ms: f64,
}

pub async fn export_metrics() -> Result<Response<String>, StatusCode> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => Ok(Response::builder()
            .header("content-type", "text/plain; version=0.0.4")
            .body(metrics)
            .unwrap()),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
