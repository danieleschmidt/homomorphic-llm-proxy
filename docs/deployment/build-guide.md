# Build Guide

## Overview

This guide covers building, containerizing, and deploying the FHE LLM Proxy across different environments.

## Prerequisites

### System Requirements
- **OS**: Linux (Ubuntu 20.04+), macOS (10.15+), or Windows (WSL2)
- **CPU**: x86_64 with AES-NI and AVX2 support
- **Memory**: 8GB+ RAM (16GB+ recommended)
- **GPU**: NVIDIA GPU with CUDA 12.0+ (optional but recommended)
- **Storage**: 20GB+ free space

### Software Dependencies
- **Rust**: 1.75+ with CUDA support
- **Docker**: 20.10+ with GPU support
- **Git**: For source code management
- **NVIDIA Docker**: For GPU containerization

## Build Methods

### Local Build

#### Standard Build
```bash
# Build release version
cargo build --release --features gpu

# Build debug version
cargo build --features gpu

# Build without GPU support
cargo build --release
```

#### Build with Just
```bash
# Install just command runner
cargo install just

# Build release
just build

# Build debug
just build-debug

# Build with all features
just ci-check
```

#### Feature Flags
```toml
# Available features in Cargo.toml
[features]
default = ["cuda"]
gpu = ["cuda", "gpu-acceleration"]
cuda = ["cudarc", "cuda-sys"]
opencl = ["opencl-sys"]
test-utils = ["proptest", "test-harness"]
profiling = ["pprof", "flame"]
```

### Container Build

#### Production Container
```bash
# Build production container
docker build -t fhe-llm-proxy:latest .

# Build with specific tag
docker build -t fhe-llm-proxy:v0.2.0 .

# Build multi-architecture
docker buildx build --platform linux/amd64,linux/arm64 -t fhe-llm-proxy:latest .
```

#### Development Container
```bash
# Build test container
docker build -f Dockerfile.test -t fhe-llm-proxy:test .

# Build with dev dependencies
docker build --target builder -t fhe-llm-proxy:dev .
```

#### Container Optimization
```dockerfile
# Multi-stage build optimization
FROM nvidia/cuda:12.0-devel-ubuntu22.04 as builder
# ... build stage ...

FROM nvidia/cuda:12.0-runtime-ubuntu22.04 as runtime
# ... runtime stage ...

# Security best practices
RUN groupadd -r fheproxy && useradd -r -g fheproxy fheproxy
USER fheproxy
```

### Cross-Platform Build

#### Linux (x86_64)
```bash
# Native build
cargo build --release --target x86_64-unknown-linux-gnu

# With GPU support
cargo build --release --features gpu --target x86_64-unknown-linux-gnu
```

#### Linux (ARM64)
```bash
# Cross-compile for ARM64
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

#### macOS
```bash
# Native macOS build (CPU only)
cargo build --release --target x86_64-apple-darwin

# Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

## Build Configuration

### Cargo Configuration
```toml
# .cargo/config.toml
[build]
target-dir = "target"
incremental = true

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "target-cpu=native"]

[target.'cfg(target_env = "cuda")']
rustflags = ["-C", "link-args=-Wl,-rpath,$ORIGIN"]
```

### Environment Variables
```bash
# Build optimization
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=abort

# CUDA configuration
export CUDA_ROOT=/usr/local/cuda
export CUDA_LIB_PATH=/usr/local/cuda/lib64
export LD_LIBRARY_PATH=$CUDA_LIB_PATH:$LD_LIBRARY_PATH

# Performance tuning
export RUSTFLAGS="-C target-cpu=native -C target-feature=+aes,+avx2"
```

### Build Profiles

#### Release Profile
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

#### Development Profile
```toml
[profile.dev]
opt-level = 0
debug = true
panic = "unwind"
incremental = true
```

#### Benchmark Profile
```toml
[profile.bench]
opt-level = 3
lto = true
codegen-units = 1
debug = true
```

## Build Verification

### Binary Verification
```bash
# Check binary size
ls -lh target/release/homomorphic-llm-proxy

# Verify dependencies
ldd target/release/homomorphic-llm-proxy

# Check CUDA support
./target/release/homomorphic-llm-proxy --version --features

# Security analysis
cargo audit
```

### Container Verification
```bash
# Inspect container
docker inspect fhe-llm-proxy:latest

# Check container size
docker images fhe-llm-proxy

# Scan for vulnerabilities
docker scan fhe-llm-proxy:latest

# Test container
docker run --rm --gpus all fhe-llm-proxy:latest --version
```

### Performance Testing
```bash
# Benchmark build
cargo bench --features gpu

# Profile binary
perf record target/release/homomorphic-llm-proxy --bench-mode
perf report

# Memory analysis
valgrind --tool=massif target/release/homomorphic-llm-proxy
```

## Deployment Builds

### Cloud Native Build
```bash
# Build for Kubernetes
docker build --target production -t fhe-llm-proxy:k8s .

# Build with health checks
docker build --build-arg ENABLE_HEALTH_CHECK=true -t fhe-llm-proxy:health .

# Build with monitoring
docker build --build-arg ENABLE_METRICS=true -t fhe-llm-proxy:metrics .
```

### Distribution Build
```bash
# Create distribution package
cargo build --release --all-features
tar czf fhe-llm-proxy-v0.2.0-linux-x86_64.tar.gz \
  target/release/homomorphic-llm-proxy \
  config/ \
  docs/ \
  scripts/

# Create checksums
sha256sum fhe-llm-proxy-v0.2.0-linux-x86_64.tar.gz > checksums.txt
```

### CI/CD Build
```yaml
# GitHub Actions example
name: Build and Test
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
          
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Build
        run: cargo build --release --all-features
        
      - name: Test
        run: cargo test --all-features
        
      - name: Build container
        run: docker build -t fhe-llm-proxy:${{ github.sha }} .
```

## Build Optimization

### Compilation Speed
```bash
# Parallel compilation
export CARGO_BUILD_JOBS=$(nproc)

# Use sccache for caching
export RUSTC_WRAPPER=sccache

# Incremental builds
export CARGO_INCREMENTAL=1
```

### Binary Size Optimization
```toml
[profile.release]
opt-level = "z"  # Optimize for size
lto = true
codegen-units = 1
panic = "abort"
strip = "symbols"
```

### Performance Optimization
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
target-cpu = "native"
```

## Troubleshooting

### Common Build Issues

#### CUDA Not Found
```bash
# Fix CUDA path
export CUDA_ROOT=/usr/local/cuda-12.0
export PATH=$CUDA_ROOT/bin:$PATH
export LD_LIBRARY_PATH=$CUDA_ROOT/lib64:$LD_LIBRARY_PATH
```

#### Linking Errors
```bash
# Install missing libraries
sudo apt-get install libssl-dev pkg-config build-essential

# Fix library paths
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig
```

#### Out of Memory
```bash
# Reduce parallel jobs
export CARGO_BUILD_JOBS=2

# Use less memory-intensive profile
cargo build --profile dev
```

#### Container Build Failures
```bash
# Clear Docker cache
docker system prune -af

# Build with no cache
docker build --no-cache -t fhe-llm-proxy:latest .

# Check disk space
df -h
```

### Debug Information

#### Verbose Build
```bash
# Verbose cargo output
cargo build --release --verbose

# Show compiler commands
cargo build --release -vv
```

#### Environment Debugging
```bash
# Show environment
cargo version --verbose
rustc --print cfg
nvidia-smi

# Show build configuration
cargo metadata --format-version 1 | jq '.packages[0].features'
```

## Security Considerations

### Build Security
- Use official base images
- Verify checksums and signatures
- Scan for vulnerabilities
- Use minimal runtime images
- Run as non-root user

### Supply Chain Security
```bash
# Audit dependencies
cargo audit

# Check for known vulnerabilities
cargo deny check

# Generate SBOM
cargo cyclonedx
```

### Container Security
```dockerfile
# Security best practices
FROM scratch  # Minimal base
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /etc/passwd /etc/passwd
USER nobody
```

## Monitoring and Metrics

### Build Metrics
- Build time tracking
- Binary size monitoring
- Test coverage tracking
- Performance regression detection

### Deployment Metrics
- Container startup time
- Resource utilization
- Error rates
- Performance benchmarks