//! 🌊 Solana Stream - Real-time WebSocket Monitoring
//! 
//! Advanced WebSocket client for monitoring Solana blockchain events in real-time,
//! replacing expensive polling with efficient push notifications.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{info, warn, error, debug};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;

/// 🎯 Solana WebSocket subscription types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionType {
    AccountChange { pubkey: String },
    ProgramChange { program_id: String },
    LogsSubscribe { mentions: Vec<String> },
    SignatureSubscribe { signature: String },
    SlotSubscribe,
}

/// 📊 WebSocket event from Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaStreamEvent {
    pub subscription_id: u64,
    pub event_type: String,
    pub data: serde_json::Value,
    pub slot: Option<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 🔄 Stream configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub websocket_url: String,
    pub max_reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    pub ping_interval_ms: u64,
    pub buffer_size: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            websocket_url: "wss://api.mainnet-beta.solana.com/".to_string(),
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 5000,
            ping_interval_ms: 30000,
            buffer_size: 1000,
        }
    }
}

/// 🌊 Solana Stream Client
pub struct SolanaStreamClient {
    config: StreamConfig,
    subscriptions: Arc<RwLock<HashMap<u64, SubscriptionType>>>,
    event_sender: mpsc::UnboundedSender<SolanaStreamEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SolanaStreamEvent>>>>,
    next_id: Arc<RwLock<u64>>,
}

impl SolanaStreamClient {
    /// 🚀 Initialize Solana stream client
    pub fn new(config: StreamConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Self {
            config,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// 🔌 Connect to Solana WebSocket
    pub async fn connect(&self) -> Result<()> {
        let mut reconnect_attempts = 0;
        
        while reconnect_attempts < self.config.max_reconnect_attempts {
            match self.try_connect().await {
                Ok(_) => {
                    info!("🔌 Connected to Solana WebSocket: {}", self.config.websocket_url);
                    return Ok(());
                }
                Err(e) => {
                    warn!("❌ WebSocket connection failed (attempt {}): {}", reconnect_attempts + 1, e);
                    reconnect_attempts += 1;
                    
                    if reconnect_attempts < self.config.max_reconnect_attempts {
                        tokio::time::sleep(tokio::time::Duration::from_millis(self.config.reconnect_delay_ms)).await;
                    }
                }
            }
        }
        
        Err(anyhow!("Failed to connect after {} attempts", self.config.max_reconnect_attempts))
    }

    /// 🔗 Attempt WebSocket connection
    async fn try_connect(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.config.websocket_url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        let event_sender = self.event_sender.clone();
        let subscriptions = self.subscriptions.clone();
        
        // Spawn message handler
        let read_handle = tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = Self::handle_message(&text, &event_sender, &subscriptions).await {
                            error!("❌ Error handling WebSocket message: {}", e);
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        debug!("🏓 Received ping, sending pong");
                        // Pong will be sent automatically by tungstenite
                    }
                    Ok(Message::Close(_)) => {
                        warn!("🔌 WebSocket connection closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("❌ WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        // Spawn ping sender
        let ping_interval = self.config.ping_interval_ms;
        let ping_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(ping_interval));
            loop {
                interval.tick().await;
                if let Err(e) = write.send(Message::Ping(vec![])).await {
                    error!("❌ Failed to send ping: {}", e);
                    break;
                }
            }
        });
        
        // Wait for either task to complete
        tokio::select! {
            _ = read_handle => {
                warn!("📖 Read handler completed");
            }
            _ = ping_handle => {
                warn!("🏓 Ping handler completed");
            }
        }
        
        Ok(())
    }

    /// 📨 Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        event_sender: &mpsc::UnboundedSender<SolanaStreamEvent>,
        subscriptions: &Arc<RwLock<HashMap<u64, SubscriptionType>>>,
    ) -> Result<()> {
        let message: serde_json::Value = serde_json::from_str(text)?;
        
        // Handle subscription notifications
        if let Some(params) = message.get("params") {
            if let Some(result) = params.get("result") {
                if let Some(subscription_id) = params.get("subscription").and_then(|s| s.as_u64()) {
                    let event = SolanaStreamEvent {
                        subscription_id,
                        event_type: "notification".to_string(),
                        data: result.clone(),
                        slot: result.get("context").and_then(|c| c.get("slot")).and_then(|s| s.as_u64()),
                        timestamp: chrono::Utc::now(),
                    };
                    
                    if let Err(e) = event_sender.send(event) {
                        error!("❌ Failed to send event: {}", e);
                    }
                }
            }
        }
        
        // Handle subscription confirmations
        if let Some(id) = message.get("id") {
            if let Some(result) = message.get("result") {
                debug!("✅ Subscription confirmed: id={}, result={}", id, result);
            }
        }
        
        Ok(())
    }

    /// 📡 Subscribe to program account changes
    pub async fn subscribe_program(&self, program_id: &str) -> Result<u64> {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        
        let subscription = SubscriptionType::ProgramChange {
            program_id: program_id.to_string(),
        };
        
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(id, subscription);
        }
        
        info!("📡 Subscribed to program changes: {} (id: {})", program_id, id);
        Ok(id)
    }

    /// 📡 Subscribe to account changes
    pub async fn subscribe_account(&self, pubkey: &str) -> Result<u64> {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        
        let subscription = SubscriptionType::AccountChange {
            pubkey: pubkey.to_string(),
        };
        
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(id, subscription);
        }
        
        info!("📡 Subscribed to account changes: {} (id: {})", pubkey, id);
        Ok(id)
    }

    /// 📡 Subscribe to logs mentioning specific strings
    pub async fn subscribe_logs(&self, mentions: Vec<String>) -> Result<u64> {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        
        let subscription = SubscriptionType::LogsSubscribe {
            mentions: mentions.clone(),
        };
        
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(id, subscription);
        }
        
        info!("📡 Subscribed to logs mentioning: {:?} (id: {})", mentions, id);
        Ok(id)
    }

    /// 📡 Subscribe to pump.fun token creation events
    pub async fn subscribe_pump_fun_tokens(&self) -> Result<u64> {
        let pump_fun_program = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
        self.subscribe_program(pump_fun_program).await
    }

    /// 📡 Subscribe to token mint events
    pub async fn subscribe_token_mints(&self) -> Result<u64> {
        let token_program = "TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj";
        self.subscribe_logs(vec!["mint".to_string(), "create".to_string()]).await
    }

    /// 📥 Get event receiver
    pub async fn get_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<SolanaStreamEvent>> {
        let mut receiver_guard = self.event_receiver.write().await;
        receiver_guard.take()
    }

    /// 📊 Get subscription statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let subscriptions = self.subscriptions.read().await;
        let mut stats = HashMap::new();
        
        stats.insert("total_subscriptions".to_string(), json!(subscriptions.len()));
        stats.insert("websocket_url".to_string(), json!(self.config.websocket_url));
        stats.insert("max_reconnect_attempts".to_string(), json!(self.config.max_reconnect_attempts));
        
        // Count subscription types
        let mut type_counts = HashMap::new();
        for subscription in subscriptions.values() {
            let type_name = match subscription {
                SubscriptionType::AccountChange { .. } => "account_change",
                SubscriptionType::ProgramChange { .. } => "program_change",
                SubscriptionType::LogsSubscribe { .. } => "logs_subscribe",
                SubscriptionType::SignatureSubscribe { .. } => "signature_subscribe",
                SubscriptionType::SlotSubscribe => "slot_subscribe",
            };
            *type_counts.entry(type_name.to_string()).or_insert(0) += 1;
        }
        
        stats.insert("subscription_types".to_string(), json!(type_counts));
        stats
    }

    /// 🔌 Disconnect and cleanup
    pub async fn disconnect(&self) {
        info!("🔌 Disconnecting Solana WebSocket stream");
        // Cleanup will happen automatically when tasks are dropped
    }
}

/// 🎯 Stream event handler trait
#[async_trait::async_trait]
pub trait StreamEventHandler: Send + Sync {
    async fn handle_event(&self, event: SolanaStreamEvent) -> Result<()>;
}

/// 🚀 Stream manager for handling multiple subscriptions
pub struct SolanaStreamManager {
    client: Arc<SolanaStreamClient>,
    handlers: Arc<RwLock<Vec<Arc<dyn StreamEventHandler>>>>,
}

impl SolanaStreamManager {
    /// 🚀 Initialize stream manager
    pub fn new(config: StreamConfig) -> Self {
        let client = Arc::new(SolanaStreamClient::new(config));
        
        Self {
            client,
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// ➕ Add event handler
    pub async fn add_handler(&self, handler: Arc<dyn StreamEventHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
    }

    /// 🚀 Start stream processing
    pub async fn start(&self) -> Result<()> {
        // Connect to WebSocket
        self.client.connect().await?;
        
        // Get event receiver
        if let Some(mut receiver) = self.client.get_event_receiver().await {
            let handlers = self.handlers.clone();
            
            // Spawn event processing task
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    let handlers_guard = handlers.read().await;
                    
                    for handler in handlers_guard.iter() {
                        if let Err(e) = handler.handle_event(event.clone()).await {
                            error!("❌ Handler error: {}", e);
                        }
                    }
                }
            });
        }
        
        Ok(())
    }

    /// 📡 Subscribe to pump.fun tokens
    pub async fn subscribe_pump_fun(&self) -> Result<u64> {
        self.client.subscribe_pump_fun_tokens().await
    }

    /// 📡 Subscribe to token mints
    pub async fn subscribe_token_mints(&self) -> Result<u64> {
        self.client.subscribe_token_mints().await
    }

    /// 📊 Get statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        self.client.get_stats().await
    }
}
