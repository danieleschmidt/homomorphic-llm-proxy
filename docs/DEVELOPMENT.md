# Development Guide

## Prerequisites

### System Requirements
* **Operating System**: Linux, macOS, or Windows with WSL2
* **CUDA**: Version 12.0+ (for GPU acceleration)
* **GPU**: NVIDIA GPU with compute capability ≥ 7.0
* **Memory**: 16GB+ RAM recommended, 8GB+ GPU memory for testing

### Software Dependencies
* **Rust**: 1.75+ with Cargo
* **Python**: 3.9+ (for client SDK development)
* **Docker**: Latest version with GPU support
* **Git**: Latest version

### Development Tools
* **IDE**: VS Code (recommended) or your preferred editor
* **Debugger**: GDB, LLDB, or VS Code debugger
* **Profiler**: cargo-flamegraph, perf, nvidia-nsight

## Setup

### 1. Clone Repository
```bash
git clone https://github.com/your-org/homomorphic-llm-proxy
cd homomorphic-llm-proxy
```

### 2. Install Rust Dependencies
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install development tools
cargo install cargo-flamegraph
cargo install cargo-audit
cargo install cargo-outdated
```

### 3. Setup Development Environment
```bash
# Run setup script
./scripts/dev-setup.sh

# Install pre-commit hooks
pip install pre-commit
pre-commit install
```

### 4. Build Project
```bash
# Development build
cargo build

# Release build with GPU support
cargo build --release --features gpu

# Build with all features
cargo build --all-features
```

### 5. Run Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# All tests with GPU features
cargo test --features gpu
```

## Development Workflow

### 1. Feature Development
1. **Create feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Follow coding standards**:
   - Use `cargo fmt` for formatting
   - Run `cargo clippy` for lints
   - Write tests for new functionality
   - Update documentation as needed

3. **Test your changes**:
   ```bash
   # Quick development test
   ./scripts/quick-test.sh
   
   # Full test suite
   ./scripts/full-test.sh
   ```

4. **Commit using conventional format**:
   ```bash
   git commit -m "feat: add homomorphic encryption for prompts"
   git commit -m "fix: resolve GPU memory leak in batch processing"
   git commit -m "docs: update API documentation for streaming"
   ```

5. **Push and create pull request**:
   ```bash
   git push origin feature/your-feature-name
   # Create PR via GitHub web interface
   ```

### 2. Code Review Process
- All PRs require approval from core maintainers
- Automated checks must pass (CI/CD pipeline)
- Security review required for cryptographic changes
- Performance review required for GPU/optimization changes

## Project Structure

```
homomorphic-llm-proxy/
├── src/                      # Rust source code
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types
│   ├── fhe.rs               # FHE operations
│   └── proxy.rs             # Proxy server implementation
├── crates/                   # Workspace crates
│   ├── fhe-core/            # Core FHE functionality
│   └── fhe-gpu/             # GPU acceleration
├── tests/                    # Test suites
│   ├── unit/                # Unit tests
│   ├── integration/         # Integration tests
│   └── e2e/                 # End-to-end tests
├── docs/                     # Documentation
│   ├── DEVELOPMENT.md       # This file
│   ├── ARCHITECTURE.md      # System architecture
│   └── workflows/           # CI/CD documentation
├── scripts/                  # Development scripts
├── benchmarks/              # Performance benchmarks
└── k8s-manifests/           # Kubernetes deployment
```

## Useful Commands

### Development Commands
```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Check documentation
cargo doc --no-deps --document-private-items

# Run benchmarks
cargo bench

# Profile performance
cargo flamegraph --bin homomorphic-llm-proxy
```

### Testing Commands
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# End-to-end tests (requires full setup)
cargo test --test e2e -- --ignored

# Test with GPU features
cargo test --features gpu

# Test coverage
cargo tarpaulin --out Html
```

### Docker Commands
```bash
# Build development image
docker build -t fhe-llm-proxy:dev .

# Run with GPU support
docker run --gpus all -p 8080:8080 fhe-llm-proxy:dev

# Run test environment
docker-compose -f docker-compose.test.yml up
```

## Configuration

### Development Configuration
Create `config.toml` for local development:
```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 1

[encryption]
poly_modulus_degree = 4096  # Smaller for development
coeff_modulus_bits = [40, 30, 40]
scale_bits = 30

[llm]
provider = "mock"
endpoint = "http://localhost:3001/mock"
timeout_seconds = 30

[privacy]
epsilon_per_query = 0.1
max_queries_per_user = 100
```

### Environment Variables
```bash
# Logging
export RUST_LOG=debug
export RUST_BACKTRACE=1

# GPU configuration
export CUDA_VISIBLE_DEVICES=0

# Development flags
export FHE_DEV_MODE=true
export FHE_MOCK_PROVIDER=true
```

## Debugging

### Rust Debugging
```bash
# Debug build with symbols
cargo build --debug

# Run with debugger
gdb target/debug/homomorphic-llm-proxy

# Debug tests
cargo test --no-run
gdb target/debug/deps/test_name
```

### GPU Debugging
```bash
# Check GPU availability
nvidia-smi

# Profile GPU usage
nvprof ./target/release/homomorphic-llm-proxy

# Debug CUDA kernels
cuda-gdb ./target/release/homomorphic-llm-proxy
```

### Performance Profiling
```bash
# CPU profiling
perf record --call-graph=dwarf ./target/release/homomorphic-llm-proxy
perf report

# Memory profiling
valgrind --tool=massif ./target/debug/homomorphic-llm-proxy

# Flamegraph generation
cargo flamegraph --bin homomorphic-llm-proxy
```

## IDE Setup

### VS Code Configuration
Install recommended extensions:
- rust-analyzer
- CodeLLDB
- GitLens
- Docker
- YAML

Workspace settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.cargo.features": ["gpu"],
    "rust-analyzer.check.command": "clippy",
    "editor.formatOnSave": true,
    "editor.rulers": [100]
}
```

### Debug Configuration (`.vscode/launch.json`)
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug FHE Proxy",
            "cargo": {
                "args": ["build", "--bin=homomorphic-llm-proxy"],
                "filter": {
                    "name": "homomorphic-llm-proxy",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug"
            }
        }
    ]
}
```

## Contributing Guidelines

### Code Style
- Follow Rust conventions and `cargo fmt` formatting
- Use meaningful variable and function names
- Add comprehensive documentation for public APIs
- Write unit tests for all new functionality
- Include integration tests for complex features

### Security Guidelines
- Never commit secrets or keys to version control
- Use secure random number generation for cryptographic operations
- Validate all inputs and handle errors gracefully
- Follow cryptographic best practices for FHE implementation
- Regular security audits for cryptographic code

### Performance Guidelines
- Profile before optimizing
- Use benchmarks to validate performance improvements
- Consider GPU memory usage and bandwidth
- Optimize for both latency and throughput
- Document performance characteristics

## Resources

* [Project Architecture](../ARCHITECTURE.md)
* [Contributing Guidelines](../CONTRIBUTING.md)
* [Security Architecture](SECURITY_ARCHITECTURE.md)
* [GitHub Actions Setup](workflows/github-actions-setup.md)
* [Rust Book](https://doc.rust-lang.org/book/)
* [CUDA Programming Guide](https://docs.nvidia.com/cuda/cuda-c-programming-guide/)
* [Microsoft SEAL Documentation](https://microsoft.github.io/SEAL/)

## Troubleshooting

### Common Issues

#### Build Failures
```bash
# Clear cargo cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

#### GPU Issues
```bash
# Check CUDA installation
nvcc --version

# Verify GPU visibility
nvidia-smi

# Test CUDA samples
cd /usr/local/cuda/samples/1_Utilities/deviceQuery
make && ./deviceQuery
```

#### Test Failures
```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Run tests serially (avoid resource conflicts)
cargo test -- --test-threads=1

# Skip GPU tests if hardware unavailable
cargo test -- --skip gpu
```

### Getting Help

- **Documentation**: Check docs/ directory first
- **Issues**: Search existing GitHub issues
- **Discussions**: Use GitHub Discussions for questions
- **Discord**: Join our developer Discord server
- **Email**: Contact fhe-support@your-org.com for urgent issues