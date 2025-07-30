---
type: "manual"
description: "Example description"
---
# Project Rules & Guidelines for AI Assistance (e.g., Augment Code, Claude Code)

## General Architecture
1.  The project consists of multiple Rust services (e.g., `hft-ninja`, `cerebro-bff`) and external services (Qdrant, Kestra, Grafana).
2.  Core business logic (AI, trading decisions, signal processing) resides in `hft-ninja` and `cerebro-bff`.
3.  External API interactions (Helius, QuickNode, Jito) are handled in `hft-ninja`.
4.  Data persistence uses TimescaleDB (feedback, paper trading) and Qdrant (context, vectors).
5.  Communication between services is primarily via HTTP APIs or message queues (to be implemented).

## Rust Coding Standards
1.  Follow Rust idioms: Use `async`/`.await` for I/O, prefer `Result<T, E>` for error handling.
2.  Use `tokio` for async runtime.
3.  Prefer strong typing and `serde` for serialization/deserialization.
4.  Minimize `clone()` calls, especially for large data structures. Use references or `Arc` where appropriate.
5.  Handle errors gracefully. Avoid `unwrap()` in production paths.
6.  Write unit tests for core logic modules (e.g., `sniper_profile`, `metrics`).

## HFT/Near-Real-Time Considerations
1.  Latency is critical. Avoid blocking operations on the main async task.
2.  Use efficient data structures (e.g., `HashMap`, `VecDeque`).
3.  Offload heavy CPU work to `tokio::task::spawn_blocking` if necessary.
4.  Minimize allocations in hot paths.
5.  Profile code regularly for performance bottlenecks.

## Webhook Handling (`hft-ninja`)
1.  Webhook handlers (`webhook_handler.rs`) must be fast and non-blocking.
2.  Perform only essential validation and parsing in the handler itself.
3.  Use rate-limiting to protect the service.
4.  Forward processed events to downstream systems (e.g., Kestra, `cerebro-bff`, internal queues) asynchronously.

## AI Integration (`cerebro-bff`)
1.  AI agents should receive well-structured, pre-processed data (e.g., from `SniperProfileEngine`).
2.  Context sent to LLMs must be optimized (filtered, weighted using TF-IDF, potentially shuffled to mitigate 'context rot').
3.  Implement timeouts for LLM calls.
4.  Log AI inputs, decisions, and latencies for the feedback loop.

## Data Flow & Orchestration
1.  Use Kestra for complex workflow orchestration triggered by events.
2.  Prefer asynchronous communication patterns (e.g., queues, pub/sub) over synchronous request/response where possible to improve resilience and decouple services.

## Configuration & Secrets
1.  Use environment variables for configuration.
2.  Never commit secrets (API keys, private keys) to the repository. Use Vault or `.env` files locally (ensure `.env` is in `.gitignore`).
3.  Validate environment variables at application startup.

## Testing & Deployment
1.  Write tests for critical components.
2.  Use Docker for containerization. Ensure `Dockerfile` is optimized (multi-stage builds).
3.  Define services and dependencies clearly in `docker-compose.yml`.
4.  Monitor application health and performance using Prometheus metrics and Grafana dashboards.

## Security
1.  Validate all inputs, especially from external sources (webhooks, APIs).
2.  Sanitize data before processing or storing.
3.  Protect sensitive endpoints with authentication (e.g., Bearer tokens for webhooks).