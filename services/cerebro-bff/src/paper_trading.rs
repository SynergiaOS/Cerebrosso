//! 📈 Paper Trading Engine - Virtual Portfolio & Execution Simulation
//! 
//! Silnik paper trading umożliwiający testowanie strategii AI bez ryzyka finansowego.
//! Symuluje rzeczywiste warunki rynkowe z uwzględnieniem slippage, gas fees i market impact.

use crate::config::Config;
use crate::ai_agent::{AIDecision, AgentType};
use crate::feedback_system::{TradeResult, MarketSnapshot, PerformanceMetrics};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualPortfolio {
    pub id: Uuid,
    pub name: String,
    pub sol_balance: f64,
    pub token_holdings: HashMap<String, TokenHolding>,
    pub total_value_usd: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub max_drawdown: f64,
    pub sharpe_ratio: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenHolding {
    pub token_address: String,
    pub symbol: String,
    pub amount: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VirtualTradeOrder {
    pub id: Uuid,
    pub portfolio_id: Uuid,
    pub agent_type: AgentType,
    pub order_type: OrderType,
    pub token_address: String,
    pub amount_sol: f64,
    pub expected_tokens: f64,
    pub max_slippage: f64,
    pub created_at: DateTime<Utc>,
    pub status: OrderStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OrderStatus {
    Pending,
    Executed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionSimulation {
    pub base_latency_ms: u64,
    pub network_congestion_factor: f64,
    pub gas_price_gwei: f64,
    pub slippage_model: SlippageModel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlippageModel {
    pub base_slippage: f64,
    pub liquidity_factor: f64,
    pub volatility_multiplier: f64,
    pub market_impact_threshold: f64,
}

pub struct PaperTradingEngine {
    config: Arc<Config>,
    db_pool: PgPool,
    portfolios: Arc<RwLock<HashMap<Uuid, VirtualPortfolio>>>,
    execution_sim: ExecutionSimulation,
    market_data_cache: Arc<RwLock<HashMap<String, MarketSnapshot>>>,
}

impl PaperTradingEngine {
    /// 🚀 Initialize paper trading engine
    pub async fn new(config: Arc<Config>, db_pool: PgPool) -> Result<Self> {
        info!("📈 Initializing Paper Trading Engine v2.0");
        
        let portfolios = Arc::new(RwLock::new(HashMap::new()));
        
        // Default execution simulation parameters
        let execution_sim = ExecutionSimulation {
            base_latency_ms: 50,
            network_congestion_factor: 1.0,
            gas_price_gwei: 0.000005, // Solana fees are much lower
            slippage_model: SlippageModel {
                base_slippage: 0.001, // 0.1% base slippage
                liquidity_factor: 0.5,
                volatility_multiplier: 1.5,
                market_impact_threshold: 1000.0, // $1000 threshold
            },
        };
        
        let market_data_cache = Arc::new(RwLock::new(HashMap::new()));
        
        let engine = PaperTradingEngine {
            config,
            db_pool,
            portfolios,
            execution_sim,
            market_data_cache,
        };
        
        // Load existing portfolios from database
        engine.load_portfolios().await?;
        
        info!("✅ Paper Trading Engine initialized successfully");
        Ok(engine)
    }
    
    /// 📊 Create new virtual portfolio
    pub async fn create_portfolio(&self, name: String, initial_sol: f64) -> Result<Uuid> {
        let portfolio_id = Uuid::new_v4();
        let now = Utc::now();
        
        let portfolio = VirtualPortfolio {
            id: portfolio_id,
            name: name.clone(),
            sol_balance: initial_sol,
            token_holdings: HashMap::new(),
            total_value_usd: initial_sol * 200.0, // Assume SOL = $200
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            total_trades: 0,
            winning_trades: 0,
            max_drawdown: 0.0,
            sharpe_ratio: None,
            created_at: now,
            last_updated: now,
        };
        
        // Save to database
        sqlx::query(
            r#"
            INSERT INTO virtual_portfolio (id, portfolio_name, sol_balance, token_holdings, 
                                         total_value_usd, unrealized_pnl, realized_pnl, 
                                         total_trades, winning_trades, max_drawdown)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(portfolio_id)
        .bind(&name)
        .bind(initial_sol)
        .bind(serde_json::to_value(&portfolio.token_holdings)?)
        .bind(portfolio.total_value_usd)
        .bind(portfolio.unrealized_pnl)
        .bind(portfolio.realized_pnl)
        .bind(portfolio.total_trades as i32)
        .bind(portfolio.winning_trades as i32)
        .bind(portfolio.max_drawdown)
        .execute(&self.db_pool)
        .await?;
        
        // Add to cache
        self.portfolios.write().await.insert(portfolio_id, portfolio);
        
        info!("📊 Created virtual portfolio: {} ({})", name, portfolio_id);
        Ok(portfolio_id)
    }
    
    /// 🎯 Execute virtual trade based on AI decision
    pub async fn execute_virtual_trade(
        &self,
        portfolio_id: Uuid,
        decision: &AIDecision,
        market_snapshot: &MarketSnapshot,
    ) -> Result<TradeResult> {
        let start_time = Instant::now();
        
        // Get portfolio
        let mut portfolios = self.portfolios.write().await;
        let portfolio = portfolios.get_mut(&portfolio_id)
            .ok_or_else(|| anyhow::anyhow!("Portfolio not found: {}", portfolio_id))?;
        
        // Determine trade parameters based on decision
        let (order_type, amount_sol, token_address) = self.parse_decision(decision, market_snapshot)?;
        
        // Simulate market conditions and execution
        let execution_result = self.simulate_execution(
            &order_type,
            amount_sol,
            &token_address,
            market_snapshot,
            portfolio,
        ).await?;
        
        // Update portfolio based on execution
        if execution_result.executed {
            self.update_portfolio_after_trade(portfolio, &execution_result, &order_type).await?;
            
            // Update portfolio in database
            self.save_portfolio_to_db(portfolio).await?;
        }
        
        let execution_time = start_time.elapsed();
        
        info!(
            "🎯 Virtual trade executed: {} {} SOL for {} (P&L: {:.4})",
            if matches!(order_type, OrderType::Buy) { "BUY" } else { "SELL" },
            amount_sol,
            token_address,
            execution_result.amount_sol
        );
        
        Ok(TradeResult {
            executed: execution_result.executed,
            transaction_hash: Some(format!("virtual_{}", Uuid::new_v4())),
            amount_sol: execution_result.amount_sol,
            amount_tokens: execution_result.amount_tokens,
            execution_price: execution_result.execution_price,
            gas_fee: execution_result.gas_fee,
            slippage: execution_result.slippage,
            market_impact: execution_result.market_impact,
            execution_time,
        })
    }
    
    /// 🔄 Simulate trade execution with realistic conditions
    async fn simulate_execution(
        &self,
        order_type: &OrderType,
        amount_sol: f64,
        token_address: &str,
        market_snapshot: &MarketSnapshot,
        portfolio: &VirtualPortfolio,
    ) -> Result<SimulatedExecution> {
        // Check if portfolio has sufficient balance
        match order_type {
            OrderType::Buy => {
                if portfolio.sol_balance < amount_sol {
                    return Ok(SimulatedExecution {
                        executed: false,
                        amount_sol: 0.0,
                        amount_tokens: 0.0,
                        execution_price: market_snapshot.price_usd,
                        gas_fee: 0.0,
                        slippage: 0.0,
                        market_impact: 0.0,
                        failure_reason: Some("Insufficient SOL balance".to_string()),
                    });
                }
            }
            OrderType::Sell => {
                if !portfolio.token_holdings.contains_key(token_address) {
                    return Ok(SimulatedExecution {
                        executed: false,
                        amount_sol: 0.0,
                        amount_tokens: 0.0,
                        execution_price: market_snapshot.price_usd,
                        gas_fee: 0.0,
                        slippage: 0.0,
                        market_impact: 0.0,
                        failure_reason: Some("No tokens to sell".to_string()),
                    });
                }
            }
        }
        
        // Calculate slippage based on trade size and market conditions
        let trade_value_usd = amount_sol * 200.0; // Assume SOL = $200
        let slippage = self.calculate_slippage(trade_value_usd, market_snapshot);
        
        // Calculate market impact
        let market_impact = self.calculate_market_impact(trade_value_usd, market_snapshot);
        
        // Calculate execution price with slippage
        let price_impact = match order_type {
            OrderType::Buy => slippage + market_impact,
            OrderType::Sell => -(slippage + market_impact),
        };
        
        let execution_price = market_snapshot.price_usd * (1.0 + price_impact);
        
        // Calculate token amount
        let amount_tokens = match order_type {
            OrderType::Buy => amount_sol * 200.0 / execution_price, // SOL to tokens
            OrderType::Sell => {
                // For sell, amount_sol represents the SOL value we want to get
                let holding = portfolio.token_holdings.get(token_address).unwrap();
                (amount_sol * 200.0 / execution_price).min(holding.amount)
            }
        };
        
        // Simulate gas fee (very low on Solana)
        let gas_fee = 0.000005; // ~0.000005 SOL
        
        // Simulate network latency
        let network_delay = Duration::from_millis(
            (self.execution_sim.base_latency_ms as f64 * self.execution_sim.network_congestion_factor) as u64
        );
        tokio::time::sleep(network_delay).await;
        
        Ok(SimulatedExecution {
            executed: true,
            amount_sol,
            amount_tokens,
            execution_price,
            gas_fee,
            slippage,
            market_impact,
            failure_reason: None,
        })
    }
    
    /// 📊 Calculate slippage based on trade size and market conditions
    fn calculate_slippage(&self, trade_value_usd: f64, market_snapshot: &MarketSnapshot) -> f64 {
        let base_slippage = self.execution_sim.slippage_model.base_slippage;
        let liquidity_factor = market_snapshot.liquidity_usd.unwrap_or(1_000_000.0);
        
        // Higher slippage for larger trades relative to liquidity
        let size_impact = (trade_value_usd / liquidity_factor) * self.execution_sim.slippage_model.liquidity_factor;
        
        // Volatility increases slippage
        let volatility_impact = market_snapshot.volatility.unwrap_or(0.02) 
            * self.execution_sim.slippage_model.volatility_multiplier;
        
        base_slippage + size_impact + volatility_impact
    }
    
    /// 📈 Calculate market impact
    fn calculate_market_impact(&self, trade_value_usd: f64, market_snapshot: &MarketSnapshot) -> f64 {
        if trade_value_usd < self.execution_sim.slippage_model.market_impact_threshold {
            return 0.0;
        }

        let liquidity = market_snapshot.liquidity_usd.unwrap_or(1_000_000.0);
        (trade_value_usd / liquidity) * 0.001 // 0.1% impact per 1% of liquidity
    }

    /// 🎯 Parse AI decision into trade parameters
    fn parse_decision(
        &self,
        decision: &AIDecision,
        market_snapshot: &MarketSnapshot,
    ) -> Result<(OrderType, f64, String)> {
        let order_type = match decision.action.as_str() {
            "buy" | "execute" => OrderType::Buy,
            "sell" => OrderType::Sell,
            _ => return Err(anyhow::anyhow!("Invalid decision action: {}", decision.action)),
        };

        // Calculate trade size based on confidence and risk assessment
        let base_amount = 1.0; // Base 1 SOL trade
        let confidence_multiplier = decision.confidence.max(0.1).min(1.0);
        let risk_multiplier = (1.0 - decision.risk_assessment).max(0.1).min(1.0);

        let amount_sol = base_amount * confidence_multiplier * risk_multiplier;
        let token_address = market_snapshot.token_address.clone();

        Ok((order_type, amount_sol, token_address))
    }

    /// 📊 Update portfolio after successful trade
    async fn update_portfolio_after_trade(
        &self,
        portfolio: &mut VirtualPortfolio,
        execution: &SimulatedExecution,
        order_type: &OrderType,
    ) -> Result<()> {
        match order_type {
            OrderType::Buy => {
                // Deduct SOL and gas fee
                portfolio.sol_balance -= execution.amount_sol + execution.gas_fee;

                // Add or update token holding
                let token_address = "dummy_token".to_string(); // Would be real token address
                if let Some(holding) = portfolio.token_holdings.get_mut(&token_address) {
                    // Update existing holding with weighted average price
                    let total_value = (holding.amount * holding.average_price) +
                                    (execution.amount_tokens * execution.execution_price);
                    let total_amount = holding.amount + execution.amount_tokens;

                    holding.amount = total_amount;
                    holding.average_price = total_value / total_amount;
                    holding.current_price = execution.execution_price;
                    holding.last_updated = Utc::now();
                } else {
                    // Create new holding
                    portfolio.token_holdings.insert(token_address.clone(), TokenHolding {
                        token_address: token_address.clone(),
                        symbol: "TOKEN".to_string(),
                        amount: execution.amount_tokens,
                        average_price: execution.execution_price,
                        current_price: execution.execution_price,
                        unrealized_pnl: 0.0,
                        last_updated: Utc::now(),
                    });
                }
            }
            OrderType::Sell => {
                // Add SOL from sale (minus gas fee)
                let sol_received = execution.amount_tokens * execution.execution_price / 200.0; // Convert to SOL
                portfolio.sol_balance += sol_received - execution.gas_fee;

                // Update or remove token holding
                let token_address = "dummy_token".to_string();
                if let Some(holding) = portfolio.token_holdings.get_mut(&token_address) {
                    holding.amount -= execution.amount_tokens;

                    // Calculate realized P&L
                    let realized_pnl = (execution.execution_price - holding.average_price) * execution.amount_tokens;
                    portfolio.realized_pnl += realized_pnl;

                    // Remove holding if amount is zero
                    if holding.amount <= 0.0001 {
                        portfolio.token_holdings.remove(&token_address);
                    } else {
                        holding.last_updated = Utc::now();
                    }
                }
            }
        }

        // Update portfolio statistics
        portfolio.total_trades += 1;
        if execution.amount_sol > 0.0 {
            portfolio.winning_trades += 1;
        }
        portfolio.last_updated = Utc::now();

        // Recalculate total value and unrealized P&L
        self.recalculate_portfolio_value(portfolio).await;

        Ok(())
    }

    /// 💰 Recalculate portfolio total value and unrealized P&L
    async fn recalculate_portfolio_value(&self, portfolio: &mut VirtualPortfolio) {
        let sol_value_usd = portfolio.sol_balance * 200.0; // Assume SOL = $200
        let mut tokens_value_usd = 0.0;
        let mut unrealized_pnl = 0.0;

        for holding in portfolio.token_holdings.values_mut() {
            let current_value = holding.amount * holding.current_price;
            let cost_basis = holding.amount * holding.average_price;

            tokens_value_usd += current_value;
            holding.unrealized_pnl = current_value - cost_basis;
            unrealized_pnl += holding.unrealized_pnl;
        }

        portfolio.total_value_usd = sol_value_usd + tokens_value_usd;
        portfolio.unrealized_pnl = unrealized_pnl;
    }

    /// 💾 Save portfolio to database
    async fn save_portfolio_to_db(&self, portfolio: &VirtualPortfolio) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE virtual_portfolio
            SET sol_balance = $1, token_holdings = $2, total_value_usd = $3,
                unrealized_pnl = $4, realized_pnl = $5, total_trades = $6,
                winning_trades = $7, max_drawdown = $8, last_updated = $9
            WHERE id = $10
            "#
        )
        .bind(portfolio.sol_balance)
        .bind(serde_json::to_value(&portfolio.token_holdings)?)
        .bind(portfolio.total_value_usd)
        .bind(portfolio.unrealized_pnl)
        .bind(portfolio.realized_pnl)
        .bind(portfolio.total_trades as i32)
        .bind(portfolio.winning_trades as i32)
        .bind(portfolio.max_drawdown)
        .bind(portfolio.last_updated)
        .bind(portfolio.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// 📂 Load portfolios from database
    async fn load_portfolios(&self) -> Result<()> {
        let rows = sqlx::query(
            r#"
            SELECT id, portfolio_name, sol_balance, token_holdings, total_value_usd,
                   unrealized_pnl, realized_pnl, total_trades, winning_trades,
                   max_drawdown, last_updated
            FROM virtual_portfolio
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut portfolios = self.portfolios.write().await;

        for row in rows {
            let id: Uuid = row.get("id");
            let token_holdings_json: Option<serde_json::Value> = row.get("token_holdings");
            let token_holdings: HashMap<String, TokenHolding> = token_holdings_json
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            let portfolio = VirtualPortfolio {
                id,
                name: row.get("portfolio_name"),
                sol_balance: row.get("sol_balance"),
                token_holdings,
                total_value_usd: row.get("total_value_usd"),
                unrealized_pnl: row.get("unrealized_pnl"),
                realized_pnl: row.get("realized_pnl"),
                total_trades: row.get::<i32, _>("total_trades") as u32,
                winning_trades: row.get::<i32, _>("winning_trades") as u32,
                max_drawdown: row.get("max_drawdown"),
                sharpe_ratio: None,
                created_at: Utc::now(), // Would be loaded from DB in real implementation
                last_updated: row.get("last_updated"),
            };

            portfolios.insert(id, portfolio);
        }

        info!("📂 Loaded {} virtual portfolios from database", portfolios.len());
        Ok(())
    }

    /// 📊 Get portfolio by ID
    pub async fn get_portfolio(&self, portfolio_id: Uuid) -> Option<VirtualPortfolio> {
        self.portfolios.read().await.get(&portfolio_id).cloned()
    }

    /// 📈 Get portfolio performance metrics
    pub async fn get_portfolio_performance(&self, portfolio_id: Uuid) -> Result<PortfolioPerformance> {
        let portfolio = self.get_portfolio(portfolio_id).await
            .ok_or_else(|| anyhow::anyhow!("Portfolio not found"))?;

        let total_pnl = portfolio.realized_pnl + portfolio.unrealized_pnl;
        let win_rate = if portfolio.total_trades > 0 {
            portfolio.winning_trades as f64 / portfolio.total_trades as f64
        } else {
            0.0
        };

        // Calculate ROI (simplified)
        let initial_value = 10.0 * 200.0; // Assume started with 10 SOL
        let roi = (portfolio.total_value_usd - initial_value) / initial_value * 100.0;

        Ok(PortfolioPerformance {
            portfolio_id,
            total_value_usd: portfolio.total_value_usd,
            total_pnl,
            realized_pnl: portfolio.realized_pnl,
            unrealized_pnl: portfolio.unrealized_pnl,
            roi_percentage: roi,
            win_rate,
            total_trades: portfolio.total_trades,
            winning_trades: portfolio.winning_trades,
            max_drawdown: portfolio.max_drawdown,
            sharpe_ratio: portfolio.sharpe_ratio,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SimulatedExecution {
    executed: bool,
    amount_sol: f64,
    amount_tokens: f64,
    execution_price: f64,
    gas_fee: f64,
    slippage: f64,
    market_impact: f64,
    failure_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioPerformance {
    pub portfolio_id: Uuid,
    pub total_value_usd: f64,
    pub total_pnl: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub roi_percentage: f64,
    pub win_rate: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub max_drawdown: f64,
    pub sharpe_ratio: Option<f64>,
}
