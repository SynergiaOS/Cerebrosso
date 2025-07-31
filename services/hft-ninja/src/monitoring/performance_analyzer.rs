//! 📈 Performance Analysis & Risk Metrics
//! 
//! Zaawansowana analiza wydajności i ryzyka trading bota

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, debug, instrument};
use chrono::{DateTime, Utc, Duration, Timelike};

use super::trading_metrics::{TradeRecord, TradingMetrics};

/// 📊 Performance Analysis Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    // 📈 Risk Metrics
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown_percentage: f64,
    pub value_at_risk_95: f64,
    pub beta: f64,
    pub alpha: f64,
    
    // 📊 Statistical Metrics
    pub volatility: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub correlation_with_sol: f64,
    
    // 🎯 Trading Efficiency
    pub profit_factor: f64,
    pub recovery_factor: f64,
    pub calmar_ratio: f64,
    pub sterling_ratio: f64,
    
    // ⏱️ Time-based Analysis
    pub best_trading_hours: Vec<u32>,
    pub worst_trading_hours: Vec<u32>,
    pub average_hold_time_minutes: f64,
    pub trade_frequency_per_hour: f64,
    
    // 🎯 Strategy Efficiency
    pub strategy_rankings: Vec<StrategyRanking>,
    pub token_performance: HashMap<String, TokenPerformance>,
    
    // 🚨 Risk Warnings
    pub risk_warnings: Vec<RiskWarning>,
    pub recommendations: Vec<String>,
}

/// 🎯 Strategy Performance Ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRanking {
    pub strategy_name: String,
    pub rank: u32,
    pub score: f64,
    pub profit_loss_sol: f64,
    pub win_rate: f64,
    pub risk_adjusted_return: f64,
    pub trades_count: u64,
}

/// 💰 Token Performance Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPerformance {
    pub token_symbol: String,
    pub total_pnl_sol: f64,
    pub trades_count: u64,
    pub win_rate: f64,
    pub average_profit_per_trade: f64,
    pub best_trade_sol: f64,
    pub worst_trade_sol: f64,
    pub volatility: f64,
}

/// 🚨 Risk Warning Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskWarning {
    pub severity: RiskSeverity,
    pub category: RiskCategory,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    Drawdown,
    Volatility,
    Concentration,
    Frequency,
    Execution,
    Liquidity,
}

/// 📈 Performance Analyzer
pub struct PerformanceAnalyzer {
    risk_free_rate: f64, // Annual risk-free rate (e.g., 0.05 for 5%)
}

impl PerformanceAnalyzer {
    /// 🚀 Initialize Performance Analyzer
    pub fn new(risk_free_rate: f64) -> Self {
        info!("📈 Initializing Performance Analyzer with risk-free rate: {:.2}%", risk_free_rate * 100.0);
        
        PerformanceAnalyzer {
            risk_free_rate,
        }
    }

    /// 📊 Analyze Trading Performance
    #[instrument(skip(self, trades))]
    pub async fn analyze_performance(
        &self,
        trades: &[TradeRecord],
        metrics: &TradingMetrics,
    ) -> Result<PerformanceAnalysis> {
        info!("📊 Analyzing performance for {} trades", trades.len());

        if trades.is_empty() {
            return Ok(self.empty_analysis());
        }

        let returns = self.calculate_returns(trades);
        let risk_metrics = self.calculate_risk_metrics(&returns, metrics);
        let time_analysis = self.analyze_time_patterns(trades);
        let strategy_rankings = self.rank_strategies(trades);
        let token_performance = self.analyze_token_performance(trades);
        let risk_warnings = self.generate_risk_warnings(metrics, &risk_metrics);
        let recommendations = self.generate_recommendations(&risk_warnings, metrics);

        let analysis = PerformanceAnalysis {
            sharpe_ratio: risk_metrics.sharpe_ratio,
            sortino_ratio: risk_metrics.sortino_ratio,
            max_drawdown_percentage: risk_metrics.max_drawdown_percentage,
            value_at_risk_95: risk_metrics.value_at_risk_95,
            beta: risk_metrics.beta,
            alpha: risk_metrics.alpha,
            volatility: risk_metrics.volatility,
            skewness: risk_metrics.skewness,
            kurtosis: risk_metrics.kurtosis,
            correlation_with_sol: risk_metrics.correlation_with_sol,
            profit_factor: risk_metrics.profit_factor,
            recovery_factor: risk_metrics.recovery_factor,
            calmar_ratio: risk_metrics.calmar_ratio,
            sterling_ratio: risk_metrics.sterling_ratio,
            best_trading_hours: time_analysis.best_hours,
            worst_trading_hours: time_analysis.worst_hours,
            average_hold_time_minutes: time_analysis.avg_hold_time,
            trade_frequency_per_hour: time_analysis.frequency_per_hour,
            strategy_rankings,
            token_performance,
            risk_warnings,
            recommendations,
        };

        info!("✅ Performance analysis completed. Sharpe ratio: {:.2}, Max drawdown: {:.1}%", 
              analysis.sharpe_ratio, analysis.max_drawdown_percentage);

        Ok(analysis)
    }

    /// 📊 Calculate Returns from Trades
    fn calculate_returns(&self, trades: &[TradeRecord]) -> Vec<f64> {
        trades.iter()
            .map(|trade| trade.profit_loss_percentage / 100.0)
            .collect()
    }

    /// 📈 Calculate Risk Metrics
    fn calculate_risk_metrics(&self, returns: &[f64], metrics: &TradingMetrics) -> RiskMetrics {
        if returns.is_empty() {
            return RiskMetrics::default();
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt();

        // Sharpe Ratio
        let excess_return = mean_return - (self.risk_free_rate / 365.0); // Daily risk-free rate
        let sharpe_ratio = if volatility > 0.0 {
            excess_return / volatility
        } else {
            0.0
        };

        // Sortino Ratio (downside deviation)
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .cloned()
            .collect();
        
        let downside_deviation = if !downside_returns.is_empty() {
            let downside_variance = downside_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / downside_returns.len() as f64;
            downside_variance.sqrt()
        } else {
            volatility
        };

        let sortino_ratio = if downside_deviation > 0.0 {
            excess_return / downside_deviation
        } else {
            0.0
        };

        // Max Drawdown
        let mut peak = 0.0;
        let mut max_drawdown = 0.0;
        let mut cumulative_return = 0.0;

        for &ret in returns {
            cumulative_return += ret;
            if cumulative_return > peak {
                peak = cumulative_return;
            }
            let drawdown = (peak - cumulative_return) / peak.max(1e-10);
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Value at Risk (95%)
        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let var_95_index = (returns.len() as f64 * 0.05) as usize;
        let value_at_risk_95 = if var_95_index < sorted_returns.len() {
            sorted_returns[var_95_index]
        } else {
            0.0
        };

        // Profit Factor
        let gross_profit: f64 = returns.iter().filter(|&&r| r > 0.0).sum();
        let gross_loss: f64 = returns.iter().filter(|&&r| r < 0.0).map(|r| r.abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Recovery Factor
        let recovery_factor = if max_drawdown > 0.0 {
            metrics.total_profit_loss_sol / max_drawdown
        } else {
            0.0
        };

        // Calmar Ratio
        let calmar_ratio = if max_drawdown > 0.0 {
            mean_return / max_drawdown
        } else {
            0.0
        };

        RiskMetrics {
            sharpe_ratio,
            sortino_ratio,
            max_drawdown_percentage: max_drawdown * 100.0,
            value_at_risk_95,
            beta: 1.0, // Simplified - would need market data for real calculation
            alpha: mean_return - self.risk_free_rate / 365.0,
            volatility,
            skewness: self.calculate_skewness(returns, mean_return),
            kurtosis: self.calculate_kurtosis(returns, mean_return, variance),
            correlation_with_sol: 0.0, // Would need SOL price data
            profit_factor,
            recovery_factor,
            calmar_ratio,
            sterling_ratio: calmar_ratio, // Simplified
        }
    }

    /// 📊 Calculate Skewness
    fn calculate_skewness(&self, returns: &[f64], mean: f64) -> f64 {
        if returns.len() < 3 {
            return 0.0;
        }

        let n = returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / n;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        let skewness = returns.iter()
            .map(|r| ((r - mean) / std_dev).powi(3))
            .sum::<f64>() / n;

        skewness
    }

    /// 📊 Calculate Kurtosis
    fn calculate_kurtosis(&self, returns: &[f64], mean: f64, variance: f64) -> f64 {
        if returns.len() < 4 || variance == 0.0 {
            return 0.0;
        }

        let n = returns.len() as f64;
        let std_dev = variance.sqrt();

        let kurtosis = returns.iter()
            .map(|r| ((r - mean) / std_dev).powi(4))
            .sum::<f64>() / n;

        kurtosis - 3.0 // Excess kurtosis
    }

    /// ⏱️ Analyze Time Patterns
    fn analyze_time_patterns(&self, trades: &[TradeRecord]) -> TimeAnalysis {
        let mut hourly_pnl: HashMap<u32, f64> = HashMap::new();
        let mut hourly_counts: HashMap<u32, u32> = HashMap::new();
        let mut total_hold_time = 0.0;
        let mut hold_time_count = 0;

        for trade in trades {
            let hour = trade.timestamp.hour();
            *hourly_pnl.entry(hour).or_insert(0.0) += trade.profit_loss_sol;
            *hourly_counts.entry(hour).or_insert(0) += 1;

            if let Some(exit_time) = trade.exit_timestamp {
                let hold_duration = exit_time.signed_duration_since(trade.timestamp);
                total_hold_time += hold_duration.num_minutes() as f64;
                hold_time_count += 1;
            }
        }

        // Find best and worst hours
        let mut hour_performance: Vec<(u32, f64)> = hourly_pnl.into_iter().collect();
        hour_performance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best_hours = hour_performance.iter().take(3).map(|(h, _)| *h).collect();
        let worst_hours = hour_performance.iter().rev().take(3).map(|(h, _)| *h).collect();

        let avg_hold_time = if hold_time_count > 0 {
            total_hold_time / hold_time_count as f64
        } else {
            0.0
        };

        let frequency_per_hour = trades.len() as f64 / 24.0; // Simplified

        TimeAnalysis {
            best_hours,
            worst_hours,
            avg_hold_time,
            frequency_per_hour,
        }
    }

    /// 🎯 Rank Trading Strategies
    fn rank_strategies(&self, trades: &[TradeRecord]) -> Vec<StrategyRanking> {
        let mut strategy_stats: HashMap<String, StrategyStats> = HashMap::new();

        for trade in trades {
            let stats = strategy_stats.entry(trade.strategy.clone()).or_insert(StrategyStats::default());
            stats.total_pnl += trade.profit_loss_sol;
            stats.trades_count += 1;
            if trade.profit_loss_sol > 0.0 {
                stats.winning_trades += 1;
            }
            stats.returns.push(trade.profit_loss_percentage / 100.0);
        }

        let mut rankings: Vec<StrategyRanking> = strategy_stats.into_iter()
            .map(|(name, stats)| {
                let win_rate = if stats.trades_count > 0 {
                    (stats.winning_trades as f64 / stats.trades_count as f64) * 100.0
                } else {
                    0.0
                };

                let volatility = if stats.returns.len() > 1 {
                    let mean = stats.returns.iter().sum::<f64>() / stats.returns.len() as f64;
                    let variance = stats.returns.iter()
                        .map(|r| (r - mean).powi(2))
                        .sum::<f64>() / stats.returns.len() as f64;
                    variance.sqrt()
                } else {
                    0.0
                };

                let risk_adjusted_return = if volatility > 0.0 {
                    stats.total_pnl / volatility
                } else {
                    stats.total_pnl
                };

                // Score combines profitability, win rate, and risk adjustment
                let score = stats.total_pnl * 0.4 + (win_rate / 100.0) * 0.3 + risk_adjusted_return * 0.3;

                StrategyRanking {
                    strategy_name: name,
                    rank: 0, // Will be set after sorting
                    score,
                    profit_loss_sol: stats.total_pnl,
                    win_rate,
                    risk_adjusted_return,
                    trades_count: stats.trades_count,
                }
            })
            .collect();

        rankings.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        for (i, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = (i + 1) as u32;
        }

        rankings
    }

    /// 💰 Analyze Token Performance
    fn analyze_token_performance(&self, trades: &[TradeRecord]) -> HashMap<String, TokenPerformance> {
        let mut token_stats: HashMap<String, Vec<&TradeRecord>> = HashMap::new();

        for trade in trades {
            token_stats.entry(trade.token_symbol.clone()).or_insert_with(Vec::new).push(trade);
        }

        token_stats.into_iter()
            .map(|(symbol, token_trades)| {
                let total_pnl: f64 = token_trades.iter().map(|t| t.profit_loss_sol).sum();
                let trades_count = token_trades.len() as u64;
                let winning_trades = token_trades.iter().filter(|t| t.profit_loss_sol > 0.0).count();
                let win_rate = (winning_trades as f64 / trades_count as f64) * 100.0;
                let average_profit = total_pnl / trades_count as f64;
                
                let best_trade = token_trades.iter()
                    .map(|t| t.profit_loss_sol)
                    .fold(f64::NEG_INFINITY, f64::max);
                
                let worst_trade = token_trades.iter()
                    .map(|t| t.profit_loss_sol)
                    .fold(f64::INFINITY, f64::min);

                let returns: Vec<f64> = token_trades.iter()
                    .map(|t| t.profit_loss_percentage / 100.0)
                    .collect();
                
                let volatility = if returns.len() > 1 {
                    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
                    let variance = returns.iter()
                        .map(|r| (r - mean).powi(2))
                        .sum::<f64>() / returns.len() as f64;
                    variance.sqrt()
                } else {
                    0.0
                };

                let performance = TokenPerformance {
                    token_symbol: symbol.clone(),
                    total_pnl_sol: total_pnl,
                    trades_count,
                    win_rate,
                    average_profit_per_trade: average_profit,
                    best_trade_sol: best_trade,
                    worst_trade_sol: worst_trade,
                    volatility,
                };

                (symbol, performance)
            })
            .collect()
    }

    /// 🚨 Generate Risk Warnings
    fn generate_risk_warnings(&self, metrics: &TradingMetrics, risk_metrics: &RiskMetrics) -> Vec<RiskWarning> {
        let mut warnings = Vec::new();
        let now = Utc::now();

        // Max Drawdown Warning
        if risk_metrics.max_drawdown_percentage > 20.0 {
            warnings.push(RiskWarning {
                severity: if risk_metrics.max_drawdown_percentage > 50.0 { RiskSeverity::Critical } else { RiskSeverity::High },
                category: RiskCategory::Drawdown,
                message: format!("High drawdown detected: {:.1}%", risk_metrics.max_drawdown_percentage),
                metric_value: risk_metrics.max_drawdown_percentage,
                threshold: 20.0,
                timestamp: now,
            });
        }

        // Low Win Rate Warning
        if metrics.win_rate_percentage < 40.0 && metrics.total_trades > 10 {
            warnings.push(RiskWarning {
                severity: RiskSeverity::Medium,
                category: RiskCategory::Execution,
                message: format!("Low win rate: {:.1}%", metrics.win_rate_percentage),
                metric_value: metrics.win_rate_percentage,
                threshold: 40.0,
                timestamp: now,
            });
        }

        // High Volatility Warning
        if risk_metrics.volatility > 0.1 {
            warnings.push(RiskWarning {
                severity: RiskSeverity::Medium,
                category: RiskCategory::Volatility,
                message: format!("High volatility detected: {:.3}", risk_metrics.volatility),
                metric_value: risk_metrics.volatility,
                threshold: 0.1,
                timestamp: now,
            });
        }

        warnings
    }

    /// 💡 Generate Recommendations
    fn generate_recommendations(&self, warnings: &[RiskWarning], metrics: &TradingMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        for warning in warnings {
            match warning.category {
                RiskCategory::Drawdown => {
                    recommendations.push("Consider reducing position sizes to limit drawdown".to_string());
                    recommendations.push("Implement stricter stop-loss levels".to_string());
                }
                RiskCategory::Execution => {
                    recommendations.push("Review and optimize trading strategies".to_string());
                    recommendations.push("Consider adjusting entry/exit criteria".to_string());
                }
                RiskCategory::Volatility => {
                    recommendations.push("Implement volatility-based position sizing".to_string());
                    recommendations.push("Consider diversifying across more tokens".to_string());
                }
                _ => {}
            }
        }

        if metrics.total_trades < 50 {
            recommendations.push("Collect more trading data for reliable analysis".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Performance looks good! Continue monitoring".to_string());
        }

        recommendations.into_iter().take(5).collect() // Limit to 5 recommendations
    }

    /// 📊 Empty Analysis for No Data
    fn empty_analysis(&self) -> PerformanceAnalysis {
        PerformanceAnalysis {
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown_percentage: 0.0,
            value_at_risk_95: 0.0,
            beta: 0.0,
            alpha: 0.0,
            volatility: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
            correlation_with_sol: 0.0,
            profit_factor: 0.0,
            recovery_factor: 0.0,
            calmar_ratio: 0.0,
            sterling_ratio: 0.0,
            best_trading_hours: vec![],
            worst_trading_hours: vec![],
            average_hold_time_minutes: 0.0,
            trade_frequency_per_hour: 0.0,
            strategy_rankings: vec![],
            token_performance: HashMap::new(),
            risk_warnings: vec![],
            recommendations: vec!["No trading data available yet".to_string()],
        }
    }
}

// Helper structs
#[derive(Debug, Default)]
struct RiskMetrics {
    sharpe_ratio: f64,
    sortino_ratio: f64,
    max_drawdown_percentage: f64,
    value_at_risk_95: f64,
    beta: f64,
    alpha: f64,
    volatility: f64,
    skewness: f64,
    kurtosis: f64,
    correlation_with_sol: f64,
    profit_factor: f64,
    recovery_factor: f64,
    calmar_ratio: f64,
    sterling_ratio: f64,
}

#[derive(Debug)]
struct TimeAnalysis {
    best_hours: Vec<u32>,
    worst_hours: Vec<u32>,
    avg_hold_time: f64,
    frequency_per_hour: f64,
}

#[derive(Debug, Default)]
struct StrategyStats {
    total_pnl: f64,
    trades_count: u64,
    winning_trades: u64,
    returns: Vec<f64>,
}
