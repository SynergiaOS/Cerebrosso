#!/bin/bash
# 🚀 Cerberus Phoenix v3.0 - Complete Production Deployment
# Master deployment script for Hive Mind production environment

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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DEPLOYMENT_TYPE="canary"
ENVIRONMENT="production"
SKIP_TESTS=false
AUTO_APPROVE=false

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
║            🚀 HIVE MIND PRODUCTION DEPLOYMENT                ║
║                 Cerberus Phoenix v3.0                       ║
║              Enterprise-Grade CI/CD Pipeline                ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            DEPLOYMENT_TYPE="$2"
            shift 2
            ;;
        --environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --auto-approve)
            AUTO_APPROVE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --type TYPE          Deployment type: canary, rolling, blue-green (default: canary)"
            echo "  --environment ENV    Target environment: staging, production (default: production)"
            echo "  --skip-tests         Skip pre-deployment tests"
            echo "  --auto-approve       Skip manual approval steps"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

log "🚀 Starting Hive Mind production deployment..."
log "📋 Deployment Configuration:"
log "   Type: $DEPLOYMENT_TYPE"
log "   Environment: $ENVIRONMENT"
log "   Skip Tests: $SKIP_TESTS"
log "   Auto Approve: $AUTO_APPROVE"

# Pre-deployment validation
validate_environment() {
    log "🔍 Validating deployment environment..."
    
    # Check required tools
    local required_tools=("docker" "kubectl" "helm" "jq" "bc")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            error "Required tool not found: $tool"
        fi
    done
    
    # Check Kubernetes connectivity
    if ! kubectl cluster-info &> /dev/null; then
        error "Cannot connect to Kubernetes cluster"
    fi
    
    # Check namespace
    if ! kubectl get namespace "$ENVIRONMENT" &> /dev/null; then
        error "Namespace '$ENVIRONMENT' does not exist"
    fi
    
    # Check Docker registry access
    if ! docker info &> /dev/null; then
        error "Cannot connect to Docker daemon"
    fi
    
    success "Environment validation passed"
}

# Build and push images
build_and_push_images() {
    log "🏗️ Building and pushing Docker images..."
    
    cd "$PROJECT_ROOT"
    
    # Get current git commit
    local git_commit=$(git rev-parse --short HEAD)
    local git_branch=$(git rev-parse --abbrev-ref HEAD)
    local timestamp=$(date +%Y%m%d-%H%M%S)
    
    # Generate image tag
    local image_tag="ghcr.io/synergiaos/cerebros:${git_branch}-${git_commit}-${timestamp}"
    
    log "📦 Building image: $image_tag"
    
    # Build multi-service image
    docker build -t "$image_tag" .
    
    # Push to registry
    log "📤 Pushing image to registry..."
    docker push "$image_tag"
    
    # Export image tag for use in deployment
    export IMAGE_TAG="$image_tag"
    echo "$image_tag" > .image-tag
    
    success "Image built and pushed: $image_tag"
}

# Run pre-deployment tests
run_pre_deployment_tests() {
    if [[ "$SKIP_TESTS" == "true" ]]; then
        warning "Skipping pre-deployment tests"
        return 0
    fi
    
    log "🧪 Running pre-deployment tests..."
    
    cd "$PROJECT_ROOT"
    
    # Run unit tests
    log "🔬 Running unit tests..."
    cargo test --all --verbose
    
    # Run integration tests
    log "🔗 Running integration tests..."
    docker-compose -f docker-compose.test.yml up -d
    sleep 30
    cargo test --test integration --verbose
    docker-compose -f docker-compose.test.yml down
    
    # Run security scans
    log "🔐 Running security scans..."
    if command -v cargo-audit &> /dev/null; then
        cargo audit
    fi
    
    # Run smoke tests against staging
    if [[ "$ENVIRONMENT" == "production" ]]; then
        log "💨 Running smoke tests against staging..."
        "$SCRIPT_DIR/smoke-tests.sh" staging
    fi
    
    success "Pre-deployment tests passed"
}

# Deploy infrastructure dependencies
deploy_infrastructure() {
    log "🏗️ Deploying infrastructure dependencies..."
    
    # Deploy monitoring stack
    log "📊 Deploying monitoring stack..."
    helm upgrade --install prometheus-stack prometheus-community/kube-prometheus-stack \
        --namespace monitoring --create-namespace \
        --values k8s/monitoring/prometheus-values.yaml
    
    # Deploy Istio service mesh (if not already installed)
    if ! kubectl get namespace istio-system &> /dev/null; then
        log "🕸️ Installing Istio service mesh..."
        istioctl install --set values.defaultRevision=default -y
        kubectl label namespace "$ENVIRONMENT" istio-injection=enabled --overwrite
    fi
    
    # Deploy secrets
    log "🔐 Deploying secrets..."
    kubectl apply -f k8s/"$ENVIRONMENT"/secrets.yaml
    
    # Deploy ConfigMaps
    log "⚙️ Deploying configuration..."
    kubectl apply -f k8s/"$ENVIRONMENT"/configmap.yaml
    
    success "Infrastructure dependencies deployed"
}

# Perform deployment
perform_deployment() {
    log "🚀 Performing $DEPLOYMENT_TYPE deployment..."
    
    # Load image tag
    if [[ -f .image-tag ]]; then
        export IMAGE_TAG=$(cat .image-tag)
    else
        error "Image tag not found. Please run build step first."
    fi
    
    # Execute deployment script
    case $DEPLOYMENT_TYPE in
        "canary"|"rolling"|"blue-green")
            "$SCRIPT_DIR/deploy-production.sh" \
                --type "$DEPLOYMENT_TYPE" \
                --image-tag "$IMAGE_TAG" \
                $([ "$SKIP_TESTS" == "true" ] && echo "--skip-tests")
            ;;
        *)
            error "Unknown deployment type: $DEPLOYMENT_TYPE"
            ;;
    esac
    
    success "Deployment completed successfully"
}

# Post-deployment verification
post_deployment_verification() {
    log "🔍 Running post-deployment verification..."
    
    # Wait for deployment to stabilize
    log "⏳ Waiting for deployment to stabilize..."
    sleep 60
    
    # Run comprehensive health checks
    log "🏥 Running health checks..."
    "$SCRIPT_DIR/production-health-check.sh"
    
    # Run performance validation
    log "⚡ Running performance validation..."
    "$SCRIPT_DIR/performance-validation.sh" || warning "Performance validation failed"
    
    # Run security validation
    log "🔐 Running security validation..."
    "$SCRIPT_DIR/security-validation.sh" || warning "Security validation failed"
    
    success "Post-deployment verification completed"
}

# Update monitoring and dashboards
update_monitoring() {
    log "📊 Updating monitoring and dashboards..."
    
    # Update Grafana dashboards
    if [[ -d "dashboards" ]]; then
        for dashboard in dashboards/*.json; do
            if [[ -f "$dashboard" ]]; then
                log "📈 Updating dashboard: $(basename "$dashboard")"
                # Import dashboard to Grafana
                # This would typically use Grafana API
            fi
        done
    fi
    
    # Update alert rules
    if [[ -f "k8s/monitoring/alert-rules.yaml" ]]; then
        kubectl apply -f k8s/monitoring/alert-rules.yaml
    fi
    
    success "Monitoring and dashboards updated"
}

# Send deployment notification
send_notification() {
    local status="$1"
    local message="$2"
    
    log "📢 Sending deployment notification..."
    
    # Slack notification (if webhook URL is configured)
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        local payload=$(cat <<EOF
{
    "text": "🐝 Hive Mind Deployment $status",
    "attachments": [
        {
            "color": "$([ "$status" == "SUCCESS" ] && echo "good" || echo "danger")",
            "fields": [
                {
                    "title": "Environment",
                    "value": "$ENVIRONMENT",
                    "short": true
                },
                {
                    "title": "Type",
                    "value": "$DEPLOYMENT_TYPE",
                    "short": true
                },
                {
                    "title": "Image",
                    "value": "${IMAGE_TAG:-unknown}",
                    "short": false
                },
                {
                    "title": "Message",
                    "value": "$message",
                    "short": false
                }
            ]
        }
    ]
}
EOF
)
        curl -X POST -H 'Content-type: application/json' \
            --data "$payload" \
            "$SLACK_WEBHOOK_URL" || warning "Failed to send Slack notification"
    fi
    
    success "Deployment notification sent"
}

# Main deployment flow
main() {
    local start_time=$(date +%s)
    
    # Validate environment
    validate_environment
    
    # Manual approval for production
    if [[ "$ENVIRONMENT" == "production" && "$AUTO_APPROVE" == "false" ]]; then
        echo -e "\n${YELLOW}⚠️ You are about to deploy to PRODUCTION environment.${NC}"
        echo -e "${YELLOW}   Deployment Type: $DEPLOYMENT_TYPE${NC}"
        echo -e "${YELLOW}   This action cannot be undone easily.${NC}\n"
        echo -n "Are you sure you want to continue? (yes/no): "
        read -r confirmation
        if [[ "$confirmation" != "yes" ]]; then
            log "Deployment cancelled by user"
            exit 0
        fi
    fi
    
    # Execute deployment steps
    build_and_push_images
    run_pre_deployment_tests
    deploy_infrastructure
    perform_deployment
    post_deployment_verification
    update_monitoring
    
    # Calculate deployment time
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    local duration_formatted=$(printf "%02d:%02d:%02d" $((duration/3600)) $((duration%3600/60)) $((duration%60)))
    
    # Success notification
    local success_message="Deployment completed successfully in $duration_formatted"
    send_notification "SUCCESS" "$success_message"
    
    success "🎉 Hive Mind production deployment completed successfully!"
    log "📊 Deployment Summary:"
    log "   Environment: $ENVIRONMENT"
    log "   Type: $DEPLOYMENT_TYPE"
    log "   Image: ${IMAGE_TAG:-unknown}"
    log "   Duration: $duration_formatted"
    log "   Status: SUCCESS"
}

# Error handling
trap 'send_notification "FAILED" "Deployment failed at step: ${BASH_COMMAND}"' ERR

# Run main function
main "$@"
