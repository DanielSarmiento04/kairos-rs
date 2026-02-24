# Web UI Dashboard

Kairos Gateway includes a modern, production-ready web-based administration interface built with **Leptos 0.8**. The UI provides real-time monitoring, metrics visualization, and system health insights.

## Features

- 📊 **Real-time Dashboard**: Live metrics with auto-refresh every 30 seconds.
- 📈 **Metrics Visualization**: Comprehensive performance analytics and charts.
- 🏥 **Health Monitoring**: Detailed health checks and system status.
- ⚡ **High Performance**: Server-side rendering (SSR) with client-side hydration.
- 🛡️ **Type-safe**: Full type safety from backend to frontend using shared Rust models.

## Quick Start

### Prerequisites

1. **Kairos Gateway must be running** on port 5900.
2. **Rust and Cargo** must be installed.
3. **cargo-leptos** must be installed (one-time setup):

```bash
cargo install cargo-leptos
```

### Running the UI

Start the UI development server with hot reload:

```bash
cd crates/kairos-ui
cargo leptos serve
```

The UI will be available at: **http://localhost:3000**

### Building for Production

To build optimized production artifacts:

```bash
cd crates/kairos-ui
cargo leptos build --release
./target/release/kairos-ui
```

## Dashboard Overview

The main dashboard (`/`) provides a comprehensive view of your gateway's performance:

- **Real-time Metrics**: Total request counts, success rates, and average response times.
- **Error Breakdown**: Visual breakdown of 4xx/5xx errors, timeouts, and connection failures.
- **Response Time Distribution**: Histogram showing performance buckets (e.g., <50ms, 50-200ms, >200ms).
- **Circuit Breakers**: Live status of all circuit breakers across your backends.
- **Data Transfer**: Total bytes sent and received.

## Health Monitoring

The Health page (`/health`) provides detailed diagnostics:

- **General Status**: Overall gateway health, version, and uptime.
- **Readiness Probe**: Kubernetes-compatible readiness status.
- **Liveness Probe**: Kubernetes-compatible liveness status.
- **Backend Connectivity**: Status of individual backend services.

## Configuration

The UI connects to the Kairos Gateway API. You can configure the connection using environment variables:

```bash
# Gateway base URL (default: http://localhost:5900)
KAIROS_GATEWAY_URL=http://localhost:5900

# Leptos server address (default: 127.0.0.1:3000)
LEPTOS_SITE_ADDR=127.0.0.1:3000
```

## Architecture

The Kairos UI is built using Leptos 0.8's Server-Side Rendering (SSR) capabilities. This provides:

- Fast initial page loads.
- SEO-friendly content.
- Progressive enhancement.
- Client-side hydration for rich interactivity.

API communication is handled seamlessly through Leptos server functions, ensuring type safety and efficient data fetching from the gateway's Prometheus metrics and management endpoints.
