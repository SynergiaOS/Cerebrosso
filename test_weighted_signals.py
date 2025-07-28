#!/usr/bin/env python3
"""
🧪 Test script for Weighted Signals implementation
Tests the new flow: HFT-Ninja (Sniper Engine) -> Cerebro-BFF (AI Analysis)
"""

import requests
import json
import time
from datetime import datetime

# Configuration
HFT_NINJA_URL = "http://localhost:8090"
CEREBRO_BFF_URL = "http://localhost:3000"

def test_sniper_engine():
    """Test the enhanced SniperProfileEngine with weighted signals"""
    print("🎯 Testing SniperProfileEngine with weighted signals...")
    
    try:
        response = requests.get(f"{HFT_NINJA_URL}/test/sniper", timeout=10)
        if response.status_code == 200:
            data = response.json()
            print("✅ SniperEngine test successful!")
            print(f"📊 Engine stats: {data.get('engine_stats', {})}")
            
            # Check if we have test results with weighted signals
            test_results = data.get('test_results', [])
            for result in test_results:
                if result.get('status') == 'passed':
                    profile = result.get('profile', {})
                    print(f"🔍 Token: {result.get('mint')}")
                    print(f"   Weighted Score: {profile.get('weighted_score', 0):.3f}")
                    print(f"   Potential Score: {profile.get('potential_score', 0):.3f}")
                    print(f"   Risk Score: {profile.get('risk_score', 0):.3f}")
                    print(f"   Top Signals: {len(profile.get('top_signals', []))}")
                    
                    # Show top signals
                    for signal in profile.get('top_signals', [])[:3]:
                        print(f"     - {signal.get('signal_name')}: {signal.get('weighted_strength', 0):.3f}")
            
            return True
        else:
            print(f"❌ SniperEngine test failed: {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ SniperEngine test error: {e}")
        return False

def test_cerebro_bff_endpoint():
    """Test the new /api/v1/analyze/tokens endpoint"""
    print("\n🧠 Testing Cerebro-BFF analyze tokens endpoint...")
    
    # Mock token profiles data (simulating what HFT-Ninja would send)
    mock_profiles = [
        {
            "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "score": 0.85,
            "signals": [
                {
                    "signal_type": "VolumeSpike",
                    "strength": 0.8,
                    "confidence": 0.9,
                    "source": "volume_analysis",
                    "weight": 0.7,
                    "weighted_strength": 0.504,
                    "signal_name": "volume_spike"
                }
            ],
            "risk_level": "Medium",
            "analysis_timestamp": int(time.time()),
            "recommended_action": "SendToCerebro",
            "top_signals": [
                {
                    "signal_type": "HighLiquidity",
                    "strength": 0.9,
                    "confidence": 0.95,
                    "source": "liquidity_analysis",
                    "weight": 0.7,
                    "weighted_strength": 0.5985,
                    "signal_name": "high_liquidity"
                },
                {
                    "signal_type": "VolumeSpike",
                    "strength": 0.8,
                    "confidence": 0.9,
                    "source": "volume_analysis",
                    "weight": 0.7,
                    "weighted_strength": 0.504,
                    "signal_name": "volume_spike"
                }
            ],
            "potential_score": 0.75,
            "risk_score": 0.25,
            "weighted_score": 0.85
        }
    ]
    
    payload = {
        "token_profiles": mock_profiles,
        "source": "sniper_engine",
        "timestamp": int(time.time())
    }
    
    try:
        response = requests.post(
            f"{CEREBRO_BFF_URL}/api/v1/analyze/tokens",
            json=payload,
            timeout=15
        )
        
        if response.status_code == 200:
            data = response.json()
            print("✅ Cerebro-BFF analyze tokens successful!")
            print(f"📊 Analysis summary: {data.get('summary', {})}")
            
            # Check AI decisions
            ai_decisions = data.get('ai_decisions', [])
            for decision in ai_decisions:
                if 'ai_decision' in decision:
                    ai_dec = decision['ai_decision']
                    print(f"🤖 AI Decision for {decision.get('mint')}:")
                    print(f"   Action: {ai_dec.get('action')}")
                    print(f"   Confidence: {ai_dec.get('confidence', 0):.2f}")
                    print(f"   Agent: {ai_dec.get('agent_type')}")
                    print(f"   Latency: {ai_dec.get('latency_ms')}ms")
                elif 'error' in decision:
                    print(f"❌ AI Decision failed for {decision.get('mint')}: {decision.get('error')}")
            
            return True
        else:
            print(f"❌ Cerebro-BFF test failed: {response.status_code}")
            print(f"Response: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Cerebro-BFF test error: {e}")
        return False

def test_health_checks():
    """Test health endpoints of both services"""
    print("\n🏥 Testing health endpoints...")
    
    # Test HFT-Ninja health
    try:
        response = requests.get(f"{HFT_NINJA_URL}/health", timeout=5)
        if response.status_code == 200:
            print("✅ HFT-Ninja health check passed")
        else:
            print(f"❌ HFT-Ninja health check failed: {response.status_code}")
    except Exception as e:
        print(f"❌ HFT-Ninja health check error: {e}")
    
    # Test Cerebro-BFF health
    try:
        response = requests.get(f"{CEREBRO_BFF_URL}/health", timeout=5)
        if response.status_code == 200:
            print("✅ Cerebro-BFF health check passed")
        else:
            print(f"❌ Cerebro-BFF health check failed: {response.status_code}")
    except Exception as e:
        print(f"❌ Cerebro-BFF health check error: {e}")

def main():
    """Run all tests"""
    print("🚀 Starting Weighted Signals Integration Tests")
    print("=" * 60)
    
    # Test health first
    test_health_checks()
    
    # Test SniperEngine
    sniper_success = test_sniper_engine()
    
    # Test Cerebro-BFF endpoint
    cerebro_success = test_cerebro_bff_endpoint()
    
    print("\n" + "=" * 60)
    print("📋 Test Summary:")
    print(f"   SniperEngine: {'✅ PASS' if sniper_success else '❌ FAIL'}")
    print(f"   Cerebro-BFF:  {'✅ PASS' if cerebro_success else '❌ FAIL'}")
    
    if sniper_success and cerebro_success:
        print("\n🎉 All tests passed! Weighted signals implementation is working correctly.")
        return 0
    else:
        print("\n⚠️  Some tests failed. Check the services and try again.")
        return 1

if __name__ == "__main__":
    exit(main())
