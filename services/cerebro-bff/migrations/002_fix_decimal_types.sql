-- 🔧 Fix DECIMAL types to FLOAT8 for Rust compatibility
-- This migration converts DECIMAL columns to FLOAT8 to match Rust f64 types

-- 📊 Fix trade_decisions table
ALTER TABLE trade_decisions 
    ALTER COLUMN confidence TYPE FLOAT8;

-- 📈 Fix trade_results table  
ALTER TABLE trade_results
    ALTER COLUMN pnl TYPE FLOAT8,
    ALTER COLUMN roi_percentage TYPE FLOAT8,
    ALTER COLUMN slippage TYPE FLOAT8,
    ALTER COLUMN market_impact TYPE FLOAT8,
    ALTER COLUMN confidence_accuracy TYPE FLOAT8;

-- 🤖 Fix agent_performance table
ALTER TABLE agent_performance
    ALTER COLUMN success_rate TYPE FLOAT8,
    ALTER COLUMN avg_roi TYPE FLOAT8,
    ALTER COLUMN avg_latency_ms TYPE FLOAT8,
    ALTER COLUMN confidence_calibration TYPE FLOAT8;
