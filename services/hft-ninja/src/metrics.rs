//! 📊 Metryki i monitoring

use axum::{http::StatusCode, response::Response};
use prometheus::{Encoder, TextEncoder, Counter, Histogram, Gauge, Registry};
use std::sync::Arc;
use once_cell::sync::Lazy;

static REGISTRY: Lazy<Registry> = Lazy::new(|| Registry::new());

static TRANSACTIONS_TOTAL: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::new("hft_ninja_transactions_total", "Total number of transactions").unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static EXECUTION_DURATION: Lazy<Histogram> = Lazy::new(|| {
    let histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new("hft_ninja_execution_duration_seconds", "Transaction execution duration")
    ).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static ACTIVE_STRATEGIES: Lazy<Gauge> = Lazy::new(|| {
    let gauge = Gauge::new("hft_ninja_active_strategies", "Number of active strategies").unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

pub struct MetricsCollector;

impl MetricsCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn increment_transactions(&self) {
        TRANSACTIONS_TOTAL.inc();
    }

    pub fn record_execution_time(&self, duration: f64) {
        EXECUTION_DURATION.observe(duration);
    }

    pub fn set_active_strategies(&self, count: f64) {
        ACTIVE_STRATEGIES.set(count);
    }
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
