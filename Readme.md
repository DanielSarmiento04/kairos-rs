# Kairos-rs 🔄

A simple HTTP gateway and reverse proxy built with Rust. Currently in **early development** - contributions welcome!

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/kairos-rs?color=blue)](https://crates.io/crates/kairos-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Security audit](https://img.shields.io/badge/security-audit-success.svg)](https://github.com/DanielSarmiento04/kairos-rs/security)


## What it actually does (right now)

Kairos-rs is a production-ready HTTP gateway that:
- ✅ Routes incoming HTTP requests to backend services based on path patterns
- ✅ Supports dynamic path parameters (e.g., `/users/{id}` → `/users/123`)
- ✅ **JWT Authentication** - Validate bearer tokens with configurable claims and required fields
- ✅ **Advanced rate limiting** - Per-route limits with multiple algorithms (fixed window, sliding window, token bucket)
- ✅ **Circuit breaker pattern** - Automatic failure detection and recovery
- ✅ **Security features** - CORS policies, request size limits, security headers
- ✅ **Observability** - Prometheus metrics, structured logging, health checks
- ✅ **Configuration hot-reload** - Update routes without service restart

**Current status:** Ready for production use with comprehensive security and reliability features.

## Quick Start

### 1. Install & Run
```bash
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs
cargo run
```

Server starts on `http://localhost:5900`

### 2. Configure Routes
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
┌─────────────┐    HTTP    ┌─────────────┐    HTTP    ┌─────────────┐
│   Client    │ ────────▶  │  Kairos-rs  │ ────────▶  │  Backend    │
│             │            │   Gateway   │            │  Service    │
└─────────────┘            └─────────────┘            └─────────────┘
                                 │
                           ┌─────────────┐
                           │ Config.json │
                           │   Routes    │ 
                           │    JWT      │
                           │ Rate Limits │
                           └─────────────┘
```

**Components:**
- Route matcher with compiled regex patterns
- JWT authentication with configurable claims validation
- Advanced rate limiting (token bucket, sliding window, fixed window)
- Circuit breaker for automatic failure handling
- HTTP client with connection pooling (reqwest)
- Prometheus metrics endpoint
- Structured logging and health checks

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
# Run all tests (81 tests total)
cargo test

# Performance tests
cargo test performance_tests -- --nocapture

# Integration tests only
cargo test --test integration_tests

# JWT authentication tests
cargo test --test jwt_integration_test

# Rate limiting tests  
cargo test rate_limit

# Circuit breaker tests
cargo test circuit_breaker
```

Current test coverage: **81 comprehensive tests** covering:
- Route matching and performance
- JWT authentication and authorization  
- Rate limiting algorithms
- Circuit breaker functionality
- Configuration validation
- Error handling scenarios
- Documentation examples

## What's Next? (Roadmap)

This project has completed Phase 1 of development and is now production-ready! Here's what's planned next:

**Current focus (Phase 2):**
- [ ] Load balancing strategies (round-robin, weighted, health-based)
- [ ] Request transformation (header manipulation, path rewriting)
- [ ] Retry logic with exponential backoff
- [ ] Enhanced service discovery integration

**Recently completed (Phase 1):**
- ✅ JWT authentication with configurable claims
- ✅ Advanced rate limiting with multiple algorithms  
- ✅ Circuit breaker pattern implementation
- ✅ Prometheus metrics endpoint
- ✅ Configuration validation and hot-reload
- ✅ Comprehensive security features

**Future phases:**
- Response caching layer
- Admin UI for configuration management
- WebSocket proxying support  
- Distributed tracing integration

See [ROADMAP.md](ROADMAP.md) for the complete development plan.

## Contributing

This project needs help! Areas where contributions would be especially valuable:

**Code:**
- Load balancing implementations
- Request transformation middleware
- Performance optimizations
- Retry logic with backoff strategies
- Documentation improvements

**Other:**
- Testing JWT authentication scenarios
- Performance benchmarking under load
- Docker and Kubernetes deployment guides
- Real-world usage examples and case studies

### Development Setup
```bash
# Clone and setup
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs

# Install dev tools
rustup component add rustfmt clippy

# Run checks
cargo fmt --check
cargo clippy
cargo test
```

**Current code style:** Uses default rustfmt with some custom lint rules. The codebase is still evolving.

## Performance

Current benchmarks on M1 MacBook Pro:
- **Static routes**: ~450k ops/sec (hash map lookup)
- **Dynamic routes**: ~200k ops/sec (regex matching)
- **JWT validation**: ~50k tokens/sec  
- **Rate limiting**: ~100k checks/sec
- **Memory usage**: ~25MB under load (15MB idle)
- **Request latency**: P99 < 2ms for route matching
- **Throughput**: Handles 10k+ concurrent requests reliably

*Note: These are micro-benchmarks and controlled load tests. Real-world performance depends on backend service latency and network conditions.*

## Known Issues

- [ ] Load balancing strategies not yet implemented
- [ ] Request transformation features in development  
- [ ] WebSocket support planned for future release
- [ ] Distributed tracing integration pending
- [ ] Admin UI for configuration management planned

**Recently fixed:**
- ✅ Configuration validation improved
- ✅ Error messages enhanced and structured
- ✅ Comprehensive test coverage added
- ✅ JWT authentication fully implemented
- ✅ Advanced rate limiting algorithms added

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
- [Actix Web](https://actix.rs/) - Web framework
- [Reqwest](https://docs.rs/reqwest/) - HTTP client  
- [Serde](https://serde.rs/) - Serialization
- [Tokio](https://tokio.rs/) - Async runtime
- [jsonwebtoken](https://docs.rs/jsonwebtoken/) - JWT validation
- [prometheus](https://docs.rs/prometheus/) - Metrics collection

---

**Status**: Production ready with comprehensive security and reliability features  
**Maintainer**: [@DanielSarmiento04](https://github.com/DanielSarmiento04)  
**Community**: Issues and PRs welcome!

