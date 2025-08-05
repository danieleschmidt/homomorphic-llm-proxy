# 🚀 Deployment Guide: Homomorphic LLM Proxy

## Quick Start

### Prerequisites
- Docker Engine 20.10+
- Docker Compose v2.0+
- NVIDIA GPU with CUDA support (optional, for GPU acceleration)
- 8GB+ RAM recommended
- SSL certificates for production deployment

### 1. Production Deployment

```bash
# Clone the repository
git clone https://github.com/terragonlabs/homomorphic-llm-proxy
cd homomorphic-llm-proxy

# Set environment variables
export OPENAI_API_KEY="your-openai-api-key"
export ANTHROPIC_API_KEY="your-anthropic-api-key"
export GRAFANA_PASSWORD="secure-password"

# Deploy with Docker Compose
docker-compose -f docker-compose.prod.yml up -d

# Check service status
docker-compose -f docker-compose.prod.yml ps
```

### 2. Kubernetes Deployment

```bash
# Apply Kubernetes manifests
kubectl apply -f k8s-manifests/

# Check deployment status
kubectl get pods -l app=fhe-proxy

# Port forward for testing
kubectl port-forward svc/fhe-proxy-service 8080:8080
```

### 3. Configuration

Create `config.toml`:

```toml
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

[gpu]
enabled = true
device_id = 0
batch_size = 32
kernel_optimization = "aggressive"

[privacy]
epsilon_per_query = 0.1
delta = 1e-5
max_queries_per_user = 1000
track_privacy_budget = true

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

## Architecture Overview

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Client    │───▶│    Nginx     │───▶│   FHE Proxy    │
│ Application │    │ Load Balancer│    │   (Rust)       │
└─────────────┘    └──────────────┘    └─────────────────┘
                                                │
                   ┌──────────────┐            │
                   │   Redis      │◀───────────┘
                   │   Cache      │
                   └──────────────┘
                                     
┌─────────────────────────────────────────────────────────┐
│                Monitoring Stack                         │
├─────────────┬─────────────┬─────────────┬──────────────┤
│ Prometheus  │   Grafana   │    Logs     │   Metrics    │
│  (Metrics)  │    (UI)     │  (JSON)     │ (OpenTel)    │
└─────────────┴─────────────┴─────────────┴──────────────┘
```

## Scaling Configuration

### Horizontal Pod Autoscaler (HPA)

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
  minReplicas: 2
  maxReplicas: 10
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

### Load Balancing

```nginx
upstream fhe_proxy_backend {
    least_conn;
    server fhe-proxy-1:8080 max_fails=3 fail_timeout=30s;
    server fhe-proxy-2:8080 max_fails=3 fail_timeout=30s;
    server fhe-proxy-3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://fhe_proxy_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
}
```

## Security Configuration

### 1. TLS Setup

```bash
# Generate self-signed certificates (development only)
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/ssl/private.key \
  -out nginx/ssl/certificate.crt

# For production, use Let's Encrypt
certbot certonly --standalone -d your-domain.com
```

### 2. Environment Security

```bash
# Use Docker secrets for sensitive data
echo "your-openai-api-key" | docker secret create openai_api_key -
echo "your-anthropic-key" | docker secret create anthropic_api_key -

# Update docker-compose.prod.yml to use secrets
secrets:
  - openai_api_key
  - anthropic_api_key
```

### 3. Network Security

```yaml
# Kubernetes NetworkPolicy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: fhe-proxy-netpol
spec:
  podSelector:
    matchLabels:
      app: fhe-proxy
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: nginx
    ports:
    - protocol: TCP
      port: 8080
```

## Monitoring & Observability

### 1. Prometheus Metrics

Access metrics at: `http://localhost:9090`

Key metrics to monitor:
- `fhe_requests_total` - Total FHE requests
- `fhe_encryption_duration_seconds` - Encryption latency
- `fhe_decryption_duration_seconds` - Decryption latency
- `fhe_active_sessions` - Active client sessions
- `fhe_privacy_budget_remaining` - Privacy budget usage

### 2. Grafana Dashboard

Access dashboard at: `http://localhost:3000`

Pre-configured dashboards:
- FHE Operations Overview
- Performance Metrics
- Security Events
- Privacy Budget Tracking

### 3. Structured Logging

Logs are output in JSON format for easy parsing:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "target": "fhe_operations",
  "message": "Encryption completed",
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "duration_ms": 45,
  "operation": "encrypt"
}
```

## Performance Tuning

### 1. FHE Parameters

```toml
# For higher security (slower)
[encryption]
poly_modulus_degree = 32768
security_level = 192

# For better performance (lower security)
[encryption]
poly_modulus_degree = 8192
security_level = 128
```

### 2. Connection Pool Sizing

```toml
[server]
workers = 8  # 2x CPU cores
max_connections = 2000  # Adjust based on memory

[gpu]
batch_size = 64  # Increase for better GPU utilization
```

### 3. Caching Strategy

```toml
[cache]
enabled = true
max_size = 10000  # Number of cached ciphertexts
ttl_seconds = 3600  # Cache TTL
```

## Health Checks

### Application Health

```bash
# Basic health check
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/health/ready

# Liveness probe
curl http://localhost:8080/health/live
```

### Component Health

```bash
# Check FHE engine status
curl http://localhost:8080/v1/params

# Check metrics endpoint
curl http://localhost:9090/metrics

# Check Redis connection
redis-cli -h localhost ping
```

## Backup & Recovery

### 1. Key Backup

```bash
# Backup FHE keys
docker run --rm -v fhe_keys:/data -v $(pwd):/backup \
  alpine tar czf /backup/fhe-keys-backup-$(date +%Y%m%d).tar.gz -C /data .
```

### 2. Configuration Backup

```bash
# Backup configuration
cp config.toml backups/config-$(date +%Y%m%d).toml
```

### 3. Disaster Recovery

```bash
# Restore from backup
docker run --rm -v fhe_keys:/data -v $(pwd):/backup \
  alpine tar xzf /backup/fhe-keys-backup-20240115.tar.gz -C /data
```

## Troubleshooting

### Common Issues

1. **GPU Not Detected**
   ```bash
   # Check GPU availability
   nvidia-smi
   
   # Verify Docker GPU support
   docker run --gpus all nvidia/cuda:11.0-base nvidia-smi
   ```

2. **High Memory Usage**
   ```bash
   # Monitor memory usage
   docker stats fhe-proxy-prod
   
   # Reduce batch size or connection pool
   ```

3. **Slow Performance**
   ```bash
   # Check FHE parameters
   curl http://localhost:8080/v1/params
   
   # Monitor encryption times
   curl http://localhost:8080/metrics/detailed
   ```

### Debugging

```bash
# Enable debug logging
export RUST_LOG=debug

# Access container logs
docker logs -f fhe-proxy-prod

# Execute into container
docker exec -it fhe-proxy-prod bash
```

## Production Checklist

- [ ] SSL/TLS certificates configured
- [ ] API keys secured with secrets management
- [ ] Network policies applied
- [ ] Monitoring and alerting configured
- [ ] Backup strategy implemented
- [ ] Load balancing configured
- [ ] Auto-scaling policies set
- [ ] Security headers enabled
- [ ] Rate limiting configured
- [ ] Privacy budget monitoring active
- [ ] Disaster recovery plan documented
- [ ] Performance benchmarks established

## Support

For deployment issues:
- 📧 Email: devops@terragonlabs.ai
- 📖 Documentation: [https://docs.terragonlabs.ai/fhe-proxy](https://docs.terragonlabs.ai/fhe-proxy)
- 🎫 Support Portal: [https://support.terragonlabs.ai](https://support.terragonlabs.ai)