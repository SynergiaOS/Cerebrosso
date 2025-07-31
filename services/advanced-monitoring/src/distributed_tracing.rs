//! 🔍 Distributed Tracing - OpenTelemetry Integration
//! 
//! Advanced distributed tracing with OpenTelemetry and Jaeger integration

use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug, instrument, Span};
use opentelemetry::{global, trace::{TraceError, Tracer}, KeyValue};
use opentelemetry_jaeger::JaegerTraceRuntime;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{config::Config, MonitoringError};

/// 🔍 Trace Context Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID
    pub parent_span_id: Option<String>,
    /// Service name
    pub service_name: String,
    /// Operation name
    pub operation_name: String,
    /// Start timestamp
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End timestamp
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration in microseconds
    pub duration_us: Option<u64>,
    /// Tags/attributes
    pub tags: HashMap<String, String>,
    /// Status
    pub status: TraceStatus,
}

/// 📊 Trace Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    /// Trace is active
    Active,
    /// Trace completed successfully
    Success,
    /// Trace completed with error
    Error(String),
    /// Trace was cancelled
    Cancelled,
}

/// 🔍 Trace Span Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    /// Span context
    pub context: TraceContext,
    /// Child spans
    pub children: Vec<TraceSpan>,
    /// Span events
    pub events: Vec<SpanEvent>,
    /// Span links
    pub links: Vec<SpanLink>,
}

/// 📝 Span Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event attributes
    pub attributes: HashMap<String, String>,
}

/// 🔗 Span Link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    /// Linked trace ID
    pub trace_id: String,
    /// Linked span ID
    pub span_id: String,
    /// Link attributes
    pub attributes: HashMap<String, String>,
}

/// 📊 Tracing Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingStats {
    /// Total traces collected
    pub total_traces: u64,
    /// Active traces count
    pub active_traces: u32,
    /// Average trace duration (microseconds)
    pub avg_duration_us: f64,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
    /// Sampling rate (0.0 - 1.0)
    pub sampling_rate: f64,
}

/// 🔍 Distributed Tracing Manager
pub struct TracingManager {
    /// Configuration
    config: Arc<Config>,
    /// OpenTelemetry tracer
    tracer: Box<dyn Tracer + Send + Sync>,
    /// Active traces
    active_traces: Arc<RwLock<HashMap<String, TraceContext>>>,
    /// Tracing statistics
    stats: Arc<RwLock<TracingStats>>,
    /// Service name
    service_name: String,
}

impl TracingManager {
    /// Creates new tracing manager
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self, MonitoringError> {
        info!("🔍 Initializing Distributed Tracing Manager...");
        
        // Initialize OpenTelemetry tracer
        let tracer = Self::init_tracer(&config).await?;
        
        let manager = Self {
            config: config.clone(),
            tracer,
            active_traces: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(TracingStats {
                total_traces: 0,
                active_traces: 0,
                avg_duration_us: 0.0,
                error_rate: 0.0,
                sampling_rate: config.tracing.sampling_rate,
            })),
            service_name: config.tracing.service_name.clone(),
        };
        
        // Start background tasks
        manager.start_trace_cleanup_task().await;
        manager.start_stats_update_task().await;
        
        info!("✅ Distributed Tracing Manager initialized");
        Ok(manager)
    }
    
    /// Starts a new trace span
    #[instrument(skip(self))]
    pub async fn start_span(
        &self,
        operation_name: &str,
        parent_context: Option<&TraceContext>,
    ) -> Result<TraceContext, MonitoringError> {
        debug!("🔍 Starting trace span: {}", operation_name);
        
        // Create span using OpenTelemetry
        let span = if let Some(parent) = parent_context {
            // Create child span
            self.tracer.start_with_context(operation_name, &self.create_otel_context(parent))
        } else {
            // Create root span
            self.tracer.start(operation_name)
        };
        
        let span_context = span.span_context();
        let trace_id = format!("{:x}", span_context.trace_id());
        let span_id = format!("{:x}", span_context.span_id());
        
        let trace_context = TraceContext {
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            parent_span_id: parent_context.map(|p| p.span_id.clone()),
            service_name: self.service_name.clone(),
            operation_name: operation_name.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_us: None,
            tags: HashMap::new(),
            status: TraceStatus::Active,
        };
        
        // Store active trace
        {
            let mut active_traces = self.active_traces.write().await;
            active_traces.insert(trace_id.clone(), trace_context.clone());
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_traces += 1;
            stats.active_traces = self.active_traces.read().await.len() as u32;
        }
        
        debug!("✅ Trace span started: {} (trace: {})", operation_name, trace_id);
        Ok(trace_context)
    }
    
    /// Finishes a trace span
    #[instrument(skip(self, context))]
    pub async fn finish_span(
        &self,
        context: &mut TraceContext,
        status: TraceStatus,
    ) -> Result<(), MonitoringError> {
        debug!("🏁 Finishing trace span: {} (trace: {})", context.operation_name, context.trace_id);
        
        let end_time = chrono::Utc::now();
        let duration_us = (end_time - context.start_time).num_microseconds().unwrap_or(0) as u64;
        
        // Update context
        context.end_time = Some(end_time);
        context.duration_us = Some(duration_us);
        context.status = status.clone();
        
        // Remove from active traces
        {
            let mut active_traces = self.active_traces.write().await;
            active_traces.remove(&context.trace_id);
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_traces = self.active_traces.read().await.len() as u32;
            
            // Update average duration
            let total_duration = stats.avg_duration_us * (stats.total_traces - 1) as f64 + duration_us as f64;
            stats.avg_duration_us = total_duration / stats.total_traces as f64;
            
            // Update error rate
            if matches!(status, TraceStatus::Error(_)) {
                let total_errors = stats.error_rate * (stats.total_traces - 1) as f64 + 1.0;
                stats.error_rate = total_errors / stats.total_traces as f64;
            } else {
                let total_errors = stats.error_rate * (stats.total_traces - 1) as f64;
                stats.error_rate = total_errors / stats.total_traces as f64;
            }
        }
        
        debug!("✅ Trace span finished: {} (duration: {}μs)", context.operation_name, duration_us);
        Ok(())
    }
    
    /// Adds tags to a trace span
    #[instrument(skip(self, context, tags))]
    pub async fn add_span_tags(
        &self,
        context: &mut TraceContext,
        tags: HashMap<String, String>,
    ) -> Result<(), MonitoringError> {
        debug!("🏷️ Adding tags to span: {} (count: {})", context.span_id, tags.len());
        
        // Add tags to context
        for (key, value) in tags {
            context.tags.insert(key, value);
        }
        
        // Update active trace
        {
            let mut active_traces = self.active_traces.write().await;
            if let Some(active_context) = active_traces.get_mut(&context.trace_id) {
                active_context.tags.extend(context.tags.clone());
            }
        }
        
        debug!("✅ Tags added to span: {}", context.span_id);
        Ok(())
    }
    
    /// Records a span event
    #[instrument(skip(self, context, attributes))]
    pub async fn record_span_event(
        &self,
        context: &TraceContext,
        event_name: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), MonitoringError> {
        debug!("📝 Recording span event: {} (span: {})", event_name, context.span_id);
        
        let _event = SpanEvent {
            name: event_name.to_string(),
            timestamp: chrono::Utc::now(),
            attributes,
        };
        
        // In a real implementation, this would record the event in OpenTelemetry
        // For now, we'll just log it
        debug!("✅ Span event recorded: {}", event_name);
        Ok(())
    }
    
    /// Gets current tracing statistics
    pub async fn get_stats(&self) -> TracingStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Lists active traces
    pub async fn list_active_traces(&self) -> Vec<TraceContext> {
        let active_traces = self.active_traces.read().await;
        active_traces.values().cloned().collect()
    }
    
    /// Initializes OpenTelemetry tracer
    async fn init_tracer(config: &Config) -> Result<Box<dyn Tracer + Send + Sync>, MonitoringError> {
        debug!("🔧 Initializing OpenTelemetry tracer...");
        
        // Initialize Jaeger tracer
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name(&config.tracing.service_name)
            .with_endpoint(&config.tracing.jaeger_endpoint)
            .with_trace_config(
                opentelemetry::sdk::trace::config()
                    .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(
                        config.tracing.sampling_rate,
                    ))
                    .with_resource(opentelemetry::sdk::Resource::new(vec![
                        KeyValue::new("service.name", config.tracing.service_name.clone()),
                        KeyValue::new("service.version", "3.0.0"),
                    ])),
            )
            .install_batch(JaegerTraceRuntime::Tokio)
            .map_err(|e| MonitoringError::DistributedTracing(format!("Failed to initialize tracer: {}", e)))?;
        
        // Set global tracer
        global::set_tracer_provider(tracer.provider().unwrap());
        
        debug!("✅ OpenTelemetry tracer initialized");
        Ok(Box::new(tracer))
    }
    
    /// Creates OpenTelemetry context from trace context
    fn create_otel_context(&self, _context: &TraceContext) -> opentelemetry::Context {
        // In a real implementation, this would create proper OpenTelemetry context
        opentelemetry::Context::current()
    }
    
    /// Starts trace cleanup task
    async fn start_trace_cleanup_task(&self) {
        let active_traces = self.active_traces.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                // Clean up old active traces (older than 1 hour)
                let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(1);
                let mut traces_to_remove = Vec::new();
                
                {
                    let traces = active_traces.read().await;
                    for (trace_id, context) in traces.iter() {
                        if context.start_time < cutoff_time {
                            traces_to_remove.push(trace_id.clone());
                        }
                    }
                }
                
                if !traces_to_remove.is_empty() {
                    let mut traces = active_traces.write().await;
                    for trace_id in traces_to_remove {
                        traces.remove(&trace_id);
                    }
                    debug!("🧹 Cleaned up old active traces");
                }
            }
        });
    }
    
    /// Starts statistics update task
    async fn start_stats_update_task(&self) {
        let stats = self.stats.clone();
        let active_traces = self.active_traces.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Update active traces count
                {
                    let mut stats_guard = stats.write().await;
                    let active_count = active_traces.read().await.len() as u32;
                    stats_guard.active_traces = active_count;
                }
            }
        });
    }
}
