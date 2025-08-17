#!/bin/bash
# Global deployment script for FHE LLM Proxy
# Deploys to multiple regions with compliance configurations

set -euo pipefail

# Configuration
IMAGE_NAME="fhe-llm-proxy"
IMAGE_TAG="${IMAGE_TAG:-latest}"
REGISTRY="${REGISTRY:-ghcr.io/your-org}"

# Define regions and their compliance requirements
declare -A REGIONS=(
    ["us-east-1"]="ccpa,gdpr"
    ["us-west-2"]="ccpa,gdpr" 
    ["eu-west-1"]="gdpr"
    ["eu-central-1"]="gdpr"
    ["ap-northeast-1"]="gdpr"
    ["ap-southeast-1"]="gdpr"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    command -v docker >/dev/null 2>&1 || error "Docker is required but not installed"
    command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
    command -v helm >/dev/null 2>&1 || error "Helm is required but not installed"
    
    log "Prerequisites check passed"
}

# Build multi-arch image
build_image() {
    log "Building multi-architecture image..."
    
    docker buildx create --use --name fhe-builder 2>/dev/null || true
    
    docker buildx build \
        --platform linux/amd64,linux/arm64 \
        --file Dockerfile.multi-region \
        --tag "${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}" \
        --push \
        .
    
    log "Image built and pushed: ${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}"
}

# Deploy to specific region
deploy_region() {
    local region=$1
    local compliance=$2
    
    log "Deploying to region: ${region} with compliance: ${compliance}"
    
    # Set region-specific compliance flags
    local gdpr_enabled=false
    local ccpa_enabled=false
    local hipaa_enabled=false
    
    if [[ $compliance == *"gdpr"* ]]; then
        gdpr_enabled=true
    fi
    if [[ $compliance == *"ccpa"* ]]; then
        ccpa_enabled=true
    fi
    if [[ $compliance == *"hipaa"* ]]; then
        hipaa_enabled=true
    fi
    
    # Create namespace if it doesn't exist
    kubectl create namespace "fhe-proxy-${region}" --dry-run=client -o yaml | kubectl apply -f -
    
    # Apply region-specific configuration
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: fhe-proxy-config
  namespace: fhe-proxy-${region}
data:
  config.toml: |
    [server]
    host = "0.0.0.0"
    port = 8080
    workers = 4
    max_connections = 1000
    request_timeout_seconds = 300

    [encryption]
    poly_modulus_degree = 16384
    coeff_modulus_bits = [60, 40, 40, 60]
    scale_bits = 40
    security_level = 128

    [privacy]
    epsilon_per_query = 0.1
    delta = 1e-5
    max_queries_per_user = 1000
    track_privacy_budget = true
    noise_multiplier = 1.1

    [monitoring]
    metrics_enabled = true
    metrics_port = 9090
    trace_sampling_rate = 0.1
    log_level = "info"

    [scaling]
    auto_scaling_enabled = true
    min_instances = 2
    max_instances = 10
    target_cpu_utilization = 70.0
    scale_up_threshold = 80.0
    scale_down_threshold = 40.0
    cooldown_period_seconds = 300
    connection_pool_size = 4
    max_concurrent_requests = 1000

    [performance]
    cache_enabled = true
    cache_size_mb = 512
    cache_ttl_seconds = 3600
    batch_processing_enabled = true
    batch_size = 32
    batch_timeout_ms = 100
    compression_enabled = true
    prefetch_enabled = true
    async_processing = true

    [compliance]
    gdpr_enabled = ${gdpr_enabled}
    ccpa_enabled = ${ccpa_enabled}
    hipaa_enabled = ${hipaa_enabled}
    audit_logs_enabled = true
    data_encryption_at_rest = true
    pii_detection_enabled = true
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fhe-proxy
  namespace: fhe-proxy-${region}
  labels:
    app: fhe-proxy
    region: ${region}
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fhe-proxy
  template:
    metadata:
      labels:
        app: fhe-proxy
        region: ${region}
    spec:
      containers:
      - name: fhe-proxy
        image: ${REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: FHE_REGION
          value: "${region}"
        - name: FHE_COMPLIANCE_MODE
          value: "${compliance}"
        - name: GDPR_ENABLED
          value: "${gdpr_enabled}"
        - name: CCPA_ENABLED
          value: "${ccpa_enabled}"
        - name: HIPAA_ENABLED
          value: "${hipaa_enabled}"
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: logs
          mountPath: /app/logs
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: config
        configMap:
          name: fhe-proxy-config
      - name: logs
        emptyDir: {}
---
apiVersion: v1
kind: Service
metadata:
  name: fhe-proxy-service
  namespace: fhe-proxy-${region}
  labels:
    app: fhe-proxy
spec:
  selector:
    app: fhe-proxy
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: LoadBalancer
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: fhe-proxy-hpa
  namespace: fhe-proxy-${region}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: fhe-proxy
  minReplicas: 2
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
EOF

    # Wait for deployment to be ready
    kubectl wait --for=condition=available --timeout=300s deployment/fhe-proxy -n "fhe-proxy-${region}"
    
    log "Deployment to ${region} completed successfully"
}

# Deploy monitoring stack
deploy_monitoring() {
    log "Deploying global monitoring stack..."
    
    # Add Prometheus Helm repo
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    # Install Prometheus Operator
    helm upgrade --install prometheus-operator prometheus-community/kube-prometheus-stack \
        --namespace monitoring \
        --create-namespace \
        --set grafana.enabled=true \
        --set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false \
        --set prometheus.prometheusSpec.podMonitorSelectorNilUsesHelmValues=false
    
    # Configure ServiceMonitor for FHE Proxy
    cat <<EOF | kubectl apply -f -
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: fhe-proxy-metrics
  namespace: monitoring
  labels:
    app: fhe-proxy
spec:
  selector:
    matchLabels:
      app: fhe-proxy
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
  namespaceSelector:
    matchNames:
$(for region in "${!REGIONS[@]}"; do echo "    - fhe-proxy-${region}"; done)
EOF

    log "Monitoring stack deployed successfully"
}

# Main deployment function
main() {
    log "Starting global deployment of FHE LLM Proxy"
    
    check_prerequisites
    build_image
    
    # Deploy to all regions
    for region in "${!REGIONS[@]}"; do
        compliance="${REGIONS[$region]}"
        deploy_region "$region" "$compliance"
    done
    
    deploy_monitoring
    
    log "Global deployment completed successfully!"
    
    info "Deployment Summary:"
    for region in "${!REGIONS[@]}"; do
        compliance="${REGIONS[$region]}"
        info "  Region: ${region} - Compliance: ${compliance}"
    done
    
    info "Monitor deployments with:"
    info "  kubectl get pods -A -l app=fhe-proxy"
    info "  kubectl get services -A -l app=fhe-proxy"
    
    info "Access Grafana dashboard:"
    info "  kubectl port-forward -n monitoring svc/prometheus-operator-grafana 3000:80"
}

# Run deployment
main "$@"