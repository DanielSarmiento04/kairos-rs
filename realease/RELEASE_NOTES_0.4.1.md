# Release Notes v0.4.1

## 🚀 Major Highlights

**Kairos-rs v0.4.1** delivers a major leap in AI orchestration, real-time observability, response caching, HTTP compression, and the Vue 3 + TypeScript administration UI—backed by **420 automated tests** across the Rust workspace and frontend SPA.

---

## 🧠 AI Orchestration, SSE Streaming & Failover (`rig-core` 0.44.0)

- **Upgraded to `rig-core 0.44.0`**: Modernized LLM provider client initialization (`ProviderClient::from_env()`, `Nothing` for local/Ollama endpoints) across OpenAI, Anthropic, Cohere, Perplexity, Mistral, Groq, and xAI.
- **Multi-Provider Automated Failover Chains**:
  - Added `FallbackProviderConfig` at both the global (`ai.fallback_providers`) and per-route (`ai_policy.fallback_providers`) levels.
  - Automatic sequential fallback across backup providers and models upon rate limits (`429`), `5xx` errors, or network timeouts.
- **Real-Time SSE & Token Streaming Passthrough**:
  - Zero-buffering chunked streaming via `futures_util::TryStream` when upstream returns `text/event-stream`, `application/x-ndjson`, or when `ai_policy.streaming: true` is enabled on a route.
  - Automatically injects downstream anti-buffering headers (`X-Accel-Buffering: no`, `Cache-Control: no-cache, no-transform`, `X-Streaming: true`) and bypasses the in-memory response cache.

---

## ⚡ Per-Route Response Caching, Regex Invalidation & HTTP Compression

- **Granular Per-Route Cache Policy (`CacheConfig`)**:
  - Configure `enabled`, `ttl_seconds`, `vary_headers`, `cache_control_respect` (`no-store`/`private`), `skip_if_Set_Cookie`, and `max_body_bytes` per route.
  - Emits RFC-compliant `Age`, `X-Cache` (`HIT`/`MISS`), and `X-Cache-TTL` headers.
- **Targeted Cache Invalidation (`POST /api/cache/invalidate`)**:
  - Purge cache entries by key prefix or regular expression (`is_regex: true`), in addition to full cache clear (`POST /api/cache/clear`) and live stats (`GET /api/cache/stats`).
- **Global HTTP Compression Middleware (`CompressionConfig`)**:
  - Optional `compression` block in `config.json` supporting `gzip`, `br` (Brotli), and `deflate` with configurable `min_bytes` threshold and compression `level`.

---

## 📈 Latency Percentiles (p50 / p95 / p99) & Time-Series Observability

- **Histogram Percentile Engine (`services::percentile`)**:
  - Computes p50, p95, and p99 latencies via linear CDF interpolation over Prometheus-style cumulative histogram buckets (`MetricValue::Histogram`).
- **REST & WebSocket Percentile Streaming**:
  - New `GET /api/metrics/percentiles` endpoint for historical percentile queries across configurable intervals (`1m`, `5m`, `15m`, `1h`, `1d`).
  - Enhanced `/ws/metrics` WebSocket protocol with `subscribe_percentiles: true` and real-time `Percentiles` events.
- **Automatic Metrics Recorder**:
  - Background task recording `requests_total`, `requests_error`, `active_connections`, `response_time_avg`, `cache_hits_total`, `cache_misses_total`, and `http_request_duration_ms` into `MetricsStore` every 5 seconds.

---

## 🖥️ Web Admin UI (`frontend/kairos-ui`)

- **Interactive Time-Series Charts (`uPlot`)**:
  - Reusable `TimeSeriesChart.vue` component with dark theme, responsive `ResizeObserver`, and live ring-buffer updates.
  - **Dashboard (`/`)**: 4 live charts (Requests/sec, Active Connections, Latency p50/p95/p99, and Cache Hit/Miss Rate).
  - **Metrics View (`/metrics`)**: Time-range presets (`15m`, `1h`, `6h`, `24h`), auto-selected aggregation intervals, route filter, and one-click **CSV Export**.
- **Cache Management UI**:
  - `CachePolicyEditor.vue` inside the Route Editor drawer for per-route cache policies and `Vary` header tags.
  - `CacheInvalidationCard.vue` on the Dashboard for instant prefix/regex cache invalidation and full purge.
- **AI Streaming & Failover Controls**:
  - Configure global and per-route fallback provider chains in `ConfigView.vue` and `RoutesView.vue`.
  - Real-time SSE token streaming reader (`ReadableStream` + `TextDecoder`) in `PlaygroundView.vue`.

---

## 🧪 Testing & Quality Assurance (420 Automated Tests)

- **Rust Workspace (`299` tests)**:
  - **198 Unit & Integration Tests**: Covering AI routing & failover (`ai_service_tests`), response caching & invalidation (`cache_integration_tests`), HTTP compression (`compression_tests`), latency percentiles (`percentile_endpoint_tests`), transformations (`transform_integration_tests`), JWT auth (`jwt_integration_test`), WebSocket/FTP/DNS protocols, circuit breakers, and rate limiting.
  - **101 Documentation Tests (`cargo test --doc`)**: All public API examples verified and compiled.
- **Frontend SPA (`121` tests)**:
  - **121 Vitest Unit & Component Tests** across 14 test suites (`TimeSeriesChart`, `CachePolicyEditor`, `CacheInvalidationCard`, `DashboardView`, `MetricsView`, `RoutesView`, `gateway` store, `prometheus` parser, `csv` exporter, etc.).

---

## 🛠 CI/CD & Documentation

- **Docker Multi-Platform Build**: Upgraded `Dockerfile` builder image to `rust:1.99.0-slim-bookworm` (`linux/amd64` and `linux/arm64`) to support `rig-core 0.44.0` MSRV (`>= 1.95.0`) while maintaining `glibc` compatibility with `debian:bookworm-slim`.
- **Documentation**: Added [SSL Offloading & TLS Termination Guide](../docs/src/SSL_OFFLOADING_GUIDE.md) and updated Architecture, Configuration, AI Routing, WebSocket Metrics, UI Dashboard, and Development guides.
