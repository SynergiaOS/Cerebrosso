//! 🔐 Infisical Client - Secure Secrets Management
//! 
//! Integration with Infisical for secure storage and retrieval of sensitive data

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};
use chrono::{DateTime, Utc, Duration};

/// 🔐 Infisical Client Configuration
#[derive(Debug, Clone)]
pub struct InfisicalConfig {
    pub api_url: String,
    pub project_id: String,
    pub environment: String,
    pub client_id: String,
    pub client_secret: String,
    pub cache_ttl_seconds: u64,
}

/// 🔐 Infisical Authentication Response
#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
    #[serde(rename = "tokenType")]
    token_type: String,
}

/// 🔐 Secret Response from Infisical
#[derive(Debug, Deserialize)]
struct SecretResponse {
    secrets: Vec<Secret>,
}

#[derive(Debug, Clone, Deserialize)]
struct Secret {
    #[serde(rename = "_id")]
    id: String,
    workspace: String,
    environment: String,
    #[serde(rename = "secretKey")]
    secret_key: String,
    #[serde(rename = "secretValue")]
    secret_value: String,
    #[serde(rename = "secretComment")]
    secret_comment: Option<String>,
}

/// 🔐 Cached Secret with TTL
#[derive(Debug, Clone)]
struct CachedSecret {
    value: String,
    expires_at: DateTime<Utc>,
}

/// 🔐 Infisical Client
pub struct InfisicalClient {
    config: InfisicalConfig,
    http_client: Client,
    access_token: Arc<RwLock<Option<String>>>,
    token_expires_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    secret_cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
}

impl InfisicalClient {
    /// 🚀 Initialize Infisical Client
    #[instrument(skip(config))]
    pub fn new(config: InfisicalConfig) -> Result<Self> {
        info!("🔐 Initializing Infisical Client for project: {}", config.project_id);

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(InfisicalClient {
            config,
            http_client,
            access_token: Arc::new(RwLock::new(None)),
            token_expires_at: Arc::new(RwLock::new(None)),
            secret_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// 🔐 Get Secret by Key
    #[instrument(skip(self))]
    pub async fn get_secret(&self, key: &str) -> Result<String> {
        debug!("🔐 Retrieving secret: {}", key);

        // Check cache first
        if let Some(cached) = self.get_from_cache(key).await {
            debug!("✅ Secret found in cache: {}", key);
            return Ok(cached);
        }

        // Ensure we have a valid token
        self.ensure_authenticated().await?;

        // Fetch secret from Infisical
        let secret_value = self.fetch_secret_from_api(key).await?;

        // Cache the secret
        self.cache_secret(key, &secret_value).await;

        info!("✅ Secret retrieved successfully: {}", key);
        Ok(secret_value)
    }

    /// 🔐 Get Multiple Secrets
    #[instrument(skip(self))]
    pub async fn get_secrets(&self, keys: &[&str]) -> Result<HashMap<String, String>> {
        info!("🔐 Retrieving {} secrets", keys.len());

        let mut results = HashMap::new();
        let mut missing_keys = Vec::new();

        // Check cache for all keys
        for &key in keys {
            if let Some(cached) = self.get_from_cache(key).await {
                results.insert(key.to_string(), cached);
            } else {
                missing_keys.push(key);
            }
        }

        if !missing_keys.is_empty() {
            // Ensure we have a valid token
            self.ensure_authenticated().await?;

            // Fetch missing secrets
            for &key in &missing_keys {
                match self.fetch_secret_from_api(key).await {
                    Ok(value) => {
                        self.cache_secret(key, &value).await;
                        results.insert(key.to_string(), value);
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to retrieve secret {}: {}", key, e);
                    }
                }
            }
        }

        info!("✅ Retrieved {}/{} secrets successfully", results.len(), keys.len());
        Ok(results)
    }

    /// 🔐 Get All Secrets for Environment
    #[instrument(skip(self))]
    pub async fn get_all_secrets(&self) -> Result<HashMap<String, String>> {
        info!("🔐 Retrieving all secrets for environment: {}", self.config.environment);

        self.ensure_authenticated().await?;

        let token = self.access_token.read().await
            .as_ref()
            .ok_or_else(|| anyhow!("No access token available"))?
            .clone();

        let url = format!(
            "{}/api/v3/secrets/raw?workspaceId={}&environment={}",
            self.config.api_url, self.config.project_id, self.config.environment
        );

        let response = self.http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch secrets: {}", response.status()));
        }

        let secret_response: SecretResponse = response.json().await?;
        let mut secrets = HashMap::new();

        for secret in secret_response.secrets {
            secrets.insert(secret.secret_key.clone(), secret.secret_value.clone());
            
            // Cache each secret
            self.cache_secret(&secret.secret_key, &secret.secret_value).await;
        }

        info!("✅ Retrieved {} secrets from Infisical", secrets.len());
        Ok(secrets)
    }

    /// 🔐 Authenticate with Infisical
    #[instrument(skip(self))]
    async fn authenticate(&self) -> Result<()> {
        info!("🔐 Authenticating with Infisical");

        let auth_payload = serde_json::json!({
            "clientId": self.config.client_id,
            "clientSecret": self.config.client_secret
        });

        let url = format!("{}/api/v1/auth/universal-auth/login", self.config.api_url);

        let response = self.http_client
            .post(&url)
            .json(&auth_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Authentication failed: {}", response.status()));
        }

        let auth_response: AuthResponse = response.json().await?;

        // Store token and expiration
        {
            let mut token = self.access_token.write().await;
            *token = Some(auth_response.access_token);
        }

        {
            let mut expires_at = self.token_expires_at.write().await;
            *expires_at = Some(Utc::now() + Duration::seconds(auth_response.expires_in as i64));
        }

        info!("✅ Successfully authenticated with Infisical");
        Ok(())
    }

    /// 🔐 Ensure we have a valid authentication token
    async fn ensure_authenticated(&self) -> Result<()> {
        let token_expires_at = self.token_expires_at.read().await.clone();
        
        let needs_auth = match token_expires_at {
            Some(expires_at) => Utc::now() >= expires_at - Duration::minutes(5), // Refresh 5 min early
            None => true,
        };

        if needs_auth {
            self.authenticate().await?;
        }

        Ok(())
    }

    /// 🔐 Fetch secret from Infisical API
    async fn fetch_secret_from_api(&self, key: &str) -> Result<String> {
        let token = self.access_token.read().await
            .as_ref()
            .ok_or_else(|| anyhow!("No access token available"))?
            .clone();

        let url = format!(
            "{}/api/v3/secrets/raw/{}?workspaceId={}&environment={}",
            self.config.api_url, key, self.config.project_id, self.config.environment
        );

        let response = self.http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!("Secret '{}' not found", key));
        }

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch secret '{}': {}", key, response.status()));
        }

        let secret: Secret = response.json().await?;
        Ok(secret.secret_value)
    }

    /// 🔐 Get secret from cache
    async fn get_from_cache(&self, key: &str) -> Option<String> {
        let cache = self.secret_cache.read().await;
        
        if let Some(cached) = cache.get(key) {
            if Utc::now() < cached.expires_at {
                return Some(cached.value.clone());
            }
        }

        None
    }

    /// 🔐 Cache secret with TTL
    async fn cache_secret(&self, key: &str, value: &str) {
        let mut cache = self.secret_cache.write().await;
        
        let cached_secret = CachedSecret {
            value: value.to_string(),
            expires_at: Utc::now() + Duration::seconds(self.config.cache_ttl_seconds as i64),
        };

        cache.insert(key.to_string(), cached_secret);
    }

    /// 🧹 Clear expired secrets from cache
    #[instrument(skip(self))]
    pub async fn cleanup_cache(&self) {
        let mut cache = self.secret_cache.write().await;
        let now = Utc::now();
        
        let before_count = cache.len();
        cache.retain(|_, cached| now < cached.expires_at);
        let after_count = cache.len();
        
        if before_count > after_count {
            debug!("🧹 Cleaned up {} expired secrets from cache", before_count - after_count);
        }
    }

    /// 📊 Get cache statistics
    pub async fn get_cache_stats(&self) -> serde_json::Value {
        let cache = self.secret_cache.read().await;
        let now = Utc::now();
        
        let total_secrets = cache.len();
        let expired_secrets = cache.values()
            .filter(|cached| now >= cached.expires_at)
            .count();
        
        serde_json::json!({
            "total_cached_secrets": total_secrets,
            "expired_secrets": expired_secrets,
            "valid_secrets": total_secrets - expired_secrets,
            "cache_ttl_seconds": self.config.cache_ttl_seconds
        })
    }
}

/// 🔐 Infisical Client Builder
pub struct InfisicalClientBuilder {
    api_url: Option<String>,
    project_id: Option<String>,
    environment: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    cache_ttl_seconds: u64,
}

impl InfisicalClientBuilder {
    pub fn new() -> Self {
        Self {
            api_url: None,
            project_id: None,
            environment: None,
            client_id: None,
            client_secret: None,
            cache_ttl_seconds: 300, // 5 minutes default
        }
    }

    pub fn api_url(mut self, url: String) -> Self {
        self.api_url = Some(url);
        self
    }

    pub fn project_id(mut self, id: String) -> Self {
        self.project_id = Some(id);
        self
    }

    pub fn environment(mut self, env: String) -> Self {
        self.environment = Some(env);
        self
    }

    pub fn client_credentials(mut self, client_id: String, client_secret: String) -> Self {
        self.client_id = Some(client_id);
        self.client_secret = Some(client_secret);
        self
    }

    pub fn cache_ttl(mut self, seconds: u64) -> Self {
        self.cache_ttl_seconds = seconds;
        self
    }

    pub fn build(self) -> Result<InfisicalClient> {
        let config = InfisicalConfig {
            api_url: self.api_url.ok_or_else(|| anyhow!("API URL is required"))?,
            project_id: self.project_id.ok_or_else(|| anyhow!("Project ID is required"))?,
            environment: self.environment.ok_or_else(|| anyhow!("Environment is required"))?,
            client_id: self.client_id.ok_or_else(|| anyhow!("Client ID is required"))?,
            client_secret: self.client_secret.ok_or_else(|| anyhow!("Client Secret is required"))?,
            cache_ttl_seconds: self.cache_ttl_seconds,
        };

        InfisicalClient::new(config)
    }
}
