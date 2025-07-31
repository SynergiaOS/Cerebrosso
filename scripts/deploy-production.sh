#!/bin/bash
# 🚀 Cerberus Phoenix v3.0 - Production Deployment Script
# Enterprise-grade production deployment with canary rollout and rollback capabilities

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
CANARY_NAMESPACE="production"
DEPLOYMENT_NAME="cerberus-hive-mind"
CANARY_DEPLOYMENT_NAME="cerberus-hive-mind-canary"
HEALTH_CHECK_TIMEOUT=300
CANARY_TRAFFIC_PERCENTAGE=10
ROLLBACK_ON_FAILURE=true

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
    exit 1
}

# Banner
echo -e "${PURPLE}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                🚀 PRODUCTION DEPLOYMENT                      ║
║                 Cerberus Phoenix v3.0                       ║
║                   Hive Mind System                          ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Parse command line arguments
DEPLOYMENT_TYPE="canary"
IMAGE_TAG=""
SKIP_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            DEPLOYMENT_TYPE="$2"
            shift 2
            ;;
        --image-tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --type TYPE          Deployment type: canary, rolling, blue-green (default: canary)"
            echo "  --image-tag TAG      Docker image tag to deploy"
            echo "  --skip-tests         Skip pre-deployment tests"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Validate required parameters
if [[ -z "$IMAGE_TAG" ]]; then
    error "Image tag is required. Use --image-tag to specify."
fi

log "🚀 Starting production deployment..."
log "📋 Deployment Configuration:"
log "   Type: $DEPLOYMENT_TYPE"
log "   Image Tag: $IMAGE_TAG"
log "   Namespace: $NAMESPACE"
log "   Skip Tests: $SKIP_TESTS"

# Pre-deployment checks
log "🔍 Running pre-deployment checks..."

# Check kubectl connectivity
if ! kubectl cluster-info &> /dev/null; then
    error "Cannot connect to Kubernetes cluster"
fi

# Check namespace exists
if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
    error "Namespace '$NAMESPACE' does not exist"
fi

# Check if image exists
log "🔍 Verifying image exists: $IMAGE_TAG"
if ! docker manifest inspect "$IMAGE_TAG" &> /dev/null; then
    error "Image '$IMAGE_TAG' does not exist or is not accessible"
fi

success "Pre-deployment checks passed"

# Run pre-deployment tests
if [[ "$SKIP_TESTS" == "false" ]]; then
    log "🧪 Running pre-deployment tests..."
    
    # Run smoke tests against staging
    if ! ./scripts/smoke-tests.sh staging; then
        error "Pre-deployment tests failed"
    fi
    
    success "Pre-deployment tests passed"
else
    warning "Skipping pre-deployment tests"
fi

# Backup current deployment
log "💾 Creating deployment backup..."
kubectl get deployment "$DEPLOYMENT_NAME" -n "$NAMESPACE" -o yaml > "backup-$(date +%Y%m%d-%H%M%S).yaml"
success "Deployment backup created"

# Deploy based on type
case $DEPLOYMENT_TYPE in
    "canary")
        deploy_canary
        ;;
    "rolling")
        deploy_rolling
        ;;
    "blue-green")
        deploy_blue_green
        ;;
    *)
        error "Unknown deployment type: $DEPLOYMENT_TYPE"
        ;;
esac

# Post-deployment verification
log "🔍 Running post-deployment verification..."
verify_deployment

# Update monitoring and dashboards
log "📊 Updating monitoring dashboards..."
update_monitoring

success "🎉 Production deployment completed successfully!"
log "📊 Deployment Summary:"
log "   Image: $IMAGE_TAG"
log "   Type: $DEPLOYMENT_TYPE"
log "   Namespace: $NAMESPACE"
log "   Status: SUCCESS"

# Deployment functions
deploy_canary() {
    log "🎯 Starting canary deployment..."
    
    # Deploy canary version
    log "🚀 Deploying canary version..."
    envsubst < k8s/production/canary-deployment.yaml | kubectl apply -f -
    
    # Wait for canary rollout
    log "⏳ Waiting for canary rollout..."
    kubectl rollout status deployment/"$CANARY_DEPLOYMENT_NAME" -n "$CANARY_NAMESPACE" --timeout="${HEALTH_CHECK_TIMEOUT}s"
    
    # Configure traffic split
    log "🚦 Configuring traffic split (${CANARY_TRAFFIC_PERCENTAGE}% to canary)..."
    configure_traffic_split "$CANARY_TRAFFIC_PERCENTAGE"
    
    # Monitor canary metrics
    log "📊 Monitoring canary metrics..."
    if ! monitor_canary_metrics; then
        error "Canary metrics validation failed"
    fi
    
    # Promote canary to production
    log "🚀 Promoting canary to production..."
    promote_canary
    
    success "Canary deployment completed"
}

deploy_rolling() {
    log "🔄 Starting rolling deployment..."
    
    # Update deployment with new image
    kubectl set image deployment/"$DEPLOYMENT_NAME" \
        cerberus-hive-mind="$IMAGE_TAG" \
        -n "$NAMESPACE"
    
    # Wait for rollout
    log "⏳ Waiting for rolling update..."
    kubectl rollout status deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout="${HEALTH_CHECK_TIMEOUT}s"
    
    success "Rolling deployment completed"
}

deploy_blue_green() {
    log "🔵🟢 Starting blue-green deployment..."
    
    # Create green deployment
    log "🟢 Creating green deployment..."
    envsubst < k8s/production/green-deployment.yaml | kubectl apply -f -
    
    # Wait for green deployment
    kubectl rollout status deployment/"${DEPLOYMENT_NAME}-green" -n "$NAMESPACE" --timeout="${HEALTH_CHECK_TIMEOUT}s"
    
    # Switch traffic to green
    log "🚦 Switching traffic to green deployment..."
    kubectl patch service "$DEPLOYMENT_NAME" -n "$NAMESPACE" \
        -p '{"spec":{"selector":{"version":"green"}}}'
    
    # Cleanup blue deployment
    log "🔵 Cleaning up blue deployment..."
    kubectl delete deployment "${DEPLOYMENT_NAME}-blue" -n "$NAMESPACE" --ignore-not-found=true
    
    success "Blue-green deployment completed"
}

configure_traffic_split() {
    local canary_percentage=$1
    local stable_percentage=$((100 - canary_percentage))
    
    # Update Istio VirtualService for traffic splitting
    cat <<EOF | kubectl apply -f -
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: cerberus-hive-mind
  namespace: $NAMESPACE
spec:
  hosts:
  - cerberus-hive-mind
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: cerberus-hive-mind
        subset: canary
  - route:
    - destination:
        host: cerberus-hive-mind
        subset: stable
      weight: $stable_percentage
    - destination:
        host: cerberus-hive-mind
        subset: canary
      weight: $canary_percentage
EOF
}

monitor_canary_metrics() {
    log "📊 Monitoring canary metrics for 5 minutes..."
    
    local start_time=$(date +%s)
    local end_time=$((start_time + 300)) # 5 minutes
    
    while [[ $(date +%s) -lt $end_time ]]; do
        # Check error rate
        local error_rate=$(kubectl exec -n "$NAMESPACE" deployment/prometheus -- \
            promtool query instant 'rate(http_requests_total{job="cerberus-canary",status=~"5.."}[5m]) / rate(http_requests_total{job="cerberus-canary"}[5m])' | \
            grep -oP '\d+\.\d+' | head -1)
        
        # Check latency
        local p95_latency=$(kubectl exec -n "$NAMESPACE" deployment/prometheus -- \
            promtool query instant 'histogram_quantile(0.95, rate(http_request_duration_seconds_bucket{job="cerberus-canary"}[5m]))' | \
            grep -oP '\d+\.\d+' | head -1)
        
        log "📈 Canary Metrics - Error Rate: ${error_rate:-0}%, P95 Latency: ${p95_latency:-0}ms"
        
        # Validate metrics
        if (( $(echo "${error_rate:-0} > 0.05" | bc -l) )); then
            error "Canary error rate too high: ${error_rate}%"
        fi
        
        if (( $(echo "${p95_latency:-0} > 0.1" | bc -l) )); then
            error "Canary latency too high: ${p95_latency}s"
        fi
        
        sleep 30
    done
    
    success "Canary metrics validation passed"
    return 0
}

promote_canary() {
    log "🚀 Promoting canary to production..."
    
    # Update main deployment with canary image
    kubectl set image deployment/"$DEPLOYMENT_NAME" \
        cerberus-hive-mind="$IMAGE_TAG" \
        -n "$NAMESPACE"
    
    # Wait for rollout
    kubectl rollout status deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout="${HEALTH_CHECK_TIMEOUT}s"
    
    # Remove canary deployment
    kubectl delete deployment "$CANARY_DEPLOYMENT_NAME" -n "$CANARY_NAMESPACE"
    
    # Reset traffic routing
    kubectl delete virtualservice cerberus-hive-mind -n "$NAMESPACE"
    
    success "Canary promoted to production"
}

verify_deployment() {
    log "🔍 Verifying deployment health..."
    
    # Check pod status
    local ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app="$DEPLOYMENT_NAME" --field-selector=status.phase=Running -o name | wc -l)
    local total_pods=$(kubectl get pods -n "$NAMESPACE" -l app="$DEPLOYMENT_NAME" -o name | wc -l)
    
    log "📊 Pod Status: $ready_pods/$total_pods ready"
    
    if [[ $ready_pods -eq 0 ]]; then
        error "No pods are ready"
    fi
    
    # Run health checks
    log "🏥 Running health checks..."
    if ! ./scripts/production-health-check.sh; then
        error "Health checks failed"
    fi
    
    success "Deployment verification passed"
}

update_monitoring() {
    log "📊 Updating monitoring dashboards..."
    
    # Update Grafana dashboards
    if command -v grafana-cli &> /dev/null; then
        grafana-cli admin reset-admin-password admin
        # Import updated dashboards
        for dashboard in dashboards/*.json; do
            curl -X POST \
                -H "Content-Type: application/json" \
                -d @"$dashboard" \
                http://admin:admin@grafana.cerberus-phoenix.com/api/dashboards/db
        done
    fi
    
    success "Monitoring dashboards updated"
}

# Export functions for use in other scripts
export -f log success warning error
export -f deploy_canary deploy_rolling deploy_blue_green
export -f configure_traffic_split monitor_canary_metrics promote_canary
export -f verify_deployment update_monitoring
