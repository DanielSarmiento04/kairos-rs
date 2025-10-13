# Kairos UI - Admin Interface for Kairos Gateway 🎨

A modern, production-ready web-based admin interface for the Kairos API Gateway, built with **Leptos 0.8** and following Rust best practices.

[![Leptos 0.8](https://img.shields.io/badge/Leptos-0.8-purple)](https://leptos.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

### 🎯 Core Functionality
- **Real-time Dashboard** - Live metrics with auto-refresh every 30 seconds
- **Route Management** - View, create, edit, and delete gateway routes (coming soon)
- **Metrics Visualization** - Comprehensive performance analytics and charts
- **Configuration Editor** - Manage JWT, rate limiting, and security settings (coming soon)
- **Health Monitoring** - Detailed health checks and system status

### 🚀 Technical Highlights
- **Leptos 0.8 SSR** - Server-side rendering with hydration for optimal performance
- **Type-safe API** - Full type safety from backend to frontend using shared models
- **Modern UI/UX** - Professional design with responsive layout and smooth animations
- **Real-time Updates** - Automatic data refresh for live monitoring
- **Error Boundaries** - Graceful error handling with user-friendly messages
- **Production Ready** - Follows Rust and Leptos best practices for reliability

## 🚀 Quick Start

### Prerequisites

1. **Kairos Gateway must be running** on port 5900:
   ```bash
   # Terminal 1: Start the gateway
   cd crates/kairos-gateway
   cargo run
   ```

2. **Install cargo-leptos** (one-time setup):
   ```bash
   cargo install cargo-leptos
   ```

### Development Mode

Start the UI development server with hot reload:

```bash
# Terminal 2: Start UI dev server
cd crates/kairos-ui
cargo leptos serve
```

The UI will be available at: **http://localhost:3000**

### Building for Production

Build optimized production artifacts:

```bash
# Build with cargo-leptos (recommended)
cargo leptos build --release

# Run the production server
./target/release/kairos-ui
```

## 📁 Project Structure

```
crates/kairos-ui/
├── src/
│   ├── app.rs              # Main app with routing and layout
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Server entry point (SSR)
│   ├── models/             # Data models (Router, Settings, Metrics, Health)
│   │   ├── router.rs
│   │   ├── settings.rs
│   │   ├── metrics.rs
│   │   └── health.rs
│   ├── server_functions/   # API integration with gateway
│   │   └── api.rs
│   ├── components/         # Reusable UI components
│   │   ├── navbar.rs
│   │   ├── sidebar.rs
│   │   ├── metric_card.rs
│   │   ├── status_badge.rs
│   │   ├── loading.rs
│   │   └── error_boundary.rs
│   └── pages/              # Route-specific pages
│       ├── dashboard.rs    # Main dashboard with live metrics
│       ├── routes_page.rs  # Route management (placeholder)
│       ├── metrics_page.rs # Advanced metrics (placeholder)
│       ├── config_page.rs  # Configuration editor (placeholder)
│       └── health_page.rs  # Health monitoring
├── style/
│   └── main.scss           # Comprehensive SCSS styling
├── assets/
│   └── favicon.ico
├── Cargo.toml              # Leptos configuration
└── README.md
```

## 🎨 Pages Overview

### Dashboard (/)
- **Real-time Metrics**: Request counts, success rates, response times
- **Error Breakdown**: 4xx/5xx errors, timeouts, connection failures
- **Response Time Distribution**: Histogram showing performance buckets
- **Circuit Breakers**: Status of all circuit breakers
- **Data Transfer**: Request/response byte counts
- **Auto-refresh**: Updates every 30 seconds

### Routes (/routes)
_Coming Soon_ - Manage gateway routes with full CRUD operations

### Metrics (/metrics)
_Coming Soon_ - Advanced metrics with historical charts and per-route breakdown

### Configuration (/config)
_Coming Soon_ - Edit JWT settings, rate limiting, CORS, and security policies

### Health (/health)
- General health status with version and uptime
- Kubernetes readiness probe status
- Kubernetes liveness probe status
- Detailed diagnostics

## 🔧 Configuration

### Environment Variables

```bash
# Gateway base URL (default: http://localhost:5900)
KAIROS_GATEWAY_URL=http://localhost:5900

# Leptos server address (default: 127.0.0.1:3000)
LEPTOS_SITE_ADDR=127.0.0.1:3000
```

### Leptos Configuration

The `Cargo.toml` includes comprehensive Leptos metadata configuration:

```toml
[package.metadata.leptos]
output-name = "kairos-ui"
site-root = "target/site"
site-pkg-dir = "pkg"
style-file = "style/main.scss"
assets-dir = "assets"
site-addr = "127.0.0.1:3000"
reload-port = 3001
env = "DEV"
```

## 🏗️ Architecture

### Server-Side Rendering (SSR)

The UI uses Leptos 0.8's SSR capabilities for:
- Fast initial page load
- SEO-friendly content
- Progressive enhancement
- Client-side hydration for interactivity

### Server Functions

API communication is handled through Leptos server functions:

```rust
#[server(GetMetrics, "/api")]
pub async fn get_metrics() -> Result<MetricsData, ServerFnError> {
    // Fetches and parses Prometheus metrics from gateway
}
```

### Reactive State Management

Leptos signals and resources provide reactive state:

```rust
let metrics_resource = Resource::new(
    move || refresh_trigger.get(),
    |_| async move { get_metrics().await }
);
```

## 🎯 Development Guidelines

### Following Rust Best Practices

This project adheres to the guidelines in `/kairos-rs/llm.txt` and `/crates/kairos-ui/llm.txt`:

- **Type Safety**: Full type safety from backend to frontend
- **Error Handling**: Comprehensive error boundaries and user feedback
- **Documentation**: Inline documentation for all public APIs
- **Testing**: Unit and integration tests (to be expanded)
- **Performance**: Optimized builds with LTO and code splitting

### Code Style

```bash
# Format code
cargo fmt --all

# Lint with Clippy
cargo clippy --all-targets --all-features

# Run tests
cargo test
```

## 🚀 Roadmap

### Phase 1: Foundation ✅ COMPLETED
- ✅ Project structure and Leptos setup
- ✅ Shared models mirroring backend
- ✅ Server functions for API integration
- ✅ Reusable component library
- ✅ Dashboard with real-time metrics
- ✅ Health monitoring page
- ✅ Comprehensive SCSS styling
- ✅ Routing and navigation

### Phase 2: Full CRUD Operations (Next)
- [ ] Routes management with full CRUD
- [ ] Route validation and testing
- [ ] Configuration editor for JWT/rate limiting
- [ ] Hot-reload trigger support
- [ ] Form validation and error handling

### Phase 3: Advanced Features
- [ ] Historical metrics with charts
- [ ] Per-route performance breakdown
- [ ] WebSocket support for real-time updates
- [ ] Dark mode support
- [ ] Export metrics data
- [ ] Custom dashboards

### Phase 4: Backend Integration
- [ ] Implement configuration endpoints in gateway backend
- [ ] Add route management endpoints
- [ ] WebSocket server for live updates
- [ ] Authentication and authorization
- [ ] Audit logging

## 🐛 Troubleshooting

### "Failed to connect to gateway" error

**Solution:**
1. Ensure Kairos Gateway is running: `cargo run --bin kairos-gateway`
2. Verify it's listening on port 5900
3. Check CORS settings if running on different domains
4. Check browser console for detailed errors

### WASM build fails

**Solution:**
1. Never build WASM with `--features ssr` (server features are incompatible)
2. Use `cargo leptos build` which handles features correctly
3. Ensure `wasm-bindgen-cli` is installed: `cargo install wasm-bindgen-cli`

### Styles not loading

**Solution:**
1. Check that `style/main.scss` exists
2. Verify `cargo-leptos` is processing SCSS correctly
3. Clear browser cache and rebuild: `cargo leptos build --release`

### Hot reload not working

**Solution:**
1. Ensure reload port 3001 is not blocked
2. Check firewall settings
3. Try restarting the dev server: `cargo leptos serve`

## 📚 Resources

- [Leptos Documentation](https://leptos.dev/)
- [Leptos Book](https://book.leptos.dev/)
- [Kairos Gateway Documentation](../../Readme.md)
- [Actix Web](https://actix.rs/)
- [Rust Documentation](https://doc.rust-lang.org/)

## 🤝 Contributing

Contributions are welcome! Areas where help would be appreciated:

- **Route Management**: Implement full CRUD for routes
- **Configuration Editor**: Build the config management UI
- **Charts & Visualizations**: Add historical metrics charts
- **Testing**: Expand test coverage
- **Documentation**: Improve examples and guides
- **Accessibility**: Ensure WCAG compliance

### Development Setup

```bash
# Clone and setup
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs/crates/kairos-ui

# Install dependencies
cargo install cargo-leptos

# Run dev server
cargo leptos serve

# Run tests
cargo test

# Format and lint
cargo fmt --check
cargo clippy
```

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

## 🙏 Acknowledgments

Built with these excellent tools:

- **[Leptos](https://leptos.dev/)** - Reactive web framework
- **[Actix Web](https://actix.rs/)** - Web server framework
- **[Serde](https://serde.rs/)** - Serialization framework
- **[Reqwest](https://docs.rs/reqwest/)** - HTTP client

---

**Status**: Production-ready foundation with working dashboard and health monitoring  
**Maintainer**: [@DanielSarmiento04](https://github.com/DanielSarmiento04)  
**Issues and PRs**: Welcome!

*Built following Rust best practices as outlined in [llm.txt](llm.txt)*