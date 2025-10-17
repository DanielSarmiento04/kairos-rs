# Kairos-rs 🔄🤖

A production-ready HTTP gateway and reverse proxy built with Rust, featuring a **modern web-based admin interface** and pioneering AI-powered routing capabilities. **The future of intelligent API gateways!**

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/kairos-rs?color=blue)](https://crates.io/crates/kairos-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Security audit](https://img.shields.io/badge/security-audit-success.svg)](https://github.com/DanielSarmiento04/kairos-rs/security)


## What it actually does (right now)

Kairos-rs is a production-ready HTTP gateway with modern web UI that:
- ✅ Routes incoming HTTP requests to backend services based on path patterns
- ✅ Supports dynamic path parameters (e.g., `/users/{id}` → `/users/123`)
- ✅ **JWT Authentication** - Validate bearer tokens with configurable claims and required fields
- ✅ **Advanced rate limiting** - Per-route limits with multiple algorithms (fixed window, sliding window, token bucket)
- ✅ **Circuit breaker pattern** - Automatic failure detection and recovery
- ✅ **Security features** - CORS policies, request size limits, security headers
- ✅ **Observability** - Prometheus metrics, structured logging, health checks
- ✅ **Configuration hot-reload** - Update routes without service restart
- ✅ **Web Admin UI** - Modern Leptos-based interface with real-time dashboard and metrics
- ✅ **Modular Architecture** - Workspace with separate crates for gateway, UI, CLI, and client

**Current status:** Ready for production use with comprehensive security, reliability features, and web-based management interface.

## Quick Start

### 1. Install & Run Gateway
```bash
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs
cargo run --bin kairos-gateway
```

Gateway starts on `http://localhost:5900`

### 2. Start Web Admin UI (Optional)
```bash
# Install cargo-leptos (one-time)
cargo install cargo-leptos

# Start UI in development mode
cd crates/kairos-ui
cargo leptos serve
```

Admin UI available at `http://localhost:3000`

### 3. Configure Routes
Create a `config.json` file:

```json
{
  "version": 1,
  "jwt_secret": "your-secret-key-here",
  "rate_limit": {
    "algorithm": "token_bucket",
    "requests_per_second": 100,
    "burst_size": 10
  },
  "routers": [
    {
      "host": "https://http.cat",
      "port": 443,
      "external_path": "/cats/{id}",
      "internal_path": "/{id}",
      "methods": ["GET"],
      "auth_required": false
    },
    {
      "host": "https://api.example.com",
      "port": 443,
      "external_path": "/api/secure/{id}",
      "internal_path": "/v1/data/{id}",
      "methods": ["GET", "POST"],
      "auth_required": true
    }
  ]
}
```

### 3. Test It
```bash
# Public endpoint (no auth required)
curl http://localhost:5900/cats/200

# Secure endpoint (requires JWT)
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     http://localhost:5900/api/secure/123

# Or use the Admin UI at http://localhost:3000 to:
# - View real-time metrics and dashboard
# - Monitor health status
# - See request/response statistics
# - Track circuit breaker status
```

## How Dynamic Routing Works

```rust
// Example route configuration
{
  "external_path": "/api/users/{user_id}/posts/{post_id}",
  "internal_path": "/users/{user_id}/posts/{post_id}"
}

// Request: GET /api/users/123/posts/456
// Forwards to: GET /users/123/posts/456
```

The route matcher:
- Uses regex for pattern matching
- Supports unlimited path parameters
- Falls back to static routes for better performance

## Current Architecture

```
                           ┌─────────────────┐
                           │   Web Admin UI  │
                           │  (Leptos 0.8)   │
                           │  Port: 3000     │
                           └────────┬────────┘
                                    │ HTTP
┌─────────────┐    HTTP    ┌───────▼─────────┐    HTTP    ┌─────────────┐
│   Client    │ ────────▶  │  Kairos Gateway │ ────────▶  │  Backend    │
│             │            │   Port: 5900    │            │  Service    │
└─────────────┘            └─────────┬───────┘            └─────────────┘
                                     │
                              ┌──────┴───────┐
                              │ Config.json  │
                              │   Routes     │ 
                              │     JWT      │
                              │ Rate Limits  │
                              └──────────────┘
```

**Architecture Components:**

### Workspace Structure:
```
kairos-rs/
├── crates/
│   ├── kairos-rs/        # Core library (models, routing logic)
│   ├── kairos-gateway/   # Gateway binary (HTTP server)
│   ├── kairos-ui/        # Web admin interface (Leptos SSR)
│   ├── kairos-cli/       # Command-line interface
│   └── kairos-client/    # Rust client library
```

### Core Features:
- Route matcher with compiled regex patterns
- JWT authentication with configurable claims validation
- Advanced rate limiting (token bucket, sliding window, fixed window)
- Circuit breaker for automatic failure handling
- HTTP client with connection pooling (reqwest)
- Prometheus metrics endpoint
- Structured logging and health checks
- **Real-time web dashboard** with metrics visualization
- **Server-side rendering** with client hydration for optimal performance

## Configuration

### Full Configuration Example
```json
{
  "version": 1,
  "jwt_secret": "your-256-bit-secret-key-here",
  "rate_limit": {
    "algorithm": "token_bucket",
    "requests_per_second": 100,
    "burst_size": 50
  },
  "routers": [
    {
      "host": "http://backend-service.com",
      "port": 8080,
      "external_path": "/api/v1/users/{id}",
      "internal_path": "/users/{id}",
      "methods": ["GET", "PUT", "DELETE"],
      "auth_required": true
    },
    {
      "host": "https://public-api.com",
      "port": 443,
      "external_path": "/public/status",
      "internal_path": "/health",
      "methods": ["GET"],
      "auth_required": false
    }
  ]
}
```

### Rate Limiting Algorithms
- **fixed_window**: Fixed time windows with request quotas
- **sliding_window**: Smooth rate limiting with sliding time windows  
- **token_bucket**: Burst-friendly with token replenishment

### JWT Configuration
- Supports standard JWT claims validation
- Configurable required claims and audience
- Bearer token extraction from Authorization header

### Environment Variables
```bash
KAIROS_HOST=0.0.0.0          # Server bind address
KAIROS_PORT=5900             # Server port
KAIROS_CONFIG_PATH=./config.json  # Config file path  
RUST_LOG=info                # Log level
```

## Testing

```bash
# Run all tests in workspace (85+ tests total)
cargo test --workspace

# Run gateway tests only
cargo test --package kairos-rs

# Run UI tests only
cd crates/kairos-ui && cargo test

# Performance tests
cargo test performance_tests -- --nocapture

# Integration tests
cargo test --test integration_tests

# JWT authentication tests
cargo test --test jwt_integration_test

# Rate limiting tests  
cargo test rate_limit

# Circuit breaker tests
cargo test circuit_breaker
```

Current test coverage: **85+ comprehensive tests** covering:
- Route matching and performance
- JWT authentication and authorization  
- Rate limiting algorithms
- Circuit breaker functionality
- Configuration validation
- Error handling scenarios
- Documentation examples
- UI model validation

## What's Next? (Roadmap)

This project has completed Phase 1 (Gateway Core) and is now working on Phase 2 (UI Completion & Advanced Routing)! Here's what's planned:

**Current focus (Phase 2 - v0.2.7):**
- [ ] Route management backend endpoints (CRUD operations via API)
- [ ] Configuration editor UI (JWT, rate limiting, CORS settings)
- [ ] WebSocket real-time updates (replace polling with live connections)
- [ ] Form validation (client and server-side)
- [ ] Load balancing strategies (round-robin, weighted, health-based)
- [ ] Request transformation (header manipulation, path rewriting)
- [ ] Retry logic with exponential backoff

**Recently completed (Phase 1 + UI Foundation):**
- ✅ JWT authentication with configurable claims
- ✅ Advanced rate limiting with multiple algorithms  
- ✅ Circuit breaker pattern implementation
- ✅ Prometheus metrics endpoint
- ✅ Configuration validation and hot-reload
- ✅ Comprehensive security features
- ✅ **Web Admin UI** with real-time dashboard
- ✅ **Workspace architecture** with modular crates
- ✅ **Health monitoring** pages

**Future phases:**
- **Phase 3:** Response caching, historical metrics, distributed tracing
- **Phase 4:** AI-powered routing, LLM integration, smart load balancing
- **Phase 5:** Enterprise features (auth, RBAC, multi-gateway support)

**🚀 Exciting AI Vision:**
Kairos-rs is pioneering the integration of AI/LLM capabilities into API gateway functionality:
- **Smart Routing**: AI-driven backend selection based on request content analysis
- **LLM Integration**: Intelligent request/response transformation using language models
- **Predictive Load Balancing**: ML-based routing decisions for optimal performance
- **Behavioral Security**: AI-powered threat detection and dynamic rate limiting

This makes Kairos-rs potentially the **first AI-powered open source API gateway** - combining traditional gateway reliability with cutting-edge AI capabilities.

See [ROADMAP.md](ROADMAP.md) for the complete development plan.

## Contributing

This project needs help! Areas where contributions would be especially valuable:

**Code:**
- **Backend endpoints** - Implement route management API (high priority)
- **UI components** - Build configuration editor forms
- Load balancing implementations
- Request transformation middleware
- Performance optimizations
- Retry logic with backoff strategies
- Historical metrics with charts
- Documentation improvements

**Other:**
- Testing JWT authentication scenarios
- Performance benchmarking under load
- Docker and Kubernetes deployment guides
- Real-world usage examples and case studies
- UI/UX improvements and accessibility

### Development Setup
```bash
# Clone and setup
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs

# Install dev tools
rustup component add rustfmt clippy
cargo install cargo-leptos  # For UI development

# Run checks
cargo fmt --check --all
cargo clippy --all-targets --all-features
cargo test --workspace

# Start gateway
cargo run --bin kairos-gateway

# Start UI (separate terminal)
cd crates/kairos-ui
cargo leptos serve
```

**Current code style:** Uses default rustfmt with workspace configuration. The codebase follows Rust best practices.

## Performance

Current benchmarks on M1 MacBook Pro:

**Gateway Performance:**
- **Static routes**: ~450k ops/sec (hash map lookup)
- **Dynamic routes**: ~200k ops/sec (regex matching)
- **JWT validation**: ~50k tokens/sec  
- **Rate limiting**: ~100k checks/sec
- **Request latency**: P99 < 2ms for route matching
- **Throughput**: Handles 10k+ concurrent requests reliably

**Resource Usage:**
- **Memory**: ~25MB under load (15MB idle)
- **CPU**: Efficient async runtime with tokio

**UI Performance:**
- **WASM bundle size**: ~500KB (gzipped)
- **Initial page load**: <1s (server-side rendering)
- **Time to interactive**: <2s (client hydration)
- **Dashboard refresh**: Real-time updates every 30s

*Note: These are micro-benchmarks and controlled load tests. Real-world performance depends on backend service latency and network conditions.*

## Known Issues

**Gateway:**
- [ ] Load balancing strategies not yet implemented
- [ ] Request transformation features in development  
- [ ] WebSocket proxying support planned for future release
- [ ] Distributed tracing integration pending

**Admin UI:**
- [ ] Route management needs backend API endpoints (CRUD operations)
- [ ] Configuration editor in development
- [ ] Historical metrics not yet implemented
- [ ] Authentication/authorization for UI pending

**Recently fixed:**
- ✅ Configuration validation improved
- ✅ Error messages enhanced and structured
- ✅ Comprehensive test coverage added
- ✅ JWT authentication fully implemented
- ✅ Advanced rate limiting algorithms added
- ✅ Web UI foundation completed with real-time dashboard

## License

MIT License - see [LICENSE](LICENSE) file.

## AI Development Transparency

This project utilizes AI assistance for:

- **Code Review**: AI tools help identify potential issues and suggest improvements
- **Documentation**: AI assists in maintaining clear and comprehensive documentation  
- **Testing**: AI helps generate test cases and identify edge cases
- **Optimization**: AI provides suggestions for performance improvements

**Human Oversight**: All AI suggestions are reviewed, tested, and validated by human developers before implementation. The core architecture decisions and project direction remain under human control.

We believe in transparency about development tools and methods used in open source projects.

See the prompt guidance in [llm.txt](./llm.txt) for more details.

## Acknowledgments

Built with these excellent Rust crates:

**Core Gateway:**
- [Actix Web](https://actix.rs/) - Web framework
- [Reqwest](https://docs.rs/reqwest/) - HTTP client  
- [Serde](https://serde.rs/) - Serialization
- [Tokio](https://tokio.rs/) - Async runtime
- [jsonwebtoken](https://docs.rs/jsonwebtoken/) - JWT validation
- [prometheus](https://docs.rs/prometheus/) - Metrics collection

**Admin UI:**
- [Leptos](https://leptos.dev/) - Reactive web framework
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) - Development tooling
- [WASM-bindgen](https://rustwasm.github.io/wasm-bindgen/) - WASM/JS interop

---

**Status**: Production ready with comprehensive security, reliability features, and modern web admin interface  
**Version**: 0.2.6  
**Maintainer**: [@DanielSarmiento04](https://github.com/DanielSarmiento04)  
**Community**: Issues and PRs welcome!

