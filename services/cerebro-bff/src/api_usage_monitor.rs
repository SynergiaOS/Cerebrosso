//! 📊 API Usage Monitor - Real-time Helius API Usage Tracking
//! 
//! Advanced monitoring system for tracking API usage, costs, and optimization
//! effectiveness with real-time alerts and cost projections.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error};
use chrono::{DateTime, Utc, Datelike};

/// 📊 API usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageStats {
    pub requests_this_hour: u32,
    pub requests_today: u32,
    pub requests_this_month: u32,
    pub rpm_current: f64,
    pub monthly_limit: u32,
    pub usage_percentage: f64,
    pub estimated_monthly_usage: u32,
    pub cost_this_month: f64,
    pub estimated_monthly_cost: f64,
    pub last_updated: DateTime<Utc>,
}

/// 💰 Cost tracking configuration
#[derive(Debug, Clone)]
pub struct CostConfig {
    pub helius_cost_per_request: f64,
    pub quicknode_cost_per_request: f64,
    pub free_tier_limit: u32,
    pub premium_tier_cost: f64,
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            helius_cost_per_request: 0.001, // $0.001 per request after free tier
            quicknode_cost_per_request: 0.0015,
            free_tier_limit: 1_000_000, // 1M requests free
            premium_tier_cost: 99.0, // $99/month for premium
        }
    }
}

/// 🎯 Optimization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    pub webhook_requests_saved: u32,
    pub batch_requests_saved: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub rpc_failovers: u32,
    pub total_requests_saved: u32,
    pub cost_savings: f64,
    pub efficiency_percentage: f64,
}

/// 📈 Usage trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrend {
    pub timestamp: DateTime<Utc>,
    pub requests_count: u32,
    pub cost: f64,
    pub optimization_savings: f64,
}

/// 📊 API Usage Monitor
pub struct ApiUsageMonitor {
    stats: Arc<RwLock<ApiUsageStats>>,
    optimization_metrics: Arc<RwLock<OptimizationMetrics>>,
    cost_config: CostConfig,
    usage_history: Arc<Mutex<Vec<UsageTrend>>>,
    alert_threshold: f64,
    last_alert_time: Arc<Mutex<Option<Instant>>>,
}

impl ApiUsageMonitor {
    /// 🚀 Initialize API usage monitor
    pub fn new(monthly_limit: u32, alert_threshold: f64) -> Self {
        let stats = ApiUsageStats {
            requests_this_hour: 0,
            requests_today: 0,
            requests_this_month: 0,
            rpm_current: 0.0,
            monthly_limit,
            usage_percentage: 0.0,
            estimated_monthly_usage: 0,
            cost_this_month: 0.0,
            estimated_monthly_cost: 0.0,
            last_updated: Utc::now(),
        };

        let optimization_metrics = OptimizationMetrics {
            webhook_requests_saved: 0,
            batch_requests_saved: 0,
            cache_hits: 0,
            cache_misses: 0,
            rpc_failovers: 0,
            total_requests_saved: 0,
            cost_savings: 0.0,
            efficiency_percentage: 0.0,
        };

        Self {
            stats: Arc::new(RwLock::new(stats)),
            optimization_metrics: Arc::new(RwLock::new(optimization_metrics)),
            cost_config: CostConfig::default(),
            usage_history: Arc::new(Mutex::new(Vec::new())),
            alert_threshold,
            last_alert_time: Arc::new(Mutex::new(None)),
        }
    }

    /// 📈 Record API request
    pub async fn record_request(&self, provider: &str, cost: f64) -> Result<()> {
        let mut stats = self.stats.write().await;
        
        stats.requests_this_hour += 1;
        stats.requests_today += 1;
        stats.requests_this_month += 1;
        stats.cost_this_month += cost;
        stats.last_updated = Utc::now();
        
        // Calculate usage percentage
        stats.usage_percentage = (stats.requests_this_month as f64 / stats.monthly_limit as f64) * 100.0;
        
        // Estimate monthly usage based on current trend
        let days_in_month = 30.0;
        let current_day = Utc::now().day() as f64;
        if current_day > 0.0 {
            stats.estimated_monthly_usage = ((stats.requests_this_month as f64 / current_day) * days_in_month) as u32;
            stats.estimated_monthly_cost = (stats.cost_this_month / current_day) * days_in_month;
        }
        
        // Update RPM (requests per minute) - simplified calculation
        stats.rpm_current = stats.requests_this_hour as f64 / 60.0;
        
        info!("📈 API request recorded: {} (${:.4}) - Usage: {:.2}%", 
              provider, cost, stats.usage_percentage);
        
        // Check for alerts
        self.check_usage_alerts().await?;
        
        // Record trend data
        self.record_trend_data(1, cost, 0.0).await?;
        
        Ok(())
    }

    /// 🎯 Record optimization savings
    pub async fn record_optimization(&self, optimization_type: &str, requests_saved: u32, cost_saved: f64) -> Result<()> {
        let mut metrics = self.optimization_metrics.write().await;
        
        match optimization_type {
            "webhook" => metrics.webhook_requests_saved += requests_saved,
            "batch" => metrics.batch_requests_saved += requests_saved,
            "cache_hit" => metrics.cache_hits += requests_saved,
            "cache_miss" => metrics.cache_misses += requests_saved,
            "rpc_failover" => metrics.rpc_failovers += requests_saved,
            _ => {}
        }
        
        metrics.total_requests_saved += requests_saved;
        metrics.cost_savings += cost_saved;
        
        // Calculate efficiency percentage
        let total_potential_requests = metrics.total_requests_saved + self.get_total_requests().await;
        if total_potential_requests > 0 {
            metrics.efficiency_percentage = (metrics.total_requests_saved as f64 / total_potential_requests as f64) * 100.0;
        }
        
        info!("🎯 Optimization recorded: {} - Saved {} requests (${:.4})", 
              optimization_type, requests_saved, cost_saved);
        
        // Record trend data
        self.record_trend_data(0, 0.0, cost_saved).await?;
        
        Ok(())
    }

    /// 🚨 Check for usage alerts
    async fn check_usage_alerts(&self) -> Result<()> {
        let stats = self.stats.read().await;
        let mut last_alert = self.last_alert_time.lock().await;
        
        // Check if we should send an alert
        let should_alert = stats.usage_percentage >= (self.alert_threshold * 100.0) &&
            last_alert.map_or(true, |last| last.elapsed() > Duration::from_secs(3600));
        
        if should_alert {
            warn!("🚨 API Usage Alert: {:.2}% of monthly limit used!", stats.usage_percentage);
            warn!("   Current usage: {}/{} requests", stats.requests_this_month, stats.monthly_limit);
            warn!("   Estimated monthly: {} requests (${:.2})", stats.estimated_monthly_usage, stats.estimated_monthly_cost);
            
            *last_alert = Some(Instant::now());
            
            // Here you could send notifications (email, Slack, etc.)
            self.send_alert_notification(&stats).await?;
        }
        
        Ok(())
    }

    /// 📧 Send alert notification (placeholder)
    async fn send_alert_notification(&self, stats: &ApiUsageStats) -> Result<()> {
        // Placeholder for notification system
        // Could integrate with Slack, email, Discord, etc.
        
        let alert_message = format!(
            "🚨 Cerberus Phoenix API Usage Alert\n\
             Usage: {:.2}% ({}/{})\n\
             Estimated monthly: {} requests\n\
             Estimated cost: ${:.2}",
            stats.usage_percentage,
            stats.requests_this_month,
            stats.monthly_limit,
            stats.estimated_monthly_usage,
            stats.estimated_monthly_cost
        );
        
        info!("📧 Alert notification: {}", alert_message);
        Ok(())
    }

    /// 📊 Record trend data
    async fn record_trend_data(&self, requests: u32, cost: f64, savings: f64) -> Result<()> {
        let mut history = self.usage_history.lock().await;
        
        let trend = UsageTrend {
            timestamp: Utc::now(),
            requests_count: requests,
            cost,
            optimization_savings: savings,
        };
        
        history.push(trend);
        
        // Keep only last 24 hours of data
        let cutoff = Utc::now() - chrono::Duration::hours(24);
        history.retain(|trend| trend.timestamp > cutoff);
        
        Ok(())
    }

    /// 📈 Get current statistics
    pub async fn get_stats(&self) -> ApiUsageStats {
        self.stats.read().await.clone()
    }

    /// 🎯 Get optimization metrics
    pub async fn get_optimization_metrics(&self) -> OptimizationMetrics {
        self.optimization_metrics.read().await.clone()
    }

    /// 📊 Get usage trends
    pub async fn get_usage_trends(&self, hours: u32) -> Vec<UsageTrend> {
        let history = self.usage_history.lock().await;
        let cutoff = Utc::now() - chrono::Duration::hours(hours as i64);
        
        history.iter()
            .filter(|trend| trend.timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// 🔢 Get total requests count
    async fn get_total_requests(&self) -> u32 {
        self.stats.read().await.requests_this_month
    }

    /// 💰 Calculate cost savings
    pub async fn calculate_cost_savings(&self) -> f64 {
        let metrics = self.optimization_metrics.read().await;
        
        let webhook_savings = metrics.webhook_requests_saved as f64 * self.cost_config.helius_cost_per_request;
        let batch_savings = metrics.batch_requests_saved as f64 * self.cost_config.helius_cost_per_request;
        let cache_savings = metrics.cache_hits as f64 * self.cost_config.helius_cost_per_request;
        
        webhook_savings + batch_savings + cache_savings
    }

    /// 📊 Generate comprehensive report
    pub async fn generate_report(&self) -> Result<serde_json::Value> {
        let stats = self.get_stats().await;
        let metrics = self.get_optimization_metrics().await;
        let trends = self.get_usage_trends(24).await;
        let cost_savings = self.calculate_cost_savings().await;
        
        let report = serde_json::json!({
            "api_usage": {
                "current_stats": stats,
                "optimization_metrics": metrics,
                "cost_savings_total": cost_savings,
                "efficiency_rating": if metrics.efficiency_percentage > 80.0 { "Excellent" } 
                                   else if metrics.efficiency_percentage > 60.0 { "Good" }
                                   else if metrics.efficiency_percentage > 40.0 { "Fair" }
                                   else { "Needs Improvement" }
            },
            "trends": {
                "last_24h": trends.len(),
                "avg_requests_per_hour": if !trends.is_empty() { 
                    trends.iter().map(|t| t.requests_count).sum::<u32>() as f64 / 24.0 
                } else { 0.0 },
                "total_savings_24h": trends.iter().map(|t| t.optimization_savings).sum::<f64>()
            },
            "projections": {
                "monthly_usage_projection": stats.estimated_monthly_usage,
                "monthly_cost_projection": stats.estimated_monthly_cost,
                "will_exceed_free_tier": stats.estimated_monthly_usage > self.cost_config.free_tier_limit,
                "days_until_limit": if stats.rpm_current > 0.0 {
                    ((stats.monthly_limit - stats.requests_this_month) as f64 / (stats.rpm_current * 60.0 * 24.0)).max(0.0)
                } else { f64::INFINITY }
            },
            "recommendations": self.generate_recommendations(&stats, &metrics).await
        });
        
        Ok(report)
    }

    /// 💡 Generate optimization recommendations
    async fn generate_recommendations(&self, stats: &ApiUsageStats, metrics: &OptimizationMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if stats.usage_percentage > 80.0 {
            recommendations.push("🚨 Critical: API usage above 80% - consider upgrading plan".to_string());
        }
        
        if metrics.cache_hits < metrics.cache_misses {
            recommendations.push("💾 Improve cache hit rate - consider longer TTL for stable data".to_string());
        }
        
        if metrics.webhook_requests_saved < 1000 {
            recommendations.push("🔔 Configure more webhooks to reduce polling requests".to_string());
        }
        
        if metrics.efficiency_percentage < 50.0 {
            recommendations.push("🎯 Low optimization efficiency - review batching and caching strategies".to_string());
        }
        
        if stats.rpm_current > 8.0 {
            recommendations.push("⚡ High RPM detected - implement request queuing".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("✅ Excellent optimization! System is running efficiently".to_string());
        }
        
        recommendations
    }

    /// 🔄 Reset monthly counters (call at start of each month)
    pub async fn reset_monthly_counters(&self) {
        let mut stats = self.stats.write().await;
        stats.requests_this_month = 0;
        stats.cost_this_month = 0.0;
        stats.usage_percentage = 0.0;
        
        let mut metrics = self.optimization_metrics.write().await;
        *metrics = OptimizationMetrics {
            webhook_requests_saved: 0,
            batch_requests_saved: 0,
            cache_hits: 0,
            cache_misses: 0,
            rpc_failovers: 0,
            total_requests_saved: 0,
            cost_savings: 0.0,
            efficiency_percentage: 0.0,
        };
        
        info!("🔄 Monthly counters reset");
    }
}
