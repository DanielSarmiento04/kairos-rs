# Kairos Gateway

[![Crates.io](https://img.shields.io/crates/v/kairos-gateway.svg)](https://crates.io/crates/kairos-gateway)
[![Documentation](https://docs.rs/kairos-gateway/badge.svg)](https://docs.rs/kairos-gateway)
[![License](https://img.shields.io/crates/l/kairos-gateway.svg)](https://github.com/DanielSarmiento04/kairos-rs/blob/main/License)

High-performance multi-protocol API gateway built with Rust and Actix-Web.

## Overview

`kairos-gateway` is the main executable binary for the Kairos API Gateway. It provides a production-ready, feature-rich gateway with support for multiple protocols including HTTP, WebSocket, FTP, and DNS.

## Features

- 🚀 **High Performance**: Built with Actix-Web for maximum throughput
- 🔄 **Multi-Protocol Support**: HTTP, WebSocket, FTP, DNS
- ⚖️ **Load Balancing**: Round-robin, least connections, random, IP hash strategies
- 🔐 **Security**: JWT authentication, rate limiting, request validation
- 🏥 **Health Checks**: Automatic backend health monitoring
- 🔁 **Retry Logic**: Configurable retry with exponential backoff
- 📊 **Metrics & Monitoring**: Built-in metrics collection and health endpoints
- 🔧 **Hot Reload**: Configuration updates without downtime
- 🌐 **Web UI**: Modern admin interface for management

## Installation

### From crates.io

```bash
cargo install kairos-gateway
```

### From Docker

```bash
docker pull ghcr.io/danielsarmiento04/kairos-rs:latest
```

### From Source

```bash
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs
cargo build --release --bin kairos-gateway
```

## Quick Start

### 1. Create Configuration File

Create a `config.json` file:

```json
{
  "version": 1,
  "routers": [
    {
      "protocol": "http",
      "external_path": "/api/users",
      "internal_path": "/users",
      "methods": ["GET", "POST"],
      "backends": [
        {
          "host": "http://localhost",
          "port": 8080,
          "weight": 1
        }
      ],
      "load_balancing_strategy": "round_robin",
      "auth_required": false
    }
  ]
}
```

### 2. Run the Gateway

```bash
# Using binary
kairos-gateway

# Using Docker
docker run -d \
  -p 5900:5900 \
  -v $(pwd)/config.json:/app/config.json:ro \
  ghcr.io/danielsarmiento04/kairos-rs:latest

# Using cargo
cargo run --bin kairos-gateway
```

### 3. Test the Gateway

```bash
# Health check
curl http://localhost:5900/health

# Test your route
curl http://localhost:5900/api/users
```

## Configuration

The gateway uses a JSON configuration file. See the [main documentation](../../Readme.md) for complete configuration reference.

### Environment Variables

- `RUST_LOG`: Log level (debug, info, warn, error)
- `KAIROS_HOST`: Server host (default: 0.0.0.0)
- `KAIROS_PORT`: Server port (default: 5900)
- `CONFIG_PATH`: Path to config.json (default: ./config.json)

### Example with Environment Variables

```bash
RUST_LOG=debug KAIROS_PORT=8000 kairos-gateway
```

## Protocol Support

### HTTP/HTTPS
- Request proxying and transformation
- Header manipulation
- Path rewriting
- Method filtering

### WebSocket
- Bidirectional message forwarding
- Connection upgrading
- Binary and text frame support
- Close frame handling

### FTP
- File upload/download
- Directory listing
- Passive mode support

### DNS
- Query forwarding
- Response caching
- Custom DNS resolution

## Advanced Features

### Load Balancing

Configure load balancing strategy per route:

```json
{
  "load_balancing_strategy": "least_connections",
  "backends": [
    {"host": "http://backend1", "port": 8080, "weight": 2},
    {"host": "http://backend2", "port": 8080, "weight": 1}
  ]
}
```

Available strategies:
- `round_robin`: Distribute evenly across backends
- `least_connections`: Send to backend with fewest active connections
- `random`: Random backend selection
- `ip_hash`: Consistent hashing based on client IP

### Circuit Breaker

Automatic failure detection and recovery:

```json
{
  "retry": {
    "max_retries": 3,
    "retry_delay_ms": 100,
    "backoff_multiplier": 2.0,
    "circuit_breaker_threshold": 5,
    "circuit_breaker_timeout_ms": 30000
  }
}
```

### JWT Authentication

Secure routes with JWT tokens:

```json
{
  "jwt": {
    "secret": "your-secret-key",
    "algorithm": "HS256",
    "issuer": "kairos-gateway",
    "audience": "api-clients"
  },
  "routers": [
    {
      "external_path": "/api/protected",
      "auth_required": true
    }
  ]
}
```

### Rate Limiting

Protect your backends from overload:

```json
{
  "rate_limit": {
    "requests_per_second": 100,
    "burst_size": 200
  }
}
```

## Monitoring

### Health Endpoints

- `GET /health` - Overall gateway health
- `GET /health/ready` - Readiness check
- `GET /health/live` - Liveness check

### Metrics Endpoints

- `GET /metrics` - Prometheus-compatible metrics
- `GET /metrics/requests` - Request statistics
- `GET /metrics/backends` - Backend health status

### Web UI

Access the admin interface at `http://localhost:5900/` (when enabled in config)

## Docker Deployment

### Docker Compose Example

```yaml
services:
  kairos-gateway:
    image: ghcr.io/danielsarmiento04/kairos-rs:0.2.10
    container_name: kairos-gateway
    restart: unless-stopped
    ports:
      - "5900:5900"
    volumes:
      - ./config.json:/app/config.json:ro
    environment:
      - RUST_LOG=info
      - KAIROS_HOST=0.0.0.0
      - KAIROS_PORT=5900
```

### Multi-Platform Support

The Docker image supports both AMD64 and ARM64 architectures:

```bash
# Pull for your platform automatically
docker pull ghcr.io/danielsarmiento04/kairos-rs:latest

# Or specify platform
docker pull --platform linux/arm64 ghcr.io/danielsarmiento04/kairos-rs:latest
```

## Performance

Benchmarks on a 4-core machine:

- **Throughput**: ~50K requests/second
- **Latency (p50)**: <1ms
- **Latency (p99)**: <10ms
- **Memory**: ~50MB base + ~10MB per 10K concurrent connections

## Troubleshooting

### Enable Debug Logging

```bash
RUST_LOG=debug kairos-gateway
```

### Check Backend Connectivity

```bash
curl http://localhost:5900/metrics/backends
```

### Exec into Docker Container

```bash
docker exec -it kairos-gateway sh
```

## Dependencies

This crate depends on [`kairos-rs`](https://crates.io/crates/kairos-rs) which provides the core gateway functionality.

## Documentation

- [Main Documentation](../../Readme.md)
- [Guides](../../docs/src/GUIDES.md)
- [API Documentation](https://docs.rs/kairos-gateway)
- [Examples](../../examples/)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/DanielSarmiento04/kairos-rs) for contribution guidelines.

## License

Licensed under MIT License. See [LICENSE](../../License) for details.

## Links

- [GitHub Repository](https://github.com/DanielSarmiento04/kairos-rs)
- [Issue Tracker](https://github.com/DanielSarmiento04/kairos-rs/issues)
- [Crates.io](https://crates.io/crates/kairos-gateway)
- [Documentation](https://docs.rs/kairos-gateway)
- [Docker Images](https://github.com/DanielSarmiento04/kairos-rs/pkgs/container/kairos-rs)
