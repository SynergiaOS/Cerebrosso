#!/bin/bash
# 🏥 Cerberus Phoenix v3.0 - Production Health Check
# Comprehensive health monitoring for production deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="production"
HEALTH_CHECK_TIMEOUT=30
PERFORMANCE_THRESHOLD_MS=100
ERROR_RATE_THRESHOLD=0.05

# Service endpoints
declare -A SERVICES=(
    ["swarm-coordinator"]="8090"
    ["agent-strateg"]="8100"
    ["context-engine"]="8200"
    ["synk"]="8300"
    ["chainguardia"]="8400"
    ["performance-optimizer"]="8500"
    ["security-hardening"]="8600"
    ["advanced-monitoring"]="8700"
    ["hft-ninja"]="8080"
    ["cerebro-bff"]="3000"
)

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

# Banner
echo -e "${CYAN}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                🏥 PRODUCTION HEALTH CHECK                    ║
║                 Cerberus Phoenix v3.0                       ║
║                   Hive Mind System                          ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

log "🏥 Starting comprehensive production health check..."

# Initialize counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

# Health check functions
check_kubernetes_connectivity() {
    log "🔍 Checking Kubernetes connectivity..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if kubectl cluster-info &> /dev/null; then
        success "Kubernetes cluster is accessible"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        error "Cannot connect to Kubernetes cluster"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

check_namespace_status() {
    log "🔍 Checking namespace status..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if kubectl get namespace "$NAMESPACE" &> /dev/null; then
        success "Namespace '$NAMESPACE' exists and is accessible"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        error "Namespace '$NAMESPACE' does not exist or is not accessible"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

check_pod_status() {
    log "🔍 Checking pod status..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    local ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app=cerberus-hive-mind --field-selector=status.phase=Running -o name | wc -l)
    local total_pods=$(kubectl get pods -n "$NAMESPACE" -l app=cerberus-hive-mind -o name | wc -l)
    
    log "📊 Pod Status: $ready_pods/$total_pods ready"
    
    if [[ $ready_pods -eq $total_pods && $ready_pods -gt 0 ]]; then
        success "All pods are running and ready"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    elif [[ $ready_pods -gt 0 ]]; then
        warning "Some pods are not ready: $ready_pods/$total_pods"
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
        return 1
    else
        error "No pods are running"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
}

check_service_endpoints() {
    log "🔍 Checking service endpoints..."
    
    local service_ip=$(kubectl get service cerberus-hive-mind -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
    
    if [[ -z "$service_ip" ]]; then
        # Try to get cluster IP
        service_ip=$(kubectl get service cerberus-hive-mind -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "")
    fi
    
    if [[ -z "$service_ip" ]]; then
        error "Cannot determine service IP address"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        return 1
    fi
    
    log "📍 Service IP: $service_ip"
    
    # Check each service endpoint
    local endpoint_failures=0
    for service in "${!SERVICES[@]}"; do
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        local port="${SERVICES[$service]}"
        
        log "🔗 Checking $service endpoint (port $port)..."
        
        if timeout "$HEALTH_CHECK_TIMEOUT" curl -f -s "http://$service_ip:$port/health" &> /dev/null; then
            success "$service endpoint is healthy"
            PASSED_CHECKS=$((PASSED_CHECKS + 1))
        else
            error "$service endpoint is not responding"
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
            endpoint_failures=$((endpoint_failures + 1))
        fi
    done
    
    if [[ $endpoint_failures -eq 0 ]]; then
        success "All service endpoints are healthy"
        return 0
    else
        error "$endpoint_failures service endpoints failed health checks"
        return 1
    fi
}

check_performance_metrics() {
    log "🔍 Checking performance metrics..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    # Get performance metrics from performance-optimizer
    local service_ip=$(kubectl get service cerberus-hive-mind -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}')
    
    if [[ -n "$service_ip" ]]; then
        local metrics_response=$(curl -s "http://$service_ip:8500/metrics" 2>/dev/null || echo "{}")
        
        if [[ "$metrics_response" != "{}" ]]; then
            # Parse latency metrics (simplified)
            local p95_latency=$(echo "$metrics_response" | jq -r '.latency.p95_ms // 0' 2>/dev/null || echo "0")
            local accuracy=$(echo "$metrics_response" | jq -r '.accuracy.current_accuracy // 0' 2>/dev/null || echo "0")
            
            log "📊 Performance Metrics:"
            log "   P95 Latency: ${p95_latency}ms"
            log "   Accuracy: ${accuracy}"
            
            # Check performance thresholds
            if (( $(echo "$p95_latency <= $PERFORMANCE_THRESHOLD_MS" | bc -l 2>/dev/null || echo "1") )); then
                success "Performance metrics within acceptable thresholds"
                PASSED_CHECKS=$((PASSED_CHECKS + 1))
                return 0
            else
                warning "Performance metrics exceed thresholds (P95: ${p95_latency}ms > ${PERFORMANCE_THRESHOLD_MS}ms)"
                WARNING_CHECKS=$((WARNING_CHECKS + 1))
                return 1
            fi
        else
            warning "Cannot retrieve performance metrics"
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            return 1
        fi
    else
        warning "Cannot determine service IP for metrics check"
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
        return 1
    fi
}

check_security_status() {
    log "🔍 Checking security status..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    local service_ip=$(kubectl get service cerberus-hive-mind -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}')
    
    if [[ -n "$service_ip" ]]; then
        local security_response=$(curl -s "http://$service_ip:8600/status" 2>/dev/null || echo "{}")
        
        if [[ "$security_response" != "{}" ]]; then
            local security_level=$(echo "$security_response" | jq -r '.security_level // "Unknown"' 2>/dev/null || echo "Unknown")
            local active_threats=$(echo "$security_response" | jq -r '.active_threats // 0' 2>/dev/null || echo "0")
            
            log "🛡️ Security Status:"
            log "   Security Level: $security_level"
            log "   Active Threats: $active_threats"
            
            if [[ "$active_threats" == "0" ]]; then
                success "No active security threats detected"
                PASSED_CHECKS=$((PASSED_CHECKS + 1))
                return 0
            else
                warning "$active_threats active security threats detected"
                WARNING_CHECKS=$((WARNING_CHECKS + 1))
                return 1
            fi
        else
            warning "Cannot retrieve security status"
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            return 1
        fi
    else
        warning "Cannot determine service IP for security check"
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
        return 1
    fi
}

check_monitoring_status() {
    log "🔍 Checking monitoring status..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    local service_ip=$(kubectl get service cerberus-hive-mind -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}')
    
    if [[ -n "$service_ip" ]]; then
        local monitoring_response=$(curl -s "http://$service_ip:8700/system/health" 2>/dev/null || echo "{}")
        
        if [[ "$monitoring_response" != "{}" ]]; then
            local overall_status=$(echo "$monitoring_response" | jq -r '.overall_status // "Unknown"' 2>/dev/null || echo "Unknown")
            local performance_score=$(echo "$monitoring_response" | jq -r '.performance_score // 0' 2>/dev/null || echo "0")
            
            log "📊 Monitoring Status:"
            log "   Overall Status: $overall_status"
            log "   Performance Score: $performance_score"
            
            if [[ "$overall_status" == "Healthy" ]]; then
                success "System monitoring reports healthy status"
                PASSED_CHECKS=$((PASSED_CHECKS + 1))
                return 0
            else
                warning "System monitoring reports non-healthy status: $overall_status"
                WARNING_CHECKS=$((WARNING_CHECKS + 1))
                return 1
            fi
        else
            warning "Cannot retrieve monitoring status"
            WARNING_CHECKS=$((WARNING_CHECKS + 1))
            return 1
        fi
    else
        warning "Cannot determine service IP for monitoring check"
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
        return 1
    fi
}

check_resource_usage() {
    log "🔍 Checking resource usage..."
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    # Get resource usage for pods
    local resource_output=$(kubectl top pods -n "$NAMESPACE" -l app=cerberus-hive-mind 2>/dev/null || echo "")
    
    if [[ -n "$resource_output" ]]; then
        log "📊 Resource Usage:"
        echo "$resource_output" | while read -r line; do
            if [[ "$line" != "NAME"* ]]; then
                log "   $line"
            fi
        done
        
        success "Resource usage information retrieved"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        warning "Cannot retrieve resource usage information (metrics-server may not be available)"
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
        return 1
    fi
}

# Run all health checks
main() {
    log "🏥 Running comprehensive health checks..."
    
    check_kubernetes_connectivity
    check_namespace_status
    check_pod_status
    check_service_endpoints
    check_performance_metrics
    check_security_status
    check_monitoring_status
    check_resource_usage
    
    # Generate health report
    echo -e "\n${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                    HEALTH CHECK REPORT                       ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    
    log "📊 Health Check Summary:"
    log "   Total Checks: $TOTAL_CHECKS"
    success "   Passed: $PASSED_CHECKS"
    if [[ $WARNING_CHECKS -gt 0 ]]; then
        warning "   Warnings: $WARNING_CHECKS"
    fi
    if [[ $FAILED_CHECKS -gt 0 ]]; then
        error "   Failed: $FAILED_CHECKS"
    fi
    
    # Calculate health score
    local health_score=$(echo "scale=2; ($PASSED_CHECKS + $WARNING_CHECKS * 0.5) / $TOTAL_CHECKS * 100" | bc -l 2>/dev/null || echo "0")
    log "   Health Score: ${health_score}%"
    
    # Determine overall status
    if [[ $FAILED_CHECKS -eq 0 && $WARNING_CHECKS -eq 0 ]]; then
        success "🎉 All health checks passed! System is fully operational."
        return 0
    elif [[ $FAILED_CHECKS -eq 0 ]]; then
        warning "⚠️ Health checks completed with warnings. System is operational but needs attention."
        return 1
    else
        error "❌ Health checks failed. System requires immediate attention."
        return 2
    fi
}

# Run main function
main "$@"
