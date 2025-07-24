-- 📊 Cerberus Phoenix v2.0 - Feedback System Database Schema
-- TimescaleDB initialization for AI trading feedback and performance analytics

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- 🎯 Trade Decisions Table
-- Stores all AI agent decisions with context and market conditions
CREATE TABLE trade_decisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type VARCHAR(50) NOT NULL,
    decision_data JSONB NOT NULL,
    market_conditions JSONB NOT NULL,
    confidence DECIMAL(5,4) NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    reasoning TEXT,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    CONSTRAINT valid_agent_type CHECK (agent_type IN ('FastDecision', 'ContextAnalysis', 'RiskAssessment', 'DeepAnalysis'))
);

-- 📈 Trade Results Table  
-- Stores execution results and performance metrics for each decision
CREATE TABLE trade_results (
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
-- Aggregated performance metrics for each AI agent
CREATE TABLE agent_performance (
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

-- 📊 Market Snapshots Table
-- Store market conditions for correlation analysis
CREATE TABLE market_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_address VARCHAR(44) NOT NULL,
    price_usd DECIMAL(20,8) NOT NULL,
    volume_24h DECIMAL(20,2),
    liquidity_usd DECIMAL(20,2),
    volatility DECIMAL(8,6),
    market_cap DECIMAL(20,2),
    holder_count INTEGER,
    dex_data JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 🎯 Virtual Portfolio Table
-- Track paper trading portfolio state
CREATE TABLE virtual_portfolio (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    portfolio_name VARCHAR(100) NOT NULL DEFAULT 'default',
    sol_balance DECIMAL(15,8) NOT NULL DEFAULT 0,
    token_holdings JSONB NOT NULL DEFAULT '{}',
    total_value_usd DECIMAL(15,2) NOT NULL DEFAULT 0,
    unrealized_pnl DECIMAL(15,8) NOT NULL DEFAULT 0,
    realized_pnl DECIMAL(15,8) NOT NULL DEFAULT 0,
    total_trades INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    max_drawdown DECIMAL(8,4) NOT NULL DEFAULT 0,
    sharpe_ratio DECIMAL(8,4),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Convert tables to TimescaleDB hypertables for time-series optimization
SELECT create_hypertable('trade_decisions', 'timestamp', chunk_time_interval => INTERVAL '1 day');
SELECT create_hypertable('trade_results', 'timestamp', chunk_time_interval => INTERVAL '1 day');
SELECT create_hypertable('market_snapshots', 'timestamp', chunk_time_interval => INTERVAL '1 hour');

-- 📊 Indexes for performance optimization
CREATE INDEX idx_trade_decisions_agent_type ON trade_decisions(agent_type, timestamp DESC);
CREATE INDEX idx_trade_decisions_confidence ON trade_decisions(confidence, timestamp DESC);
CREATE INDEX idx_trade_results_pnl ON trade_results(pnl, timestamp DESC);
CREATE INDEX idx_trade_results_decision_id ON trade_results(decision_id);
CREATE INDEX idx_market_snapshots_token ON market_snapshots(token_address, timestamp DESC);
CREATE INDEX idx_virtual_portfolio_name ON virtual_portfolio(portfolio_name);

-- 🎯 Performance Views for Analytics
CREATE VIEW agent_performance_summary AS
SELECT 
    agent_type,
    COUNT(*) as total_decisions,
    AVG(confidence) as avg_confidence,
    COUNT(tr.id) as executed_trades,
    AVG(tr.pnl) as avg_pnl,
    AVG(tr.roi_percentage) as avg_roi,
    AVG(tr.execution_latency_ms) as avg_latency,
    STDDEV(tr.roi_percentage) as roi_volatility,
    COUNT(CASE WHEN tr.pnl > 0 THEN 1 END)::DECIMAL / NULLIF(COUNT(tr.id), 0) as win_rate
FROM trade_decisions td
LEFT JOIN trade_results tr ON td.id = tr.decision_id
WHERE td.timestamp >= NOW() - INTERVAL '30 days'
GROUP BY agent_type;

-- 📈 Daily Performance View
CREATE VIEW daily_performance AS
SELECT 
    DATE(timestamp) as trade_date,
    agent_type,
    COUNT(*) as trades_count,
    SUM(pnl) as daily_pnl,
    AVG(roi_percentage) as avg_roi,
    MAX(pnl) as best_trade,
    MIN(pnl) as worst_trade,
    AVG(execution_latency_ms) as avg_latency
FROM trade_results tr
JOIN trade_decisions td ON tr.decision_id = td.id
WHERE tr.timestamp >= NOW() - INTERVAL '90 days'
GROUP BY DATE(timestamp), agent_type
ORDER BY trade_date DESC, agent_type;

-- 🚀 Initialize default agent performance records
INSERT INTO agent_performance (agent_type, performance_data, optimal_parameters) VALUES
('FastDecision', '{"initialized": true}', '{"temperature": 0.3, "max_tokens": 1024}'),
('ContextAnalysis', '{"initialized": true}', '{"temperature": 0.5, "max_tokens": 2048}'),
('RiskAssessment', '{"initialized": true}', '{"temperature": 0.4, "max_tokens": 1536}'),
('DeepAnalysis', '{"initialized": true}', '{"temperature": 0.6, "max_tokens": 4096}');

-- 💼 Initialize default virtual portfolio
INSERT INTO virtual_portfolio (portfolio_name, sol_balance, total_value_usd) VALUES
('paper_trading_main', 10.0, 2000.0);

-- 🔧 Functions for performance calculations
CREATE OR REPLACE FUNCTION calculate_sharpe_ratio(agent_type_param VARCHAR(50), days_back INTEGER DEFAULT 30)
RETURNS DECIMAL(8,4) AS $$
DECLARE
    avg_return DECIMAL(8,4);
    return_stddev DECIMAL(8,4);
    sharpe DECIMAL(8,4);
BEGIN
    SELECT 
        AVG(roi_percentage),
        STDDEV(roi_percentage)
    INTO avg_return, return_stddev
    FROM trade_results tr
    JOIN trade_decisions td ON tr.decision_id = td.id
    WHERE td.agent_type = agent_type_param
    AND tr.timestamp >= NOW() - (days_back || ' days')::INTERVAL;
    
    IF return_stddev IS NULL OR return_stddev = 0 THEN
        RETURN NULL;
    END IF;
    
    sharpe := avg_return / return_stddev;
    RETURN sharpe;
END;
$$ LANGUAGE plpgsql;

-- 📊 Function to update agent performance metrics
CREATE OR REPLACE FUNCTION update_agent_performance(agent_type_param VARCHAR(50))
RETURNS VOID AS $$
DECLARE
    perf_record RECORD;
BEGIN
    SELECT 
        COUNT(*) as total,
        COUNT(CASE WHEN tr.pnl > 0 THEN 1 END) as profitable,
        AVG(tr.roi_percentage) as avg_roi,
        AVG(tr.execution_latency_ms) as avg_latency,
        COUNT(CASE WHEN tr.pnl > 0 THEN 1 END)::DECIMAL / NULLIF(COUNT(*), 0) as success_rate
    INTO perf_record
    FROM trade_results tr
    JOIN trade_decisions td ON tr.decision_id = td.id
    WHERE td.agent_type = agent_type_param
    AND tr.timestamp >= NOW() - INTERVAL '30 days';
    
    UPDATE agent_performance SET
        total_trades = perf_record.total,
        profitable_trades = perf_record.profitable,
        avg_roi = COALESCE(perf_record.avg_roi, 0),
        avg_latency_ms = COALESCE(perf_record.avg_latency, 0),
        success_rate = COALESCE(perf_record.success_rate, 0),
        last_updated = NOW()
    WHERE agent_type = agent_type_param;
END;
$$ LANGUAGE plpgsql;

-- 🎯 Trigger to auto-update agent performance on new trade results
CREATE OR REPLACE FUNCTION trigger_update_agent_performance()
RETURNS TRIGGER AS $$
BEGIN
    -- Update performance metrics for the agent that made this decision
    PERFORM update_agent_performance(
        (SELECT agent_type FROM trade_decisions WHERE id = NEW.decision_id)
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_agent_performance_trigger
    AFTER INSERT ON trade_results
    FOR EACH ROW
    EXECUTE FUNCTION trigger_update_agent_performance();

-- 🔧 Cleanup old data (retention policy)
SELECT add_retention_policy('trade_decisions', INTERVAL '1 year');
SELECT add_retention_policy('trade_results', INTERVAL '1 year');
SELECT add_retention_policy('market_snapshots', INTERVAL '6 months');

COMMIT;
