//! 📊 Metryki i monitoring

use axum::{http::StatusCode, response::Response};
use prometheus::{Encoder, TextEncoder, Counter, Histogram, Gauge, Registry};
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};

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

#[derive(Default)]
pub struct Metrics {
    pub webhooks_received: AtomicU64,
    pub tokens_analyzed: AtomicU64,
    pub promising_tokens_found: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_webhooks(&self) {
        self.webhooks_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_tokens_analyzed(&self) {
        self.tokens_analyzed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_promising_tokens(&self) {
        self.promising_tokens_found.fetch_add(1, Ordering::Relaxed);
    }

    pub fn to_prometheus(&self) -> String {
        format!(
            "# HELP webhooks_received_total Total webhooks received\n\
             # TYPE webhooks_received_total counter\n\
             webhooks_received_total {}\n\
             # HELP tokens_analyzed_total Total tokens analyzed\n\
             # TYPE tokens_analyzed_total counter\n\
             tokens_analyzed_total {}\n\
             # HELP promising_tokens_found_total Promising tokens found\n\
             # TYPE promising_tokens_found_total counter\n\
             promising_tokens_found_total {}\n",
            self.webhooks_received.load(Ordering::Relaxed),
            self.tokens_analyzed.load(Ordering::Relaxed),
            self.promising_tokens_found.load(Ordering::Relaxed)
        )
    }
}
