//! 📊 Metryki i monitoring dla Cerebro-BFF

use axum::{http::StatusCode, response::Response};
use prometheus::{TextEncoder, Counter, Histogram, HistogramOpts, Gauge, Registry};
use once_cell::sync::Lazy;

static REGISTRY: Lazy<Registry> = Lazy::new(|| Registry::new());

static AI_DECISIONS_TOTAL: Lazy<Counter> = Lazy::new(|| {
    let counter = Counter::new("cerebro_ai_decisions_total", "Total number of AI decisions").unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

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

pub struct MetricsCollector;

impl MetricsCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn increment_ai_decisions(&self) {
        AI_DECISIONS_TOTAL.inc();
    }

    pub fn record_context_processing_time(&self, duration: f64) {
        CONTEXT_PROCESSING_DURATION.observe(duration);
    }

    pub fn set_active_contexts(&self, count: f64) {
        ACTIVE_CONTEXTS.set(count);
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
