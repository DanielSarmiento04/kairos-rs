# Configuration Guide

Kairos Gateway uses a JSON configuration file (`config.json`) to define its behavior, routing rules, security policies, and more. This guide covers the structure and available options.

## Global Settings

The root of the configuration file contains global settings for the gateway.

```json
{
  "version": 1,
  "server": {
    "host": "0.0.0.0",
    "port": 5900
  },
  "metrics": {
    "enabled": true,
    "path": "/metrics"
  },
  "cors": {
    "allowed_origins": ["*"],
    "allowed_methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    "allowed_headers": ["Authorization", "Content-Type"],
    "max_age": 3600
  },
  "routers": [
    // Route definitions...
  ]
}
```

### Server Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `host` | string | `"0.0.0.0"` | The IP address to bind the gateway to. |
| `port` | number | `5900` | The port to listen on. |

### Metrics Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable or disable Prometheus metrics. |
| `path` | string | `"/metrics"` | The endpoint path for metrics scraping. |

### CORS Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `allowed_origins` | array | `["*"]` | List of allowed origins. |
| `allowed_methods` | array | `["GET", "POST", "PUT", "DELETE", "OPTIONS"]` | List of allowed HTTP methods. |
| `allowed_headers` | array | `["Authorization", "Content-Type"]` | List of allowed HTTP headers. |
| `max_age` | number | `3600` | Preflight request cache duration in seconds. |

## Route Configuration

The `routers` array contains the routing rules for the gateway. Each route defines how incoming requests are matched and forwarded to backend services.

```json
{
  "routers": [
    {
      "external_path": "/api/users",
      "internal_path": "/users",
      "methods": ["GET", "POST"],
      "protocol": "http",
      "backends": [
        {
          "host": "http://backend1",
          "port": 8080,
          "weight": 2
        },
        {
          "host": "http://backend2",
          "port": 8080,
          "weight": 1
        }
      ],
      "load_balancing_strategy": "round_robin",
      "auth_required": true,
      "rate_limit": {
        "requests_per_second": 100,
        "burst_size": 20
      },
      "retry": {
        "max_retries": 3,
        "base_delay_ms": 100,
        "max_delay_ms": 2000,
        "retryable_status_codes": [500, 502, 503, 504]
      }
    }
  ]
}
```

### Route Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `external_path` | string | Yes | The path the client requests (e.g., `/api/users`). Supports regex and path parameters. |
| `internal_path` | string | Yes | The path forwarded to the backend (e.g., `/users`). |
| `methods` | array | Yes | Allowed HTTP methods (e.g., `["GET", "POST"]`). |
| `protocol` | string | No | The protocol to use (`http`, `websocket`, `ftp`, `dns`). Default is `http`. |
| `backends` | array | Yes | List of backend servers to route to. |
| `load_balancing_strategy` | string | No | Strategy for distributing traffic. Default is `round_robin`. |
| `auth_required` | boolean | No | Whether JWT authentication is required. Default is `false`. |
| `rate_limit` | object | No | Rate limiting configuration for this route. |
| `retry` | object | No | Retry logic configuration for this route. |

### Load Balancing Strategies

Kairos supports multiple load balancing strategies:

- `round_robin`: Distributes requests sequentially across all backends.
- `least_connections`: Routes to the backend with the fewest active connections.
- `random`: Selects a backend at random.
- `weighted`: Distributes traffic based on the `weight` assigned to each backend.
- `ip_hash`: Consistently routes the same client IP to the same backend.

### Retry Logic

Configure automatic retries for failed requests:

- `max_retries`: Maximum number of retry attempts.
- `base_delay_ms`: Initial delay before the first retry.
- `max_delay_ms`: Maximum delay between retries (uses exponential backoff).
- `retryable_status_codes`: List of HTTP status codes that trigger a retry.

## Security Configuration

### JWT Authentication

To secure routes, configure the global `jwt` settings and set `auth_required: true` on specific routes.

```json
{
  "jwt": {
    "secret": "your-super-secure-jwt-secret-key-must-be-at-least-32-characters-long",
    "issuer": "kairos-gateway",
    "audience": "your-app",
    "required_claims": ["sub", "exp", "role"]
  }
}
```

### Rate Limiting

Protect your backends from overload by configuring rate limits per route.

```json
"rate_limit": {
  "requests_per_second": 100,
  "burst_size": 20
}
```

## Hot Reload

Kairos Gateway supports hot reloading of its configuration without dropping active connections.

To reload the configuration, send a `POST` request to the management endpoint:

```bash
curl -X POST http://localhost:5900/api/config/reload
```

This will read the `config.json` file from disk and apply the new routing rules immediately.
