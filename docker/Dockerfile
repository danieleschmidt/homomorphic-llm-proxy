# Multi-stage build for optimized production image
FROM rust:1.79-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN echo "// dummy" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --bin fhe-proxy
RUN rm -rf src

# Copy real source code
COPY src ./src
COPY tests ./tests
COPY config.toml ./

# Build the actual application
RUN cargo build --release --bin fhe-proxy

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r fhe && useradd -r -g fhe -s /bin/false fhe

# Set working directory
WORKDIR /app

# Copy binary and configuration
COPY --from=builder /app/target/release/fhe-proxy /app/
COPY --from=builder /app/config.toml /app/
COPY docker/entrypoint.sh /app/

# Create directories with proper permissions
RUN mkdir -p /var/log/fhe-proxy /etc/ssl/fhe-proxy && \
    chown -R fhe:fhe /app /var/log/fhe-proxy && \
    chmod +x /app/entrypoint.sh

# Switch to non-root user
USER fhe

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Expose port
EXPOSE 8080 9090

# Set entry point
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["/app/fhe-proxy"]