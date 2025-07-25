#!/bin/bash

# 🧪 Simple Webhook Test for Devnet
echo "🧪 Testing HFT-Ninja Webhook on Devnet"
echo "======================================"

# Test 1: Health Check
echo "1. Testing Health Check..."
curl -s http://localhost:8090/health | jq '.'
echo ""

# Test 2: Webhook Metrics (before)
echo "2. Webhook Metrics (before test)..."
curl -s http://localhost:8090/webhooks/metrics | jq '.'
echo ""

# Test 3: Simple Webhook Test
echo "3. Testing Webhook Endpoint..."
response=$(curl -s -w "%{http_code}" \
  -X POST http://localhost:8090/webhooks/helius \
  -H "Authorization: Bearer test_devnet_token" \
  -H "Content-Type: application/json" \
  -d '{
    "account_addresses": ["9cBo5UJhAGcE9YVLbxUPihSX24DVhPkmqaRGKCJvHM7s"],
    "transaction_types": ["token_transfer"],
    "events": [{
      "transaction": {
        "signature": "DevnetSimpleTest123",
        "timestamp": 1721782286,
        "slot": 123456789,
        "fee": 5000,
        "fee_payer": "9cBo5UJhAGcE9YVLbxUPihSX24DVhPkmqaRGKCJvHM7s"
      },
      "token_transfers": [{
        "from_user_account": "9cBo5UJhAGcE9YVLbxUPihSX24DVhPkmqaRGKCJvHM7s",
        "to_user_account": "11111111111111111111111111111111",
        "token_amount": 1000.0,
        "mint": "DevnetTestToken123",
        "token_standard": "Fungible"
      }]
    }],
    "webhook_type": "devnet_test"
  }' \
  -o /tmp/webhook_response.json)

http_code="${response: -3}"
echo "HTTP Status: $http_code"

if [ -f /tmp/webhook_response.json ]; then
  echo "Response:"
  cat /tmp/webhook_response.json | jq '.' 2>/dev/null || cat /tmp/webhook_response.json
  rm -f /tmp/webhook_response.json
fi
echo ""

# Test 4: Webhook Metrics (after)
echo "4. Webhook Metrics (after test)..."
curl -s http://localhost:8090/webhooks/metrics | jq '.'
echo ""

# Test 5: Large Volume Test
echo "5. Testing Large Volume Detection..."
curl -s -X POST http://localhost:8090/webhooks/helius \
  -H "Authorization: Bearer test_devnet_token" \
  -H "Content-Type: application/json" \
  -d '{
    "events": [{
      "token_transfers": [{
        "from_user_account": "9cBo5UJhAGcE9YVLbxUPihSX24DVhPkmqaRGKCJvHM7s",
        "to_user_account": "11111111111111111111111111111111",
        "token_amount": 50000.0,
        "mint": "LargeVolumeToken123"
      }]
    }]
  }' | jq '.' 2>/dev/null || echo "Large volume test completed"
echo ""

# Test 6: Authentication Test
echo "6. Testing Authentication (should fail)..."
auth_response=$(curl -s -w "%{http_code}" \
  -X POST http://localhost:8090/webhooks/helius \
  -H "Authorization: Bearer invalid_token" \
  -H "Content-Type: application/json" \
  -d '{"test": "auth"}' \
  -o /dev/null)

auth_code="${auth_response: -3}"
echo "HTTP Status with invalid token: $auth_code"
if [ "$auth_code" = "401" ]; then
  echo "✅ Authentication working correctly"
else
  echo "❌ Authentication test failed"
fi
echo ""

# Final Metrics
echo "7. Final Webhook Metrics..."
curl -s http://localhost:8090/webhooks/metrics | jq '.'

echo ""
echo "🎉 Webhook Testing Complete!"
