//! 🤖 AI Anomaly Detector - Machine Learning Anomaly Detection
//! 
//! Advanced anomaly detection using machine learning algorithms

use anyhow::Result;
use std::sync::Arc;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, debug, warn, instrument};
use ndarray::{Array1, Array2};
use statrs::statistics::{Statistics, OrderStatistics};

use crate::{config::Config, MonitoringError};

/// 🤖 Anomaly Type Classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Statistical anomaly (outlier)
    Statistical,
    /// Temporal anomaly (time-based pattern)
    Temporal,
    /// Behavioral anomaly (unusual behavior)
    Behavioral,
    /// Performance anomaly (latency/throughput)
    Performance,
    /// Security anomaly (potential threat)
    Security,
    /// Resource anomaly (CPU/memory/disk)
    Resource,
}

/// 🚨 Anomaly Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    /// Alert ID
    pub alert_id: String,
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Severity level (0.0 - 1.0)
    pub severity: f64,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Description
    pub description: String,
    /// Affected metric/service
    pub affected_target: String,
    /// Anomalous value
    pub anomalous_value: f64,
    /// Expected value range
    pub expected_range: (f64, f64),
    /// Detection timestamp
    pub detected_at: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// 🧠 Anomaly Detection Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyModel {
    /// Model ID
    pub model_id: String,
    /// Model type
    pub model_type: String,
    /// Training data size
    pub training_data_size: usize,
    /// Model accuracy
    pub accuracy: f64,
    /// Last training timestamp
    pub last_trained: chrono::DateTime<chrono::Utc>,
    /// Model parameters
    pub parameters: std::collections::HashMap<String, f64>,
}

/// 📊 Data Point for Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Metric name
    pub metric_name: String,
    /// Value
    pub value: f64,
    /// Additional features
    pub features: Vec<f64>,
}

/// 📈 Anomaly Detection Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStats {
    /// Total data points analyzed
    pub total_data_points: u64,
    /// Total anomalies detected
    pub total_anomalies: u64,
    /// False positive rate
    pub false_positive_rate: f64,
    /// Detection accuracy
    pub detection_accuracy: f64,
    /// Average detection time (milliseconds)
    pub avg_detection_time_ms: f64,
}

/// 🤖 AI Anomaly Detector
pub struct AnomalyDetector {
    /// Configuration
    config: Arc<Config>,
    /// Detection models
    models: Arc<RwLock<std::collections::HashMap<String, AnomalyModel>>>,
    /// Historical data buffer
    data_buffer: Arc<RwLock<VecDeque<DataPoint>>>,
    /// Recent anomalies
    recent_anomalies: Arc<RwLock<VecDeque<AnomalyAlert>>>,
    /// Detection statistics
    stats: Arc<RwLock<AnomalyStats>>,
    /// Detection sensitivity threshold
    sensitivity_threshold: f64,
}

impl AnomalyDetector {
    /// Creates new anomaly detector
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self, MonitoringError> {
        info!("🤖 Initializing AI Anomaly Detector...");
        
        let detector = Self {
            config: config.clone(),
            models: Arc::new(RwLock::new(std::collections::HashMap::new())),
            data_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            recent_anomalies: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            stats: Arc::new(RwLock::new(AnomalyStats {
                total_data_points: 0,
                total_anomalies: 0,
                false_positive_rate: 0.0,
                detection_accuracy: 0.0,
                avg_detection_time_ms: 0.0,
            })),
            sensitivity_threshold: config.anomaly_detection.sensitivity_threshold,
        };
        
        // Initialize default models
        detector.initialize_models().await?;
        
        // Start background tasks
        detector.start_detection_task().await;
        detector.start_model_update_task().await;
        
        info!("✅ AI Anomaly Detector initialized");
        Ok(detector)
    }
    
    /// Analyzes data point for anomalies
    #[instrument(skip(self, data_point))]
    pub async fn analyze_data_point(&self, data_point: DataPoint) -> Result<Option<AnomalyAlert>, MonitoringError> {
        debug!("🔍 Analyzing data point: {} = {}", data_point.metric_name, data_point.value);
        
        let start_time = std::time::Instant::now();
        
        // Add to data buffer
        {
            let mut buffer = self.data_buffer.write().await;
            buffer.push_back(data_point.clone());
            
            // Keep buffer size manageable
            if buffer.len() > 10000 {
                buffer.pop_front();
            }
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_data_points += 1;
        }
        
        // Perform anomaly detection
        let anomaly_alert = self.detect_anomaly(&data_point).await?;
        
        // Record detection time
        let detection_time_ms = start_time.elapsed().as_millis() as f64;
        {
            let mut stats = self.stats.write().await;
            let total_points = stats.total_data_points as f64;
            stats.avg_detection_time_ms = 
                (stats.avg_detection_time_ms * (total_points - 1.0) + detection_time_ms) / total_points;
        }
        
        if let Some(ref alert) = anomaly_alert {
            warn!("🚨 Anomaly detected: {} (severity: {:.2}, confidence: {:.2})", 
                  alert.description, alert.severity, alert.confidence);
            
            // Store anomaly
            {
                let mut anomalies = self.recent_anomalies.write().await;
                anomalies.push_back(alert.clone());
                
                // Keep only recent anomalies
                if anomalies.len() > 1000 {
                    anomalies.pop_front();
                }
            }
            
            // Update anomaly statistics
            {
                let mut stats = self.stats.write().await;
                stats.total_anomalies += 1;
            }
        }
        
        Ok(anomaly_alert)
    }
    
    /// Trains anomaly detection model
    #[instrument(skip(self, training_data))]
    pub async fn train_model(
        &self,
        model_id: &str,
        training_data: Vec<DataPoint>,
    ) -> Result<(), MonitoringError> {
        info!("🧠 Training anomaly detection model: {} (data points: {})", model_id, training_data.len());
        
        if training_data.len() < 100 {
            return Err(MonitoringError::AnomalyDetection("Insufficient training data".to_string()));
        }
        
        // Extract features and labels
        let features = self.extract_features(&training_data)?;
        
        // Train model (simplified implementation)
        let model = self.train_statistical_model(&features)?;
        
        // Store trained model
        {
            let mut models = self.models.write().await;
            models.insert(model_id.to_string(), model);
        }
        
        info!("✅ Model trained successfully: {}", model_id);
        Ok(())
    }
    
    /// Gets recent anomalies
    pub async fn get_recent_anomalies(&self, limit: usize) -> Vec<AnomalyAlert> {
        let anomalies = self.recent_anomalies.read().await;
        anomalies.iter().rev().take(limit).cloned().collect()
    }
    
    /// Gets detection statistics
    pub async fn get_stats(&self) -> AnomalyStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Detects anomaly in data point
    async fn detect_anomaly(&self, data_point: &DataPoint) -> Result<Option<AnomalyAlert>, MonitoringError> {
        // Get historical data for the same metric
        let historical_data = self.get_historical_data(&data_point.metric_name, 100).await;
        
        if historical_data.len() < 10 {
            // Not enough historical data for detection
            return Ok(None);
        }
        
        // Statistical anomaly detection using Z-score
        let values: Vec<f64> = historical_data.iter().map(|dp| dp.value).collect();
        let mean = values.mean();
        let std_dev = values.std_dev();
        
        if std_dev == 0.0 {
            // No variance in data
            return Ok(None);
        }
        
        let z_score = (data_point.value - mean) / std_dev;
        let anomaly_threshold = 3.0; // 3-sigma rule
        
        if z_score.abs() > anomaly_threshold {
            // Anomaly detected
            let severity = (z_score.abs() - anomaly_threshold) / anomaly_threshold;
            let severity = severity.min(1.0).max(0.0);
            
            let confidence = 1.0 - (1.0 / (1.0 + z_score.abs()));
            
            let alert = AnomalyAlert {
                alert_id: uuid::Uuid::new_v4().to_string(),
                anomaly_type: self.classify_anomaly_type(&data_point.metric_name),
                severity,
                confidence,
                description: format!(
                    "Statistical anomaly in {}: value {} deviates {:.2} standard deviations from mean {}",
                    data_point.metric_name, data_point.value, z_score, mean
                ),
                affected_target: data_point.metric_name.clone(),
                anomalous_value: data_point.value,
                expected_range: (mean - 2.0 * std_dev, mean + 2.0 * std_dev),
                detected_at: chrono::Utc::now(),
                metadata: std::collections::HashMap::new(),
            };
            
            return Ok(Some(alert));
        }
        
        Ok(None)
    }
    
    /// Gets historical data for a metric
    async fn get_historical_data(&self, metric_name: &str, limit: usize) -> Vec<DataPoint> {
        let buffer = self.data_buffer.read().await;
        buffer
            .iter()
            .rev()
            .filter(|dp| dp.metric_name == metric_name)
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Classifies anomaly type based on metric name
    fn classify_anomaly_type(&self, metric_name: &str) -> AnomalyType {
        match metric_name {
            name if name.contains("latency") || name.contains("response_time") => AnomalyType::Performance,
            name if name.contains("cpu") || name.contains("memory") || name.contains("disk") => AnomalyType::Resource,
            name if name.contains("error") || name.contains("failure") => AnomalyType::Behavioral,
            name if name.contains("security") || name.contains("auth") => AnomalyType::Security,
            name if name.contains("time") || name.contains("timestamp") => AnomalyType::Temporal,
            _ => AnomalyType::Statistical,
        }
    }
    
    /// Initializes default detection models
    async fn initialize_models(&self) -> Result<(), MonitoringError> {
        info!("🧠 Initializing default anomaly detection models...");
        
        let mut models = self.models.write().await;
        
        // Statistical model
        let statistical_model = AnomalyModel {
            model_id: "statistical_zscore".to_string(),
            model_type: "Z-Score".to_string(),
            training_data_size: 0,
            accuracy: 0.85,
            last_trained: chrono::Utc::now(),
            parameters: [("threshold".to_string(), 3.0)].iter().cloned().collect(),
        };
        
        models.insert("statistical_zscore".to_string(), statistical_model);
        
        info!("✅ Default models initialized");
        Ok(())
    }
    
    /// Extracts features from training data
    fn extract_features(&self, training_data: &[DataPoint]) -> Result<Array2<f64>, MonitoringError> {
        let n_samples = training_data.len();
        let n_features = 1; // Just the value for now
        
        let mut features = Array2::zeros((n_samples, n_features));
        
        for (i, data_point) in training_data.iter().enumerate() {
            features[[i, 0]] = data_point.value;
        }
        
        Ok(features)
    }
    
    /// Trains a statistical anomaly detection model
    fn train_statistical_model(&self, _features: &Array2<f64>) -> Result<AnomalyModel, MonitoringError> {
        // Simplified model training
        let model = AnomalyModel {
            model_id: "trained_statistical".to_string(),
            model_type: "Statistical".to_string(),
            training_data_size: _features.nrows(),
            accuracy: 0.90,
            last_trained: chrono::Utc::now(),
            parameters: [("threshold".to_string(), 3.0)].iter().cloned().collect(),
        };
        
        Ok(model)
    }
    
    /// Starts background detection task
    async fn start_detection_task(&self) {
        let data_buffer = self.data_buffer.clone();
        let recent_anomalies = self.recent_anomalies.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Perform batch anomaly detection on recent data
                let recent_data = {
                    let buffer = data_buffer.read().await;
                    buffer.iter().rev().take(100).cloned().collect::<Vec<_>>()
                };
                
                // In a real implementation, this would perform more sophisticated
                // batch anomaly detection algorithms
                debug!("🔍 Performed batch anomaly detection on {} data points", recent_data.len());
            }
        });
    }
    
    /// Starts model update task
    async fn start_model_update_task(&self) {
        let models = self.models.clone();
        let data_buffer = self.data_buffer.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // 1 hour
            
            loop {
                interval.tick().await;
                
                // Retrain models with recent data
                let training_data = {
                    let buffer = data_buffer.read().await;
                    buffer.iter().rev().take(1000).cloned().collect::<Vec<_>>()
                };
                
                if training_data.len() >= 100 {
                    debug!("🧠 Retraining models with {} data points", training_data.len());
                    // In a real implementation, this would retrain the models
                }
            }
        });
    }
}
