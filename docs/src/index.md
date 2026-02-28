# Kairos Gateway

Welcome to the official documentation for **Kairos Gateway**, a high-performance, multi-protocol API gateway built with Rust and Actix-Web.

[![Crates.io](https://img.shields.io/crates/v/kairos-gateway.svg)](https://crates.io/crates/kairos-gateway)
[![Documentation](https://docs.rs/kairos-gateway/badge.svg)](https://docs.rs/kairos-gateway)
[![License](https://img.shields.io/crates/l/kairos-gateway.svg)](https://github.com/DanielSarmiento04/kairos-rs/blob/main/LICENSE)

## What is Kairos?

Kairos is designed to be a modern, production-ready gateway that handles not just HTTP, but multiple protocols including WebSocket, FTP, and DNS. It incorporates intelligent AI-driven routing capabilities, making it a forward-thinking choice for modern microservices architectures.

### Key Features

- 🚀 **High Performance**: Built with Rust and Actix-Web for maximum throughput and minimal latency.
- 🧠 **AI-Powered Routing**: Route requests dynamically based on content complexity using LLMs.
- 🔄 **Multi-Protocol Support**: Seamlessly proxy HTTP/HTTPS, WebSockets, FTP, and DNS.
- ⚖️ **Advanced Load Balancing**: Round-robin, least connections, random, weighted, and IP hash strategies.
- 🛡️ **Robust Security**: Built-in JWT authentication, rate limiting, and request validation.
- 🏥 **Resilience**: Circuit breakers, automatic health checks, and configurable retry logic with exponential backoff.
- 📊 **Observability**: Prometheus metrics, real-time WebSocket metrics streaming, and a modern Web UI dashboard.
- 🔧 **Hot Reload**: Update your routing configuration without dropping a single connection.

## Quick Start

### 1. Installation

You can run Kairos Gateway using Docker (recommended) or install it via Cargo.

=== "Docker"

    ```bash
    docker pull ghcr.io/danielsarmiento04/kairos-rs:latest
    ```

=== "Cargo"

    ```bash
    cargo install kairos-gateway
    ```

=== "Source"

    ```bash
    git clone https://github.com/DanielSarmiento04/kairos-rs.git
    cd kairos-rs
    cargo build --release --bin kairos-gateway
    ```

### 2. Basic Configuration

Create a `config.json` file to define your routes:

```json
{
  "version": 1,
  "routers": [
    {
      "external_path": "/api/users",
      "internal_path": "/users",
      "methods": ["GET", "POST"],
      "backends": [
        {
          "host": "http://localhost",
          "port": 8080,
          "weight": 1
        }
      ]
    }
  ]
}
```

> **Note:** The gateway bind address and port are configured via the `KAIROS_HOST` (default: `0.0.0.0`) and `KAIROS_PORT` (default: `5900`) environment variables, not in the config file.

### 3. Run the Gateway

=== "Docker"

    ```bash
    docker run -d \
      -p 5900:5900 \
      -e KAIROS_HOST=0.0.0.0 \
      -e KAIROS_PORT=5900 \
      -v $(pwd)/config.json:/app/config.json:ro \
      ghcr.io/danielsarmiento04/kairos-rs:latest
    ```

=== "Binary"

    ```bash
    KAIROS_HOST=0.0.0.0 KAIROS_PORT=5900 kairos-gateway --config ./config.json
    ```

### 4. Verify

Check the health of your gateway:

```bash
curl http://localhost:5900/health
```

## Next Steps

Explore the guides to unlock the full potential of Kairos Gateway:

- [Configuration Guide](CONFIGURATION.md): Learn how to configure load balancing, rate limiting, and circuit breakers.
- [AI Routing Guide](AI_ROUTING_GUIDE.md): Discover how to use LLMs for smart request routing.
- [UI Dashboard](UI_DASHBOARD.md): Set up and use the real-time web administration interface.
- [Examples](EXAMPLES.md): Try out ready-to-run Docker Compose examples.
