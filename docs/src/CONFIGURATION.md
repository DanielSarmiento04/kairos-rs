# Configuration Guide

Kairos Gateway uses a JSON configuration file (`config.json`) to define its behavior, routing rules, security policies, and more. This guide covers the structure and available options.

## Global Settings

The root of the configuration file contains global settings for the gateway.

```json
{
  "version": 1,
  "jwt": {
    "secret": "your-secret-key-at-least-32-characters",
    "issuer": "kairos-gateway",
    "audience": "api-clients",
    "required_claims": ["sub", "exp"]
  },
  "rate_limit": {
    "strategy": "PerIP",
    "requests_per_window": 100,
    "window_duration": 60,
    "burst_allowance": 20,
    "window_type": "SlidingWindow"
  },
  "routers": [
    // Route definitions...
  ]
}
```

> **Note:** The gateway bind address and port are **not** set in the config file. Use the `KAIROS_HOST` (default: `0.0.0.0`) and `KAIROS_PORT` (default: `5900`) environment variables to configure where the gateway listens.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KAIROS_HOST` | `0.0.0.0` | The IP address to bind the gateway to. |
| `KAIROS_PORT` | `5900` | The port to listen on. |

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
| `external_path` | string | Yes | The path the client requests (e.g., `/api/users`). Supports `{param}` path placeholders (e.g., `/api/users/{id}`); regular expressions are not supported. |
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
{
  "routers": [
    {
      "external_path": "/api/users",
      "internal_path": "/users",
      "methods": ["GET"],
      "backends": [
        {
          "host": "http://backend1",
          "port": 8080
        }
      ],
      "rate_limit": {
        "requests_per_second": 100,
        "burst_size": 20
      }
    }
  ]
}
```

## Hot Reload

Kairos Gateway supports hot reloading of its configuration without dropping active connections.

To reload the configuration, send a `POST` request to the management endpoint:

```bash
curl -X POST http://localhost:5900/api/config/reload
```

This will read the `config.json` file from disk and apply the new routing rules immediately.
