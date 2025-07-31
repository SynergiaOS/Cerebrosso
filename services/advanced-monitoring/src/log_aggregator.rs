//! 📝 Log Aggregator

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{config::Config, MonitoringError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub level: Option<LogLevel>,
    pub service: Option<String>,
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
}

pub struct LogAggregator {
    config: Arc<Config>,
}

impl LogAggregator {
    pub async fn new(config: Arc<Config>) -> Result<Self, MonitoringError> {
        Ok(Self { config })
    }
}
