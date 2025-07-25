---
type: "manual"
description: "Example description"
---
# Cerberus Phoenix v2.0 - User Guide & Project Overview

## Project Goal
Cerberus Phoenix v2.0 is an advanced High-Frequency Trading (HFT) system for the Solana blockchain. Its primary goal is to identify and capitalize on opportunities in new and volatile tokens (especially memecoins like those on pump.fun) with minimal cost and maximum speed. It uses a hybrid multi-model AI system to make trading decisions.

## Core Strategy: "Piranha Surf MAX"
This strategy focuses on ultra-fast detection and sniping of newly launched tokens, particularly on platforms like pump.fun.
1.  **Discovery:** Listen for new token events via Helius webhooks.
2.  **Pre-Screening:** Quickly filter tokens using rule-based logic (`SniperProfileEngine`) to identify potential "sniper" candidates based on known risk/potential factors (e.g., dev allocation <10%, no freeze function).
3.  **AI Analysis:** Send promising candidates to `cerebro-bff` for deeper AI-driven analysis using specialized LLM agents.
4.  **Execution:** Execute trades rapidly via `hft-ninja` using Jito Bundles for MEV protection and speed.
5.  **Risk Management:** Use burner wallets, predefined take-profit/stop-loss orders.
6.  **Learning:** Feed trade results and market data back into the system to refine AI models and rules (`Feedback System`).

## Key Components

### `hft-ninja` (Rust)
*   **Role:** The core engine for real-time operations.
*   **Responsibilities:**
    *   Receiving and validating Helius webhooks.
    *   Pre-processing token data and applying initial filters (`SniperProfileEngine`).
    *   Executing trades via Solana RPC (QuickNode) and Jito.
    *   Collecting real-time metrics.
    *   Interfacing with Kestra for workflow triggers.

### `cerebro-bff` (TypeScript/Rust - backend for frontend)
*   **Role:** Orchestration and AI decision-making hub.
*   **Responsibilities:**
    *   Receiving pre-screened token data from `hft-ninja`.
    *   Managing interactions with multi-model AI agents (Phi-3, Llama3, Mistral).
    *   Applying Context Engine principles (TF-IDF, shuffle, Apriori rules) to optimize LLM inputs.
    *   Generating final trading decisions.
    *   Feeding decisions back to `hft-ninja` for execution.

### `Context Engine` (Concept/Part of `cerebro-bff`)
*   **Purpose:** Mitigate "context rot" by optimizing the information sent to LLMs.
*   **Techniques:**
    *   Noise filtering.
    *   TF-IDF weighting of signals.
    *   Shuffling data order (`shuffle haystacks`).
    *   Applying rule-based guardrails (Apriori-derived rules).

### `Feedback System` (Rust + TimescaleDB)
*   **Purpose:** Collect data on AI decisions and trade outcomes to enable learning and system improvement.
*   **Data Stored:** Trade decisions, market conditions at time of decision, actual trade results (P&L), agent performance metrics.

### `Paper Trading Engine` (Rust + TimescaleDB)
*   **Purpose:** Simulate trades and strategies without financial risk.
*   **Features:** Virtual portfolio management, realistic execution simulation (slippage, fees), performance analytics.

### External Integrations
*   **Helius:** Primary data source for Solana events (webhooks), API for on-chain data.
*   **QuickNode / Alchemy:** RPC providers for submitting transactions.
*   **Jito:** MEV protection and transaction bundling.
*   **Qdrant:** Vector database for storing context and potentially rules.
*   **Kestra:** Workflow orchestration engine.
*   **Prometheus + Grafana:** Monitoring and alerting stack.

## Development Workflow
1.  Make changes to the codebase.
2.  Ensure code compiles (`cargo build`).
3.  Run tests (`cargo test`).
4.  Update `docker-compose.yml` if services or dependencies change.
5.  Build and run services using Docker Compose (`docker-compose up`).
6.  Monitor logs and dashboards (Grafana) for issues.
7.  Commit code, ensuring `.env` and secrets are not included.

## Deployment
The system is designed for deployment on cloud providers (like Oracle Cloud Free Tier) or dedicated servers for low latency. Docker Compose is used for orchestration.

## Key Concepts from Research & Strategy Docs
*   **Context Rot:** LLM performance degrades with long, unfiltered input. Mitigated by Context Engine.
*   **Memecoin Sniping:** High risk, high reward. Success rate ~10%, but potential gains are 100x-10000x.
*   **Risk Management:** Holder concentration, dev allocation, contract features (freeze, renounce) are critical filters.
*   **Tooling:** Platforms like Axiom Pro, Soul Meteor, Pump.fun, Soul Scan are used for manual research and inspiration for automated rules.