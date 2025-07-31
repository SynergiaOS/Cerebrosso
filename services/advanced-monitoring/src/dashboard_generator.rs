//! 📊 Dashboard Generator

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{config::Config, MonitoringError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub dashboard_id: String,
    pub title: String,
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub widget_id: String,
    pub title: String,
    pub chart: Chart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Chart {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
}

pub struct DashboardGenerator {
    config: Arc<Config>,
}

impl DashboardGenerator {
    pub async fn new(config: Arc<Config>) -> Result<Self, MonitoringError> {
        Ok(Self { config })
    }
}
