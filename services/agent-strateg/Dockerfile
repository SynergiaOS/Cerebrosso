# 👑 Agent-Strateg Dockerfile
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
RUN useradd -m -u 1001 strateg

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
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 strateg

# Create directories
RUN mkdir -p /app/logs /app/data && \
    chown -R strateg:strateg /app

# Copy binary from builder
COPY --from=builder /app/target/release/agent-strateg /app/agent-strateg

# Set ownership
RUN chown strateg:strateg /app/agent-strateg

# Switch to app user
USER strateg

# Set working directory
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8100/health || exit 1

# Expose port
EXPOSE 8100

# Set environment variables
ENV RUST_LOG=info
ENV STRATEG_HOST=0.0.0.0
ENV STRATEG_PORT=8100

# Run the application
CMD ["./agent-strateg"]
