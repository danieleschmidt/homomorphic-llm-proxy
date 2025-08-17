# FHE LLM Proxy - Production Deployment Guide

This guide covers deploying the FHE LLM Proxy to production environments with full security, monitoring, and compliance features.

## 🚀 Quick Start

```bash
# 1. Clone and build
git clone https://github.com/your-org/fhe-llm-proxy
cd fhe-llm-proxy

# 2. Configure environment
cp .env.production .env
# Edit .env with your settings

# 3. Deploy
docker-compose -f docker-compose.prod.yml up -d
```

## 📋 Prerequisites

### System Requirements

- **CPU**: 4+ cores (8+ recommended)
- **Memory**: 8GB RAM minimum (16GB+ recommended)
- **Storage**: 100GB+ SSD
- **Network**: Public IP with ports 80, 443, 8080, 9090 accessible
- **OS**: Linux (Ubuntu 22.04+ recommended)

### Dependencies

- Docker 24.0+
- Docker Compose 2.20+
- NVIDIA GPU (optional, for acceleration)
- SSL certificates (Let's Encrypt or custom)

## 🔧 Configuration

### 1. Environment Setup

```bash
# Copy and customize environment file
cp .env.production .env
```

Edit `.env` with your specific configuration:

```bash
# Required API keys
OPENAI_API_KEY=sk-your-key-here
ANTHROPIC_API_KEY=your-anthropic-key

# Database connections
DATABASE_URL=postgresql://user:pass@host:5432/db
REDIS_URL=redis://localhost:6379

# Security
GRAFANA_PASSWORD=secure-admin-password

# Regional compliance
REGION=us-east-1
COMPLIANCE_MODE=gdpr
GDPR_ENABLED=true
CCPA_ENABLED=true
```

### 2. SSL Certificates

#### Option A: Let's Encrypt (Recommended)

```bash
# Install certbot
sudo apt install certbot

# Generate certificates
sudo certbot certonly --standalone -d your-domain.com

# Copy certificates
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem ./certs/
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem ./certs/
```

#### Option B: Custom Certificates

```bash
# Place your certificates in the certs directory
mkdir -p certs
cp your-cert.pem certs/fullchain.pem
cp your-key.pem certs/privkey.pem
```

### 3. Configuration File

Update `config.toml` for production:

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4
max_connections = 1000
enable_cors = true
cors_allowed_origins = ["https://your-domain.com"]

[security]
require_https = true
tls_cert_path = "/app/certs/fullchain.pem"
tls_key_path = "/app/certs/privkey.pem"
require_api_key = true

[compliance]
gdpr_enabled = true
ccpa_enabled = true
audit_logs_enabled = true
data_encryption_at_rest = true
```

## 🚀 Deployment Methods

### Method 1: Docker Compose (Recommended)

```bash
# Deploy full stack
docker-compose -f docker-compose.prod.yml up -d

# Check status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f fhe-proxy
```

### Method 2: Kubernetes

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# Check deployment
kubectl get pods -n fhe-proxy

# Scale deployment
kubectl scale deployment fhe-proxy --replicas=5 -n fhe-proxy
```

### Method 3: Global Multi-Region

```bash
# Deploy to multiple regions
./scripts/deploy-global.sh

# Monitor deployment
kubectl get pods -A -l app=fhe-proxy
```

## 📊 Monitoring & Observability

### Access Dashboards

- **Main API**: https://your-domain.com
- **Health Check**: https://your-domain.com/health
- **Metrics**: https://your-domain.com:9090/metrics
- **Grafana**: https://your-domain.com:3000
- **Prometheus**: https://your-domain.com:9091

### Key Metrics to Monitor

```yaml
SLI/SLO Targets:
  - API Availability: >99.9%
  - Response Time P95: <200ms
  - Error Rate: <1%
  - Privacy Budget Utilization: <90%
  - FHE Operation Success: >99%
```

### Alerting Rules

```yaml
Critical Alerts:
  - Service down for >5 minutes
  - Error rate >5% for >2 minutes
  - Response time P95 >1s for >5 minutes
  - Privacy budget exhausted
  - FHE operation failures >10%

Warning Alerts:
  - High CPU usage >80% for >10 minutes
  - High memory usage >85% for >10 minutes
  - Disk space >90%
  - SSL certificate expiring <30 days
```

## 🔐 Security Hardening

### 1. Network Security

```bash
# Firewall configuration
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw enable
```

### 2. Container Security

```yaml
# Security configurations applied:
security_opt:
  - no-new-privileges:true
  - seccomp:unconfined

# Non-root user
user: "1001:1001"

# Read-only filesystem
read_only: true
```

### 3. Secret Management

```bash
# Use Docker secrets
echo "your-api-key" | docker secret create openai_api_key -

# Or external secret management
export OPENAI_API_KEY=$(vault kv get -field=key secret/openai)
```

## 🌍 Compliance & Privacy

### GDPR Compliance

- ✅ Data minimization
- ✅ Right to erasure
- ✅ Privacy by design
- ✅ Audit logging
- ✅ Data retention policies

### CCPA Compliance

- ✅ Privacy rights disclosure
- ✅ Opt-out mechanisms
- ✅ Data category tracking
- ✅ Retention limits

### HIPAA Compliance (Optional)

- ✅ PHI encryption
- ✅ Access controls
- ✅ Audit trails
- ✅ Data integrity

## 🔄 Backup & Recovery

### Automated Backups

```bash
# Database backup
docker exec fhe-postgres pg_dump -U postgres fhe_db | gzip > backup.sql.gz

# Configuration backup
tar -czf config-backup.tar.gz config.toml locales/ certs/

# Upload to S3
aws s3 cp backup.sql.gz s3://your-backup-bucket/$(date +%Y%m%d)/
```

### Disaster Recovery

```bash
# Complete system restore
docker-compose -f docker-compose.prod.yml down
docker-compose -f docker-compose.prod.yml up -d

# Database restore
gunzip < backup.sql.gz | docker exec -i fhe-postgres psql -U postgres -d fhe_db
```

## 📈 Scaling

### Horizontal Scaling

```bash
# Scale with Docker Compose
docker-compose -f docker-compose.prod.yml up -d --scale fhe-proxy=5

# Scale with Kubernetes
kubectl scale deployment fhe-proxy --replicas=10
```

### Auto-scaling Configuration

```yaml
# HPA configuration
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
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

## 🔍 Troubleshooting

### Common Issues

**Container Won't Start**
```bash
# Check logs
docker-compose -f docker-compose.prod.yml logs fhe-proxy

# Check configuration
docker-compose -f docker-compose.prod.yml config
```

**High Memory Usage**
```bash
# Monitor memory
docker stats fhe-proxy-prod

# Adjust memory limits
export FHE_MEMORY_LIMIT=16G
docker-compose -f docker-compose.prod.yml up -d
```

**SSL Certificate Issues**
```bash
# Verify certificate
openssl x509 -in certs/fullchain.pem -text -noout

# Test SSL connection
openssl s_client -connect your-domain.com:443
```

### Performance Tuning

```bash
# CPU optimization
export RUST_MIN_STACK=8388608
export RAYON_NUM_THREADS=4

# Memory optimization
export MALLOC_ARENA_MAX=4

# GPU optimization (if available)
export CUDA_VISIBLE_DEVICES=0
export FHE_GPU_ENABLED=true
```

## 📞 Support

### Health Checks

```bash
# Application health
curl https://your-domain.com/health

# Component health
curl https://your-domain.com/health/components

# Metrics endpoint
curl https://your-domain.com/metrics
```

### Logs

```bash
# Application logs
docker-compose -f docker-compose.prod.yml logs -f fhe-proxy

# System logs
journalctl -u docker -f

# Error logs only
docker-compose -f docker-compose.prod.yml logs fhe-proxy | grep ERROR
```

### Getting Help

- 📧 Email: support@terragonlabs.com
- 💬 Slack: #fhe-proxy-support
- 🐛 Issues: GitHub Issues
- 📖 Docs: [Full Documentation](./docs/)

## 🔄 Updates & Maintenance

### Rolling Updates

```bash
# Pull latest image
docker-compose -f docker-compose.prod.yml pull

# Rolling restart
docker-compose -f docker-compose.prod.yml up -d --no-deps fhe-proxy
```

### Security Updates

```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Update Docker images
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d

# Restart with new images
docker-compose -f docker-compose.prod.yml restart
```

---

**🔒 Security Notice**: This deployment includes comprehensive security measures, but always review and adapt to your organization's security requirements.

**📊 Performance Note**: For high-traffic production environments, consider dedicated infrastructure and load balancing.

**🌍 Compliance Note**: Ensure your deployment configuration matches your regulatory requirements (GDPR, CCPA, HIPAA, etc.).