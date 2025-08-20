#!/bin/bash

# Production deployment script for FHE LLM Proxy
# Terragon Labs - SDLC v4.0 Autonomous Deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="fhe-proxy"
CONTAINER_REGISTRY="ghcr.io/terragon-labs"
NAMESPACE="fhe-system"
ENVIRONMENT="${ENVIRONMENT:-production}"
VERSION="${VERSION:-$(git rev-parse --short HEAD)}"

log() {
    echo -e "${GREEN}[DEPLOY]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    command -v docker >/dev/null 2>&1 || error "Docker is required but not installed"
    command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
    command -v cargo >/dev/null 2>&1 || error "Cargo is required but not installed"
    
    # Check if we can access the container registry
    docker info >/dev/null 2>&1 || error "Docker daemon is not running"
    
    log "Prerequisites check passed ✓"
}

# Build and test
build_and_test() {
    log "Building and testing application..."
    
    # Run tests
    cargo test --features tokio-runtime || error "Tests failed"
    
    # Build release
    cargo build --release --features tokio-runtime || error "Build failed"
    
    log "Build and test completed ✓"
}

# Build Docker image
build_docker_image() {
    log "Building Docker image..."
    
    IMAGE_TAG="${CONTAINER_REGISTRY}/${PROJECT_NAME}:${VERSION}"
    LATEST_TAG="${CONTAINER_REGISTRY}/${PROJECT_NAME}:latest"
    
    docker build \
        -f docker/Dockerfile.production \
        -t "${IMAGE_TAG}" \
        -t "${LATEST_TAG}" \
        . || error "Docker build failed"
    
    log "Docker image built: ${IMAGE_TAG} ✓"
}

# Push Docker image
push_docker_image() {
    log "Pushing Docker image to registry..."
    
    IMAGE_TAG="${CONTAINER_REGISTRY}/${PROJECT_NAME}:${VERSION}"
    LATEST_TAG="${CONTAINER_REGISTRY}/${PROJECT_NAME}:latest"
    
    docker push "${IMAGE_TAG}" || error "Failed to push versioned image"
    docker push "${LATEST_TAG}" || error "Failed to push latest image"
    
    log "Docker image pushed ✓"
}

# Create Kubernetes namespace
create_namespace() {
    log "Creating Kubernetes namespace..."
    
    kubectl create namespace "${NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -
    
    log "Namespace ${NAMESPACE} ready ✓"
}

# Deploy to Kubernetes
deploy_kubernetes() {
    log "Deploying to Kubernetes..."
    
    # Update image tag in deployment
    sed -i.bak "s|ghcr.io/terragon-labs/fhe-proxy:latest|${CONTAINER_REGISTRY}/${PROJECT_NAME}:${VERSION}|g" k8s/deployment.yaml
    
    # Apply Kubernetes manifests
    kubectl apply -f k8s/ -n "${NAMESPACE}" || error "Kubernetes deployment failed"
    
    # Restore original deployment file
    mv k8s/deployment.yaml.bak k8s/deployment.yaml
    
    log "Kubernetes deployment completed ✓"
}

# Wait for deployment
wait_for_deployment() {
    log "Waiting for deployment to be ready..."
    
    kubectl rollout status deployment/"${PROJECT_NAME}" -n "${NAMESPACE}" --timeout=600s || error "Deployment failed to become ready"
    
    log "Deployment is ready ✓"
}

# Health check
health_check() {
    log "Performing health check..."
    
    # Get service URL
    SERVICE_URL=$(kubectl get service "${PROJECT_NAME}" -n "${NAMESPACE}" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
    
    if [ -z "${SERVICE_URL}" ]; then
        warn "LoadBalancer URL not available yet, using port-forward for health check"
        kubectl port-forward service/"${PROJECT_NAME}" 8080:80 -n "${NAMESPACE}" &
        PF_PID=$!
        sleep 5
        SERVICE_URL="localhost:8080"
    fi
    
    # Perform health check
    for i in {1..30}; do
        if curl -f "http://${SERVICE_URL}/health" >/dev/null 2>&1; then
            log "Health check passed ✓"
            [ -n "${PF_PID:-}" ] && kill "${PF_PID}" >/dev/null 2>&1 || true
            return 0
        fi
        sleep 2
    done
    
    [ -n "${PF_PID:-}" ] && kill "${PF_PID}" >/dev/null 2>&1 || true
    error "Health check failed after 60 seconds"
}

# Show deployment status
show_status() {
    log "Deployment Status:"
    echo
    kubectl get pods -n "${NAMESPACE}" -l app="${PROJECT_NAME}"
    echo
    kubectl get service -n "${NAMESPACE}" "${PROJECT_NAME}"
    echo
    log "Deployment completed successfully! 🚀"
}

# Main deployment pipeline
main() {
    log "Starting deployment pipeline for ${PROJECT_NAME} v${VERSION}"
    
    check_prerequisites
    build_and_test
    build_docker_image
    push_docker_image
    create_namespace
    deploy_kubernetes
    wait_for_deployment
    health_check
    show_status
}

# Handle script arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "build")
        check_prerequisites
        build_and_test
        build_docker_image
        ;;
    "push")
        push_docker_image
        ;;
    "k8s")
        create_namespace
        deploy_kubernetes
        wait_for_deployment
        ;;
    "health")
        health_check
        ;;
    "status")
        show_status
        ;;
    *)
        echo "Usage: $0 [deploy|build|push|k8s|health|status]"
        echo "  deploy - Full deployment pipeline (default)"
        echo "  build  - Build and test only"
        echo "  push   - Push Docker image only"
        echo "  k8s    - Deploy to Kubernetes only"
        echo "  health - Health check only"
        echo "  status - Show deployment status"
        exit 1
        ;;
esac