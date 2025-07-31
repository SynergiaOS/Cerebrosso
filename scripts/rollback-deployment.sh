#!/bin/bash
# 🔄 Cerberus Phoenix v3.0 - Deployment Rollback Script
# Emergency rollback capabilities with automated recovery

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
DEPLOYMENT_NAME="cerberus-hive-mind"
ROLLBACK_TIMEOUT=300

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
echo -e "${RED}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                🔄 EMERGENCY ROLLBACK                         ║
║                 Cerberus Phoenix v3.0                       ║
║                   Hive Mind System                          ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Parse command line arguments
ROLLBACK_TYPE="auto"
TARGET_REVISION=""
FORCE_ROLLBACK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            ROLLBACK_TYPE="$2"
            shift 2
            ;;
        --revision)
            TARGET_REVISION="$2"
            shift 2
            ;;
        --force)
            FORCE_ROLLBACK=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --type TYPE          Rollback type: auto, manual, emergency (default: auto)"
            echo "  --revision REV       Target revision to rollback to"
            echo "  --force              Force rollback without confirmation"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

log "🔄 Starting deployment rollback..."
log "📋 Rollback Configuration:"
log "   Type: $ROLLBACK_TYPE"
log "   Target Revision: ${TARGET_REVISION:-auto}"
log "   Namespace: $NAMESPACE"
log "   Force: $FORCE_ROLLBACK"

# Pre-rollback checks
log "🔍 Running pre-rollback checks..."

# Check kubectl connectivity
if ! kubectl cluster-info &> /dev/null; then
    error "Cannot connect to Kubernetes cluster"
fi

# Check deployment exists
if ! kubectl get deployment "$DEPLOYMENT_NAME" -n "$NAMESPACE" &> /dev/null; then
    error "Deployment '$DEPLOYMENT_NAME' does not exist in namespace '$NAMESPACE'"
fi

# Get current deployment status
CURRENT_REVISION=$(kubectl rollout history deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --revision=0 | tail -1 | awk '{print $1}')
ROLLOUT_STATUS=$(kubectl rollout status deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout=10s 2>/dev/null || echo "FAILED")

log "📊 Current Deployment Status:"
log "   Current Revision: $CURRENT_REVISION"
log "   Rollout Status: $ROLLOUT_STATUS"

# Determine rollback strategy
case $ROLLBACK_TYPE in
    "auto")
        perform_auto_rollback
        ;;
    "manual")
        perform_manual_rollback
        ;;
    "emergency")
        perform_emergency_rollback
        ;;
    *)
        error "Unknown rollback type: $ROLLBACK_TYPE"
        ;;
esac

# Post-rollback verification
log "🔍 Running post-rollback verification..."
verify_rollback

# Update monitoring
log "📊 Updating monitoring alerts..."
update_rollback_monitoring

success "🎉 Deployment rollback completed successfully!"

# Rollback functions
perform_auto_rollback() {
    log "🤖 Performing automatic rollback..."
    
    # Get previous revision
    if [[ -z "$TARGET_REVISION" ]]; then
        TARGET_REVISION=$(kubectl rollout history deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" | tail -2 | head -1 | awk '{print $1}')
    fi
    
    if [[ -z "$TARGET_REVISION" ]]; then
        error "Cannot determine target revision for rollback"
    fi
    
    log "🎯 Rolling back to revision: $TARGET_REVISION"
    
    # Perform rollback
    kubectl rollout undo deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --to-revision="$TARGET_REVISION"
    
    # Wait for rollback to complete
    log "⏳ Waiting for rollback to complete..."
    kubectl rollout status deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout="${ROLLBACK_TIMEOUT}s"
    
    success "Automatic rollback completed"
}

perform_manual_rollback() {
    log "👤 Performing manual rollback..."
    
    # Show rollout history
    log "📋 Deployment History:"
    kubectl rollout history deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE"
    
    # Get target revision if not specified
    if [[ -z "$TARGET_REVISION" ]]; then
        echo -n "Enter target revision to rollback to: "
        read -r TARGET_REVISION
    fi
    
    # Confirm rollback
    if [[ "$FORCE_ROLLBACK" == "false" ]]; then
        echo -n "Are you sure you want to rollback to revision $TARGET_REVISION? (y/N): "
        read -r confirmation
        if [[ "$confirmation" != "y" && "$confirmation" != "Y" ]]; then
            log "Rollback cancelled by user"
            exit 0
        fi
    fi
    
    log "🎯 Rolling back to revision: $TARGET_REVISION"
    
    # Perform rollback
    kubectl rollout undo deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --to-revision="$TARGET_REVISION"
    
    # Wait for rollback to complete
    kubectl rollout status deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout="${ROLLBACK_TIMEOUT}s"
    
    success "Manual rollback completed"
}

perform_emergency_rollback() {
    log "🚨 Performing emergency rollback..."
    
    # Emergency rollback to last known good state
    log "🚨 Emergency: Rolling back to previous revision immediately"
    
    # Scale down current deployment
    log "📉 Scaling down current deployment..."
    kubectl scale deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --replicas=0
    
    # Wait for scale down
    kubectl wait --for=delete pod -l app="$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout=60s
    
    # Rollback to previous revision
    kubectl rollout undo deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE"
    
    # Scale back up
    log "📈 Scaling up rolled back deployment..."
    kubectl scale deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --replicas=3
    
    # Wait for rollback to complete
    kubectl rollout status deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --timeout="${ROLLBACK_TIMEOUT}s"
    
    success "Emergency rollback completed"
}

verify_rollback() {
    log "🔍 Verifying rollback success..."
    
    # Check pod status
    local ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app="$DEPLOYMENT_NAME" --field-selector=status.phase=Running -o name | wc -l)
    local total_pods=$(kubectl get pods -n "$NAMESPACE" -l app="$DEPLOYMENT_NAME" -o name | wc -l)
    
    log "📊 Pod Status: $ready_pods/$total_pods ready"
    
    if [[ $ready_pods -eq 0 ]]; then
        error "No pods are ready after rollback"
    fi
    
    # Check deployment status
    local new_revision=$(kubectl rollout history deployment/"$DEPLOYMENT_NAME" -n "$NAMESPACE" --revision=0 | tail -1 | awk '{print $1}')
    log "📊 New Revision: $new_revision"
    
    # Run health checks
    log "🏥 Running post-rollback health checks..."
    if ! ./scripts/production-health-check.sh; then
        warning "Health checks failed after rollback - manual intervention may be required"
    fi
    
    # Check service endpoints
    log "🔗 Verifying service endpoints..."
    local service_ip=$(kubectl get service "$DEPLOYMENT_NAME" -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
    if [[ -n "$service_ip" ]]; then
        if curl -f "http://$service_ip/health" &> /dev/null; then
            success "Service endpoints are responding"
        else
            warning "Service endpoints are not responding properly"
        fi
    fi
    
    success "Rollback verification completed"
}

update_rollback_monitoring() {
    log "📊 Updating monitoring for rollback event..."
    
    # Send rollback event to monitoring
    local rollback_event=$(cat <<EOF
{
    "event_type": "deployment_rollback",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "deployment": "$DEPLOYMENT_NAME",
    "namespace": "$NAMESPACE",
    "rollback_type": "$ROLLBACK_TYPE",
    "target_revision": "$TARGET_REVISION",
    "status": "completed"
}
EOF
)
    
    # Send to monitoring webhook
    if [[ -n "${MONITORING_WEBHOOK_URL:-}" ]]; then
        curl -X POST \
            -H "Content-Type: application/json" \
            -d "$rollback_event" \
            "$MONITORING_WEBHOOK_URL" || warning "Failed to send rollback event to monitoring"
    fi
    
    # Update Grafana annotations
    if [[ -n "${GRAFANA_URL:-}" ]]; then
        curl -X POST \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer ${GRAFANA_API_KEY:-}" \
            -d '{
                "text": "Deployment Rollback: '"$DEPLOYMENT_NAME"'",
                "tags": ["rollback", "deployment"],
                "time": '$(date +%s000)'
            }' \
            "${GRAFANA_URL}/api/annotations" || warning "Failed to create Grafana annotation"
    fi
    
    success "Monitoring updated for rollback event"
}

# Cleanup function
cleanup() {
    log "🧹 Performing cleanup..."
    
    # Remove any temporary files
    rm -f /tmp/rollback-*.yaml
    
    # Reset any temporary configurations
    kubectl delete configmap rollback-temp -n "$NAMESPACE" --ignore-not-found=true
}

# Set trap for cleanup
trap cleanup EXIT

# Export functions for use in other scripts
export -f log success warning error
export -f perform_auto_rollback perform_manual_rollback perform_emergency_rollback
export -f verify_rollback update_rollback_monitoring cleanup
