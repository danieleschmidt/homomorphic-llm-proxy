# Multi-stage build for GPU-accelerated FHE proxy
FROM nvidia/cuda:12.0-devel-ubuntu22.04 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --features gpu
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY crates ./crates

# Build the application
RUN cargo build --release --features gpu

# Runtime stage
FROM nvidia/cuda:12.0-runtime-ubuntu22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r fheproxy \
    && useradd -r -g fheproxy fheproxy

# Copy binary from builder stage
COPY --from=builder /app/target/release/homomorphic-llm-proxy /usr/local/bin/

# Create necessary directories
RUN mkdir -p /app/config /app/data \
    && chown -R fheproxy:fheproxy /app

# Switch to non-root user
USER fheproxy

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set default command
CMD ["homomorphic-llm-proxy", "--config", "/app/config/config.toml"]