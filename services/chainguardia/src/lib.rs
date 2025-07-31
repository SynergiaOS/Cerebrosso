//! 🛡️ Chainguardia - Advanced Security Monitoring Library
//! 
//! Enterprise-grade security monitoring and threat detection for blockchain operations

pub mod config;
pub mod security_monitor;
pub mod threat_detector;
pub mod anomaly_detector;
pub mod wallet_guardian;
pub mod transaction_analyzer;
pub mod risk_assessor;
pub mod alert_manager;
pub mod compliance_checker;
pub mod metrics;

// Core exports
pub use config::Config;
pub use security_monitor::{SecurityMonitor, SecurityState, SecurityEvent};
pub use threat_detector::{ThreatDetector, ThreatLevel, ThreatType, ThreatAlert};
pub use anomaly_detector::{AnomalyDetector, AnomalyType, AnomalyScore};
pub use wallet_guardian::{WalletGuardian, WalletSecurity, WalletRisk};
pub use transaction_analyzer::{TransactionAnalyzer, TxRisk, TxPattern};
pub use risk_assessor::{RiskAssessor, RiskLevel, RiskFactor};
pub use alert_manager::{AlertManager, Alert, AlertSeverity, AlertChannel};
pub use compliance_checker::{ComplianceChecker, ComplianceStatus, ComplianceRule};
pub use metrics::{SecurityMetrics, ThreatMetrics, ComplianceMetrics};

/// 🎯 Core Chainguardia Result Type
pub type GuardiaResult<T> = Result<T, SecurityError>;

/// ❌ Security Error Types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Threat detection error: {0}")]
    ThreatDetection(String),
    
    #[error("Anomaly detection error: {0}")]
    AnomalyDetection(String),
    
    #[error("Wallet security error: {0}")]
    WalletSecurity(String),
    
    #[error("Transaction analysis error: {0}")]
    TransactionAnalysis(String),
    
    #[error("Risk assessment error: {0}")]
    RiskAssessment(String),
    
    #[error("Alert management error: {0}")]
    AlertManagement(String),
    
    #[error("Compliance check error: {0}")]
    ComplianceCheck(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),
    
    #[error("Network security error: {0}")]
    NetworkSecurity(String),
}

/// 🚨 Security Event Types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SecurityEventType {
    /// Suspicious transaction detected
    SuspiciousTransaction,
    /// Unusual wallet activity
    UnusualWalletActivity,
    /// Potential MEV attack
    PotentialMevAttack,
    /// Sandwich attack detected
    SandwichAttack,
    /// Front-running detected
    FrontRunning,
    /// Large position movement
    LargePositionMovement,
    /// Rapid trading pattern
    RapidTradingPattern,
    /// Wallet compromise suspected
    WalletCompromise,
    /// API key exposure
    ApiKeyExposure,
    /// Unusual network activity
    UnusualNetworkActivity,
    /// Compliance violation
    ComplianceViolation,
}

/// 🔒 Security Level
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SecurityLevel {
    /// Low security risk
    Low,
    /// Medium security risk
    Medium,
    /// High security risk
    High,
    /// Critical security risk
    Critical,
    /// Emergency - immediate action required
    Emergency,
}

/// 📊 Security Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityStatus {
    /// Overall security level
    pub overall_level: SecurityLevel,
    /// Active threats count
    pub active_threats: u32,
    /// Anomalies detected in last hour
    pub recent_anomalies: u32,
    /// Compliance status
    pub compliance_status: ComplianceStatus,
    /// Last security scan timestamp
    pub last_scan: chrono::DateTime<chrono::Utc>,
    /// Security score (0.0 - 1.0)
    pub security_score: f64,
}

/// 🛡️ Security Constants
pub mod constants {
    /// Maximum threat alerts to keep in memory
    pub const MAX_THREAT_ALERTS: usize = 10000;
    
    /// Default anomaly detection threshold
    pub const DEFAULT_ANOMALY_THRESHOLD: f64 = 0.95;
    
    /// Maximum transaction analysis batch size
    pub const MAX_TX_ANALYSIS_BATCH: usize = 1000;
    
    /// Default risk assessment interval (seconds)
    pub const DEFAULT_RISK_ASSESSMENT_INTERVAL: u64 = 300;
    
    /// Maximum wallet monitoring count
    pub const MAX_MONITORED_WALLETS: usize = 10000;
    
    /// Default alert retention period (hours)
    pub const DEFAULT_ALERT_RETENTION_HOURS: u64 = 168; // 7 days
    
    /// Minimum confidence for threat alerts
    pub const MIN_THREAT_CONFIDENCE: f64 = 0.7;
    
    /// Maximum concurrent security scans
    pub const MAX_CONCURRENT_SCANS: usize = 10;
    
    /// Default encryption key size
    pub const DEFAULT_KEY_SIZE: usize = 32;
    
    /// Security metrics update interval (seconds)
    pub const METRICS_UPDATE_INTERVAL: u64 = 60;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8400,
                host: "localhost".to_string(),
            },
            security: config::SecurityConfig {
                threat_detection_enabled: true,
                anomaly_detection_enabled: true,
                wallet_monitoring_enabled: true,
                compliance_checking_enabled: true,
                threat_threshold: 0.7,
                anomaly_threshold: 0.95,
                max_monitored_wallets: 10000,
            },
            monitoring: config::MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: 9400,
                log_level: "info".to_string(),
                alert_retention_hours: 168,
            },
            notifications: config::NotificationConfig {
                email_enabled: false,
                slack_enabled: false,
                webhook_enabled: true,
                webhook_url: "http://localhost:8090/alerts".to_string(),
            },
            redis: config::RedisConfig {
                url: "redis://localhost:6379".to_string(),
                key_prefix: "chainguardia:".to_string(),
                ttl_seconds: 3600,
            },
        })
    }
    
    /// Create a mock security event
    pub fn create_mock_security_event() -> SecurityEvent {
        SecurityEvent {
            id: uuid::Uuid::new_v4(),
            event_type: SecurityEventType::SuspiciousTransaction,
            severity: SecurityLevel::Medium,
            description: "Test security event".to_string(),
            source: "test".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Create a mock threat alert
    pub fn create_mock_threat_alert() -> ThreatAlert {
        ThreatAlert {
            id: uuid::Uuid::new_v4(),
            threat_type: ThreatType::SuspiciousActivity,
            threat_level: ThreatLevel::Medium,
            confidence: 0.8,
            description: "Test threat alert".to_string(),
            affected_resources: vec!["test_wallet".to_string()],
            timestamp: chrono::Utc::now(),
            resolved: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_chainguardia_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.server.port, 8400);
        assert!(config.security.threat_detection_enabled);
        assert_eq!(config.security.threat_threshold, 0.7);
    }
    
    #[tokio::test]
    async fn test_security_event_creation() {
        let event = create_mock_security_event();
        
        assert_eq!(event.event_type, SecurityEventType::SuspiciousTransaction);
        assert_eq!(event.severity, SecurityLevel::Medium);
        assert_eq!(event.description, "Test security event");
    }
    
    #[tokio::test]
    async fn test_threat_alert_creation() {
        let alert = create_mock_threat_alert();
        
        assert_eq!(alert.threat_type, ThreatType::SuspiciousActivity);
        assert_eq!(alert.threat_level, ThreatLevel::Medium);
        assert_eq!(alert.confidence, 0.8);
        assert!(!alert.resolved);
    }
    
    #[test]
    fn test_security_level_ordering() {
        assert!(SecurityLevel::Critical > SecurityLevel::High);
        assert!(SecurityLevel::High > SecurityLevel::Medium);
        assert!(SecurityLevel::Medium > SecurityLevel::Low);
        assert!(SecurityLevel::Emergency > SecurityLevel::Critical);
    }
}
