#!/bin/bash
# 📊 Cerberus Phoenix v3.0 - Hive Mind Monitoring Script
# Real-time monitoring of all Hive Mind services

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to check service health
check_service_health() {
    local service_name=$1
    local port=$2
    local endpoint=${3:-"/health"}

    if curl -s -f "http://localhost:$port$endpoint" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $service_name${NC}"
        return 0
    else
        echo -e "${RED}❌ $service_name${NC}"
        return 1
    fi
}

# Function to get service metrics
get_service_metrics() {
    local service_name=$1
    local port=$2

    local response=$(curl -s "http://localhost:$port/metrics" 2>/dev/null || echo "{}")
    echo "$response"
}

# Function to display performance metrics
display_performance_metrics() {
    echo -e "\n${CYAN}⚡ PERFORMANCE METRICS:${NC}"

    # Performance Optimizer metrics
    local perf_metrics=$(get_service_metrics "performance-optimizer" 8500)
    if [ "$perf_metrics" != "{}" ]; then
        local latency_p95=$(echo "$perf_metrics" | jq -r '.latency.p95_ms // "N/A"' 2>/dev/null || echo "N/A")
        local accuracy=$(echo "$perf_metrics" | jq -r '.accuracy.current_accuracy // "N/A"' 2>/dev/null || echo "N/A")
        local cache_hit_rate=$(echo "$perf_metrics" | jq -r '.cache.hit_rate // "N/A"' 2>/dev/null || echo "N/A")

        echo -e "  📈 Latency P95:      ${latency_p95}ms"
        echo -e "  🎯 Accuracy:         ${accuracy}"
        echo -e "  🗄️ Cache Hit Rate:   ${cache_hit_rate}"
    else
        echo -e "  ${YELLOW}⚠️ Performance metrics unavailable${NC}"
    fi
}

# Function to display Hive Mind status
display_hive_mind_status() {
    echo -e "\n${PURPLE}🐝 HIVE MIND STATUS:${NC}"

    # SwarmCoordinator status
    local swarm_status=$(curl -s "http://localhost:8090/status" 2>/dev/null || echo "{}")
    if [ "$swarm_status" != "{}" ]; then
        local swarm_state=$(echo "$swarm_status" | jq -r '.swarm_state // "Unknown"' 2>/dev/null || echo "Unknown")
        local active_agents=$(echo "$swarm_status" | jq -r '.active_agents // "N/A"' 2>/dev/null || echo "N/A")

        echo -e "  🐝 Swarm State:      $swarm_state"
        echo -e "  👥 Active Agents:    $active_agents"
    else
        echo -e "  ${YELLOW}⚠️ SwarmCoordinator unavailable${NC}"
    fi

    # Agent-Strateg status
    local strateg_status=$(curl -s "http://localhost:8100/status" 2>/dev/null || echo "{}")
    if [ "$strateg_status" != "{}" ]; then
        local agent_role=$(echo "$strateg_status" | jq -r '.role // "Unknown"' 2>/dev/null || echo "Unknown")
        local decision_weight=$(echo "$strateg_status" | jq -r '.decision_weight // "N/A"' 2>/dev/null || echo "N/A")

        echo -e "  👑 Agent Role:       $agent_role"
        echo -e "  ⚖️ Decision Weight:  $decision_weight"
    else
        echo -e "  ${YELLOW}⚠️ Agent-Strateg unavailable${NC}"
    fi
}

# Function to display security status
display_security_status() {
    echo -e "\n${CYAN}🛡️ SECURITY STATUS:${NC}"

    # Chainguardia status
    local security_status=$(curl -s "http://localhost:8400/status" 2>/dev/null || echo "{}")
    if [ "$security_status" != "{}" ]; then
        local security_level=$(echo "$security_status" | jq -r '.security_level // "Unknown"' 2>/dev/null || echo "Unknown")
        local active_threats=$(echo "$security_status" | jq -r '.active_threats // "N/A"' 2>/dev/null || echo "N/A")
        local security_score=$(echo "$security_status" | jq -r '.security_score // "N/A"' 2>/dev/null || echo "N/A")

        echo -e "  🛡️ Security Level:   $security_level"
        echo -e "  🚨 Active Threats:   $active_threats"
        echo -e "  📊 Security Score:   $security_score"
    else
        echo -e "  ${YELLOW}⚠️ Chainguardia unavailable${NC}"
    fi
}

# Function to display network status
display_network_status() {
    echo -e "\n${CYAN}🔗 NETWORK STATUS:${NC}"

    # Synk status
    local network_status=$(curl -s "http://localhost:8300/status" 2>/dev/null || echo "{}")
    if [ "$network_status" != "{}" ]; then
        local sync_state=$(echo "$network_status" | jq -r '.sync_state // "Unknown"' 2>/dev/null || echo "Unknown")
        local current_slot=$(echo "$network_status" | jq -r '.current_slot // "N/A"' 2>/dev/null || echo "N/A")
        local network_health=$(echo "$network_status" | jq -r '.network_health // "N/A"' 2>/dev/null || echo "N/A")

        echo -e "  🔗 Sync State:       $sync_state"
        echo -e "  📊 Current Slot:     $current_slot"
        echo -e "  💚 Network Health:   $network_health"
    else
        echo -e "  ${YELLOW}⚠️ Synk unavailable${NC}"
    fi
}

# Main monitoring loop
main() {
    # Clear screen
    clear

    # Display header
    echo -e "${PURPLE}"
    cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║              🐝 HIVE MIND MONITORING DASHBOARD               ║
║                  Cerberus Phoenix v3.0                      ║
╚══════════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"

    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] Monitoring Hive Mind services...${NC}"

    # Service health checks
    echo -e "\n${CYAN}🏥 SERVICE HEALTH:${NC}"

    healthy_count=0
    total_count=0

    services=(
        "SwarmCoordinator:8090"
        "Agent-Strateg:8100"
        "Context-Engine:8200"
        "Synk:8300"
        "Chainguardia:8400"
        "Performance-Optimizer:8500"
        "HFT-Ninja:8080"
        "Cerebro-BFF:3000"
    )

    for service in "${services[@]}"; do
        service_name=$(echo $service | cut -d':' -f1)
        port=$(echo $service | cut -d':' -f2)

        if check_service_health "$service_name" "$port"; then
            ((healthy_count++))
        fi
        ((total_count++))
    done

    echo -e "\n${CYAN}📊 OVERALL HEALTH: $healthy_count/$total_count services healthy${NC}"

    # Display detailed status
    display_hive_mind_status
    display_performance_metrics
    display_security_status
    display_network_status

    # Display infrastructure status
    echo -e "\n${CYAN}🏗️ INFRASTRUCTURE:${NC}"
    check_service_health "Grafana" 3001 ""
    check_service_health "Prometheus" 9090 ""
    check_service_health "Qdrant" 6333 ""
    check_service_health "Traefik" 8082 ""

    # Display useful links
    echo -e "\n${CYAN}🔗 USEFUL LINKS:${NC}"
    echo -e "  📊 Grafana Dashboard:    http://localhost:3001"
    echo -e "  📈 Prometheus Metrics:   http://localhost:9090"
    echo -e "  🗄️ Qdrant Console:       http://localhost:6333/dashboard"
    echo -e "  🌐 Traefik Dashboard:    http://localhost:8082"

    echo -e "\n${BLUE}Press Ctrl+C to exit monitoring${NC}"
}

# Check if running in continuous mode
if [ "${1:-}" = "--continuous" ] || [ "${1:-}" = "-c" ]; then
    echo -e "${CYAN}Starting continuous monitoring (refresh every 10 seconds)...${NC}"
    while true; do
        main
        sleep 10
    done
else
    main
fi