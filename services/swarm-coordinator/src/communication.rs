//! 📡 Communication Hub - Komunikacja między agentami
//! 
//! System komunikacji w architekturze Hive Mind

use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, RwLock, broadcast};
use tracing::{info, debug, warn, error, instrument};
use redis::{Client as RedisClient, AsyncCommands};

use crate::{
    config::Config,
    task_delegation::Task,
    agent_types::AgentType,
};

/// 📨 Typ wiadomości
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Przydzielenie zadania
    TaskAssignment,
    /// Wynik zadania
    TaskResult,
    /// Heartbeat agenta
    Heartbeat,
    /// Komenda systemowa
    SystemCommand,
    /// Broadcast do wszystkich agentów
    Broadcast,
    /// Komunikacja między agentami
    AgentToAgent,
    /// Aktualizacja stanu
    StatusUpdate,
    /// Błąd/ostrzeżenie
    Error,
}

/// 🎯 Priorytet wiadomości
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Krytyczne - natychmiastowe dostarczenie
    Critical = 4,
    /// Wysokie - dostarczenie w ciągu 100ms
    High = 3,
    /// Średnie - dostarczenie w ciągu 1s
    Medium = 2,
    /// Niskie - dostarczenie w ciągu 5s
    Low = 1,
}

/// 📨 Wiadomość między agentami
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Unikalny identyfikator wiadomości
    pub id: Uuid,
    /// Typ wiadomości
    pub message_type: MessageType,
    /// Priorytet
    pub priority: MessagePriority,
    /// ID nadawcy (None dla systemu)
    pub sender_id: Option<Uuid>,
    /// ID odbiorcy (None dla broadcast)
    pub recipient_id: Option<Uuid>,
    /// Typ agenta odbiorcy (dla filtrowania)
    pub recipient_type: Option<AgentType>,
    /// Zawartość wiadomości
    pub payload: Value,
    /// Czas utworzenia
    pub created_at: DateTime<Utc>,
    /// Czas wygaśnięcia
    pub expires_at: Option<DateTime<Utc>>,
    /// Metadane wiadomości
    pub metadata: HashMap<String, String>,
    /// Czy wymaga potwierdzenia odbioru
    pub requires_ack: bool,
    /// ID wiadomości, na którą to jest odpowiedź
    pub reply_to: Option<Uuid>,
}

impl AgentMessage {
    /// Tworzy nową wiadomość
    pub fn new(
        message_type: MessageType,
        priority: MessagePriority,
        sender_id: Option<Uuid>,
        recipient_id: Option<Uuid>,
        payload: Value,
    ) -> Self {
        let now = Utc::now();
        let expires_at = match priority {
            MessagePriority::Critical => Some(now + chrono::Duration::seconds(5)),
            MessagePriority::High => Some(now + chrono::Duration::seconds(30)),
            MessagePriority::Medium => Some(now + chrono::Duration::minutes(5)),
            MessagePriority::Low => Some(now + chrono::Duration::minutes(15)),
        };
        
        Self {
            id: Uuid::new_v4(),
            message_type,
            priority,
            sender_id,
            recipient_id,
            recipient_type: None,
            payload,
            created_at: now,
            expires_at,
            metadata: HashMap::new(),
            requires_ack: false,
            reply_to: None,
        }
    }
    
    /// Tworzy wiadomość przydzielenia zadania
    pub fn new_task_assignment(agent_id: Uuid, task: Task) -> Self {
        let payload = serde_json::to_value(&task).unwrap_or(Value::Null);
        
        let mut message = Self::new(
            MessageType::TaskAssignment,
            MessagePriority::High,
            None, // System sender
            Some(agent_id),
            payload,
        );
        
        message.requires_ack = true;
        message.metadata.insert("task_id".to_string(), task.id.to_string());
        message.metadata.insert("task_type".to_string(), task.task_type);
        
        message
    }
    
    /// Tworzy wiadomość heartbeat
    pub fn new_heartbeat(agent_id: Uuid) -> Self {
        let payload = serde_json::json!({
            "agent_id": agent_id,
            "timestamp": Utc::now(),
            "status": "active"
        });
        
        Self::new(
            MessageType::Heartbeat,
            MessagePriority::Low,
            Some(agent_id),
            None, // To coordinator
            payload,
        )
    }
    
    /// Tworzy wiadomość broadcast
    pub fn new_broadcast(
        sender_id: Option<Uuid>,
        recipient_type: Option<AgentType>,
        payload: Value,
    ) -> Self {
        let mut message = Self::new(
            MessageType::Broadcast,
            MessagePriority::Medium,
            sender_id,
            None,
            payload,
        );
        
        message.recipient_type = recipient_type;
        message
    }
    
    /// Sprawdza czy wiadomość wygasła
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

/// 📡 Hub komunikacyjny
pub struct CommunicationHub {
    /// Konfiguracja
    config: Arc<Config>,
    /// Klient Redis
    redis_client: RedisClient,
    /// Kanały komunikacyjne dla agentów
    agent_channels: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<AgentMessage>>>>,
    /// Kanał broadcast
    broadcast_tx: broadcast::Sender<AgentMessage>,
    /// Kolejka wiadomości wychodzących
    outbound_queue: Arc<RwLock<Vec<AgentMessage>>>,
    /// Statystyki komunikacji
    stats: Arc<RwLock<CommunicationStats>>,
}

/// 📊 Statystyki komunikacji
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommunicationStats {
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_broadcasts: u64,
    pub total_failed_deliveries: u64,
    pub average_delivery_time_ms: f64,
    pub active_connections: usize,
    pub message_queue_size: usize,
}

impl CommunicationHub {
    /// Tworzy nowy hub komunikacyjny
    #[instrument(skip(config))]
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        info!("📡 Initializing CommunicationHub...");
        
        // Połączenie z Redis
        let redis_client = RedisClient::open(config.redis.url.as_str())?;
        
        // Test połączenia
        let mut conn = redis_client.get_async_connection().await?;
        let _: String = conn.ping().await?;
        
        // Kanał broadcast
        let (broadcast_tx, _) = broadcast::channel(config.communication.message_buffer_size);
        
        info!("✅ CommunicationHub initialized");
        
        Ok(Self {
            config,
            redis_client,
            agent_channels: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            outbound_queue: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(CommunicationStats::default())),
        })
    }
    
    /// Uruchamia hub komunikacyjny
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting CommunicationHub...");
        
        // Uruchomienie pętli przetwarzania wiadomości
        self.start_message_processing_loop().await?;
        
        // Uruchomienie pętli Redis streams
        self.start_redis_streams_loop().await?;
        
        // Uruchomienie pętli czyszczenia wygasłych wiadomości
        self.start_cleanup_loop().await?;
        
        info!("✅ CommunicationHub started");
        Ok(())
    }
    
    /// Zatrzymuje hub komunikacyjny
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down CommunicationHub...");
        
        // Wyczyść kolejki
        {
            let mut queue = self.outbound_queue.write().await;
            queue.clear();
        }
        
        // Zamknij kanały agentów
        {
            let mut channels = self.agent_channels.write().await;
            channels.clear();
        }
        
        info!("✅ CommunicationHub shutdown completed");
        Ok(())
    }
    
    /// Rejestruje agenta w systemie komunikacji
    #[instrument(skip(self))]
    pub async fn register_agent(&self, agent_id: Uuid) -> Result<mpsc::UnboundedReceiver<AgentMessage>> {
        debug!("📝 Registering agent for communication: {}", agent_id);
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        {
            let mut channels = self.agent_channels.write().await;
            channels.insert(agent_id, tx);
        }
        
        // Aktualizuj statystyki
        {
            let mut stats = self.stats.write().await;
            stats.active_connections += 1;
        }
        
        info!("✅ Agent registered for communication: {}", agent_id);
        Ok(rx)
    }
    
    /// Wyrejestrowuje agenta z systemu komunikacji
    #[instrument(skip(self))]
    pub async fn unregister_agent(&self, agent_id: Uuid) -> Result<()> {
        debug!("🗑️ Unregistering agent from communication: {}", agent_id);
        
        {
            let mut channels = self.agent_channels.write().await;
            channels.remove(&agent_id);
        }
        
        // Aktualizuj statystyki
        {
            let mut stats = self.stats.write().await;
            if stats.active_connections > 0 {
                stats.active_connections -= 1;
            }
        }
        
        info!("✅ Agent unregistered from communication: {}", agent_id);
        Ok(())
    }
    
    /// Wysyła wiadomość do agenta
    #[instrument(skip(self, message))]
    pub async fn send_message(&self, message: AgentMessage) -> Result<()> {
        let message_id = message.id;
        let recipient_id = message.recipient_id;
        
        debug!("📤 Sending message: {} to {:?}", message_id, recipient_id);
        
        // Sprawdź czy wiadomość nie wygasła
        if message.is_expired() {
            warn!("⚠️ Message expired before sending: {}", message_id);
            return Err(anyhow!("Message expired"));
        }
        
        match recipient_id {
            Some(agent_id) => {
                // Wiadomość do konkretnego agenta
                self.send_to_agent(agent_id, message).await?;
            }
            None => {
                // Broadcast
                self.broadcast_message(message).await?;
            }
        }
        
        // Aktualizuj statystyki
        {
            let mut stats = self.stats.write().await;
            stats.total_messages_sent += 1;
        }
        
        debug!("✅ Message sent: {}", message_id);
        Ok(())
    }
    
    /// Wysyła wiadomość do konkretnego agenta
    async fn send_to_agent(&self, agent_id: Uuid, message: AgentMessage) -> Result<()> {
        let channels = self.agent_channels.read().await;
        
        if let Some(tx) = channels.get(&agent_id) {
            if let Err(_) = tx.send(message.clone()) {
                warn!("⚠️ Failed to send message to agent: {}", agent_id);
                
                // Aktualizuj statystyki
                {
                    let mut stats = self.stats.write().await;
                    stats.total_failed_deliveries += 1;
                }
                
                return Err(anyhow!("Agent channel closed"));
            }
        } else {
            // Agent nie jest zarejestrowany, spróbuj przez Redis
            self.send_via_redis(message).await?;
        }
        
        Ok(())
    }
    
    /// Wysyła broadcast do wszystkich agentów
    async fn broadcast_message(&self, message: AgentMessage) -> Result<()> {
        debug!("📢 Broadcasting message: {}", message.id);
        
        // Wyślij przez kanał broadcast
        if let Err(_) = self.broadcast_tx.send(message.clone()) {
            warn!("⚠️ Failed to broadcast message");
        }
        
        // Wyślij też przez Redis dla agentów zewnętrznych
        self.send_via_redis(message).await?;
        
        // Aktualizuj statystyki
        {
            let mut stats = self.stats.write().await;
            stats.total_broadcasts += 1;
        }
        
        Ok(())
    }
    
    /// Wysyła wiadomość przez Redis
    async fn send_via_redis(&self, message: AgentMessage) -> Result<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let stream_name = match message.message_type {
            MessageType::TaskAssignment => &self.config.redis.streams.agent_commands,
            MessageType::TaskResult => &self.config.redis.streams.agent_responses,
            _ => &self.config.redis.streams.agent_commands,
        };
        
        let message_json = serde_json::to_string(&message)?;
        
        let _: String = conn.xadd(
            stream_name,
            "*",
            &[("message", message_json)]
        ).await?;
        
        debug!("📤 Message sent via Redis: {}", message.id);
        Ok(())
    }
    
    /// Otrzymuje wiadomość od agenta
    #[instrument(skip(self, message))]
    pub async fn receive_message(&self, message: AgentMessage) -> Result<()> {
        debug!("📥 Receiving message: {} from {:?}", message.id, message.sender_id);
        
        // Aktualizuj statystyki
        {
            let mut stats = self.stats.write().await;
            stats.total_messages_received += 1;
        }
        
        // Przetwórz wiadomość według typu
        match message.message_type {
            MessageType::Heartbeat => {
                self.handle_heartbeat(message).await?;
            }
            MessageType::TaskResult => {
                self.handle_task_result(message).await?;
            }
            MessageType::AgentToAgent => {
                self.handle_agent_to_agent(message).await?;
            }
            _ => {
                debug!("📨 Received message type: {:?}", message.message_type);
            }
        }
        
        Ok(())
    }
    
    /// Obsługuje heartbeat od agenta
    async fn handle_heartbeat(&self, message: AgentMessage) -> Result<()> {
        if let Some(agent_id) = message.sender_id {
            debug!("💓 Heartbeat received from agent: {}", agent_id);
            // Tutaj można zaktualizować status agenta w registry
        }
        Ok(())
    }
    
    /// Obsługuje wynik zadania od agenta
    async fn handle_task_result(&self, message: AgentMessage) -> Result<()> {
        debug!("📊 Task result received: {}", message.id);
        // Tutaj można przekazać wynik do TaskDelegator
        Ok(())
    }
    
    /// Obsługuje komunikację między agentami
    async fn handle_agent_to_agent(&self, message: AgentMessage) -> Result<()> {
        if let Some(recipient_id) = message.recipient_id {
            debug!("🔄 Forwarding agent-to-agent message to: {}", recipient_id);
            self.send_to_agent(recipient_id, message).await?;
        }
        Ok(())
    }
    
    /// Pobiera statystyki komunikacji
    pub async fn get_stats(&self) -> CommunicationStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
    
    /// Uruchamia pętlę przetwarzania wiadomości
    async fn start_message_processing_loop(&self) -> Result<()> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
    
    /// Uruchamia pętlę Redis streams
    async fn start_redis_streams_loop(&self) -> Result<()> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
    
    /// Uruchamia pętlę czyszczenia wygasłych wiadomości
    async fn start_cleanup_loop(&self) -> Result<()> {
        // Implementation będzie dodana w następnej iteracji
        Ok(())
    }
}
