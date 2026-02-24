# Examples

Kairos Gateway provides ready-to-run examples to help you get started quickly. These examples demonstrate how to configure and deploy the gateway alongside backend services using Docker Compose.

You can find the source code for these examples in the `examples/` directory of the repository.

---

## HTTP Routing Example

This example demonstrates how to use Kairos Gateway to route HTTP traffic to a Python FastAPI backend service.

### Architecture

- **Kairos Gateway**: Runs on port `5900`.
- **FastAPI Backend**: Runs on port `80` (internal Docker network).

### Running the Example

1. Navigate to the example directory:
   ```bash
   cd examples/http_routing
   ```

2. Start the services using Docker Compose:
   ```bash
   docker compose up --build
   ```

### Testing the Routing

The gateway is configured to route requests from `/test/health` to the root `/` of the FastAPI backend.

```bash
curl http://localhost:5900/test/health
```

**Expected Response:**
```json
{"Hello": "World"}
```

### Configuration Highlights

The `config.json` for this example defines a simple HTTP router:

```json
{
  "routers": [
    {
      "external_path": "/test/health",
      "internal_path": "/",
      "methods": ["GET"],
      "backends": [
        {
          "host": "http://app_http",
          "port": 80
        }
      ],
      "auth_required": false
    }
  ]
}
```

Notice how the `external_path` (`/test/health`) is rewritten to the `internal_path` (`/`) before being forwarded to the backend service (`app_http`).

---

## WebSocket Routing Example

This example demonstrates how Kairos Gateway can proxy real-time bidirectional WebSocket connections to a Bun/TypeScript backend.

### Architecture

- **Kairos Gateway**: Runs on port `5900`.
- **WebSocket Backend**: Runs on port `3000` (internal Docker network).

### Running the Example

1. Navigate to the example directory:
   ```bash
   cd examples/websocket_routing
   ```

2. Start the services using Docker Compose:
   ```bash
   docker compose up
   ```

### Testing the Connection

You can test the WebSocket connection using a tool like `wscat` or `curl`.

**Using wscat:**
```bash
wscat -c "ws://localhost:5900/ws/chat"
```

**Using curl:**
```bash
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: $(openssl rand -base64 16)" \
  http://localhost:5900/ws/chat
```

### Configuration Highlights

The `config.json` for this example defines a WebSocket router:

```json
{
  "routers": [
    {
      "protocol": "websocket",
      "external_path": "/ws/chat",
      "internal_path": "/ws",
      "backends": [
        {
          "host": "ws://api_websocket",
          "port": 3000
        }
      ]
    }
  ]
}
```

Key differences from HTTP routing:
- `protocol` is explicitly set to `"websocket"`.
- The backend host uses the `ws://` scheme.
- The gateway handles the HTTP Upgrade request and establishes a persistent bidirectional tunnel.
