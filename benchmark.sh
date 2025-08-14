#!/bin/bash
# Performance benchmark script

echo "🚀 FHE LLM Proxy Performance Benchmark"
echo "======================================"

# Check if required tools are available
if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required for benchmarking"
    exit 1
fi

# Configuration
HOST=${HOST:-127.0.0.1}
PORT=${PORT:-8080}
BASE_URL="http://$HOST:$PORT"

# Performance test function
run_benchmark() {
    local endpoint=$1
    local requests=${2:-100}
    local concurrency=${3:-10}
    
    echo "Testing $endpoint with $requests requests ($concurrency concurrent)"
    
    # Simple benchmark using curl
    start_time=$(date +%s.%N)
    
    for ((i=1; i<=requests; i++)); do
        if [ $((i % concurrency)) -eq 0 ]; then
            wait # Wait for background jobs to complete
        fi
        
        curl -s "$BASE_URL$endpoint" > /dev/null &
        
        if [ $((i % 10)) -eq 0 ]; then
            echo -n "."
        fi
    done
    
    wait # Wait for all background jobs
    end_time=$(date +%s.%N)
    
    duration=$(echo "$end_time - $start_time" | bc -l)
    rps=$(echo "scale=2; $requests / $duration" | bc -l)
    
    echo
    echo "  Requests: $requests"
    echo "  Duration: ${duration}s"
    echo "  RPS: $rps"
    echo
}

# Check service availability
echo "Checking service availability..."
if ! curl -s "$BASE_URL/health" > /dev/null; then
    echo "Error: Service not available at $BASE_URL"
    exit 1
fi

echo "Service is available. Starting benchmarks..."
echo

# Run benchmarks
run_benchmark "/health" 50 5
run_benchmark "/metrics" 50 5
run_benchmark "/v1/params" 50 5

echo "Benchmark completed!"