//! 🔧 Security Hardening Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub hsm: HSMConfig,
    pub multi_sig: MultiSigConfig,
    pub zero_trust: ZeroTrustConfig,
    pub threat_detection: ThreatDetectionConfig,
    pub compliance: ComplianceConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSMConfig {
    pub provider: String,
    pub library_path: String,
    pub slot_id: u32,
    pub pin: String,
    pub key_label_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigConfig {
    pub default_threshold: usize,
    pub max_signers: usize,
    pub key_derivation_path: String,
    pub enable_hardware_wallets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTrustConfig {
    pub jwt_secret: String,
    pub session_timeout_seconds: u64,
    pub max_failed_attempts: u32,
    pub require_mfa: bool,
    pub enable_device_fingerprinting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionConfig {
    pub enable_real_time_detection: bool,
    pub threat_signatures_file: String,
    pub detection_window_seconds: u64,
    pub alert_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub frameworks: Vec<String>,
    pub audit_interval_hours: u64,
    pub compliance_threshold: f64,
    pub enable_continuous_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_port: u16,
    pub log_level: String,
    pub audit_log_retention_days: u64,
}
