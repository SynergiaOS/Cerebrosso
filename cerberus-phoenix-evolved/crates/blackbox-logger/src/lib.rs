//! 📦 Blackbox Logger - High-performance logging for trading systems
//! 
//! Provides structured, high-throughput logging with automatic rotation and compression.

use anyhow::Result;
use cerberus_core_types::{Decision, ExecutionResult, Signal};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
    sync::mpsc,
};
use tracing::{error, info, warn};
use uuid::Uuid;

/// 📝 Log entry types
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LogEntry {
    Signal {
        id: Uuid,
        timestamp: SystemTime,
        data: Signal,
    },
    Decision {
        id: Uuid,
        timestamp: SystemTime,
        data: Decision,
    },
    Execution {
        id: Uuid,
        timestamp: SystemTime,
        data: ExecutionResult,
    },
    System {
        id: Uuid,
        timestamp: SystemTime,
        level: LogLevel,
        message: String,
        metadata: serde_json::Value,
    },
}

/// 📊 Log levels
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

/// ⚙️ Logger configuration
#[derive(Clone, Debug)]
pub struct LoggerConfig {
    pub log_dir: PathBuf,
    pub max_file_size: u64,
    pub max_files: usize,
    pub buffer_size: usize,
    pub flush_interval_ms: u64,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("./logs"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            buffer_size: 1000,
            flush_interval_ms: 1000,
        }
    }
}

/// 📦 High-performance blackbox logger
pub struct BlackboxLogger {
    sender: mpsc::UnboundedSender<LogEntry>,
    _handle: tokio::task::JoinHandle<()>,
}

impl BlackboxLogger {
    /// Create a new blackbox logger
    pub async fn new(config: LoggerConfig) -> Result<Self> {
        info!("📦 Initializing Blackbox Logger");

        // Create log directory if it doesn't exist
        tokio::fs::create_dir_all(&config.log_dir).await?;

        let (sender, receiver) = mpsc::unbounded_channel();
        let handle = tokio::spawn(Self::writer_task(config, receiver));

        Ok(Self {
            sender,
            _handle: handle,
        })
    }

    /// Log a trading signal
    pub fn log_signal(&self, signal: Signal) -> Result<()> {
        let entry = LogEntry::Signal {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            data: signal,
        };
        
        self.sender.send(entry)?;
        Ok(())
    }

    /// Log a trading decision
    pub fn log_decision(&self, decision: Decision) -> Result<()> {
        let entry = LogEntry::Decision {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            data: decision,
        };
        
        self.sender.send(entry)?;
        Ok(())
    }

    /// Log an execution result
    pub fn log_execution(&self, result: ExecutionResult) -> Result<()> {
        let entry = LogEntry::Execution {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            data: result,
        };
        
        self.sender.send(entry)?;
        Ok(())
    }

    /// Log a system message
    pub fn log_system(&self, level: LogLevel, message: String, metadata: serde_json::Value) -> Result<()> {
        let entry = LogEntry::System {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            level,
            message,
            metadata,
        };
        
        self.sender.send(entry)?;
        Ok(())
    }

    /// Background writer task
    async fn writer_task(
        config: LoggerConfig,
        mut receiver: mpsc::UnboundedReceiver<LogEntry>,
    ) {
        let mut writer = match LogWriter::new(config).await {
            Ok(w) => w,
            Err(e) => {
                error!("❌ Failed to create log writer: {}", e);
                return;
            }
        };

        while let Some(entry) = receiver.recv().await {
            if let Err(e) = writer.write_entry(&entry).await {
                error!("❌ Failed to write log entry: {}", e);
            }
        }

        if let Err(e) = writer.flush().await {
            error!("❌ Failed to flush log writer: {}", e);
        }
    }
}

/// 📝 Log file writer with rotation
struct LogWriter {
    config: LoggerConfig,
    current_file: Option<BufWriter<File>>,
    current_file_size: u64,
    current_file_path: Option<PathBuf>,
}

impl LogWriter {
    async fn new(config: LoggerConfig) -> Result<Self> {
        Ok(Self {
            config,
            current_file: None,
            current_file_size: 0,
            current_file_path: None,
        })
    }

    async fn write_entry(&mut self, entry: &LogEntry) -> Result<()> {
        // Serialize entry to JSON
        let json = serde_json::to_string(entry)?;
        let line = format!("{}\n", json);
        let line_size = line.len() as u64;

        // Check if we need to rotate the file
        if self.should_rotate(line_size).await? {
            self.rotate_file().await?;
        }

        // Ensure we have a file open
        if self.current_file.is_none() {
            self.open_new_file().await?;
        }

        // Write the entry
        if let Some(ref mut file) = self.current_file {
            file.write_all(line.as_bytes()).await?;
            self.current_file_size += line_size;
        }

        Ok(())
    }

    async fn should_rotate(&self, additional_size: u64) -> Result<bool> {
        Ok(self.current_file_size + additional_size > self.config.max_file_size)
    }

    async fn rotate_file(&mut self) -> Result<()> {
        // Flush and close current file
        if let Some(mut file) = self.current_file.take() {
            file.flush().await?;
        }

        // Clean up old files
        self.cleanup_old_files().await?;

        // Reset state
        self.current_file_size = 0;
        self.current_file_path = None;

        info!("🔄 Log file rotated");
        Ok(())
    }

    async fn open_new_file(&mut self) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("cerberus_{}.jsonl", timestamp);
        let file_path = self.config.log_dir.join(filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .await?;

        self.current_file = Some(BufWriter::new(file));
        self.current_file_path = Some(file_path.clone());
        self.current_file_size = 0;

        info!("📝 Opened new log file: {:?}", file_path);
        Ok(())
    }

    async fn cleanup_old_files(&self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.config.log_dir).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("cerberus_") && name.ends_with(".jsonl") {
                    files.push((entry.path(), entry.metadata().await?.modified()?));
                }
            }
        }

        // Sort by modification time (newest first)
        files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess files
        if files.len() > self.config.max_files {
            for (path, _) in files.iter().skip(self.config.max_files) {
                if let Err(e) = tokio::fs::remove_file(path).await {
                    warn!("⚠️ Failed to remove old log file {:?}: {}", path, e);
                } else {
                    info!("🗑️ Removed old log file: {:?}", path);
                }
            }
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(ref mut file) = self.current_file {
            file.flush().await?;
        }
        Ok(())
    }
}
