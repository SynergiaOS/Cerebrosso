-- 📊 Initial Feedback System Migration
-- This migration creates the basic feedback system tables

-- 🎯 Trade Decisions Table
CREATE TABLE IF NOT EXISTS trade_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type VARCHAR(50) NOT NULL,
    decision_data JSONB NOT NULL,
    market_conditions JSONB NOT NULL,
    confidence DECIMAL(5,4) NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    reasoning TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_agent_type CHECK (agent_type IN ('FastDecision', 'ContextAnalysis', 'RiskAssessment', 'DeepAnalysis'))
);

-- 📈 Trade Results Table  
CREATE TABLE IF NOT EXISTS trade_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    decision_id UUID NOT NULL REFERENCES trade_decisions(id) ON DELETE CASCADE,
    execution_data JSONB NOT NULL,
    performance_metrics JSONB NOT NULL,
    pnl DECIMAL(15,8) NOT NULL,
    roi_percentage DECIMAL(8,4) NOT NULL,
    execution_latency_ms INTEGER NOT NULL,
    slippage DECIMAL(8,6),
    market_impact DECIMAL(8,6),
    confidence_accuracy DECIMAL(5,4),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 🤖 Agent Performance Table
CREATE TABLE IF NOT EXISTS agent_performance (
    agent_type VARCHAR(50) PRIMARY KEY,
    performance_data JSONB NOT NULL,
    success_rate DECIMAL(5,4) NOT NULL DEFAULT 0,
    avg_roi DECIMAL(8,4) NOT NULL DEFAULT 0,
    total_trades INTEGER NOT NULL DEFAULT 0,
    profitable_trades INTEGER NOT NULL DEFAULT 0,
    avg_latency_ms DECIMAL(8,2) NOT NULL DEFAULT 0,
    confidence_calibration DECIMAL(5,4) NOT NULL DEFAULT 0,
    optimal_parameters JSONB,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT valid_agent_type_perf CHECK (agent_type IN ('FastDecision', 'ContextAnalysis', 'RiskAssessment', 'DeepAnalysis'))
);

-- 📊 Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_trade_decisions_agent_type ON trade_decisions(agent_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_trade_decisions_confidence ON trade_decisions(confidence, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_trade_results_pnl ON trade_results(pnl, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_trade_results_decision_id ON trade_results(decision_id);

-- 🚀 Initialize default agent performance records
INSERT INTO agent_performance (agent_type, performance_data, optimal_parameters) VALUES
('FastDecision', '{"initialized": true}', '{"temperature": 0.3, "max_tokens": 1024}'),
('ContextAnalysis', '{"initialized": true}', '{"temperature": 0.5, "max_tokens": 2048}'),
('RiskAssessment', '{"initialized": true}', '{"temperature": 0.4, "max_tokens": 1536}'),
('DeepAnalysis', '{"initialized": true}', '{"temperature": 0.6, "max_tokens": 4096}')
ON CONFLICT (agent_type) DO NOTHING;
