# Developer Guide

## Development Environment Setup

### Prerequisites

- Rust 1.75+ with CUDA support
- NVIDIA GPU with compute capability ≥ 7.0
- Docker and Docker Compose
- Python 3.9+ for client development

### Local Development

1. **Clone and Setup**
   ```bash
   git clone https://github.com/your-org/homomorphic-llm-proxy
   cd homomorphic-llm-proxy
   ./scripts/dev-setup.sh
   ```

2. **Development Dependencies**
   ```bash
   # Rust development
   cargo install cargo-watch cargo-nextest
   
   # Python development
   pip install -r requirements-dev.txt
   
   # Pre-commit hooks
   pre-commit install
   ```

3. **Local Testing**
   ```bash
   # Run all tests
   just test
   
   # Run with file watching
   cargo watch -x test
   
   # Run specific test suite
   cargo nextest run unit
   ```

### Architecture Overview

#### Core Components

- **FHE Gateway** (`src/proxy.rs`): Main proxy server
- **Encryption Layer** (`src/fhe.rs`): CKKS implementation
- **Configuration** (`src/config.rs`): System configuration
- **Error Handling** (`src/error.rs`): Unified error types

#### Key Data Flows

1. **Request Processing**
   ```rust
   Client → Encryption → Proxy → LLM Provider → Response → Decryption → Client
   ```

2. **Key Management**
   ```rust
   KeyGenerator → SecureStorage → RotationScheduler → ClientSync
   ```

### Development Workflow

#### Making Changes

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Implement Changes**
   - Follow Rust best practices
   - Add comprehensive tests
   - Update documentation

3. **Run Quality Checks**
   ```bash
   just lint    # Run clippy and formatting
   just test    # Run test suite
   just bench   # Run benchmarks
   ```

4. **Submit Pull Request**
   - Use conventional commit messages
   - Include tests and documentation
   - Request reviews from code owners

#### Code Standards

- **Formatting**: Use `rustfmt` with project configuration
- **Linting**: Address all `clippy` warnings
- **Testing**: Maintain >90% code coverage
- **Documentation**: Document all public APIs

### Testing Strategy

#### Test Types

1. **Unit Tests** (`tests/unit/`)
   - Individual component testing
   - Mock external dependencies
   - Fast execution (<1s per test)

2. **Integration Tests** (`tests/integration/`)
   - Component interaction testing
   - Real dependency usage
   - End-to-end scenarios

3. **End-to-End Tests** (`tests/e2e/`)
   - Full system testing
   - Production-like environment
   - Performance validation

#### Test Configuration

```toml
# nextest.toml
[profile.default]
test-threads = 4
slow-timeout = { period = "60s", terminate-after = 2 }

[profile.ci]
test-threads = 2
slow-timeout = { period = "30s" }
```

### Performance Optimization

#### Profiling

```bash
# CPU profiling
cargo build --release --features profiling
perf record --call-graph=dwarf ./target/release/fhe-proxy
perf report

# Memory profiling
valgrind --tool=massif ./target/release/fhe-proxy
```

#### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Specific benchmark
cargo bench --bench latency

# Compare against baseline
cargo bench -- --baseline main
```

### Security Considerations

#### Cryptographic Code

- Use constant-time algorithms
- Validate all input parameters
- Implement secure key generation
- Regular security audits

#### Memory Safety

- Avoid unsafe code where possible
- Use secure memory allocation
- Clear sensitive data from memory
- Prevent timing attacks

### Debugging

#### Logging Configuration

```rust
// Set log levels
RUST_LOG=debug cargo run

// Module-specific logging
RUST_LOG=fhe_proxy::fhe=trace cargo run

// JSON structured logging
RUST_LOG_FORMAT=json cargo run
```

#### Debug Tools

- **GDB**: Native debugging with Rust support
- **LLDB**: Alternative debugger
- **Flamegraph**: Performance profiling
- **Heaptrack**: Memory usage analysis

### Integration Development

#### Client SDK Development

```python
# Local development setup
cd python/
pip install -e .[dev]

# Run Python tests
pytest tests/ -v

# Type checking
mypy src/
```

#### API Testing

```bash
# Start local server
cargo run -- --config config/dev.toml

# Test endpoints
curl -X POST http://localhost:8080/v1/encrypt \
  -H "Content-Type: application/json" \
  -d '{"prompt": "test"}'
```

### Contribution Guidelines

#### Pull Request Checklist

- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Performance impact assessed
- [ ] Security review completed
- [ ] Changelog entry added

#### Code Review Process

1. **Automated Checks**: CI must pass
2. **Peer Review**: At least one approval required
3. **Security Review**: For crypto-related changes
4. **Performance Review**: For performance-critical code

### Resources

- **Rust Documentation**: https://doc.rust-lang.org/
- **SEAL Library**: https://github.com/microsoft/SEAL
- **CUDA Programming**: https://docs.nvidia.com/cuda/
- **Project Issues**: https://github.com/your-org/homomorphic-llm-proxy/issues