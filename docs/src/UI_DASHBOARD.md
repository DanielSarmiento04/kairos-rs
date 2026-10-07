# Web UI Dashboard (`frontend/kairos-ui`)

Kairos Gateway includes a decoupled, high-performance web administration interface built with **Vue 3**, **TypeScript**, and **Vite 8**. The SPA connects directly to the gateway's REST API and real-time WebSocket metrics streams (`/ws/admin/metrics`).

---

## Features

- 📊 **Real-Time Telemetry Dashboard**: Live WebSocket streaming with zero deep-reactivity overhead (`shallowRef` log streams), pause/resume, and custom zero-dependency SVG sparklines.
- ⚡ **In-Memory Cache Monitoring**: Live observability for the response cache (`ResponseCache`) showing hits, misses, hit ratio %, stored entries, and instant cache purging.
- 🗺️ **Comprehensive Route Management**: Full CRUD operations for routes with multi-backend load balancing (5 strategies: Round Robin, Least Connections, Random, Weighted, IP Hash), retry policies, AI routing policies, and pre-validation.
- 🔄 **Visual Transformation Editor**: Visual editor for request and response transformations (header injection/stripping, regex path rewriting, query parameter modification, status code mapping).
- ⚙️ **Interactive Configuration Management**: Forms for AI providers (OpenAI, Anthropic, Cohere, etc.), JWT authentication, rate limiting, CORS, server settings, Prometheus metrics, and one-click hot-reload (`/api/config/reload`).
- 🧪 **Interactive Playground**: Send test prompts and HTTP queries with customized headers, body, and Bearer token injection while inspecting latency, headers, and response payloads.
- 🔑 **Client & Token Manager**: Generate signed JWT tokens with customizable claims (`sub`, `iss`, `aud`), and decode existing tokens.
- 📈 **Time-Series Observability**: Query historical metrics (`/api/metrics/history`) with configurable aggregation intervals and inspect raw Prometheus metrics (`/metrics`).

---

## Quick Start

### Prerequisites

1. **Kairos Gateway running** on `http://localhost:5900`.
2. **Node.js** (v20+ recommended) and **npm**.

### Development Mode

Start the Vite development server with instant HMR and proxying to the gateway:

```bash
# Using the root helper script:
./dev.sh ui

# Or directly from the frontend directory:
cd frontend/kairos-ui
npm install
npm run dev
```

The UI will be available at: **http://localhost:5173**

Vite's development server automatically proxies `/api`, `/health`, `/metrics`, and `/ws` to the Rust gateway running on port `5900`.

---

## Building for Production

To create an optimized, production-ready static bundle:

```bash
cd frontend/kairos-ui
npm run build
```

This compiles optimized assets to `frontend/kairos-ui/dist` in ~250ms with minified JavaScript and CSS chunks.

### Type-Checking & Unit Testing

```bash
cd frontend/kairos-ui

# Run Vue TypeScript type checker
npm run type-check

# Run Vitest unit tests
npm run test:unit -- --run
```

---

## Architecture & Views

### 1. Dashboard (`/`)
- **Live KPIs**: Total requests, active connections, success rate, and uptime.
- **Sparklines**: Rolling time-series SVG charts for request rate and concurrency.
- **In-Memory Response Cache**: Displays current hit ratio, hits, misses, stored entries, and target latency.
- **High-Throughput Telemetry Log Stream**: Powered by `shallowRef` to prevent Vue deep-proxy overhead when ingesting hundreds of logs per second.

### 2. Routes (`/routes`)
- **Route Matrix**: Visual table displaying external paths, protocols (`http`, `websocket`, `ftp`, `dns`), upstream backends, and active features (AI policies, transformations).
- **Interactive Route Editor**:
  - **General**: External path (with `{param}` support), internal destination, protocol, allowed methods, and JWT requirement.
  - **Backends & Load Balancer**: Multi-target weighted endpoints and load-balancing strategy selection.
  - **Retry Policy**: Exponential backoff multiplier, initial/max backoff, and retryable status codes.
  - **AI Routing Policy**: Content analysis, dynamic model selection, and fallback targets.
  - **Transformations**: Request header manipulation, regex path rewriting, and response status code translations.
  - **Live Pre-Validation**: Tests route configuration against `/api/routes/validate` before saving.

### 3. Configuration (`/config`)
- Modular cards for editing gateway subsystems: AI models, JWT secrets, Rate Limiting algorithms, Server threads, CORS rules, and Response Cache.
- **Live Hot Reload**: Triggers `POST /api/config/reload` to apply updates without service downtime.
- **Raw JSON Display**: Syntax-highlighted view of `config.json` with one-click copy.

### 4. Interactive Playground (`/playground`)
- Real-time route and prompt tester.
- Custom HTTP method, path, request headers, Bearer authentication token, and JSON payload.
- Detailed response inspection: Status code, duration in milliseconds, response headers, and formatted body.

### 5. Client Token Manager (`/clients`)
- Issue new JWT tokens with configurable secret, expiry, issuer, and audience.
- Paste and inspect existing JWT tokens to view decoded header, claims, and validity status.

### 6. Metrics & Observability (`/metrics`)
- Historical time-series query interface for `requests_total`, `requests_error`, `active_connections`, and `response_time_avg`.
- Raw Prometheus exposition viewer for `/metrics`.
