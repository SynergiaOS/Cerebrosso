# 🔐 Security Hardening Dockerfile
# Multi-stage build for enterprise security

# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    softhsm2 \
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
    softhsm2 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app security-hardening

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/security-hardening /app/security-hardening

# Initialize SoftHSM
RUN softhsm2-util --init-token --slot 0 --label "cerberus" --pin 1234 --so-pin 1234

# Change ownership
RUN chown -R security-hardening:security-hardening /app

# Switch to app user
USER security-hardening

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8600/health || exit 1

# Expose ports
EXPOSE 8600 9600

# Start application
CMD ["./security-hardening"]
