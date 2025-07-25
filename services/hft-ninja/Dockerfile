# 🐺 Projekt Cerberus Phoenix v2.0 - HFT-Ninja Dockerfile
# Multi-stage build dla optymalizacji rozmiaru obrazu

# 🏗️ Build stage
FROM rust:latest as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

# 🚀 Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hft-ninja /usr/local/bin/hft-ninja

EXPOSE 8080
CMD ["hft-ninja"]
