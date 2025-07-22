//! 🎯 Strategie tradingowe

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Sandwich,
    Arbitrage,
    PiranhaSurf,
}

pub trait Strategy {
    fn name(&self) -> &str;
    fn detect_signals(&self) -> Result<Vec<crate::TradingSignal>>;
    fn execute(&self) -> Result<()>;
}

pub struct SandwichStrategy;

impl Strategy for SandwichStrategy {
    fn name(&self) -> &str {
        "sandwich"
    }

    fn detect_signals(&self) -> Result<Vec<crate::TradingSignal>> {
        // TODO: Implementacja wykrywania sygnałów sandwich
        Ok(vec![])
    }

    fn execute(&self) -> Result<()> {
        // TODO: Implementacja egzekucji sandwich
        Ok(())
    }
}
