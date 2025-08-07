#!/bin/bash
set -euo pipefail

# FHE Proxy Container Entrypoint
# Handles initialization, configuration, and graceful shutdown

# Default configuration
export FHE_CONFIG_FILE="${FHE_CONFIG_FILE:-/app/config.toml}"
export FHE_LOG_LEVEL="${FHE_LOG_LEVEL:-info}"
export RUST_LOG="${RUST_LOG:-homomorphic_llm_proxy=${FHE_LOG_LEVEL},tower_http=debug}"

# Performance tuning
export MALLOC_ARENA_MAX="${MALLOC_ARENA_MAX:-4}"
export MALLOC_MMAP_THRESHOLD_="${MALLOC_MMAP_THRESHOLD_:-131072}"

echo "=== FHE LLM Proxy Starting ==="
echo "Config file: $FHE_CONFIG_FILE"
echo "Log level: $FHE_LOG_LEVEL"
echo "Build version: ${BUILD_VERSION:-unknown}"
echo "Build timestamp: ${BUILD_TIMESTAMP:-unknown}"

# Wait for external dependencies if needed
if [ "${WAIT_FOR_DEPS:-false}" = "true" ]; then
    echo "Waiting for dependencies..."
    if [ -n "${REDIS_URL:-}" ]; then
        until nc -z ${REDIS_URL//redis:\/\//} 2>/dev/null; do
            echo "Waiting for Redis..."
            sleep 2
        done
    fi
    
    if [ -n "${DATABASE_URL:-}" ]; then
        echo "Waiting for database..."
        # Add database health check here
        sleep 2
    fi
fi

# Validate configuration
if [ ! -f "$FHE_CONFIG_FILE" ]; then
    echo "ERROR: Configuration file not found: $FHE_CONFIG_FILE"
    exit 1
fi

# Generate or load TLS certificates if required
if [ "${GENERATE_TLS_CERTS:-false}" = "true" ]; then
    echo "Generating TLS certificates..."
    mkdir -p /tmp/certs
    
    # Generate self-signed certificate (for development)
    openssl req -x509 -newkey rsa:4096 -keyout /tmp/certs/key.pem -out /tmp/certs/cert.pem \
        -days 365 -nodes -subj "/C=US/ST=State/L=City/O=Org/CN=localhost"
    
    export TLS_CERT_PATH="/tmp/certs/cert.pem"
    export TLS_KEY_PATH="/tmp/certs/key.pem"
fi

# Set up signal handlers for graceful shutdown
_term() {
    echo "Received SIGTERM, initiating graceful shutdown..."
    kill -TERM "$child" 2>/dev/null || true
    wait "$child"
}
trap _term SIGTERM SIGINT

# Start the application
echo "Starting FHE proxy server..."
exec "$@" &
child=$!

# Wait for the application
wait "$child"