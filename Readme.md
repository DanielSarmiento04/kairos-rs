# Ben 🚀

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> A high-performance HTTP gateway/proxy service built with Rust, providing intelligent request routing and forwarding capabilities.

## 📋 Table of Contents

- [Overview](#-overview)
- [Features](#-features)
- [Architecture](#-architecture)
- [Getting Started](#-getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration](#configuration)
- [Usage](#-usage)
- [API Reference](#-api-reference)
- [Logging](#-logging)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

## 🎯 Overview

Ben is a lightweight, configurable HTTP gateway service that acts as a reverse proxy, routing external requests to internal services based on path patterns. Built with [Actix Web](https://actix.rs/) and [Reqwest](https://docs.rs/reqwest/), it provides high-performance request forwarding with comprehensive error handling and logging.

### Key Use Cases

- **API Gateway**: Route external API calls to internal microservices
- **Service Mesh**: Proxy requests between services with path transformation
- **Load Balancing**: Distribute requests across multiple backend services
- **Development Proxy**: Route development requests to different environments

## ✨ Features

- 🔄 **Request Forwarding**: Intelligent HTTP request routing and proxying
- 🛣️ **Path Mapping**: Flexible external-to-internal path transformation
- 🔧 **Method Filtering**: Configurable HTTP method restrictions per route
- ⚡ **High Performance**: Built with Rust and Actix Web for maximum throughput
- 🎨 **Rich Logging**: Colored, formatted logs with configurable levels
- 📊 **Error Handling**: Comprehensive error types with proper HTTP status codes
- 🔧 **Configuration**: JSON-based route configuration
- 🚀 **Async/Await**: Full async support for concurrent request handling

## 🏗️ Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Client    │───▶│     Ben      │───▶│  Target Service │
│             │    │   Gateway    │    │                 │
└─────────────┘    └──────────────┘    └─────────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Configuration│
                   │  (config.json)│
                   └──────────────┘
```

### Components

- **RouteHandler**: Core request handling and forwarding logic
- **Configuration System**: JSON-based route definitions
- **Logger**: Custom logging with ANSI colors and formatting
- **Error Handling**: Structured error types with HTTP response mapping

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70+ (2021 edition)
- [Cargo](https://doc.rust-lang.org/cargo/) (included with Rust)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/DanielSarmiento04/Ben.git
   cd Ben
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the service**
   ```bash
   cargo run
   ```

The service will start on `http://0.0.0.0:5900` by default.

### Configuration

Create or modify `config.json` to define your routing rules:

```json
{
  "version": 1,
  "routers": [
    {
      "host": "http://localhost",
      "port": 3000,
      "external_path": "/api/v1/users",
      "internal_path": "/users",
      "methods": ["GET", "POST", "PUT", "DELETE"]
    },
    {
      "host": "https://api.example.com",
      "port": 443,
      "external_path": "/api/v2/auth",
      "internal_path": "/auth/login",
      "methods": ["POST"]
    }
  ]
}
```

#### Configuration Fields

| Field | Type | Description |
|-------|------|-------------|
| `host` | String | Target service hostname (with protocol) |
| `port` | Number | Target service port |
| `external_path` | String | Incoming request path pattern |
| `internal_path` | String | Target service path |
| `methods` | Array | Allowed HTTP methods |

## 📖 Usage

### Basic Request Flow

1. Client sends request to Ben gateway
2. Ben matches the request path against configured routes
3. Request is forwarded to the target service
4. Response is returned to the client

### Example Requests

```bash
# Request to Ben gateway
curl -X GET http://localhost:5900/api/v1/users

# Gets forwarded to
# http://localhost:3000/users
```

### Health Check

The service logs startup information and route validation results:

```
Aug 23 02:30:15 PM | [INFO]    | src/main.rs:28        | Version: 1
Aug 23 02:30:15 PM | [INFO]    | src/main.rs:32        | Logger initialized
```

## 🔧 API Reference

### Error Responses

Ben returns structured error responses:

```json
{
  "error": "Request timeout",
  "type": "timeout"
}
```

#### Error Types

| Type | HTTP Status | Description |
|------|-------------|-------------|
| `timeout` | 504 | Request to upstream service timed out |
| `internal` | 500 | Internal server error |
| `config` | 502 | Route configuration error |
| `upstream` | 502 | Upstream service error |

### Timeouts

- **Request Timeout**: 30 seconds (configurable)
- **Connection Pool**: 32 idle connections per host
- **Pool Timeout**: 30 seconds

## 📝 Logging

Ben features a custom logging system with rich formatting and colors.

### Log Levels

- **ERROR** (Red): Critical errors
- **WARN** (Yellow): Warning messages  
- **INFO** (Green): Informational messages
- **DEBUG** (Blue): Debug information
- **TRACE** (Magenta): Detailed tracing

### Usage

```bash
# Default logging (INFO and above)
cargo run

# Enable debug logging
RUST_LOG=debug cargo run

# Enable trace logging
RUST_LOG=trace cargo run

# Module-specific logging
RUST_LOG=ben=trace cargo run

# Disable colors
NO_COLOR=1 cargo run
```

### Log Format

```
Aug 23 02:30:15 PM | [INFO]    | src/main.rs:32        | Logger initialized
```

## 🛠️ Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── config/
│   ├── mod.rs
│   └── settings.rs      # Configuration loading
├── models/
│   ├── mod.rs
│   ├── router.rs        # Route definitions
│   ├── settings.rs      # Settings struct
│   ├── http.rs          # HTTP handler
│   ├── error.rs         # Error types
│   └── protocol.rs      # Protocol definitions
├── routes/
│   ├── mod.rs
│   ├── http.rs          # HTTP route configuration
│   └── websocket.rs     # WebSocket routes (future)
└── logs/
    ├── mod.rs
    └── logger.rs        # Custom logger implementation
```

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run with hot reload (requires cargo-watch)
cargo watch -x run

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy
```

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `actix-web` | 4.x | Web framework |
| `reqwest` | 0.12 | HTTP client |
| `tokio` | 1.x | Async runtime |
| `serde` | 1.0 | Serialization |
| `serde_json` | 1.0 | JSON handling |
| `log` | 0.4 | Logging facade |
| `env_logger` | 0.11 | Logger implementation |
| `chrono` | 0.4 | Date/time handling |
| `thiserror` | 1.0 | Error handling |

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Guidelines

- Follow Rust formatting conventions (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add tests for new functionality
- Update documentation as needed

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <sub>Built with ❤️ by <a href="https://github.com/DanielSarmiento04">Daniel Sarmiento</a></sub>
</div>

