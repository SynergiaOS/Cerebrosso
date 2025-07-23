# 📡 API Reference - Cerberus Phoenix v2.0

## Base URLs
- **Cerebro-BFF**: `http://localhost:3000`
- **HFT-Ninja**: `http://localhost:8090`

## Authentication
Most endpoints require API key authentication:
```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
     http://localhost:3000/api/v1/endpoint
```

---

## 🏥 Health & Status

### Health Check
```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "2.0.0",
  "services": {
    "database": "healthy",
    "cache": "healthy",
    "rpc_providers": "healthy"
  }
}
```

---

## 🔄 Multi-RPC Management

### Get RPC Providers
```http
GET /api/v1/rpc/providers
```

**Response:**
```json
{
  "routing_strategy": "CostOptimized",
  "providers": {
    "helius": {
      "requests_this_month": 15234,
      "usage_percentage": 15.2,
      "success_rate": 99.8,
      "avg_response_time_ms": 45,
      "is_healthy": true
    }
  },
  "summary": {
    "total_providers": 5,
    "healthy_providers": 5,
    "total_requests": 100357
  }
}
```

### Get RPC Performance Report
```http
GET /api/v1/rpc/performance
```

---

## 📊 Usage Monitoring

### Get Usage Report
```http
GET /api/v1/usage/report
```

**Response:**
```json
{
  "api_usage": {
    "current_stats": {
      "requests_this_month": 28500,
      "usage_percentage": 2.85,
      "cost_this_month": 28.50
    },
    "optimization_metrics": {
      "webhook_requests_saved": 38400,
      "total_requests_saved": 66800,
      "cost_savings": 127.50,
      "efficiency_percentage": 87.2
    }
  }
}
```

---

## 🎯 Risk Analysis

### Analyze Token Risk
```http
GET /api/v1/risk/analyze/:token_address
```

**Response:**
```json
{
  "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
  "risk_analysis": {
    "overall_risk_score": 0.23,
    "risk_level": "LOW",
    "confidence": 0.89
  },
  "recommendation": {
    "action": "BUY",
    "confidence": 0.89,
    "max_position_size": 0.05
  }
}
```

---

## 🤖 AI Decision Engine

### Make AI Decision
```http
POST /api/v1/ai/decide
```

**Request Body:**
```json
{
  "context": "New pump.fun token discovered",
  "signals": [
    {
      "type": "liquidity",
      "value": 50000,
      "confidence": 0.8
    }
  ],
  "risk_tolerance": 0.7
}
```

**Response:**
```json
{
  "decision": {
    "action": "BUY",
    "confidence": 0.85,
    "position_size": 0.08,
    "reasoning": "Strong liquidity signals"
  },
  "execution_plan": {
    "entry_price": 0.00123,
    "stop_loss": 0.00098,
    "take_profit": 0.00156
  }
}
```

---

## 🔍 Pump.fun Scanner

### Get Discovered Tokens
```http
GET /api/v1/pump-fun/discovered
```

**Response:**
```json
{
  "tokens": [
    {
      "token_address": "ABC123...",
      "discovery_time": "2024-01-15T10:25:00Z",
      "risk_analysis": {
        "overall_score": 0.45,
        "confidence": 0.78
      },
      "recommendation": {
        "action": "WATCH",
        "reasoning": "Moderate risk, needs more data"
      }
    }
  ],
  "total_discovered": 156
}
```

---

## 💾 Cache Management

### Get Cache Statistics
```http
GET /api/v1/cache/stats
```

**Response:**
```json
{
  "cache_performance": {
    "hit_rate": 73.5,
    "miss_rate": 26.5,
    "total_requests": 15420
  },
  "optimization_stats": {
    "requests_saved": 11334,
    "cost_savings": 11.33
  }
}
```

---

## ❌ Error Responses

### Standard Error Format
```json
{
  "error": {
    "code": "INVALID_TOKEN_ADDRESS",
    "message": "The provided token address is not valid",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Common Error Codes
- `INVALID_TOKEN_ADDRESS` - Invalid Solana token address
- `RATE_LIMIT_EXCEEDED` - API rate limit exceeded
- `PROVIDER_UNAVAILABLE` - All RPC providers are down
- `ANALYSIS_FAILED` - Risk analysis could not be completed

---

## 📊 Rate Limits

- **Standard endpoints**: 100 requests/minute
- **Batch endpoints**: 10 requests/minute
- **Analysis endpoints**: 50 requests/minute

Rate limit headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642248600
```
