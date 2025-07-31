//! 📊 Security Metrics

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub threat_metrics: ThreatMetrics,
    pub auth_metrics: AuthMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreatMetrics {
    pub threats_detected: u64,
    pub threats_blocked: u64,
    pub false_positives: u64,
    pub detection_accuracy: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthMetrics {
    pub successful_auths: u64,
    pub failed_auths: u64,
    pub active_sessions: u32,
    pub mfa_usage_rate: f64,
}

impl SecurityMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
