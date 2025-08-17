#!/bin/bash
# Health check script for FHE LLM Proxy container

set -e

# Configuration
HEALTH_URL="http://localhost:8080/health"
METRICS_URL="http://localhost:9090/metrics"
TIMEOUT=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[HEALTH] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[HEALTH] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[HEALTH] ERROR: $1${NC}"
    exit 1
}

# Check if the main health endpoint is responsive
check_health_endpoint() {
    local response
    local status_code
    
    response=$(curl -s -w "%{http_code}" --max-time "$TIMEOUT" "$HEALTH_URL" 2>/dev/null || echo "000")
    status_code="${response: -3}"
    
    if [[ "$status_code" == "200" ]]; then
        log "Health endpoint is responsive (HTTP $status_code)"
        return 0
    else
        error "Health endpoint failed (HTTP $status_code)"
        return 1
    fi
}

# Check if metrics endpoint is accessible
check_metrics_endpoint() {
    local response
    local status_code
    
    response=$(curl -s -w "%{http_code}" --max-time "$TIMEOUT" "$METRICS_URL" 2>/dev/null || echo "000")
    status_code="${response: -3}"
    
    if [[ "$status_code" == "200" ]]; then
        log "Metrics endpoint is responsive (HTTP $status_code)"
        return 0
    else
        warn "Metrics endpoint failed (HTTP $status_code)"
        return 0  # Non-critical for health check
    fi
}

# Check memory usage
check_memory_usage() {
    local memory_limit_kb=2097152  # 2GB in KB
    local memory_usage_kb
    local memory_percent
    
    if [[ -f /proc/meminfo ]]; then
        memory_usage_kb=$(awk '/MemAvailable/ {print $2}' /proc/meminfo)
        memory_percent=$((100 - (memory_usage_kb * 100 / memory_limit_kb)))
        
        if [[ $memory_percent -lt 90 ]]; then
            log "Memory usage is healthy (${memory_percent}%)"
            return 0
        else
            warn "High memory usage detected (${memory_percent}%)"
            return 0  # Non-critical for health check
        fi
    else
        log "Memory information not available"
        return 0
    fi
}

# Check CPU load
check_cpu_load() {
    local cpu_cores
    local load_avg
    local load_percent
    
    if [[ -f /proc/loadavg ]] && command -v nproc >/dev/null 2>&1; then
        cpu_cores=$(nproc)
        load_avg=$(awk '{print $1}' /proc/loadavg)
        load_percent=$(echo "$load_avg * 100 / $cpu_cores" | bc -l 2>/dev/null || echo "0")
        load_percent=${load_percent%.*}  # Remove decimal part
        
        if [[ $load_percent -lt 80 ]]; then
            log "CPU load is healthy (${load_percent}%)"
            return 0
        else
            warn "High CPU load detected (${load_percent}%)"
            return 0  # Non-critical for health check
        fi
    else
        log "CPU load information not available"
        return 0
    fi
}

# Check disk space
check_disk_space() {
    local disk_usage_percent
    
    if command -v df >/dev/null 2>&1; then
        disk_usage_percent=$(df /app | awk 'NR==2 {print $5}' | sed 's/%//')
        
        if [[ $disk_usage_percent -lt 85 ]]; then
            log "Disk space is healthy (${disk_usage_percent}% used)"
            return 0
        else
            warn "High disk usage detected (${disk_usage_percent}% used)"
            return 0  # Non-critical for health check
        fi
    else
        log "Disk space information not available"
        return 0
    fi
}

# Check if required environment variables are set
check_environment() {
    local required_vars=("FHE_HOST" "FHE_PORT")
    local missing_vars=()
    
    for var in "${required_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            missing_vars+=("$var")
        fi
    done
    
    if [[ ${#missing_vars[@]} -eq 0 ]]; then
        log "Environment variables are properly configured"
        return 0
    else
        warn "Missing environment variables: ${missing_vars[*]}"
        return 0  # Non-critical for health check
    fi
}

# Check if the process is running
check_process() {
    if pgrep -f "fhe-proxy" >/dev/null 2>&1; then
        log "FHE Proxy process is running"
        return 0
    else
        error "FHE Proxy process is not running"
        return 1
    fi
}

# Main health check function
main() {
    log "Starting health check for FHE LLM Proxy..."
    
    # Critical checks (must pass)
    check_process || exit 1
    check_health_endpoint || exit 1
    
    # Non-critical checks (warnings only)
    check_metrics_endpoint
    check_memory_usage
    check_cpu_load
    check_disk_space
    check_environment
    
    log "Health check completed successfully"
    exit 0
}

# Trap to ensure clean exit
trap 'error "Health check interrupted"' INT TERM

# Run health check
main "$@"