# Kairos-rs 🔄

A simple HTTP gateway and reverse proxy built with Rust. Currently in **early development** - contributions welcome!

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/kairos-rs?color=blue)](https://crates.io/crates/kairos-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Security audit](https://img.shields.io/badge/security-audit-success.svg)](https://github.com/DanielSarmiento04/kairos-rs/security)


## What it actually does (right now)

Kairos-rs is a basic HTTP gateway that:
- ✅ Routes incoming HTTP requests to backend services based on path patterns
- ✅ Supports dynamic path parameters (e.g., `/users/{id}` → `/users/123`)
- ✅ Has basic rate limiting (100 req/s with burst support)
- ✅ Includes request logging and security headers
- ✅ Provides health check endpoints

**What it doesn't do yet:** Most enterprise features you'd expect from a production gateway (see [roadmap](ROADMAP.md) for planned features).

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
  "routers": [
    {
      "host": "http://httpbin.org",
      "port": 80,
      "external_path": "/test/{path}",
      "internal_path": "/anything/{path}",
      "methods": ["GET", "POST"]
    }
  ]
}
```

### 3. Test It
```bash
# This request to kairos-rs:
curl http://localhost:5900/test/hello

# Gets forwarded to:
# http://httpbin.org:80/anything/hello
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
                           └─────────────┘
```

**Components:**
- Route matcher with compiled regex patterns
- HTTP client with connection pooling (reqwest)
- Basic middleware pipeline (rate limiting, logging)
- JSON configuration with hot-reload support

## Configuration

```json
{
  "version": 1,
  "routers": [
    {
      "host": "http://backend-service.com",
      "port": 8080,
      "external_path": "/api/v1/users/{id}",
      "internal_path": "/users/{id}",
      "methods": ["GET", "PUT", "DELETE"]
    }
  ]
}
```

### Environment Variables
```bash
KAIROS_HOST=0.0.0.0          # Server bind address
KAIROS_PORT=5900             # Server port  
RUST_LOG=info                # Log level
```

## Testing

```bash
# Run all tests
cargo test

# Performance tests
cargo test performance_tests -- --nocapture

# Integration tests only
cargo test --test route_matcher_tests
```

Current test coverage: **13 tests covering route matching, performance, and error handling**.

## What's Next? (Roadmap)

This is an early-stage project. Here's what's planned:

**Short term (next month):**
- [ ] JWT authentication
- [ ] Prometheus metrics endpoint  
- [ ] Circuit breaker pattern
- [ ] Load balancing strategies

**Medium term:**
- [ ] Response caching
- [ ] Request transformation
- [ ] Service discovery integration
- [ ] Admin UI

See [ROADMAP.md](ROADMAP.md) for the full development plan.

## Contributing

This project needs help! Areas where contributions would be especially valuable:

**Code:**
- Performance optimizations
- Additional middleware
- Better error handling
- Documentation improvements

**Other:**
- Testing on different platforms
- Docker improvements
- Documentation/examples
- Bug reports and feature requests

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

Basic benchmarks on M1 MacBook Pro:
- **Static routes**: ~450k ops/sec (very fast hash lookup)
- **Dynamic routes**: ~200k ops/sec (regex matching)
- **Memory usage**: ~15MB idle
- **Latency**: <1ms for route matching

*Note: These are micro-benchmarks. Real-world performance will vary.*

## Known Issues

- [ ] Configuration validation could be better
- [ ] Error messages need improvement  
- [ ] No WebSocket support yet
- [ ] Limited HTTP/2 support
- [ ] Missing comprehensive integration tests

## License

MIT License - see [LICENSE](LICENSE) file.

## Acknowledgments

Built with these excellent Rust crates:
- [Actix Web](https://actix.rs/) - Web framework
- [Reqwest](https://docs.rs/reqwest/) - HTTP client  
- [Serde](https://serde.rs/) - Serialization
- [Tokio](https://tokio.rs/) - Async runtime

---

**Status**: Early development, not production ready  
**Maintainer**: [@DanielSarmiento04](https://github.com/DanielSarmiento04)  
**Community**: Issues and PRs welcome!

---

**Status**: Early development, not production ready  
**Maintainer**: [@DanielSarmiento04](https://github.com/DanielSarmiento04)  
**Community**: Issues and PRs welcome!

