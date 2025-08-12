# 📚 API Documentation

## Overview

The FHE LLM Proxy provides a REST API for privacy-preserving language model inference using Fully Homomorphic Encryption. All sensitive data remains encrypted throughout the entire process.

## Base URL

```
https://api.fheproxy.example.com/v1
```

## Authentication

```bash
# API Key Authentication
Authorization: Bearer <your_api_key>

# Client Certificate Authentication (Enterprise)
curl --cert client.pem --key client.key https://api.fheproxy.example.com/v1/...
```

## Rate Limiting

- **Free Tier**: 100 requests/hour
- **Pro Tier**: 10,000 requests/hour  
- **Enterprise**: Custom limits

Rate limit headers:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Core Endpoints

### 1. Key Management

#### Generate Key Pair

```http
POST /v1/keys/generate
Content-Type: application/json

{
  "security_level": 128,
  "poly_modulus_degree": 16384,
  "client_id": "optional-client-identifier"
}
```

**Response:**
```json
{
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "server_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "public_key": "<base64_encoded_public_key>",
  "evaluation_keys": "<base64_encoded_evaluation_keys>",
  "parameters": {
    "poly_modulus_degree": 16384,
    "coefficient_modulus": [60, 40, 40, 60],
    "scale": 40,
    "security_level": 128
  }
}
```

#### Rotate Keys

```http
POST /v1/keys/rotate/{client_id}
Authorization: Bearer <api_key>
```

**Response:**
```json
{
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "new_server_id": "new-server-key-id",
  "rotated_at": 1640995200,
  "status": "success"
}
```

### 2. Encryption Operations

#### Encrypt Text

```http
POST /v1/encrypt
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "plaintext": "What is the meaning of life?",
  "encoding": "utf8"
}
```

**Response:**
```json
{
  "ciphertext_id": "ct_abc123def456",
  "encrypted_data": "<base64_encoded_ciphertext>",
  "size_bytes": 65536,
  "noise_budget": 45,
  "created_at": 1640995200
}
```

#### Decrypt Text

```http
POST /v1/decrypt
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "ciphertext_id": "ct_abc123def456"
}
```

**Response:**
```json
{
  "plaintext": "The meaning of life is to find purpose and create meaningful connections.",
  "encoding": "utf8",
  "decrypted_at": 1640995200
}
```

### 3. LLM Processing

#### Chat Completions

```http
POST /v1/chat/completions
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "ciphertext_id": "ct_abc123def456",
  "model": "gpt-4",
  "provider": "openai",
  "max_tokens": 150,
  "temperature": 0.7,
  "stream": false
}
```

**Response:**
```json
{
  "id": "fhe-completion-abc123",
  "object": "chat.completion",
  "created": 1640995200,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content_ciphertext_id": "ct_response_xyz789"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 28,
    "total_tokens": 40
  },
  "privacy_cost": {
    "epsilon_consumed": 0.1,
    "remaining_budget": 9.9
  }
}
```

#### Streaming Chat

```http
POST /v1/chat/stream
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "ciphertext_id": "ct_abc123def456",
  "model": "gpt-4",
  "provider": "openai",
  "max_tokens": 150,
  "temperature": 0.7
}
```

**Response (Server-Sent Events):**
```
data: {"stream_id": "stream_abc123", "status": "streaming"}

data: {"delta": {"content_ciphertext": "<encrypted_token_1>"}, "noise_budget": 44}

data: {"delta": {"content_ciphertext": "<encrypted_token_2>"}, "noise_budget": 43}

data: {"type": "done", "total_tokens": 28, "epsilon_consumed": 0.1}
```

### 4. Homomorphic Operations

#### Concatenate Ciphertexts

```http
POST /v1/concatenate
Content-Type: application/json
Authorization: Bearer <api_key>

{
  "ciphertext_a": "ct_abc123def456",
  "ciphertext_b": "ct_xyz789ghi012",
  "operation": "concat"
}
```

**Response:**
```json
{
  "result_ciphertext_id": "ct_concat_result_456",
  "input_a": "ct_abc123def456",
  "input_b": "ct_xyz789ghi012",
  "size_bytes": 131072,
  "noise_budget": 42,
  "created_at": 1640995200
}
```

#### Validate Ciphertext

```http
POST /v1/ciphertext/{ciphertext_id}/validate
Authorization: Bearer <api_key>
```

**Response:**
```json
{
  "ciphertext_id": "ct_abc123def456",
  "status": "valid",
  "noise_budget": 45,
  "size_bytes": 65536,
  "security_level": 128,
  "validated_at": 1640995200
}
```

### 5. Privacy Management

#### Get Privacy Budget

```http
GET /v1/privacy/budget/{user_id}
Authorization: Bearer <api_key>
```

**Response:**
```json
{
  "user_id": "user_12345",
  "total_epsilon": 10.0,
  "consumed_epsilon": 2.3,
  "remaining_epsilon": 7.7,
  "total_queries": 23,
  "last_query_at": 1640995200,
  "reset_at": 1640995200
}
```

#### Reset Privacy Budget

```http
POST /v1/privacy/budget/{user_id}/reset
Authorization: Bearer <api_key>
Content-Type: application/json

{
  "new_epsilon": 10.0,
  "reason": "Monthly reset"
}
```

### 6. System Monitoring

#### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1640995200,
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

#### Readiness Check

```http
GET /health/ready
```

**Response:**
```json
{
  "status": "ready",
  "checks": {
    "database": "ok",
    "gpu": "ok",
    "fhe_engine": "ok"
  }
}
```

#### Metrics

```http
GET /metrics
Accept: application/json
```

**Response:**
```json
{
  "requests_total": 12345,
  "requests_per_second": 45.2,
  "average_latency_ms": 250,
  "gpu_utilization_percent": 75.5,
  "memory_usage_mb": 4096,
  "active_connections": 23,
  "cache_hit_rate": 0.85
}
```

#### Detailed Metrics

```http
GET /metrics/detailed
Authorization: Bearer <api_key>
```

**Response:**
```json
{
  "performance": {
    "encryption_time_ms": 150,
    "decryption_time_ms": 120,
    "homomorphic_ops_per_sec": 1000,
    "throughput_tokens_per_sec": 25.5
  },
  "security": {
    "total_tracked_ips": 456,
    "currently_blocked_ips": 3,
    "high_risk_ips": 12,
    "failed_auth_attempts": 89
  },
  "resources": {
    "cpu_usage_percent": 65.2,
    "memory_usage_percent": 78.9,
    "gpu_memory_usage_percent": 82.1,
    "disk_usage_percent": 45.3
  }
}
```

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_CIPHERTEXT",
    "message": "The provided ciphertext is malformed or corrupted",
    "details": {
      "ciphertext_id": "ct_abc123def456",
      "validation_errors": ["size_mismatch", "invalid_noise_budget"]
    },
    "request_id": "req_xyz789",
    "timestamp": 1640995200
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_REQUEST` | 400 | Malformed request body |
| `UNAUTHORIZED` | 401 | Invalid or missing API key |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `RATE_LIMITED` | 429 | Rate limit exceeded |
| `INVALID_CIPHERTEXT` | 400 | Malformed ciphertext |
| `ENCRYPTION_FAILED` | 500 | Encryption operation failed |
| `PROVIDER_ERROR` | 502 | LLM provider error |
| `GPU_ERROR` | 500 | GPU computation error |
| `PRIVACY_BUDGET_EXCEEDED` | 403 | Privacy budget exhausted |

## SDKs

### Python SDK

```python
pip install fhe-llm-proxy
```

```python
from fhe_llm_proxy import FHEClient

client = FHEClient(
    api_key="your_api_key",
    base_url="https://api.fheproxy.example.com/v1"
)

# Generate keys
keys = client.generate_keys(security_level=128)

# Encrypt and process
response = client.chat(
    messages=[{"role": "user", "content": "Hello world"}],
    model="gpt-4"
)
```

### JavaScript SDK

```bash
npm install fhe-llm-proxy
```

```javascript
import { FHEClient } from 'fhe-llm-proxy';

const client = new FHEClient({
  apiKey: 'your_api_key',
  baseUrl: 'https://api.fheproxy.example.com/v1'
});

// Generate keys
const keys = await client.generateKeys({ securityLevel: 128 });

// Encrypt and process
const response = await client.chat({
  messages: [{ role: 'user', content: 'Hello world' }],
  model: 'gpt-4'
});
```

### Rust SDK

```toml
[dependencies]
fhe-llm-proxy = "1.0"
```

```rust
use fhe_llm_proxy::{FHEClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .api_key("your_api_key")
        .base_url("https://api.fheproxy.example.com/v1");
    
    let client = FHEClient::new(config)?;
    
    // Generate keys
    let keys = client.generate_keys(128).await?;
    
    // Encrypt and process
    let response = client.chat()
        .message("user", "Hello world")
        .model("gpt-4")
        .send()
        .await?;
    
    Ok(())
}
```

## OpenAPI Specification

Full OpenAPI 3.0 specification available at:
- **JSON**: `GET /v1/openapi.json`
- **YAML**: `GET /v1/openapi.yaml`
- **Interactive**: `GET /v1/docs`

## Rate Limits and Quotas

### Default Limits

| Tier | Requests/Hour | Ciphertexts/Day | Storage |
|------|---------------|-----------------|---------|
| Free | 100 | 1,000 | 1 GB |
| Pro | 10,000 | 100,000 | 100 GB |
| Enterprise | Custom | Custom | Custom |

### Request Headers

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
X-RateLimit-Retry-After: 3600
```

## Security Features

### Request Signing (Enterprise)

```python
import hmac
import hashlib
import time

def sign_request(method, url, body, secret_key):
    timestamp = str(int(time.time()))
    message = f"{method}\n{url}\n{body}\n{timestamp}"
    signature = hmac.new(
        secret_key.encode(),
        message.encode(),
        hashlib.sha256
    ).hexdigest()
    
    return {
        'X-Timestamp': timestamp,
        'X-Signature': signature
    }
```

### IP Whitelisting

```json
{
  "allowed_ips": [
    "192.168.1.0/24",
    "10.0.0.0/8",
    "203.0.113.42"
  ],
  "blocked_ips": [
    "192.0.2.1"
  ]
}
```

## Webhooks

### Event Types

- `ciphertext.created`
- `ciphertext.processed`  
- `privacy.budget_exceeded`
- `key.rotated`
- `error.occurred`

### Webhook Configuration

```http
POST /v1/webhooks
Content-Type: application/json

{
  "url": "https://your-app.com/webhook",
  "events": ["ciphertext.processed", "privacy.budget_exceeded"],
  "secret": "webhook_secret_for_verification"
}
```

### Webhook Payload

```json
{
  "id": "evt_abc123",
  "type": "ciphertext.processed",
  "created": 1640995200,
  "data": {
    "ciphertext_id": "ct_abc123def456",
    "processing_time_ms": 250,
    "tokens_generated": 42
  }
}
```

## Example Integrations

### LangChain

```python
from langchain.llms.base import LLM
from fhe_llm_proxy import FHEClient

class FHELangChainLLM(LLM):
    def __init__(self, api_key: str):
        self.client = FHEClient(api_key=api_key)
    
    def _call(self, prompt: str, stop=None) -> str:
        response = self.client.chat(
            messages=[{"role": "user", "content": prompt}]
        )
        return response.content

# Usage
llm = FHELangChainLLM(api_key="your_key")
result = llm("What is quantum computing?")
```

### OpenAI SDK Proxy

```python
import openai
from fhe_llm_proxy.middleware import FHEOpenAIProxy

# Wrap OpenAI client
openai.api_base = "https://api.fheproxy.example.com/v1/openai"
openai.api_key = "your_fhe_proxy_key"

# Use normally - encryption is transparent
response = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello world"}]
)
```

## Performance Tips

1. **Batch Requests**: Use batch processing for multiple prompts
2. **Key Reuse**: Generate keys once and reuse for session
3. **Caching**: Enable ciphertext caching for repeated operations
4. **GPU Optimization**: Use larger batch sizes with GPU acceleration
5. **Parameter Tuning**: Adjust polynomial degree based on use case

## Support and Resources

- **Documentation**: https://docs.fheproxy.example.com
- **Status Page**: https://status.fheproxy.example.com  
- **Support Email**: support@fheproxy.example.com
- **Community Discord**: https://discord.gg/fheproxy