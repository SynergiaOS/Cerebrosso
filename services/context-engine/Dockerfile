# 🧠 Context Engine (CEM) Dockerfile
# Multi-stage build for optimized production image

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
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app context-engine

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/context-engine /app/context-engine

# Copy configuration
COPY .env.example /app/.env.example

# Change ownership
RUN chown -R context-engine:context-engine /app

# Switch to app user
USER context-engine

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8200/health || exit 1

# Expose port
EXPOSE 8200

# Start application
CMD ["./context-engine"]
