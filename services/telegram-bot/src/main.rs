//! 📱 Cerberus Telegram Bot - Simple Trading Monitoring
//! 
//! Telegram bot for monitoring Cerberus Phoenix v2.0 HFT trading bot

use anyhow::Result;
use serde_json::Value;
use std::env;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, error};

/// 📱 Bot Commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Cerberus Phoenix v2.0 commands:")]
enum Command {
    #[command(description = "Show help")]
    Help,
    #[command(description = "Current trading status")]
    Status,
    #[command(description = "Profit & Loss summary")]
    Pnl,
    #[command(description = "Recent trades")]
    Trades,
    #[command(description = "Performance metrics")]
    Performance,
    #[command(description = "System health check")]
    Health,
    #[command(description = "Configuration status")]
    Config,
    #[command(description = "Emergency stop trading")]
    Stop,
    #[command(description = "Start trading")]
    Start,
}

/// 🤖 Bot State
#[derive(Clone)]
struct BotState {
    hft_ninja_url: String,
    cerebro_bff_url: String,
    http_client: reqwest::Client,
    monitoring_chat_id: ChatId,
}

impl BotState {
    fn new(hft_ninja_url: String, cerebro_bff_url: String, monitoring_chat_id: ChatId) -> Self {
        Self {
            hft_ninja_url,
            cerebro_bff_url,
            http_client: reqwest::Client::new(),
            monitoring_chat_id,
        }
    }

    /// 📊 Fetch trading status
    async fn fetch_trading_status(&self) -> Result<Value> {
        let response = self.http_client
            .get(&format!("{}/api/dashboard/summary", self.hft_ninja_url))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }

    /// 💰 Fetch P&L data
    async fn fetch_pnl_data(&self) -> Result<Value> {
        let response = self.http_client
            .get(&format!("{}/api/dashboard/performance", self.hft_ninja_url))
            .send()
            .await?;
        
        Ok(response.json().await?)
    }

    /// 🏥 Check service health
    async fn check_service_health(&self, url: &str) -> bool {
        self.http_client
            .get(url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

/// 💬 Command handler
async fn command_handler(bot: Bot, msg: Message, cmd: Command, state: BotState) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            let help_text = "🤖 *Cerberus Phoenix v2.0 Bot*\n\n\
                📊 *Monitoring Commands:*\n\
                /status - Current trading status\n\
                /pnl - Profit & Loss summary\n\
                /trades - Recent trades\n\
                /performance - Performance metrics\n\
                /health - System health check\n\
                /config - Configuration status\n\n\
                ⚙️ *Control Commands:*\n\
                /stop - Emergency stop trading\n\
                /start - Start trading\n\n\
                ℹ️ *Info:*\n\
                /help - Show this help";
            
            bot.send_message(msg.chat.id, help_text)
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        
        Command::Status => {
            match state.fetch_trading_status().await {
                Ok(status) => {
                    let pnl = status["current_pnl_sol"].as_f64().unwrap_or(0.0);
                    let win_rate = status["win_rate_percentage"].as_f64().unwrap_or(0.0);
                    let total_trades = status["total_trades"].as_u64().unwrap_or(0);
                    let success_rate = status["success_rate_percentage"].as_f64().unwrap_or(0.0);
                    let is_healthy = status["is_healthy"].as_bool().unwrap_or(false);
                    
                    let status_emoji = if is_healthy { "🟢" } else { "🔴" };
                    let pnl_emoji = if pnl >= 0.0 { "📈" } else { "📉" };
                    
                    let message = format!(
                        "{} *Trading Status*\n\n\
                        {} *P&L:* {:.4} SOL\n\
                        📊 *Win Rate:* {:.1}%\n\
                        🔢 *Total Trades:* {}\n\
                        🎯 *Success Rate:* {:.1}%\n\n\
                        🕐 *Updated:* {}",
                        status_emoji, pnl_emoji, pnl, win_rate, total_trades, success_rate,
                        chrono::Utc::now().format("%H:%M:%S UTC")
                    );
                    
                    bot.send_message(msg.chat.id, message)
                        .parse_mode(teloxide::types::ParseMode::Markdown)
                        .await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("❌ Failed to fetch status: {}", e)).await?;
                }
            }
        }
        
        Command::Pnl => {
            match state.fetch_pnl_data().await {
                Ok(pnl_data) => {
                    let total_pnl = pnl_data["total_pnl"].as_f64().unwrap_or(0.0);
                    let best_trade = pnl_data["best_trade"].as_f64().unwrap_or(0.0);
                    let worst_trade = pnl_data["worst_trade"].as_f64().unwrap_or(0.0);
                    let total_fees = pnl_data["total_fees"].as_f64().unwrap_or(0.0);
                    let sharpe_ratio = pnl_data["sharpe_ratio"].as_f64().unwrap_or(0.0);
                    
                    let pnl_emoji = if total_pnl >= 0.0 { "📈" } else { "📉" };
                    
                    let message = format!(
                        "💰 *Profit & Loss Summary*\n\n\
                        {} *Total P&L:* {:.4} SOL\n\
                        📈 *Best Trade:* {:.4} SOL\n\
                        📉 *Worst Trade:* {:.4} SOL\n\
                        💸 *Total Fees:* {:.4} SOL\n\
                        📊 *Sharpe Ratio:* {:.2}\n\n\
                        🕐 *Updated:* {}",
                        pnl_emoji, total_pnl, best_trade, worst_trade, total_fees, sharpe_ratio,
                        chrono::Utc::now().format("%H:%M:%S UTC")
                    );
                    
                    bot.send_message(msg.chat.id, message)
                        .parse_mode(teloxide::types::ParseMode::Markdown)
                        .await?;
                }
                Err(e) => {
                    bot.send_message(msg.chat.id, format!("❌ Failed to fetch P&L data: {}", e)).await?;
                }
            }
        }
        
        Command::Health => {
            let hft_health = state.check_service_health(&format!("{}/health", state.hft_ninja_url)).await;
            let cerebro_health = state.check_service_health(&format!("{}/health", state.cerebro_bff_url)).await;
            
            let overall_health = hft_health && cerebro_health;
            let status_emoji = if overall_health { "🟢" } else { "🔴" };
            
            let message = format!(
                "{} *System Health Check*\n\n\
                ⚡ *HFT-Ninja:* {}\n\
                🧠 *Cerebro-BFF:* {}\n\
                📱 *Telegram Bot:* ✅ Healthy\n\n\
                🕐 *Checked:* {}",
                status_emoji,
                if hft_health { "✅ Healthy" } else { "❌ Unhealthy" },
                if cerebro_health { "✅ Healthy" } else { "❌ Unhealthy" },
                chrono::Utc::now().format("%H:%M:%S UTC")
            );
            
            bot.send_message(msg.chat.id, message)
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        
        Command::Stop => {
            bot.send_message(msg.chat.id, "🛑 *Emergency Stop*\n\nThis would stop all trading activities.\n\n⚠️ *Note:* This is a demo command.")
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        
        Command::Start => {
            bot.send_message(msg.chat.id, "▶️ *Start Trading*\n\nThis would resume trading activities.\n\n⚠️ *Note:* This is a demo command.")
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        
        _ => {
            bot.send_message(msg.chat.id, "🚧 Command not implemented yet. Use /help for available commands.").await?;
        }
    }
    
    Ok(())
}

/// 📊 Start monitoring scheduler
async fn start_monitoring_scheduler(bot: Bot, state: BotState) -> Result<()> {
    let scheduler = JobScheduler::new().await?;
    
    // Status updates every 15 minutes
    let bot_clone = bot.clone();
    let state_clone = state.clone();
    
    scheduler.add(Job::new_async("0 */15 * * * *", move |_uuid, _l| {
        let bot = bot_clone.clone();
        let state = state_clone.clone();
        Box::pin(async move {
            if let Ok(status) = state.fetch_trading_status().await {
                let pnl = status["current_pnl_sol"].as_f64().unwrap_or(0.0);
                let win_rate = status["win_rate_percentage"].as_f64().unwrap_or(0.0);
                let total_trades = status["total_trades"].as_u64().unwrap_or(0);
                
                let pnl_emoji = if pnl >= 0.0 { "📈" } else { "📉" };
                
                let message = format!(
                    "📊 *Periodic Status Update*\n\n\
                    {} *P&L:* {:.4} SOL\n\
                    📊 *Win Rate:* {:.1}%\n\
                    🔢 *Trades:* {}\n\n\
                    🕐 {}",
                    pnl_emoji, pnl, win_rate, total_trades,
                    chrono::Utc::now().format("%H:%M UTC")
                );
                
                let _ = bot.send_message(state.monitoring_chat_id, message)
                    .parse_mode(teloxide::types::ParseMode::Markdown)
                    .send()
                    .await;
            }
        })
    })?)?;

    scheduler.start().await?;
    info!("📅 Monitoring scheduler started");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("📱 Starting Cerberus Telegram Bot v2.0...");

    // Load configuration
    let token = env::var("TELEGRAM_TOKEN")
        .expect("Expected TELEGRAM_TOKEN in environment");
    
    let chat_id: i64 = env::var("TELEGRAM_CHAT_ID")
        .expect("Expected TELEGRAM_CHAT_ID in environment")
        .parse()
        .expect("TELEGRAM_CHAT_ID must be a valid chat ID");
    
    let hft_ninja_url = env::var("HFT_NINJA_URL")
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    
    let cerebro_bff_url = env::var("CEREBRO_BFF_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    // Create bot and state
    let bot = Bot::new(token);
    let state = BotState::new(hft_ninja_url, cerebro_bff_url, ChatId(chat_id));

    // Send startup message
    bot.send_message(ChatId(chat_id), "🚀 *Cerberus Phoenix v2.0*\n\nTelegram monitoring bot started!\n\nUse /help for available commands.")
        .parse_mode(teloxide::types::ParseMode::Markdown)
        .send()
        .await?;

    // Start monitoring scheduler
    let monitoring_state = state.clone();
    let monitoring_bot = bot.clone();
    tokio::spawn(async move {
        if let Err(e) = start_monitoring_scheduler(monitoring_bot, monitoring_state).await {
            error!("Monitoring scheduler error: {}", e);
        }
    });

    info!("🚀 Cerberus Telegram Bot is running!");

    // Start command dispatcher
    Command::repl(bot, move |bot, msg, cmd| {
        let state = state.clone();
        async move { command_handler(bot, msg, cmd, state).await }
    })
    .await;

    Ok(())
}
