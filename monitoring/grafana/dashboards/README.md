# 📊 Cerberus Phoenix v2.0 - Grafana Dashboards

## 🎯 Overview

This directory contains comprehensive Grafana dashboards for monitoring the Cerberus Phoenix v2.0 AI-driven HFT trading system. The dashboards provide real-time insights into AI performance, trading results, system health, and adaptive learning progress.

## 📈 Available Dashboards

### 1. **Cerberus Phoenix Overview** (`cerberus-phoenix-overview.json`)
**Main system dashboard with high-level metrics**

**Key Panels:**
- 🎯 **AI Decision Performance** - Real-time decision rate and latency
- 💰 **Portfolio Performance** - Portfolio value and ROI trends
- 🤖 **AI Agent Latency** - Heatmap of decision latencies by agent
- 📈 **Trading Results Distribution** - Pie chart of profit/loss/neutral trades
- 🧠 **Agent Performance Comparison** - Win rates and ROI by agent type
- 🔄 **Optimization Activity** - Parameter optimization frequency
- 📊 **Market Data Health** - Data feed latency and update rates
- 🎯 **Confidence Calibration** - AI confidence vs actual accuracy
- 💹 **P&L Distribution** - Histogram of profit/loss amounts
- 🔥 **System Health Score** - Overall system health indicator
- 📈 **Real-time Trading Activity** - Live trading logs

### 2. **AI Performance Deep Dive** (`ai-performance-detailed.json`)
**Detailed AI agent analysis and optimization tracking**

**Key Panels:**
- 🎯 **Decision Accuracy by Agent** - Accuracy trends over time
- ⚡ **Decision Latency Distribution** - Latency heatmaps by agent
- 🧠 **Fast Decision Agent Performance** - Specialized metrics for speed-focused agent
- 🔍 **Context Analysis Agent Performance** - Context processing metrics
- ⚠️ **Risk Assessment Agent Performance** - Risk prediction accuracy
- 🔬 **Deep Analysis Agent Performance** - Long-term strategy effectiveness
- 📊 **Confidence Calibration Curves** - Calibration quality over time
- 🔄 **Parameter Optimization History** - Optimization events timeline
- 📈 **Optimization Improvement Trends** - Expected vs actual improvements
- 🎯 **Decision Confidence vs Success Rate** - Scatter plot analysis
- ⚡ **Model Performance Comparison** - Latency and throughput by model

## 🚨 Alert Integration

The dashboards are integrated with Prometheus alerting rules defined in `/infrastructure/prometheus/alerts/cerberus-alerts.yml`:

### Critical Alerts
- **AI Decision Latency Critical** - >200ms (HFT requirement breach)
- **Portfolio Loss Critical** - >20% loss
- **No Trading Activity** - No trades for 10+ minutes
- **Market Data Source Down** - No updates for 10+ minutes
- **System Health Low** - Multiple component failures

### Warning Alerts
- **AI Decision Latency High** - >100ms
- **AI Agent Performance Degraded** - Win rate <40%
- **Portfolio Loss High** - >10% loss
- **Market Data Latency High** - >1s latency
- **Poor Confidence Calibration** - <30% calibration accuracy

## 📊 Key Metrics Explained

### AI Performance Metrics
- **Decision Rate** - AI decisions per second
- **Latency Percentiles** - 50th, 95th, 99th percentile response times
- **Confidence Accuracy** - How well predicted confidence matches actual success
- **Win Rate** - Percentage of profitable decisions
- **Brier Score** - Calibration quality metric (lower is better)

### Trading Performance Metrics
- **Portfolio ROI** - Return on investment percentage
- **Sharpe Ratio** - Risk-adjusted returns
- **Max Drawdown** - Largest peak-to-trough decline
- **P&L Distribution** - Profit/loss histogram
- **Trade Frequency** - Trades per hour/minute

### System Health Metrics
- **Market Data Latency** - Time to receive price updates
- **Context Processing Time** - AI context analysis duration
- **Feedback Processing Rate** - Learning system throughput
- **Optimization Frequency** - Parameter tuning events

## 🔧 Dashboard Configuration

### Variables
Both dashboards support template variables for filtering:
- **Portfolio** - Filter by specific portfolio ID
- **Agent Type** - Filter by AI agent (FastDecision, ContextAnalysis, etc.)
- **Model** - Filter by AI model (phi3, llama3, etc.)

### Time Ranges
- **Overview Dashboard** - Default: Last 1 hour, 5s refresh
- **AI Performance Dashboard** - Default: Last 6 hours, 10s refresh

### Annotations
- **Parameter Optimizations** - Marked on timeline when agents are optimized
- **Trading Events** - Major trading decisions and outcomes

## 🎯 Performance Targets

### SLA Targets (monitored by alerts)
- **Fast Decision Agent** - <20ms (95th percentile)
- **Context Analysis Agent** - <50ms (95th percentile)
- **Risk Assessment Agent** - <30ms (95th percentile)
- **Deep Analysis Agent** - <200ms (95th percentile)
- **Trading Success Rate** - >85% win rate
- **System Uptime** - >99.9%

### Daily Performance Goals
- **ROI Target** - 5% daily (0.4 SOL from 8 SOL capital)
- **Max Drawdown** - <15%
- **Sharpe Ratio** - >2.0
- **Decision Accuracy** - >80% confidence calibration

## 🚀 Getting Started

1. **Start Infrastructure**
   ```bash
   cd infrastructure
   docker-compose up -d grafana prometheus
   ```

2. **Access Grafana**
   - URL: http://localhost:3001 or http://grafana.localhost
   - Username: admin
   - Password: admin

3. **Import Dashboards**
   Dashboards are automatically provisioned from this directory.

4. **Configure Data Sources**
   Prometheus is automatically configured as data source pointing to http://prometheus:9090

## 📱 Mobile Optimization

Both dashboards are optimized for mobile viewing with:
- Responsive panel layouts
- Touch-friendly controls
- Simplified mobile views
- Key metrics prioritized on small screens

## 🔍 Troubleshooting

### Common Issues
1. **No Data Showing**
   - Check Prometheus is scraping metrics from cerebro-bff:3000/metrics
   - Verify time range matches data availability

2. **High Latency Alerts**
   - Check system resources (CPU, memory)
   - Review AI model performance
   - Verify network connectivity to data sources

3. **Missing Panels**
   - Ensure all required metrics are being exported
   - Check Prometheus configuration
   - Verify dashboard JSON syntax

### Useful Queries
```promql
# AI Decision Rate
rate(cerebro_ai_decisions_total[5m])

# Average Portfolio ROI
avg(cerebro_portfolio_roi_percentage)

# System Health Score
(rate(cerebro_ai_decisions_total[5m]) > 0) * 
(avg(cerebro_agent_performance_score{metric_type="win_rate"}) > 0.5) * 
(rate(cerebro_feedback_processing_total[5m]) > 0)
```

## 📚 Additional Resources

- [Prometheus Metrics Documentation](../prometheus/README.md)
- [Alert Rules Configuration](../prometheus/alerts/README.md)
- [System Architecture Overview](../../docs/architecture.md)
- [Performance Tuning Guide](../../docs/performance.md)
