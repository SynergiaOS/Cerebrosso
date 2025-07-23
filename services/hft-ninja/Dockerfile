# 🐺 Projekt Cerberus Phoenix v2.0 - HFT-Ninja Dockerfile
# Multi-stage build dla optymalizacji rozmiaru obrazu

# 🏗️ Build stage
FROM rust:latest as builder

# 📦 Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 📁 Set working directory
WORKDIR /app

# 📋 Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# 🚀 Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 📦 Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# 📁 Copy source code
COPY src ./src

# 🏗️ Build the application
RUN cargo build --release

# 🚀 Runtime stage
FROM debian:bookworm-slim

# 📦 Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 👤 Create non-root user
RUN groupadd -r hft-ninja && useradd -r -g hft-ninja hft-ninja

# 📁 Create app directory
WORKDIR /app

# 📋 Copy binary from builder stage
COPY --from=builder /app/target/release/hft-ninja /usr/local/bin/hft-ninja

# 🔐 Set ownership and permissions
RUN chown hft-ninja:hft-ninja /usr/local/bin/hft-ninja && \
    chmod +x /usr/local/bin/hft-ninja

# 👤 Switch to non-root user
USER hft-ninja

# 🌐 Expose port
EXPOSE 8080

# 📦 Install curl for health checks
USER root
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
USER hft-ninja

# 🔧 Environment variables for debugging
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# 🏥 Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 🚀 Run the application with logging
CMD ["sh", "-c", "echo 'Starting HFT-Ninja...' && /usr/local/bin/hft-ninja"]
