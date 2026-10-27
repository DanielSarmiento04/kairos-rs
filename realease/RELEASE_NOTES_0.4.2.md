# Release Notes v0.4.2

## 🚀 Major Highlights

**Kairos-rs v0.4.2** introduces three foundational enterprise pillars: **Control Plane Security & Web UI Authentication**, **Semantic Caching for AI / LLM Requests**, and a **Distributed Redis Backend for Rate Limiting & Response Caching**—elevating Kairos into a secure, production-grade AI gateway designed for distributed clusters.

---

## 🔐 Pillar 1: Control Plane Security & Web UI Authentication

- **Admin API Key Middleware (`middleware::admin_auth`)**:
  - Restricts access to administrative REST endpoints (`/api/*`) and WebSocket admin streams (`/ws/admin/*`).
  - Validates client credentials against configured keys via the `X-Admin-Key` header or `Bearer` token.
  - Transparently bypasses public telemetry (`/metrics`, `/health`, `/api/health`, `/api/status`) and data-plane proxy traffic.
- **Dynamic Configuration & Verification**:
  - `POST /api/config/admin-auth`: Hot-reload admin authentication settings dynamically.
  - `GET /api/admin/verify`: Lightweight handshake endpoint to validate keys.
- **Vue 3 Admin SPA Security**:
  - Integrated `AdminAuthModal.vue` with persistent browser credential storage.
  - Centralized Axios/fetch request interceptors automatically injecting authentication headers and triggering re-authentication upon `401 Unauthorized`.
  - Status pill indicator in top navigation bar indicating current administrative access level.

---

## 🧠 Pillar 2: Semantic Caching for AI / LLM Requests

- **Vector Cosine Similarity Fast-Path (`services::semantic_cache`)**:
  - Intercepts incoming prompts and compares their vector embeddings against cached query representations.
  - Returns cached LLM completions in sub-millisecond latency when cosine similarity matches or exceeds `similarity_threshold` (e.g. `0.90`).
  - Injects diagnostic headers: `X-Cache: HIT`, `X-Semantic-Cache: HIT`, and `X-Cache-Similarity: <score>`.
- **Hybrid Embedding Pipeline**:
  - Generates remote embeddings via OpenAI (`text-embedding-3-small`) or configured AI providers via `AiService`.
  - Deterministic zero-dependency local vectorizer fallback using character n-gram hashing and L2 unit-norm projection.
- **Universal Prompt Extraction**:
  - Automatically extracts user prompts from OpenAI Chat Completions (`{"messages": [{"role": "user", "content": "..."}]}`), Legacy Completions (`{"prompt": "..."}`), and generic payloads (`{"query": "..."}`, `{"input": "..."}`).
- **Semantic Cache Management**:
  - Endpoints: `GET /api/cache/semantic/stats` and `POST /api/cache/semantic/clear`.
  - Unified cache flushing (`POST /api/cache/clear`) purges both standard HTTP response cache and semantic vector cache simultaneously.

---

## ⚡ Pillar 3: Distributed Redis Backend for Rate Limiting & Response Cache

- **Atomic Sliding Window Rate Limiting (`services::redis_service`)**:
  - Cluster-wide rate limiting powered by atomic Redis Lua script evaluation (`ZREMRANGEBYSCORE`, `ZCARD`, `ZADD`, `PEXPIRE`).
  - Eliminates distributed race conditions across multiple load-balanced Kairos gateway replicas.
- **Resilient Fallback Engine**:
  - If Redis is disconnected or becomes unreachable, the rate limiter and response cache gracefully degrade to the local in-memory storage without dropping user requests.
- **Distributed Response Cache**:
  - Shared response caching across gateway instances with automatic TTL management and base64 payload serialization.
  - Prefix-based invalidation (`invalidate_prefix`) propagates instantaneously across the cluster.
- **Observability & Management**:
  - `GET /api/redis/health`: Reports connection status, round-trip latency, and configured pool metrics.
  - `POST /api/config/redis`: Hot-update Redis cluster connection URLs, timeouts, and pool sizes.
  - Frontend: Redis Dashboard Card in `Config.vue` with live ping indicators and connection configuration modal.

---

## 🧪 Testing & Verification

- **428 Automated Tests Across Workspace**:
  - 100% passing Rust integration tests covering Admin Auth (`admin_auth_tests.rs`), Semantic Caching (`semantic_cache_tests.rs`), and Redis Distributed Limiting & Fallbacks (`redis_distributed_tests.rs`).
  - 121 Vitest frontend component and service unit tests.
  - All public API documentation examples verified via `cargo test --doc`.

---

## 📦 Automated Crates.io Publishing

- Added automated GitHub Actions publishing workflow (`.github/workflows/crates-publish.yml`).
- Publishes workspace crates in topological dependency order (`kairos-rs` -> `kairos-client` -> `kairos-gateway` -> `kairos-cli`) on git tag push `v*.*.*`.
