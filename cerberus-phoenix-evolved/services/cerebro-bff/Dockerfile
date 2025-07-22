# 🧠 Cerebro BFF - Multi-stage Dockerfile
# Optimized for production with minimal attack surface

# 🏗️ Build stage
FROM rustlang/rust:nightly-slim as builder

# 📦 Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 📁 Set working directory
WORKDIR /app

# 📁 Copy source code and dependencies
COPY services/cerebro-bff .

# 🏗️ Build the application
RUN cargo build --release

# 🚀 Runtime stage - Distroless for security
FROM gcr.io/distroless/cc-debian12:latest

# 📁 Create app directory
WORKDIR /app

# 👤 Copy binary from builder stage
COPY --from=builder /app/target/release/cerebro-bff /usr/local/bin/cerebro-bff

# 🌐 Expose port
EXPOSE 3000

# 🏥 Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/cerebro-bff", "--health-check"] || exit 1

# 🚀 Run the application
ENTRYPOINT ["/usr/local/bin/cerebro-bff"]
