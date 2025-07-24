# 🎣 Helius Webhook Integration - Real-time Token Event Processing

## 🎯 Overview

The **Helius Webhook Ingestor** is a high-performance, secure webhook handler integrated into HFT-Ninja that processes real-time token events from Helius API Pro. It provides intelligent filtering, signal extraction, and seamless integration with the Cerberus Phoenix v2.0 AI trading system.

## 🚀 Features

### **Real-time Event Processing**
- **Secure webhook validation** with Bearer token authentication
- **Rate limiting** (100 requests/minute by default)
- **Intelligent event filtering** for trading-relevant activities
- **Parallel processing** of Kestra workflows and Cerebro-BFF notifications

### **Advanced Signal Extraction**
- **Large volume detection** (>$1000 equivalent transfers)
- **Pump.fun program monitoring** for new token launches
- **Risk indicator analysis** (wash trading, suspicious patterns)
- **Trading signal generation** with confidence scoring

### **Comprehensive Monitoring**
- **Real-time metrics** collection and export
- **Processing latency tracking** (<10ms average)
- **Success/failure rate monitoring**
- **Grafana dashboard integration**

## 📊 Architecture

```
Helius API Pro → Webhook → HFT-Ninja → [Kestra + Cerebro-BFF]
                    ↓
              Signal Processing → AI Analysis → Trading Decisions
```

## 🔧 Configuration

### **Environment Variables**
```bash
# Required
HELIUS_AUTH_TOKEN=your_helius_webhook_auth_token
KESTRA_TRIGGER_URL=http://kestra:8080/api/v1/executions/trigger/helius-webhook
CEREBRO_BFF_URL=http://cerebro-bff:8080

# Optional
WEBHOOK_RATE_LIMIT=100  # requests per minute
```

### **Helius Webhook Setup**
1. **Create webhook** in Helius Dashboard
2. **Set endpoint**: `https://your-domain.com/webhooks/helius`
3. **Configure events**: Token transfers, account changes, program interactions
4. **Set auth token**: Use strong random token for security

## 📡 API Endpoints

### **Webhook Receiver**
```http
POST /webhooks/helius
Authorization: Bearer YOUR_HELIUS_AUTH_TOKEN
Content-Type: application/json

{
  "account_addresses": ["TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj"],
  "transaction_types": ["token_mint"],
  "events": [...]
}
```

### **Metrics Endpoint**
```http
GET /webhooks/metrics

Response:
{
  "webhook_metrics": {
    "total_received": 1250,
    "successful_processing": 1200,
    "failed_processing": 50,
    "kestra_triggers": 1200,
    "cerebro_notifications": 1180,
    "avg_processing_time_ms": 8
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## 🎯 Event Processing Flow

### **1. Webhook Validation**
- **Rate limiting** check (IP-based)
- **Bearer token** validation
- **Payload structure** verification

### **2. Event Filtering**
```rust
// Large volume transfers
if transfer.token_amount > 1000.0 {
    // Generate volume signal
}

// Pump.fun program interactions
if instruction.program_id == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" {
    // New token launch detected
}

// Risk indicators
if transfer.from_user_account == transfer.to_user_account {
    // Potential wash trading
}
```

### **3. Signal Generation**
- **Volume signals** with strength calculation
- **Risk indicators** with severity scoring
- **Confidence metrics** based on data quality
- **Metadata extraction** for context

### **4. Parallel Processing**
- **Kestra workflow** triggering for data pipeline
- **Cerebro-BFF notification** for AI analysis
- **Metrics collection** for monitoring

## 📈 Trading Signals

### **Signal Types**
```rust
pub struct TradingSignal {
    pub signal_type: String,     // "large_volume", "new_token", etc.
    pub strength: f64,           // 0.0 - 1.0
    pub confidence: f64,         // 0.0 - 1.0
    pub metadata: HashMap<String, Value>,
}
```

### **Risk Indicators**
```rust
pub struct RiskIndicator {
    pub risk_type: String,       // "self_transfer", "low_liquidity"
    pub severity: f64,           // 0.0 - 1.0
    pub description: String,
}
```

## 🔍 Monitoring & Alerting

### **Key Metrics**
- **Processing Rate**: Webhooks/second
- **Success Rate**: % successful processing
- **Latency**: Average processing time
- **Error Rate**: Failed webhook processing

### **Grafana Dashboard Panels**
- **Webhook Processing Rate** - Real-time webhook throughput
- **Processing Latency Distribution** - Latency percentiles
- **Success vs Failure Rate** - Processing reliability
- **Signal Generation Rate** - Trading signals per hour
- **Risk Detection Rate** - Risk indicators identified

### **Prometheus Alerts**
```yaml
- alert: WebhookProcessingHigh
  expr: rate(webhook_processing_total[5m]) > 10
  for: 2m
  
- alert: WebhookLatencyHigh  
  expr: histogram_quantile(0.95, webhook_processing_duration_seconds) > 0.05
  for: 1m
```

## 🧪 Testing

### **Local Testing**
```bash
# Start HFT-Ninja
cargo run

# Test webhook endpoint
curl -X POST http://localhost:8090/webhooks/helius \
  -H "Authorization: Bearer your_test_token" \
  -H "Content-Type: application/json" \
  -d @test_payload.json
```

### **Test Payload Example**
```json
{
  "account_addresses": ["TokenkegQfeZyiNwAJbNbGKLQ7d1gQ3XJQsKj1X1g8qj"],
  "transaction_types": ["token_mint"],
  "events": [
    {
      "transaction": {
        "signature": "5Signature123456789...",
        "timestamp": 1678886400
      },
      "token_transfers": [
        {
          "from_user_account": "11111111111111111111111111111111",
          "to_user_account": "22222222222222222222222222222222",
          "token_amount": 1000000000.0,
          "mint": "NewToken111111111111111111111111111111111"
        }
      ]
    }
  ]
}
```

## 🔒 Security

### **Authentication**
- **Bearer token** validation for all webhook requests
- **IP-based rate limiting** to prevent abuse
- **Request size limits** to prevent DoS attacks

### **Data Validation**
- **Payload structure** verification
- **Field type checking** and sanitization
- **Malicious content** filtering

### **Error Handling**
- **Graceful degradation** on service failures
- **Retry logic** for transient errors
- **Circuit breaker** pattern for external services

## 🚀 Deployment

### **Docker Compose**
```yaml
hft-ninja:
  environment:
    HELIUS_AUTH_TOKEN: ${HELIUS_AUTH_TOKEN}
    KESTRA_TRIGGER_URL: ${KESTRA_TRIGGER_URL}
    CEREBRO_BFF_URL: http://cerebro-bff:8080
  ports:
    - "8090:8080"
```

### **Production Considerations**
- **Load balancing** for high-volume webhooks
- **SSL termination** at reverse proxy
- **Log aggregation** for debugging
- **Backup webhook endpoints** for redundancy

## 📚 Integration Examples

### **Kestra Workflow Trigger**
```yaml
id: helius-webhook-processor
namespace: cerberus.trading

triggers:
  - id: webhook-trigger
    type: io.kestra.core.models.triggers.types.Webhook
    
tasks:
  - id: process-token-event
    type: io.kestra.plugin.scripts.python.Script
    script: |
      import json
      webhook_data = {{ trigger.body }}
      # Process token events...
```

### **Cerebro-BFF Integration**
```rust
// In Cerebro-BFF: /api/v1/webhook/token-events
async fn handle_token_events(payload: Json<Value>) -> Result<Json<Value>> {
    let events = payload.get("events").unwrap();
    
    for event in events.as_array().unwrap() {
        // Trigger AI analysis
        let decision = ai_agent.make_decision(&context, &[]).await?;
        
        // Record metrics
        metrics.record_ai_decision(&decision);
    }
    
    Ok(Json(json!({"status": "processed"})))
}
```

## 🎯 Performance Targets

- **Processing Latency**: <10ms average, <50ms 95th percentile
- **Throughput**: >100 webhooks/second
- **Availability**: >99.9% uptime
- **Success Rate**: >99% successful processing

## 🔧 Troubleshooting

### **Common Issues**
1. **Authentication Failures**: Check HELIUS_AUTH_TOKEN
2. **Rate Limiting**: Verify webhook frequency
3. **Processing Errors**: Check Kestra/Cerebro-BFF connectivity
4. **High Latency**: Monitor system resources

### **Debug Commands**
```bash
# Check webhook metrics
curl http://localhost:8090/webhooks/metrics

# View processing logs
docker logs cerberus-hft-ninja | grep webhook

# Test connectivity
curl -f http://kestra:8080/health
curl -f http://cerebro-bff:8080/health
```

## 📖 Additional Resources

- [Helius Webhook Documentation](https://docs.helius.dev/webhooks)
- [Cerberus Phoenix Architecture](../docs/architecture.md)
- [Monitoring Setup Guide](../infrastructure/grafana/README.md)
- [Security Best Practices](../docs/security.md)
