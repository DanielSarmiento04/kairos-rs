# Configuration Guide

Kairos Gateway relies heavily on a central JSON configuration file (`config.json`). This file defines everything from your routing rules to AI fallbacks, rate limiting, and security policies.

By default, the gateway looks for `./config.json`. You can override this using the `KAIROS_CONFIG_PATH` environment variable.

## Environment Variables

While routing rules and security are defined in `config.json`, the server binding properties are set through environment variables:

- `KAIROS_HOST`: Server bind address (default: `0.0.0.0`).
- `KAIROS_PORT`: Server port number (default: `5900`).
- `KAIROS_CONFIG_PATH`: Override path to `config.json`.
- `JWT_SECRET`: Overrides the default JWT secret if `jwt` block is missing in config.

## Root Configuration Structure

The root of `config.json` encapsulates global security, rate limiting, your AI provider preferences, and your `routers` array.

```json
{
  "version": 1,
  "jwt": {
    "secret": "your-super-secure-jwt-secret-key-must-be-at-least-32-characters-long",
    "issuer": "kairos-gateway",
    "audience": "api-clients",
    "required_claims": ["sub", "exp"]
  },
  "rate_limit": {
    "strategy": "PerIP",
    "requests_per_window": 100,
    "window_duration": 60,
    "burst_allowance": 20,
    "window_type": "SlidingWindow",
    "enable_redis": false,
    "redis_key_prefix": "kairos_rl"
  },
  "ai": {
    "provider": "openai",
    "model": "gpt-4"
  },
  "routers": [
    // Route definitions...
  ]
}
```

### Global Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | number | Yes | Schema version for compatibility. Should be `1`. |
| `jwt` | object | No | Global JWT authentication settings. Required if any router sets `auth_required: true`. |
| `rate_limit` | object | No | Global rate limiting configuration. |
| `ai` | object | No | Configuration for AI routing/fallback models. |
| `routers` | array | Yes | Array of route definitions. Processed in order. |

---

## Route Configuration

The `routers` array maps external client requests to your internal backend services. 

### Basic Example

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
      "retry": {
        "max_retries": 3,
        "initial_backoff_ms": 100,
        "max_backoff_ms": 5000,
        "backoff_multiplier": 2.0,
        "retry_on_status_codes": [408, 429, 502, 503, 504]
      }
    }
  ]
}
```

### Route Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `external_path` | string | Yes | The path the client requests (e.g., `/api/users`). Supports `{param}` placeholders. Must start with `/`. |
| `internal_path` | string | Yes | The path forwarded to the backend. Parameters from `external_path` translate natively. |
| `methods` | array | Yes | Allowed HTTP methods (`["GET", "POST", "PUT", "DELETE"]`). |
| `protocol` | string | No | The protocol to use. Options: `http`, `websocket`, `ftp`, `dns`. Default is `http`. |
| `backends` | array | Yes | Array of backend targets to route traffic into. |
| `load_balancing_strategy` | string | No | Traffic distribution pattern. Options: `round_robin`, `least_connections`, `random`, `weighted`, `ip_hash`. |
| `auth_required` | boolean | No | Whether requests against this route require valid JWT authentication. Default is `false`. |
| `retry` | object | No | Retry logic configuration specifically tailored for this route's resilience. |
| `request_transformation` | object | No | Transform incoming request headers, rewrite paths, or query params context. |
| `response_transformation` | object | No | Append/overwrite outgoing response headers. |
| `ai_policy` | object | No | Define AI failover cache behaviors for AI integrations. |

> **Info:** `host` and `port` inside a single Router level properties are considered legacy APIs. Always define destinations exclusively using the `backends` logical array block.

---

## Load Balancing Strategies

Kairos natively supplies algorithms optimized for distinct traffic topologies:

- `round_robin` (Default): Distributes requests systematically across all backends.
- `least_connections`: Routes incoming load to the backend with historically fewest active connections. Highly reliable against varying capacities.
- `random`: Unweighted stateless dispersal.
- `weighted`: Distributes proportionally corresponding to the static `weight` parameter bound to each backend element.
- `ip_hash`: Derives consistent affinity ("Sticky Sessions") locking client IPs deterministically against backends.

---

## Retry Configuration Payload

Safely configure exponential backoff against generic gateway fault tolerances.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_retries` | number | 3 | Maximum execution attempts before returning error |
| `initial_backoff_ms` | number | 100 | Time allocation preceding the first retry. |
| `max_backoff_ms` | number | 5000 | The cap for iterative exponential delay mechanisms. |
| `backoff_multiplier` | number | 2.0 | Factor for exponential delay sizing iterations. |
| `retry_on_status_codes` | array | `[408,429,502...504]` | Defined HTTP Code triggers warranting an automatic restart logic tree. |

---

## Rate Limiting Security

Control burst overflows comprehensively. Define `rate_limit` globally mapping across routes or users.

- `strategy`: Selection constraint logic. `PerIP`, `PerUser`, `PerRoute`, `PerIPAndRoute`, `PerUserAndRoute`.
- `requests_per_window`: Numerator count of valid requests allowable. 
- `window_duration`: Numerically expressed period measuring integer seconds logic.
- `burst_allowance`: Soft-limit extension buffer protecting sudden localized traffic spikes.
- `window_type`: `FixedWindow`, `SlidingWindow`, or `TokenBucket`.
- `enable_redis`: Expand rate limits universally via cross-container Redis (future distributed support).

---

## Hot Reload

Kairos Gateway runs memory-safe hot reloading meaning server downtime is avoided.

Initiate local re-synchronization sending an empty POST against:

```bash
curl -X POST http://localhost:5900/api/config/reload
```

Your system replaces the active configuration schema on-the-fly dynamically.
