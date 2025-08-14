#!/bin/bash
# Health check script for production deployment

# Configuration
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-8080}
TIMEOUT=${TIMEOUT:-10}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 FHE LLM Proxy Health Check"
echo "==============================="

# Function to check HTTP endpoint
check_endpoint() {
    local endpoint=$1
    local expected_status=${2:-200}
    
    echo -n "Checking $endpoint... "
    
    response=$(curl -s -o /dev/null -w "%{http_code}" \
        --connect-timeout $TIMEOUT \
        "http://$HOST:$PORT$endpoint" 2>/dev/null)
    
    if [ "$response" = "$expected_status" ]; then
        echo -e "${GREEN}✓ OK ($response)${NC}"
        return 0
    else
        echo -e "${RED}✗ FAIL (got $response, expected $expected_status)${NC}"
        return 1
    fi
}

# Function to check if service is running
check_service() {
    echo -n "Checking if service is listening on port $PORT... "
    
    if command -v nc >/dev/null 2>&1; then
        if nc -z "$HOST" "$PORT" 2>/dev/null; then
            echo -e "${GREEN}✓ Service is running${NC}"
            return 0
        else
            echo -e "${RED}✗ Service not responding${NC}"
            return 1
        fi
    else
        # Fallback to curl if nc is not available
        if curl -s --connect-timeout 3 "http://$HOST:$PORT" >/dev/null 2>&1; then
            echo -e "${GREEN}✓ Service is running${NC}"
            return 0
        else
            echo -e "${RED}✗ Service not responding${NC}"
            return 1
        fi
    fi
}

# Main health checks
failed_checks=0

# Check if service is running
if ! check_service; then
    echo -e "${RED}Service is not running. Cannot proceed with health checks.${NC}"
    exit 1
fi

# Health endpoint
if ! check_endpoint "/health"; then
    ((failed_checks++))
fi

# Metrics endpoint
if ! check_endpoint "/metrics"; then
    ((failed_checks++))
fi

# FHE parameters endpoint
if ! check_endpoint "/v1/params"; then
    ((failed_checks++))
fi

# Summary
echo "==============================="
if [ $failed_checks -eq 0 ]; then
    echo -e "${GREEN}✓ All health checks passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $failed_checks health check(s) failed${NC}"
    exit 1
fi