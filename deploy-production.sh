#!/bin/bash
# Production deployment script for FHE LLM Proxy

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${ENVIRONMENT:-production}
DOCKER_IMAGE=${DOCKER_IMAGE:-fhe-llm-proxy}
TAG=${TAG:-latest}
REPLICAS=${REPLICAS:-3}
MAX_REPLICAS=${MAX_REPLICAS:-10}

echo -e "${BLUE}🚀 FHE LLM Proxy Production Deployment${NC}"
echo "==========================================="
echo "Environment: $ENVIRONMENT"
echo "Image: $DOCKER_IMAGE:$TAG"
echo "Replicas: $REPLICAS"
echo

# Function to run command with error handling
run_command() {
    echo -e "${YELLOW}▶ $1${NC}"
    if ! eval "$1"; then
        echo -e "${RED}✗ Command failed: $1${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Success${NC}"
    echo
}

# Check prerequisites
echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

if ! command -v docker >/dev/null 2>&1; then
    echo -e "${RED}✗ Docker is not installed${NC}"
    exit 1
fi

if ! command -v kubectl >/dev/null 2>&1; then
    echo -e "${RED}✗ kubectl is not installed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Prerequisites check passed${NC}"
echo

# Generate SSL certificates if they don't exist
if [ ! -f "nginx/ssl/cert.pem" ] || [ ! -f "nginx/ssl/key.pem" ]; then
    echo -e "${BLUE}🔐 Generating SSL certificates...${NC}"
    run_command "./generate-ssl-certs.sh"
fi

# Build Docker image
echo -e "${BLUE}🏗️ Building production Docker image...${NC}"
run_command "docker build -t $DOCKER_IMAGE:$TAG ."

# Run security scan
echo -e "${BLUE}🔍 Running security scan...${NC}"
if command -v trivy >/dev/null 2>&1; then
    run_command "trivy image --exit-code 1 --severity HIGH,CRITICAL $DOCKER_IMAGE:$TAG"
else
    echo -e "${YELLOW}⚠ Trivy not found, skipping security scan${NC}"
fi

# Create Kubernetes namespace
echo -e "${BLUE}🏗️ Setting up Kubernetes resources...${NC}"
run_command "kubectl create namespace fhe-proxy --dry-run=client -o yaml | kubectl apply -f -"

# Apply Kubernetes manifests
run_command "kubectl apply -f k8s/namespace.yaml"
run_command "kubectl apply -f k8s/configmap.yaml"
run_command "kubectl apply -f k8s/secret.yaml"
run_command "kubectl apply -f k8s/deployment.yaml"
run_command "kubectl apply -f k8s/service.yaml"
run_command "kubectl apply -f k8s/ingress.yaml"
run_command "kubectl apply -f k8s/hpa.yaml"

# Apply monitoring
if [ -f "k8s-manifests/monitoring.yaml" ]; then
    run_command "kubectl apply -f k8s-manifests/monitoring.yaml"
fi

# Wait for deployment to be ready
echo -e "${BLUE}⏳ Waiting for deployment to be ready...${NC}"
run_command "kubectl wait --for=condition=available --timeout=300s deployment/fhe-proxy -n fhe-proxy"

# Run health checks
echo -e "${BLUE}🩺 Running health checks...${NC}"
SERVICE_IP=$(kubectl get service fhe-proxy -n fhe-proxy -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "localhost")
PORT=$(kubectl get service fhe-proxy -n fhe-proxy -o jsonpath='{.spec.ports[0].port}' 2>/dev/null || echo "8080")

# Port forward for local testing if LoadBalancer IP is not available
if [ "$SERVICE_IP" = "localhost" ]; then
    echo -e "${YELLOW}⚠ LoadBalancer IP not available, using port-forward${NC}"
    kubectl port-forward service/fhe-proxy 8080:8080 -n fhe-proxy &
    PORT_FORWARD_PID=$!
    sleep 5
fi

# Run health check script
if [ -f "healthcheck.sh" ]; then
    HOST=$SERVICE_IP PORT=$PORT ./healthcheck.sh
else
    echo -e "${YELLOW}⚠ Health check script not found${NC}"
fi

# Cleanup port forward if used
if [ -n "${PORT_FORWARD_PID:-}" ]; then
    kill $PORT_FORWARD_PID 2>/dev/null || true
fi

# Display deployment information
echo -e "${BLUE}📊 Deployment Summary${NC}"
echo "======================"
kubectl get pods -n fhe-proxy -o wide
echo
kubectl get services -n fhe-proxy
echo
kubectl get ingress -n fhe-proxy

echo
echo -e "${GREEN}✅ Production deployment completed successfully!${NC}"
echo
echo "📍 Access points:"
echo "  • Health check: http://$SERVICE_IP:$PORT/health"
echo "  • Metrics: http://$SERVICE_IP:$PORT/metrics"
echo "  • API: http://$SERVICE_IP:$PORT/v1/"
echo
echo "🔧 Management commands:"
echo "  • View logs: kubectl logs -f deployment/fhe-proxy -n fhe-proxy"
echo "  • Scale: kubectl scale deployment fhe-proxy --replicas=$MAX_REPLICAS -n fhe-proxy"
echo "  • Status: kubectl get pods -n fhe-proxy"