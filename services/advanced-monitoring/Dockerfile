# 📊 Advanced Monitoring Dockerfile
# Multi-stage build for monitoring and observability

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app advanced-monitoring

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/advanced-monitoring /app/advanced-monitoring

# Change ownership
RUN chown -R advanced-monitoring:advanced-monitoring /app

# Switch to app user
USER advanced-monitoring

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8700/health || exit 1

# Expose ports
EXPOSE 8700 9700

# Start application
CMD ["./advanced-monitoring"]
