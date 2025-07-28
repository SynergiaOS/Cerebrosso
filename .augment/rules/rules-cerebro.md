---
type: "always_apply"
---

# Cerberus Phoenix v2.0 - Project Rules & Guidelines

## 1. ARCHITECTURE & DESIGN PRINCIPLES

1.1. Core Tenets:
- Resilience First: System must degrade gracefully, not fail catastrophically.
- Low Latency: Every millisecond matters, especially in HFT components (`hft-ninja`).
- Cost Efficiency: Minimize API costs through smart caching, batching, and webhooks.
- Adaptive Intelligence: AI decisions must be contextual, explainable, and continuously learned from.

1.2. Service Boundaries:
- `hft-ninja`: Rust. Real-time operations (webhooks, sniping, arbitrage execution, Jito bundles). Ultra-low latency.
- `cerebro-bff`: TypeScript/Rust. AI orchestration, Context Engine, decision-making hub.
- External Integrations: Helius (primary data/webhooks), QuickNode/Alchemy (RPC fallback), Jito (MEV), Qdrant (context).

## 2. RUST CODING STANDARDS (`hft-ninja`)

2.1. Async & Performance:
- Use `tokio` for all asynchronous operations.
- Prefer `async/await`. Avoid blocking the executor.
- Use efficient data structures (`HashMap`, `VecDeque`). Minimize `clone()`.

2.2. Error Handling:
- Use `anyhow::Result<T>` or specific error enums.
- Avoid `unwrap()` and `expect()` in non-test, non-`main()` code.
- Implement graceful degradation where possible.

2.3. Resilience Patterns (see `market_data` module):
- Always wrap external API clients in a `ResilientClient`.
- Implement `Primary -> Fallback -> Mock` cascading logic.
- Use exponential backoff with jitter for retries.
- Employ circuit breaker patterns for failing dependencies.

2.4. Testing:
- Write unit tests for core logic modules.
- Use `tokio::test` for async tests.

## 3. CONTEXT ENGINE RULES (`cerebro-bff`)

3.1. Context Optimization (based on "context rot" research):
- Filter Noise: Remove irrelevant data from context payloads before sending to LLM.
- Dynamic Signal Weighting: Use TF-IDF to weight important signals.
- Shuffle Haystacks: Randomize the order of context elements to mitigate positional bias degradation.
- Minimize Context Length: Prioritize quality and relevance over quantity.

3.2. AI Agent Interaction:
- Define clear input/output schemas for each AI agent.
- Implement timeouts for all LLM calls.
- Log AI inputs, decisions, confidence, and latency for the feedback loop.

3.3. Rule-Based Guardrails (Apriori-derived):
- Maintain a set of configurable, high-confidence rules in Qdrant.
- These rules should act as sanity checks or hard filters for AI decisions.
- Dynamically update rule weights/confidence based on feedback loop data.

## 4. API & INFRASTRUCTURE OPTIMIZATION

4.1. API Consumption:
- Use webhooks over polling whenever possible (e.g., Helius).
- Implement request batching for RPC calls.
- Use Redis for caching API responses (respect TTL based on data volatility).
- Implement intelligent RPC provider rotation (Helius -> QuickNode -> Alchemy -> Public RPC).

4.2. Configuration & Secrets:
- Use environment variables for all configuration.
- Never commit secrets (API keys, private keys) to the repository.
- Use Vault or `.env` files locally (ensure `.env` is in `.gitignore`).

## 5. TRADING STRATEGIES & RISK

5.1. Strategy Implementation:
- Each strategy (Sniping, Arbitrage, Sandwich) must be a separate, well-defined module.
- Strategies must define clear entry/exit criteria and risk parameters.
- Parameters (e.g.,