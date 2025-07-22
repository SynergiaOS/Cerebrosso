# 🐺 Projekt Cerberus Phoenix v2.0 - Cerebro-BFF Dockerfile
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
    curl \
    && rm -rf /var/lib/apt/lists/*

# 👤 Create non-root user
RUN groupadd -r cerebro && useradd -r -g cerebro cerebro

# 📁 Create app directory
WORKDIR /app

# 📋 Copy binary from builder stage
COPY --from=builder /app/target/release/cerebro-bff /usr/local/bin/cerebro-bff

# 🔐 Set ownership and permissions
RUN chown cerebro:cerebro /usr/local/bin/cerebro-bff && \
    chmod +x /usr/local/bin/cerebro-bff

# 👤 Switch to non-root user
USER cerebro

# 🌐 Expose port
EXPOSE 8080

# 🏥 Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 🚀 Run the application
CMD ["cerebro-bff"]
