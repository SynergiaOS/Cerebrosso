//! 📜 Certificate Manager

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{config::Config, SecurityError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub cert_id: String,
    pub subject: String,
    pub issuer: String,
    pub valid_from: chrono::DateTime<chrono::Utc>,
    pub valid_to: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateChain {
    pub certificates: Vec<Certificate>,
}

pub struct CertificateManager {
    config: Arc<Config>,
}

impl CertificateManager {
    pub async fn new(config: Arc<Config>) -> Result<Self, SecurityError> {
        Ok(Self { config })
    }
}
