# Task: Migrate `kairos-rs` UI from Leptos to Vue 3 + TypeScript

## 1. Objective
Migrate the presentation layer of the `kairos-rs` TensorZero LLM Application Platform/Gateway from Leptos (Rust/WASM) to a decoupled Vue 3 + TypeScript Single Page Application (SPA). 
The goal is to eliminate WASM-related UI maintenance complexity, improve frontend developer experience, and maintain a high-performance dashboard for LLM telemetry and routing configuration.

## 2. Architecture Shift
*   **Current State:** Decoupled modern frontend and pure Rust API server.
*   **Target State:** 
    *   **Backend:** Pure Rust API server (Actix Web) serving JSON via REST and real-time WebSocket metrics (`/ws/admin/metrics`).
    *   **Frontend:** Vue 3 (Composition API) + Vite 8 + TypeScript + Pinia + Vue Router + Vitest.

## 3. Execution Phases

### Phase 1: API Boundary Decoupling (Rust Backend)
- [x] **Analyze Leptos Server Functions:** Scanned and removed legacy Leptos server macros from the codebase.
- [x] **Create REST API Contracts:** Implemented and exposed RESTful endpoints for routes (`/api/routes`), configuration (`/api/config/*`), and metrics (`/api/metrics/*`, `/metrics`).
- [x] **Implement CORS & Proxy:** Configured Vite proxy in `vite.config.ts` for `/api`, `/health`, `/metrics`, and `/ws`.

### Phase 2: Frontend Scaffolding
- [x] Initialize a new Vite + Vue + TS project in `/frontend/kairos-ui`.
- [x] Configure `vite.config.ts` to proxy requests to Rust backend (port 5900).
- [x] Set up Vue Router with base layout (Sidebar navigation, Header, Breadcrumbs).
- [x] Set up Pinia for global state (`useGatewayStore` with live WebSocket streaming).

### Phase 3: Core Feature Migration (TensorZero Gateway UI)
- [x] **Dashboard / Telemetry View:**
    - Created strict interfaces for LLM and Gateway metrics (requests, latency, active connections, error rate, success rate, tokens, cost).
    - Implemented live WebSocket telemetry streaming from `/ws/admin/metrics`.
    - Implemented a high-performance data table for gateway logs.
    - *Agent Constraint Fulfilled:* Used `shallowRef` instead of `ref` for arrays of telemetry data to ensure peak rendering performance for large log datasets.
    - Built lightweight zero-dependency SVG sparkline charts for instant visual metrics.
- [x] **Routing & Configuration View:**
    - Implemented full multi-backend load balancing configuration (Host, Port, Weight, Health Check Path).
    - Supported 5 load balancing strategies (`round_robin`, `least_connections`, `random`, `weighted`, `ip_hash`).
    - Configured protocols (`http`, `websocket`, `ftp`, `dns`).
    - Configured exponential backoff retry policies and AI-powered routing policies.
    - Integrated live server validation via `POST /api/routes/validate` before persistence.
- [x] **Configuration Management:**
    - Dedicated interactive configuration forms for AI Providers (`/api/config/ai`), JWT Authentication (`/api/config/jwt`), Rate Limiting (`/api/config/rate-limit`), CORS (`/api/config/cors`), Server settings (`/api/config/server`), and Metrics (`/api/config/metrics`).
    - Raw JSON configuration viewer with highlight.js syntax highlighting and one-click copy.
    - One-click backend hot-reload button (`/api/config/reload`).
- [x] **API Key / Client Management:**
    - Built `Clients.vue` for generating signed JWT tokens, configuring claim sets (`sub`, `iss`, `aud`), and real-time token decoding/inspection.
- [x] **Playground / Request Testing:**
    - Built `Playground.vue` for interactive testing of gateway routes, prompt forwarding, header customization, Bearer token injection, and response inspection (status, latency, headers, body).
- [x] **Historical Metrics & Observability:**
    - Built `Metrics.vue` for querying historical time-series data (`/api/metrics/history`), time range windowing, and Prometheus exposition scraper (`/metrics`).

### Phase 4: Styling & Cleanup
- [x] Implemented modern styling with scoped CSS and accessible color schemes.
- [x] Removed all Leptos metadata and wasm release profiles from root `Cargo.toml`.
- [x] Cleaned up obsolete template files and fixed all type-check and Vitest issues.
- [x] Validated 100% test pass rate across unit tests and production builds.

### Phase 5: Backend Optimization & Response Caching Layer
- [x] **In-Memory HTTP Response Cache (`ResponseCache`)**:
    - Built thread-safe, lock-optimized in-memory caching in `crates/kairos-rs/src/services/cache.rs` with TTL expiration, `AHashMap`, capacity management, and automatic evictions.
    - Integrated with `RouteHandler` (`crates/kairos-rs/src/services/http.rs`) for sub-millisecond responses on idempotent requests (`GET`/`HEAD`).
    - Added `X-Cache: HIT` and `X-Cache: MISS` telemetry headers, as well as `X-Cache-TTL`.
    - Added smart cache invalidation on successful mutating requests (`POST`, `PUT`, `DELETE`, `PATCH`).
    - Added administrative cache observability & purge endpoints (`GET /api/cache/stats`, `POST /api/cache/clear`) in `crates/kairos-rs/src/routes/management.rs`.
    - Initialized in `kairos-gateway` `main.rs` (5,000 entries, 60s default TTL).
- [x] **Route Matcher Optimization**:
    - Added `match_cache` (`Arc<RwLock<AHashMap<String, (Router, String)>>>`) to `RouteMatcher` (`crates/kairos-rs/src/utils/route_matcher.rs`).
    - Accelerated dynamic route matching with O(1) cached lookup prior to regex capture loops, plus `clear_cache` support.
- [x] **Upstream Client Connection Pool Tuning**:
    - Enabled `.tcp_nodelay(true)` and `.tcp_keepalive(Some(Duration::from_secs(60)))` to minimize packet latency.
    - Expanded pool size: `.pool_max_idle_per_host(128)` and `.pool_idle_timeout(Duration::from_secs(60))`.
- [x] **Complete Test Verification**:
    - 101/101 Rust workspace tests and doctests passing.
    - 12/12 Vitest frontend tests passing.
    - 0 TypeScript errors and clean production build.

## 4. Technical Constraints & Rules for the Agent
1.  **Strict Typing:** Strictly 0 `any` in TypeScript. All API responses have corresponding TypeScript interfaces mapped to the Rust backend structs.
2.  **Composition API:** Strictly used Vue 3 `<script setup>` syntax across all components and views.
3.  **State Management:** Used Vue's built-in reactivity (`ref`, `computed`, `shallowRef`) for local component state, and Pinia (`useGatewayStore`) for global gateway WebSocket metrics.
4.  **Performance:** Kept the bundle size lightweight. Built custom SVG sparkline charts without heavy third-party charting libraries.