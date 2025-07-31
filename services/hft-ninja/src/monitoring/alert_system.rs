//! 🚨 Alert System - Real-time Notifications
//! 
//! System alertów dla krytycznych zdarzeń w trading bocie

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn, error, instrument};
use chrono::{DateTime, Utc};

/// 🚨 Alert Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    ProfitTarget,
    LossLimit,
    HighDrawdown,
    LowWinRate,
    SystemError,
    HighVolatility,
    LowLiquidity,
    PositionSize,
    ExecutionDelay,
}

/// 🚨 Alert Severity Levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// 🚨 Alert Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
    pub strategy: Option<String>,
    pub token_symbol: Option<String>,
    pub acknowledged: bool,
}

/// 🚨 Alert Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub max_drawdown_percentage: f64,
    pub min_win_rate_percentage: f64,
    pub max_loss_per_trade_sol: f64,
    pub min_profit_target_sol: f64,
    pub max_execution_time_ms: u64,
    pub max_daily_loss_sol: f64,
    pub min_liquidity_usd: f64,
    pub max_position_size_sol: f64,
    pub enable_email_alerts: bool,
    pub enable_webhook_alerts: bool,
    pub webhook_url: Option<String>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        AlertConfig {
            max_drawdown_percentage: 25.0,
            min_win_rate_percentage: 35.0,
            max_loss_per_trade_sol: 2.0,
            min_profit_target_sol: 10.0,
            max_execution_time_ms: 5000,
            max_daily_loss_sol: 10.0,
            min_liquidity_usd: 50000.0,
            max_position_size_sol: 5.0,
            enable_email_alerts: false,
            enable_webhook_alerts: true,
            webhook_url: None,
        }
    }
}

/// 🚨 Alert System
pub struct AlertSystem {
    config: AlertConfig,
    alert_sender: mpsc::UnboundedSender<Alert>,
    alert_receiver: mpsc::UnboundedReceiver<Alert>,
    active_alerts: HashMap<String, Alert>,
    alert_history: Vec<Alert>,
}

impl AlertSystem {
    /// 🚀 Initialize Alert System
    pub fn new(config: AlertConfig) -> Self {
        info!("🚨 Initializing Alert System");
        
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();

        AlertSystem {
            config,
            alert_sender,
            alert_receiver,
            active_alerts: HashMap::new(),
            alert_history: Vec::new(),
        }
    }

    /// 🚨 Create Alert Sender (for other modules)
    pub fn get_alert_sender(&self) -> mpsc::UnboundedSender<Alert> {
        self.alert_sender.clone()
    }

    /// 🚨 Check Trading Metrics for Alerts
    #[instrument(skip(self))]
    pub async fn check_trading_metrics(
        &mut self,
        metrics: &super::TradingMetrics,
    ) -> Result<Vec<Alert>> {
        let mut new_alerts = Vec::new();

        // Check Max Drawdown
        if metrics.max_drawdown_percentage > self.config.max_drawdown_percentage {
            let alert = self.create_alert(
                AlertType::HighDrawdown,
                AlertSeverity::Critical,
                "High Drawdown Alert".to_string(),
                format!("Drawdown exceeded threshold: {:.1}% > {:.1}%", 
                        metrics.max_drawdown_percentage, self.config.max_drawdown_percentage),
                metrics.max_drawdown_percentage,
                self.config.max_drawdown_percentage,
                None,
                None,
            );
            new_alerts.push(alert);
        }

        // Check Win Rate
        if metrics.win_rate_percentage < self.config.min_win_rate_percentage && metrics.total_trades > 10 {
            let alert = self.create_alert(
                AlertType::LowWinRate,
                AlertSeverity::Warning,
                "Low Win Rate Alert".to_string(),
                format!("Win rate below threshold: {:.1}% < {:.1}%", 
                        metrics.win_rate_percentage, self.config.min_win_rate_percentage),
                metrics.win_rate_percentage,
                self.config.min_win_rate_percentage,
                None,
                None,
            );
            new_alerts.push(alert);
        }

        // Check Daily Loss
        if metrics.current_session_pnl < -self.config.max_daily_loss_sol {
            let alert = self.create_alert(
                AlertType::LossLimit,
                AlertSeverity::Critical,
                "Daily Loss Limit Alert".to_string(),
                format!("Daily loss exceeded limit: {:.2} SOL < -{:.2} SOL", 
                        metrics.current_session_pnl, self.config.max_daily_loss_sol),
                metrics.current_session_pnl.abs(),
                self.config.max_daily_loss_sol,
                None,
                None,
            );
            new_alerts.push(alert);
        }

        // Check Profit Target
        if metrics.current_session_pnl > self.config.min_profit_target_sol {
            let alert = self.create_alert(
                AlertType::ProfitTarget,
                AlertSeverity::Info,
                "Profit Target Reached".to_string(),
                format!("Daily profit target achieved: {:.2} SOL > {:.2} SOL", 
                        metrics.current_session_pnl, self.config.min_profit_target_sol),
                metrics.current_session_pnl,
                self.config.min_profit_target_sol,
                None,
                None,
            );
            new_alerts.push(alert);
        }

        // Process new alerts
        for alert in &new_alerts {
            self.process_alert(alert.clone()).await?;
        }

        Ok(new_alerts)
    }

    /// 🚨 Check Individual Trade for Alerts
    #[instrument(skip(self))]
    pub async fn check_trade_alert(
        &mut self,
        trade: &super::TradeRecord,
    ) -> Result<Option<Alert>> {
        // Check large loss
        if trade.profit_loss_sol < -self.config.max_loss_per_trade_sol {
            let alert = self.create_alert(
                AlertType::LossLimit,
                AlertSeverity::Warning,
                "Large Trade Loss".to_string(),
                format!("Trade loss exceeded threshold: {:.2} SOL < -{:.2} SOL for {}", 
                        trade.profit_loss_sol, self.config.max_loss_per_trade_sol, trade.token_symbol),
                trade.profit_loss_sol.abs(),
                self.config.max_loss_per_trade_sol,
                Some(trade.strategy.clone()),
                Some(trade.token_symbol.clone()),
            );
            
            self.process_alert(alert.clone()).await?;
            return Ok(Some(alert));
        }

        // Check execution delay
        if trade.execution_time_ms > self.config.max_execution_time_ms {
            let alert = self.create_alert(
                AlertType::ExecutionDelay,
                AlertSeverity::Warning,
                "Slow Trade Execution".to_string(),
                format!("Trade execution time exceeded threshold: {}ms > {}ms for {}", 
                        trade.execution_time_ms, self.config.max_execution_time_ms, trade.token_symbol),
                trade.execution_time_ms as f64,
                self.config.max_execution_time_ms as f64,
                Some(trade.strategy.clone()),
                Some(trade.token_symbol.clone()),
            );
            
            self.process_alert(alert.clone()).await?;
            return Ok(Some(alert));
        }

        // Check position size
        if trade.amount_sol > self.config.max_position_size_sol {
            let alert = self.create_alert(
                AlertType::PositionSize,
                AlertSeverity::Warning,
                "Large Position Size".to_string(),
                format!("Position size exceeded threshold: {:.2} SOL > {:.2} SOL for {}", 
                        trade.amount_sol, self.config.max_position_size_sol, trade.token_symbol),
                trade.amount_sol,
                self.config.max_position_size_sol,
                Some(trade.strategy.clone()),
                Some(trade.token_symbol.clone()),
            );
            
            self.process_alert(alert.clone()).await?;
            return Ok(Some(alert));
        }

        Ok(None)
    }

    /// 🚨 Create Alert
    fn create_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        title: String,
        message: String,
        metric_value: f64,
        threshold: f64,
        strategy: Option<String>,
        token_symbol: Option<String>,
    ) -> Alert {
        Alert {
            id: uuid::Uuid::new_v4().to_string(),
            alert_type,
            severity,
            title,
            message,
            metric_value,
            threshold,
            timestamp: Utc::now(),
            strategy,
            token_symbol,
            acknowledged: false,
        }
    }

    /// 🚨 Process Alert (send notifications)
    #[instrument(skip(self))]
    async fn process_alert(&mut self, alert: Alert) -> Result<()> {
        info!("🚨 Processing alert: {} - {}", alert.title, alert.message);

        // Add to active alerts
        self.active_alerts.insert(alert.id.clone(), alert.clone());
        
        // Add to history
        self.alert_history.push(alert.clone());

        // Send webhook notification
        if self.config.enable_webhook_alerts {
            if let Some(webhook_url) = &self.config.webhook_url {
                self.send_webhook_alert(webhook_url, &alert).await?;
            }
        }

        // Send email notification (if configured)
        if self.config.enable_email_alerts {
            self.send_email_alert(&alert).await?;
        }

        // Log alert based on severity
        match alert.severity {
            AlertSeverity::Info => info!("📢 {}: {}", alert.title, alert.message),
            AlertSeverity::Warning => warn!("⚠️ {}: {}", alert.title, alert.message),
            AlertSeverity::Critical => error!("🚨 {}: {}", alert.title, alert.message),
            AlertSeverity::Emergency => error!("🆘 EMERGENCY: {}: {}", alert.title, alert.message),
        }

        Ok(())
    }

    /// 📡 Send Webhook Alert
    #[instrument(skip(self))]
    async fn send_webhook_alert(&self, webhook_url: &str, alert: &Alert) -> Result<()> {
        let client = reqwest::Client::new();
        
        let payload = serde_json::json!({
            "alert_id": alert.id,
            "type": alert.alert_type,
            "severity": alert.severity,
            "title": alert.title,
            "message": alert.message,
            "metric_value": alert.metric_value,
            "threshold": alert.threshold,
            "timestamp": alert.timestamp,
            "strategy": alert.strategy,
            "token": alert.token_symbol
        });

        match client.post(webhook_url)
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Webhook alert sent successfully");
                } else {
                    warn!("⚠️ Webhook alert failed with status: {}", response.status());
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to send webhook alert: {}", e);
            }
        }

        Ok(())
    }

    /// 📧 Send Email Alert (placeholder)
    #[instrument(skip(self))]
    async fn send_email_alert(&self, alert: &Alert) -> Result<()> {
        // Email implementation would go here
        info!("📧 Email alert would be sent: {}", alert.title);
        Ok(())
    }

    /// 📋 Get Active Alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values().collect()
    }

    /// 📋 Get Alert History
    pub fn get_alert_history(&self, limit: Option<usize>) -> Vec<&Alert> {
        let limit = limit.unwrap_or(100);
        self.alert_history.iter().rev().take(limit).collect()
    }

    /// ✅ Acknowledge Alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            info!("✅ Alert acknowledged: {}", alert_id);
        }
        Ok(())
    }

    /// 🗑️ Clear Acknowledged Alerts
    pub fn clear_acknowledged_alerts(&mut self) {
        let before_count = self.active_alerts.len();
        self.active_alerts.retain(|_, alert| !alert.acknowledged);
        let cleared_count = before_count - self.active_alerts.len();
        
        if cleared_count > 0 {
            info!("🗑️ Cleared {} acknowledged alerts", cleared_count);
        }
    }

    /// 📊 Get Alert Statistics
    pub fn get_alert_stats(&self) -> serde_json::Value {
        let total_alerts = self.alert_history.len();
        let active_alerts = self.active_alerts.len();
        let acknowledged_alerts = self.alert_history.iter()
            .filter(|a| a.acknowledged)
            .count();

        let severity_counts = self.alert_history.iter()
            .fold(HashMap::new(), |mut acc, alert| {
                *acc.entry(format!("{:?}", alert.severity)).or_insert(0) += 1;
                acc
            });

        serde_json::json!({
            "total_alerts": total_alerts,
            "active_alerts": active_alerts,
            "acknowledged_alerts": acknowledged_alerts,
            "severity_breakdown": severity_counts,
            "last_alert": self.alert_history.last().map(|a| a.timestamp)
        })
    }
}
