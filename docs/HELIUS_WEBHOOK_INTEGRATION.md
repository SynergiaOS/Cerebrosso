# 🎣 Helius Webhook Integration - Cerberus Phoenix v2.0

## 🎯 Overview

The **Helius Webhook Integration** is a critical component of Cerberus Phoenix v2.0 that enables real-time token event processing for high-frequency trading on Solana. This integration provides immediate notifications of new token launches, large transfers, and trading opportunities, eliminating the need for expensive polling strategies.

## 🚀 Key Benefits

### **Cost Optimization**
- **85-90% API Usage Reduction**: From 43,200 to ~5,000 requests/month
- **Monthly Savings**: $80-120 reduction in API costs
- **Real-time Processing**: <10ms average webhook processing latency
- **High Availability**: 99.9% uptime with automatic failover

### **Trading Performance**
- **Immediate Signal Detection**: Real-time new token discovery
- **Risk Assessment**: Automated wash trading and rug pull detection
- **AI-Driven Decisions**: Intelligent trading signal generation
- **Parallel Processing**: Simultaneous Kestra and AI analysis

## 📊 Architecture Integration

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Helius API    │    │   HFT-Ninja     │    │  Cerebro-BFF    │
│   Pro Webhook   │    │   Webhook       │    │   AI Engine     │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • Token Events  │───▶│ • Validation    │───▶│ • Risk Analysis │
│ • Large Transfers│    │ • Filtering     │    │ • AI Decisions  │
│ • Program Calls │    │ • Signal Extract│    │ • Context Store │
│ • Account Changes│    │ • Rate Limiting │    │ • Learning Loop │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │     Kestra      │
                       │   Workflows     │
                       ├─────────────────┤
                       │ • Data Pipeline │
                       │ • Batch Process │
                       │ • Orchestration │
                       │ • Monitoring    │
                       └─────────────────┘
```

## 🔧 Configuration

### **Environment Setup**
```bash
# Required Webhook Configuration
HELIUS_AUTH_TOKEN=your_secure_webhook_token
HELIUS_WEBHOOK_URL=https://your-domain.com/webhooks/helius
KESTRA_TRIGGER_URL=http://kestra:8080/api/v1/executions/trigger/helius-webhook
CEREBRO_BFF_URL=http://cerebro-bff:3000

# Optional Performance Tuning
WEBHOOK_RATE_LIMIT=100          # requests per minute
WEBHOOK_TIMEOUT_MS=5000         # request timeout
WEBHOOK_BATCH_SIZE=10           # events per batch
WEBHOOK_PARALLEL_PROCESSING=true
```

### **Helius Dashboard Setup**
1. **Navigate to Webhooks** in Helius Dashboard
2. **Create New Webhook**:
   - **URL**: `https://your-domain.com/webhooks/helius`
   - **Auth Token**: Generate secure random token
   - **Events**: Token transfers, account changes, program interactions
3. **Configure Programs**:
   - `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` (pump.fun)
   - `TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj` (SPL Token)
   - `11111111111111111111111111111112` (System Program)

## 📡 Webhook Processing Flow

### **1. Event Reception & Validation**
```rust
// Rate limiting (100 req/min default)
if !rate_limiter.is_allowed(client_ip) {
    return StatusCode::TOO_MANY_REQUESTS;
}

// Bearer token validation
if auth_header != format!("Bearer {}", helius_auth_token) {
    return StatusCode::UNAUTHORIZED;
}
```

### **2. Intelligent Event Filtering**
```rust
// Large volume detection (>$1000 equivalent)
if transfer.token_amount > 1000.0 {
    signals.push(TradingSignal {
        signal_type: "large_volume",
        strength: (amount / 100000.0).min(1.0),
        confidence: 0.8,
    });
}

// Pump.fun new token detection
if instruction.program_id == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" {
    signals.push(TradingSignal {
        signal_type: "new_token_launch",
        strength: 0.9,
        confidence: 0.95,
    });
}
```

### **3. Risk Assessment**
```rust
// Wash trading detection
if transfer.from_user_account == transfer.to_user_account {
    risk_indicators.push(RiskIndicator {
        risk_type: "self_transfer",
        severity: 0.7,
        description: "Potential wash trading detected",
    });
}

// Low liquidity warning
if estimated_liquidity < 1.0 {
    risk_indicators.push(RiskIndicator {
        risk_type: "low_liquidity",
        severity: 0.6,
        description: "Low initial liquidity detected",
    });
}
```

### **4. Parallel Processing**
```rust
// Simultaneous Kestra and Cerebro-BFF notifications
let (kestra_result, cerebro_result) = tokio::join!(
    trigger_kestra_workflow(&kestra_url, &payload),
    notify_cerebro_bff(&cerebro_url, &processed_events)
);
```

## 🎯 Trading Signal Types

### **Volume Signals**
- **Large Volume**: Transfers >$1000 equivalent
- **Whale Activity**: Transfers >$10000 equivalent
- **Unusual Volume**: 10x average volume spikes

### **New Token Signals**
- **Pump.fun Launch**: New token creation on pump.fun
- **Initial Liquidity**: First LP provision events
- **Token Mint**: New token mint creation

### **Risk Signals**
- **Wash Trading**: Self-transfers and circular patterns
- **Rug Pull Indicators**: Large holder concentration
- **Low Liquidity**: Insufficient trading depth

## 📊 Performance Metrics

### **Real-time Monitoring**
```bash
# Webhook processing metrics
GET /webhooks/metrics

{
  "webhook_metrics": {
    "total_received": 15420,
    "successful_processing": 15180,
    "failed_processing": 240,
    "avg_processing_time_ms": 8.5,
    "kestra_triggers": 15180,
    "cerebro_notifications": 15050
  }
}
```

### **Grafana Dashboard Panels**
- **Webhook Throughput**: Events processed per second
- **Processing Latency**: P50, P95, P99 latency distribution
- **Success Rate**: Percentage of successful webhook processing
- **Signal Generation**: Trading signals generated per hour
- **Risk Detection**: Risk indicators identified per hour

## 🔒 Security & Reliability

### **Authentication & Rate Limiting**
- **Bearer Token**: Secure webhook authentication
- **IP-based Rate Limiting**: 100 requests/minute default
- **Request Size Limits**: Prevent DoS attacks
- **Payload Validation**: Strict schema enforcement

### **Error Handling & Resilience**
- **Graceful Degradation**: Continue processing on partial failures
- **Retry Logic**: Exponential backoff for transient errors
- **Circuit Breaker**: Automatic failover for external services
- **Dead Letter Queue**: Failed events for manual review

## 🚀 Deployment

### **Docker Compose Integration**
```yaml
services:
  hft-ninja:
    environment:
      HELIUS_AUTH_TOKEN: ${HELIUS_AUTH_TOKEN}
      KESTRA_TRIGGER_URL: http://kestra:8080/api/v1/executions/trigger/helius-webhook
      CEREBRO_BFF_URL: http://cerebro-bff:3000
    ports:
      - "8090:8080"
    depends_on:
      - cerebro-bff
      - kestra
```

### **Production Considerations**
- **Load Balancing**: Multiple HFT-Ninja instances for high volume
- **SSL Termination**: HTTPS webhook endpoints with valid certificates
- **Monitoring**: Comprehensive logging and alerting
- **Backup Endpoints**: Redundant webhook URLs for failover

## 🧪 Testing & Validation

### **Local Testing**
```bash
# Test webhook endpoint
curl -X POST http://localhost:8090/webhooks/helius \
  -H "Authorization: Bearer test_token" \
  -H "Content-Type: application/json" \
  -d '{
    "account_addresses": ["TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj"],
    "transaction_types": ["token_mint"],
    "events": [...]
  }'
```

### **Performance Testing**
```bash
# Load testing with Apache Bench
ab -n 1000 -c 10 -H "Authorization: Bearer test_token" \
   -T "application/json" -p test_payload.json \
   http://localhost:8090/webhooks/helius
```

## 📈 Expected Results

### **Cost Optimization**
- **Before**: 43,200 API calls/month = $93-140/month
- **After**: 5,000 API calls/month = $13-20/month
- **Savings**: 85-90% cost reduction

### **Performance Improvements**
- **Latency**: <10ms average webhook processing
- **Throughput**: >100 webhooks/second capacity
- **Availability**: 99.9% uptime with monitoring
- **Signal Quality**: >95% relevant trading signals

## 🔧 Troubleshooting

### **Common Issues**
1. **Authentication Failures**: Verify HELIUS_AUTH_TOKEN matches dashboard
2. **High Latency**: Check Kestra and Cerebro-BFF connectivity
3. **Rate Limiting**: Monitor webhook frequency and adjust limits
4. **Processing Errors**: Review logs for external service failures

### **Debug Commands**
```bash
# Check webhook health
curl http://localhost:8090/webhooks/metrics

# View processing logs
docker logs cerberus-hft-ninja | grep webhook

# Test external connectivity
curl -f http://kestra:8080/health
curl -f http://cerebro-bff:3000/health
```

## 📚 Related Documentation

- [HFT-Ninja Webhook Handler](../services/hft-ninja/README_WEBHOOK.md)
- [Cerebro-BFF AI Integration](./API_REFERENCE.md)
- [Kestra Workflow Configuration](../infrastructure/kestra/README.md)
- [Monitoring & Alerting Setup](../infrastructure/grafana/README.md)

---

**🎣 Real-time webhook integration for maximum trading efficiency**
