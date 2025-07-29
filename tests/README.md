# Testing Infrastructure

This directory contains the comprehensive testing infrastructure for the Homomorphic LLM Proxy project.

## Test Structure

```
tests/
├── unit/                 # Unit tests for individual components
│   ├── rust/            # Rust unit tests
│   └── python/          # Python unit tests
├── integration/         # Integration tests
│   ├── api/            # API endpoint tests
│   ├── fhe/            # FHE operation tests
│   └── provider/       # LLM provider integration tests
├── e2e/                # End-to-end tests
│   ├── scenarios/      # Test scenarios
│   └── fixtures/       # Test data and fixtures
├── performance/        # Performance and benchmarking tests
│   ├── latency/        # Latency benchmarks
│   ├── throughput/     # Throughput benchmarks
│   └── memory/         # Memory usage tests
├── security/           # Security and cryptographic tests
│   ├── fhe_security/   # FHE scheme security tests
│   ├── key_mgmt/       # Key management tests
│   └── privacy/        # Privacy budget tests
├── load/               # Load testing
│   ├── scenarios/      # Load test scenarios
│   └── reports/        # Load test reports
└── fixtures/           # Shared test fixtures and data
    ├── keys/           # Test encryption keys
    ├── prompts/        # Test prompts and responses
    └── configs/        # Test configurations
```

## Running Tests

### Rust Tests
```bash
# Unit tests
cargo test

# With GPU features
cargo test --features gpu

# Integration tests
cargo test --test integration

# Performance benchmarks
cargo bench
```

### Python Tests
```bash
# Install test dependencies
pip install -e ".[dev]"

# Run all tests
pytest

# With coverage
pytest --cov=fhe_llm_proxy --cov-report=html

# Specific test categories
pytest tests/unit/python/
pytest tests/integration/
pytest tests/e2e/
```

### End-to-End Tests
```bash
# Start test environment
docker-compose -f docker-compose.test.yml up -d

# Run E2E tests
pytest tests/e2e/

# Cleanup
docker-compose -f docker-compose.test.yml down
```

## Test Categories

### Unit Tests
- **Scope**: Individual functions and methods
- **Coverage Target**: >95%
- **Execution Time**: <5 minutes total
- **Dependencies**: Mocked external services

### Integration Tests
- **Scope**: Component interactions
- **Coverage Target**: >85%
- **Execution Time**: <15 minutes total
- **Dependencies**: Test containers for Redis, mock LLM

### End-to-End Tests
- **Scope**: Full user workflows
- **Coverage Target**: Critical paths covered
- **Execution Time**: <30 minutes total
- **Dependencies**: Full environment with GPU

### Performance Tests
- **Scope**: Latency, throughput, resource usage
- **Coverage Target**: All critical paths
- **Execution Time**: <60 minutes total
- **Dependencies**: GPU-enabled environment

### Security Tests
- **Scope**: Cryptographic correctness, privacy guarantees
- **Coverage Target**: All security-critical code
- **Execution Time**: <45 minutes total
- **Dependencies**: Cryptographic test vectors

## Test Data Management

### Test Keys
- Pre-generated test keys for consistent testing
- Separate keys for unit vs integration tests
- Key rotation testing scenarios

### Test Prompts
- Various prompt sizes and complexities
- Edge cases and malformed inputs
- Privacy-sensitive test data (synthetic)

### Test Configurations
- Different FHE parameter sets
- Various GPU configurations
- Performance testing configurations

## CI/CD Integration

### Test Execution Order
1. **Fast Tests** (unit, linting): <5 minutes
2. **Integration Tests**: <15 minutes
3. **Security Tests**: <45 minutes
4. **Performance Tests**: <60 minutes (on GPU runners)
5. **Load Tests**: Manual trigger only

### Quality Gates
- Unit test coverage: ≥95%
- Integration test coverage: ≥85%
- Security tests: All must pass
- Performance regression: <5% degradation
- Load tests: Meet SLA requirements

## Local Development

### Setup Test Environment
```bash
# Install dependencies
./scripts/setup-test-env.sh

# Generate test keys
./scripts/generate-test-keys.sh

# Start test services
docker-compose -f docker-compose.test.yml up -d
```

### Pre-commit Testing
```bash
# Run pre-commit hooks
pre-commit run --all-files

# Quick test suite (fast tests only)
./scripts/quick-test.sh

# Full test suite
./scripts/full-test.sh
```

### Debug Testing
```bash
# Run tests with debug output
RUST_LOG=debug cargo test
pytest -v -s tests/

# Run specific test
cargo test test_fhe_encryption
pytest tests/unit/python/test_client.py::test_encrypt_decrypt
```

## Performance Testing

### Benchmark Categories
- **Latency**: Single request response time
- **Throughput**: Requests per second
- **Concurrency**: Multiple simultaneous requests
- **Memory**: GPU and system memory usage
- **Scalability**: Performance under load

### Benchmark Execution
```bash
# Latency benchmarks
cargo bench --bench latency

# Throughput benchmarks
cargo bench --bench throughput

# Memory benchmarks
cargo bench --bench memory

# Full benchmark suite
./scripts/run-benchmarks.sh
```

### Performance Regression Detection
- Automated comparison with baseline
- Alert on >5% performance degradation
- Historical performance tracking
- GPU-specific performance metrics

## Security Testing

### Cryptographic Testing
- FHE scheme correctness verification
- Key generation and rotation testing
- Ciphertext operations validation
- Privacy budget calculations

### Security Scenarios
- Invalid key handling
- Malformed ciphertext processing
- Side-channel attack resistance
- Memory safety validation

### Privacy Testing
- Differential privacy guarantees
- Data leakage prevention
- Audit trail verification
- Compliance validation

## Load Testing

### Load Test Scenarios
- **Steady State**: Consistent load over time
- **Spike**: Sudden traffic increases
- **Ramp Up**: Gradual load increase
- **Endurance**: Extended duration testing

### Load Test Execution
```bash
# Install load testing tools
pip install locust

# Run load tests
locust -f tests/load/steady_state.py --host=http://localhost:8080

# Generate load test reports
./scripts/generate-load-report.sh
```

## Test Maintenance

### Regular Tasks
- Update test data monthly
- Review test coverage quarterly
- Update performance baselines after optimization
- Refresh test keys annually

### Test Infrastructure Updates
- Monitor test execution times
- Update test dependencies
- Optimize slow tests
- Add tests for new features

## Troubleshooting

### Common Issues
- **GPU tests failing**: Check CUDA drivers and GPU availability
- **Slow test execution**: Use test parallelization and caching
- **Flaky tests**: Implement proper test isolation and cleanup
- **Memory issues**: Monitor and optimize test resource usage

### Debug Resources
- Test execution logs in CI/CD
- Local test debugging guides
- Performance profiling tools
- Security testing utilities