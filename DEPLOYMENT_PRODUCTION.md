# 🚀 Production Deployment Guide

## Prerequisites

### Hardware Requirements
- **CPU**: 8+ cores (Intel Xeon or AMD EPYC recommended)
- **Memory**: 32GB+ RAM 
- **GPU**: NVIDIA GPU with CUDA compute capability ≥7.0 (V100, A100, RTX 30/40 series)
- **Storage**: 1TB+ SSD for ciphertext caching
- **Network**: 10Gbps+ for high-throughput scenarios

### Software Requirements
- Docker 24.0+ with NVIDIA Container Toolkit
- Kubernetes 1.25+ (for orchestrated deployment)
- NVIDIA GPU Drivers 470+
- OpenSSL 3.0+

## Quick Production Deployment

### 1. Docker Compose (Single Node)

```bash
# Clone repository
git clone https://github.com/terragonlabs/homomorphic-llm-proxy
cd homomorphic-llm-proxy

# Build production image
docker build -t fhe-proxy:latest .

# Deploy with compose
docker-compose -f docker-compose.prod.yml up -d
```

### 2. Kubernetes (Recommended)

```bash
# Apply configurations
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/hpa.yaml
```

## Configuration

### Environment Variables

```bash
# Core Settings
FHE_HOST=0.0.0.0
FHE_PORT=8080
RUST_LOG=info

# GPU Configuration
FHE_GPU_ENABLED=true
FHE_GPU_DEVICE_ID=0

# Encryption Parameters
FHE_POLY_MODULUS_DEGREE=16384
FHE_SECURITY_LEVEL=128

# LLM Provider Keys
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key

# Monitoring
FHE_METRICS_ENABLED=true
```

### Production config.toml

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 16
max_connections = 10000
request_timeout_seconds = 300

[encryption]
poly_modulus_degree = 16384
coeff_modulus_bits = [60, 40, 40, 60]
scale_bits = 40
security_level = 128

[gpu]
enabled = true
device_id = 0
batch_size = 64
kernel_optimization = "aggressive"
memory_limit_gb = 16

[privacy]
epsilon_per_query = 0.1
delta = 1e-5
max_queries_per_user = 10000
track_privacy_budget = true
noise_multiplier = 1.1

[llm]
provider = "openai"
endpoint = "https://api.openai.com/v1"
timeout_seconds = 300
max_retries = 3

[monitoring]
metrics_enabled = true
metrics_port = 9090
trace_sampling_rate = 0.1
log_level = "info"
```

## Security Hardening

### 1. Network Security

```bash
# Use TLS/SSL termination at load balancer
# Restrict network access with firewall rules
sudo ufw allow 8080/tcp  # FHE Proxy
sudo ufw allow 9090/tcp  # Metrics (internal only)
sudo ufw deny 22/tcp     # Disable SSH in production
```

### 2. Container Security

```dockerfile
# Non-root user (already implemented)
USER fheproxy

# Read-only filesystem
RUN apt-get install -y --no-install-recommends

# Security scanning
RUN apt-get update && apt-get upgrade -y
```

### 3. API Security

```yaml
# Rate limiting
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: fhe-proxy-circuit-breaker
spec:
  host: fhe-proxy
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 1000
      http:
        http1MaxPendingRequests: 100
        maxRequestsPerConnection: 10
```

## Monitoring & Observability

### 1. Prometheus Configuration

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'fhe-proxy'
      static_configs:
      - targets: ['fhe-proxy:9090']
      metrics_path: /metrics
```

### 2. Key Metrics to Monitor

- **Performance**: Request latency, throughput, GPU utilization
- **Security**: Failed authentication attempts, rate limit violations
- **Privacy**: Privacy budget consumption, differential privacy violations
- **Infrastructure**: CPU/Memory usage, disk I/O, network throughput

### 3. Alerting Rules

```yaml
groups:
- name: fhe-proxy-alerts
  rules:
  - alert: HighErrorRate
    expr: rate(fhe_proxy_errors_total[5m]) > 0.1
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"
      
  - alert: GPUUtilizationHigh
    expr: gpu_utilization > 90
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "GPU utilization consistently high"
```

## Auto-Scaling

### 1. Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: fhe-proxy-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: fhe-proxy
  minReplicas: 3
  maxReplicas: 50
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
```

### 2. Cluster Autoscaler

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cluster-autoscaler-status
data:
  nodes.max: "100"
  nodes.min: "3"
  scale-down-delay-after-add: "10m"
  scale-down-unneeded-time: "10m"
```

## Backup & Disaster Recovery

### 1. State Backup

```bash
# Backup encryption keys and configuration
kubectl create backup fhe-proxy-backup \
  --include-namespaces=fhe-proxy \
  --storage-location=s3-backup

# Schedule daily backups
kubectl create schedule fhe-daily-backup \
  --schedule="0 2 * * *" \
  --backup-template=fhe-proxy-backup
```

### 2. Multi-Region Deployment

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: fhe-proxy-dr
spec:
  source:
    repoURL: https://github.com/terragonlabs/fhe-proxy
    targetRevision: main
    path: k8s/disaster-recovery
  destination:
    server: https://dr-cluster.example.com
    namespace: fhe-proxy
```

## Performance Optimization

### 1. GPU Memory Management

```toml
[gpu]
enabled = true
batch_size = 128        # Increase for better GPU utilization
memory_limit_gb = 24    # Set based on GPU memory
kernel_optimization = "aggressive"
```

### 2. Connection Pool Tuning

```toml
[scaling]
pool_size = 8           # Number of FHE engines
max_concurrent = 1000   # Max concurrent operations
cache_size = 10000      # Ciphertext cache entries
cache_ttl_seconds = 3600
```

### 3. Load Balancing

```yaml
apiVersion: v1
kind: Service
metadata:
  name: fhe-proxy-lb
spec:
  type: LoadBalancer
  sessionAffinity: ClientIP  # Sticky sessions for caching
  ports:
  - port: 80
    targetPort: 8080
  selector:
    app: fhe-proxy
```

## Compliance & Auditing

### 1. GDPR Compliance

```toml
[privacy]
# Ensure epsilon budget aligns with privacy requirements
epsilon_per_query = 0.05    # Conservative setting
data_retention_days = 30    # Automatic data purging
audit_log_enabled = true
```

### 2. SOC 2 Controls

- **Security**: TLS encryption, API authentication
- **Availability**: Health checks, auto-scaling, backup/restore
- **Processing Integrity**: Input validation, error handling
- **Confidentiality**: FHE encryption, access controls
- **Privacy**: Differential privacy, data minimization

### 3. Audit Logging

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: audit-policy
data:
  audit-policy.yaml: |
    rules:
    - level: Metadata
      resources:
      - group: ""
        resources: ["secrets", "configmaps"]
    - level: RequestResponse
      namespaces: ["fhe-proxy"]
```

## Troubleshooting

### Common Issues

1. **GPU Not Detected**
   ```bash
   # Check NVIDIA runtime
   docker run --rm --gpus all nvidia/cuda:12.0-base-ubuntu20.04 nvidia-smi
   ```

2. **High Memory Usage**
   ```bash
   # Monitor ciphertext cache
   curl http://localhost:8080/metrics/detailed | grep cache
   ```

3. **Performance Degradation**
   ```bash
   # Check FHE operation timings
   kubectl logs -f deployment/fhe-proxy | grep "operation="
   ```

### Health Check Endpoints

- `/health` - Basic health status
- `/health/live` - Kubernetes liveness probe
- `/health/ready` - Kubernetes readiness probe
- `/metrics` - Prometheus metrics
- `/metrics/detailed` - Detailed system metrics

## Support & Maintenance

### Regular Maintenance Tasks

1. **Weekly**: Review monitoring dashboards and alerts
2. **Monthly**: Update dependencies and security patches
3. **Quarterly**: Performance review and capacity planning
4. **Annually**: Security audit and compliance review

### Getting Help

- 📧 Production Support: support@terragonlabs.ai
- 📚 Documentation: https://docs.terragonlabs.ai/fhe-proxy
- 🔧 GitHub Issues: https://github.com/terragonlabs/fhe-proxy/issues
- 💬 Discord: [Join Community](https://discord.gg/terragonlabs)

## Appendix

### A. Resource Requirements by Scale

| Scale | CPU Cores | Memory | GPU | Storage | Network |
|-------|-----------|--------|-----|---------|---------|
| Small | 8 | 32GB | 1x RTX 4090 | 500GB | 1Gbps |
| Medium | 32 | 128GB | 2x A100 | 2TB | 10Gbps |
| Large | 128 | 512GB | 8x A100 | 10TB | 25Gbps |
| Enterprise | 512+ | 2TB+ | 32+ A100 | 50TB+ | 100Gbps |

### B. Cost Estimation

- **AWS**: $5-15/hour per GPU instance (p4d.24xlarge)
- **GCP**: $3-12/hour per GPU instance (a2-highgpu)
- **Azure**: $4-14/hour per GPU instance (ND40rs_v2)
- **On-Premise**: $50K-200K initial investment per node

*Note: Costs vary by region and commitment level*