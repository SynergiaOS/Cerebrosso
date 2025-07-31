//! 🔐 Security Hardening - Enterprise Security Library
//! 
//! Enterprise-grade security hardening with HSM integration, multi-sig wallets, and zero-trust architecture

pub mod config;
pub mod hsm_manager;
pub mod multi_sig_wallet;
pub mod zero_trust_auth;
pub mod threat_detector;
pub mod secure_storage;
pub mod certificate_manager;
pub mod audit_logger;
pub mod compliance_checker;
pub mod metrics;

// Core exports
pub use config::Config;
pub use hsm_manager::{HSMManager, HSMProvider, HSMKey, HSMOperation};
pub use multi_sig_wallet::{MultiSigWallet, WalletSigner, SignatureThreshold, WalletTransaction};
pub use zero_trust_auth::{ZeroTrustAuth, AuthPolicy, AccessToken, UserContext};
pub use threat_detector::{ThreatDetector, ThreatSignature, ThreatLevel, SecurityIncident};
pub use secure_storage::{SecureStorage, EncryptedData, StorageBackend};
pub use certificate_manager::{CertificateManager, Certificate, CertificateChain};
pub use audit_logger::{AuditLogger, AuditEvent, AuditLevel};
pub use compliance_checker::{ComplianceChecker, ComplianceRule, ComplianceReport};
pub use metrics::{SecurityMetrics, ThreatMetrics, AuthMetrics};

/// 🎯 Core Security Result Type
pub type SecurityResult<T> = Result<T, SecurityError>;

/// ❌ Security Error Types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("HSM operation error: {0}")]
    HSMOperation(String),
    
    #[error("Multi-signature error: {0}")]
    MultiSignature(String),
    
    #[error("Zero-trust authentication error: {0}")]
    ZeroTrustAuth(String),
    
    #[error("Threat detection error: {0}")]
    ThreatDetection(String),
    
    #[error("Secure storage error: {0}")]
    SecureStorage(String),
    
    #[error("Certificate management error: {0}")]
    CertificateManagement(String),
    
    #[error("Audit logging error: {0}")]
    AuditLogging(String),
    
    #[error("Compliance check error: {0}")]
    ComplianceCheck(String),
    
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Hardware security module error: {0}")]
    HardwareSecurityModule(String),
    
    #[error("Access denied: {0}")]
    AccessDenied(String),
}

/// 🔐 Security Level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum SecurityLevel {
    /// Low security level
    Low = 1,
    /// Medium security level
    Medium = 2,
    /// High security level
    High = 3,
    /// Critical security level
    Critical = 4,
    /// Maximum security level
    Maximum = 5,
}

/// 🛡️ Security Policy
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityPolicy {
    /// Minimum security level required
    pub min_security_level: SecurityLevel,
    /// Require multi-factor authentication
    pub require_mfa: bool,
    /// Require HSM for key operations
    pub require_hsm: bool,
    /// Require multi-signature for transactions
    pub require_multi_sig: bool,
    /// Maximum session duration in seconds
    pub max_session_duration: u64,
    /// Require certificate-based authentication
    pub require_certificate_auth: bool,
    /// Enable zero-trust verification
    pub enable_zero_trust: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            min_security_level: SecurityLevel::High,
            require_mfa: true,
            require_hsm: true,
            require_multi_sig: true,
            max_session_duration: 3600, // 1 hour
            require_certificate_auth: true,
            enable_zero_trust: true,
        }
    }
}

/// 📊 Security Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityStatus {
    /// Overall security level
    pub security_level: SecurityLevel,
    /// HSM status
    pub hsm_status: HSMStatus,
    /// Multi-sig wallet status
    pub multi_sig_status: MultiSigStatus,
    /// Zero-trust authentication status
    pub zero_trust_status: ZeroTrustStatus,
    /// Active threats count
    pub active_threats: u32,
    /// Compliance score (0.0 - 1.0)
    pub compliance_score: f64,
    /// Last security audit timestamp
    pub last_audit: chrono::DateTime<chrono::Utc>,
}

/// 🔑 HSM Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HSMStatus {
    /// HSM provider
    pub provider: String,
    /// HSM connection status
    pub connected: bool,
    /// Number of keys stored
    pub key_count: u32,
    /// HSM health score (0.0 - 1.0)
    pub health_score: f64,
}

/// 🔐 Multi-Sig Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiSigStatus {
    /// Number of active wallets
    pub active_wallets: u32,
    /// Total signers across all wallets
    pub total_signers: u32,
    /// Pending transactions
    pub pending_transactions: u32,
    /// Multi-sig health score (0.0 - 1.0)
    pub health_score: f64,
}

/// 🛡️ Zero-Trust Status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ZeroTrustStatus {
    /// Active sessions
    pub active_sessions: u32,
    /// Failed authentication attempts in last hour
    pub failed_auth_attempts: u32,
    /// Zero-trust policies enforced
    pub policies_enforced: u32,
    /// Zero-trust health score (0.0 - 1.0)
    pub health_score: f64,
}

/// 🔐 Security Constants
pub mod constants {
    /// Maximum key size for HSM operations
    pub const MAX_HSM_KEY_SIZE: usize = 4096;
    
    /// Default multi-sig threshold
    pub const DEFAULT_MULTISIG_THRESHOLD: usize = 3;
    
    /// Maximum multi-sig signers
    pub const MAX_MULTISIG_SIGNERS: usize = 10;
    
    /// Default session timeout (seconds)
    pub const DEFAULT_SESSION_TIMEOUT: u64 = 3600;
    
    /// Maximum failed authentication attempts
    pub const MAX_FAILED_AUTH_ATTEMPTS: u32 = 5;
    
    /// Certificate validity period (days)
    pub const CERTIFICATE_VALIDITY_DAYS: u64 = 365;
    
    /// Audit log retention period (days)
    pub const AUDIT_LOG_RETENTION_DAYS: u64 = 2555; // 7 years
    
    /// Threat detection window (seconds)
    pub const THREAT_DETECTION_WINDOW: u64 = 300;
    
    /// Compliance check interval (hours)
    pub const COMPLIANCE_CHECK_INTERVAL: u64 = 24;
    
    /// Security metrics update interval (seconds)
    pub const SECURITY_METRICS_INTERVAL: u64 = 60;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::sync::Arc;
    
    /// Create a test configuration
    pub fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server: config::ServerConfig {
                port: 8600,
                host: "localhost".to_string(),
            },
            hsm: config::HSMConfig {
                provider: "SoftHSM".to_string(),
                library_path: "/usr/lib/softhsm/libsofthsm2.so".to_string(),
                slot_id: 0,
                pin: "1234".to_string(),
                key_label_prefix: "cerberus_".to_string(),
            },
            multi_sig: config::MultiSigConfig {
                default_threshold: 3,
                max_signers: 10,
                key_derivation_path: "m/44'/501'/0'/0'".to_string(),
                enable_hardware_wallets: true,
            },
            zero_trust: config::ZeroTrustConfig {
                jwt_secret: "test_secret".to_string(),
                session_timeout_seconds: 3600,
                max_failed_attempts: 5,
                require_mfa: true,
                enable_device_fingerprinting: true,
            },
            threat_detection: config::ThreatDetectionConfig {
                enable_real_time_detection: true,
                threat_signatures_file: "threat_signatures.json".to_string(),
                detection_window_seconds: 300,
                alert_threshold: 0.8,
            },
            compliance: config::ComplianceConfig {
                frameworks: vec!["SOC2".to_string(), "ISO27001".to_string()],
                audit_interval_hours: 24,
                compliance_threshold: 0.95,
                enable_continuous_monitoring: true,
            },
            monitoring: config::MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: 9600,
                log_level: "info".to_string(),
                audit_log_retention_days: 2555,
            },
        })
    }
    
    /// Create a mock security status
    pub fn create_mock_security_status() -> SecurityStatus {
        SecurityStatus {
            security_level: SecurityLevel::High,
            hsm_status: HSMStatus {
                provider: "SoftHSM".to_string(),
                connected: true,
                key_count: 25,
                health_score: 0.95,
            },
            multi_sig_status: MultiSigStatus {
                active_wallets: 5,
                total_signers: 15,
                pending_transactions: 2,
                health_score: 0.92,
            },
            zero_trust_status: ZeroTrustStatus {
                active_sessions: 10,
                failed_auth_attempts: 1,
                policies_enforced: 8,
                health_score: 0.98,
            },
            active_threats: 0,
            compliance_score: 0.96,
            last_audit: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;
    
    #[tokio::test]
    async fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        
        assert_eq!(policy.min_security_level, SecurityLevel::High);
        assert!(policy.require_mfa);
        assert!(policy.require_hsm);
        assert!(policy.require_multi_sig);
        assert!(policy.enable_zero_trust);
    }
    
    #[tokio::test]
    async fn test_security_level_ordering() {
        assert!(SecurityLevel::Maximum > SecurityLevel::Critical);
        assert!(SecurityLevel::Critical > SecurityLevel::High);
        assert!(SecurityLevel::High > SecurityLevel::Medium);
        assert!(SecurityLevel::Medium > SecurityLevel::Low);
    }
    
    #[tokio::test]
    async fn test_security_config_creation() {
        let config = create_test_config();
        
        assert_eq!(config.server.port, 8600);
        assert_eq!(config.hsm.provider, "SoftHSM");
        assert_eq!(config.multi_sig.default_threshold, 3);
        assert!(config.zero_trust.require_mfa);
    }
    
    #[tokio::test]
    async fn test_security_status_creation() {
        let status = create_mock_security_status();
        
        assert_eq!(status.security_level, SecurityLevel::High);
        assert!(status.hsm_status.connected);
        assert_eq!(status.multi_sig_status.active_wallets, 5);
        assert!(status.compliance_score > 0.9);
    }
}
