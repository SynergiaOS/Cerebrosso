//! 🎯 Execution module - Core trading execution components
//! 
//! Implementuje kluczowe moduły strategii "Certainty-First HFT":
//! - Fee & Tip Optimizer
//! - Transaction Executor
//! - Bundle Manager

pub mod fee_optimizer;

pub use fee_optimizer::FeeOptimizer;
