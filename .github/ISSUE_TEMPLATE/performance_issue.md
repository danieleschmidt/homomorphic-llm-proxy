---
name: Performance Issue
about: Report performance problems or regressions
title: '[PERFORMANCE] '
labels: performance
assignees: ''
---

## Performance Issue Description
A clear and concise description of the performance problem.

## Performance Metrics
Please provide specific metrics where possible:

### Latency
- **Current latency**: [e.g., 2000ms for GPT-2 inference]
- **Expected latency**: [e.g., <500ms based on benchmarks]
- **Baseline comparison**: [e.g., 3.8x overhead vs target 3.5x]

### Throughput
- **Current throughput**: [e.g., 45 requests/minute]
- **Expected throughput**: [e.g., 100+ requests/minute]
- **Concurrent users**: [e.g., 10 concurrent clients]

### Resource Usage
- **GPU Memory**: [e.g., 6GB used vs 4GB expected]
- **CPU Usage**: [e.g., 95% constantly vs expected 60%]
- **Network I/O**: [e.g., bandwidth utilization]

## Environment Details
- **Version**: [e.g., v0.1.0]
- **Platform**: [e.g., Ubuntu 22.04, Docker]
- **Hardware**: 
  - GPU: [e.g., NVIDIA A100, RTX 4090]
  - CPU: [e.g., Intel Xeon, AMD EPYC]
  - RAM: [e.g., 64GB DDR4]
- **Configuration**:
  - Poly modulus degree: [e.g., 16384]
  - Batch size: [e.g., 32]
  - Worker count: [e.g., 4]

## Workload Characteristics
- **Model**: [e.g., GPT-2, GPT-4, custom]
- **Input size**: [e.g., average 512 tokens]
- **Request pattern**: [e.g., steady 10 req/min, burst 100 req/min]
- **Encryption parameters**: [e.g., security level 128]

## Steps to Reproduce
1. Set up environment with [specific configuration]
2. Run workload [describe the test]
3. Monitor [specific metrics]
4. Observe [performance issue]

## Benchmark Results
If you have benchmark data, please include:
```
# Example benchmark output
Model: GPT-2
Batch size: 16
Latency: 1850ms (target: <600ms)
GPU Memory: 5.2GB (target: <4GB)
Throughput: 32 req/min (target: 100+ req/min)
```

## Expected Behavior
What performance characteristics were you expecting?

## Regression Information
- **Is this a regression?**: [Yes/No/Unknown]
- **Last known good version**: [e.g., v0.0.9]
- **When did you first notice**: [e.g., after upgrade to v0.1.0]

## Profiling Data
If available, please attach:
- [ ] CPU profiling results
- [ ] GPU profiling results (nsight, nvprof)
- [ ] Memory usage graphs
- [ ] Network timing data

## Additional Context
Any other context about the performance issue, including:
- Specific use case requirements
- Business impact
- Workarounds attempted
- Related issues or discussions