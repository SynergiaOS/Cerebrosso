#!/bin/bash

# 🐺 CERBERUS PHOENIX V2.0 - AUTOMATED TRADING WORKFLOW
# Operacja: Phoenix Liftoff - Automatyzacja bez Kestra

set -e

echo "🐺 CERBERUS PHOENIX V2.0 - PHOENIX LIFTOFF INITIATED"
echo "=================================================="

# Configuration
CEREBRO_URL="http://localhost:3000"
HFT_NINJA_URL="http://localhost:8090"
QDRANT_URL="http://localhost:6333"
PROMETHEUS_URL="http://localhost:9090"

# Trading parameters
TRADING_PAIRS=("So11111111111111111111111111111111111111112")  # SOL
AMOUNT_SOL=0.1
CONFIDENCE_THRESHOLD=0.8
MAX_ITERATIONS=100
SLEEP_INTERVAL=5

echo "🎯 Starting automated trading loop..."
echo "Target pairs: ${TRADING_PAIRS[@]}"
echo "Amount per trade: ${AMOUNT_SOL} SOL"
echo "Confidence threshold: ${CONFIDENCE_THRESHOLD}"
echo "Max iterations: ${MAX_ITERATIONS}"
echo ""

iteration=0
successful_trades=0
failed_trades=0

while [ $iteration -lt $MAX_ITERATIONS ]; do
    iteration=$((iteration + 1))
    echo "🔄 Iteration $iteration/$MAX_ITERATIONS"
    
    for token_address in "${TRADING_PAIRS[@]}"; do
        echo "  🧠 Generating signal for $token_address..."
        
        # Generate signal via Cerebro-BFF
        signal_response=$(curl -s -X POST "$CEREBRO_URL/trigger/snipe" \
            -H "Content-Type: application/json" \
            -d "{\"token_address\":\"$token_address\",\"amount\":$AMOUNT_SOL}" \
            2>/dev/null || echo '{"error":"failed"}')
        
        # Check if signal generation was successful
        if echo "$signal_response" | jq -e '.confidence' > /dev/null 2>&1; then
            confidence=$(echo "$signal_response" | jq -r '.confidence')
            signal_id=$(echo "$signal_response" | jq -r '.id')
            
            echo "  📊 Signal generated: ID=$signal_id, Confidence=$confidence"
            
            # Check confidence threshold
            if (( $(echo "$confidence >= $CONFIDENCE_THRESHOLD" | bc -l) )); then
                echo "  ✅ Confidence above threshold, executing trade..."
                
                # Execute trade via HFT-Ninja
                execution_response=$(curl -s -X POST "$HFT_NINJA_URL/execute" \
                    -H "Content-Type: application/json" \
                    -d "$signal_response" \
                    2>/dev/null || echo '{"success":false}')
                
                # Check execution result
                if echo "$execution_response" | jq -e '.success' > /dev/null 2>&1; then
                    success=$(echo "$execution_response" | jq -r '.success')
                    if [ "$success" = "true" ]; then
                        profit=$(echo "$execution_response" | jq -r '.profit_sol // 0')
                        latency=$(echo "$execution_response" | jq -r '.latency_ms // 0')
                        tx_hash=$(echo "$execution_response" | jq -r '.tx_hash // "unknown"')
                        
                        successful_trades=$((successful_trades + 1))
                        echo "  🚀 TRADE SUCCESSFUL! Profit: ${profit} SOL, Latency: ${latency}ms, TX: $tx_hash"
                    else
                        failed_trades=$((failed_trades + 1))
                        error=$(echo "$execution_response" | jq -r '.error // "unknown"')
                        echo "  ❌ Trade failed: $error"
                    fi
                else
                    failed_trades=$((failed_trades + 1))
                    echo "  ❌ Execution API error"
                fi
            else
                echo "  ⚠️  Confidence below threshold ($confidence < $CONFIDENCE_THRESHOLD), skipping"
            fi
        else
            echo "  ❌ Signal generation failed"
        fi
    done
    
    # Status summary
    total_trades=$((successful_trades + failed_trades))
    if [ $total_trades -gt 0 ]; then
        success_rate=$(echo "scale=2; $successful_trades * 100 / $total_trades" | bc)
        echo "  📈 Stats: $successful_trades/$total_trades successful (${success_rate}%)"
    fi
    
    echo "  💤 Sleeping ${SLEEP_INTERVAL}s..."
    sleep $SLEEP_INTERVAL
done

echo ""
echo "🏁 PHOENIX LIFTOFF COMPLETED"
echo "=========================="
echo "Total iterations: $MAX_ITERATIONS"
echo "Successful trades: $successful_trades"
echo "Failed trades: $failed_trades"
echo "Total trades: $total_trades"

if [ $total_trades -gt 0 ]; then
    final_success_rate=$(echo "scale=2; $successful_trades * 100 / $total_trades" | bc)
    echo "Final success rate: ${final_success_rate}%"
fi

echo "🐺 Cerberus Phoenix V2.0 - Mission Complete"
