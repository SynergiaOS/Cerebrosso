# 🐝 SwarmCoordinator Dockerfile
# Multi-stage build for optimal size and security

# Build stage
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 swarm

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (cached layer)
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
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 swarm

# Create directories
RUN mkdir -p /app/logs /app/data && \
    chown -R swarm:swarm /app

# Copy binary from builder
COPY --from=builder /app/target/release/swarm-coordinator /app/swarm-coordinator

# Set ownership
RUN chown swarm:swarm /app/swarm-coordinator

# Switch to app user
USER swarm

# Set working directory
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8090/health || exit 1

# Expose ports
EXPOSE 8090 8091

# Set environment variables
ENV RUST_LOG=info
ENV SWARM_HOST=0.0.0.0
ENV SWARM_PORT=8090

# Run the application
CMD ["./swarm-coordinator"]
