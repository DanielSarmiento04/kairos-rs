# WebSocket Proxy Guide

## Overview

Kairos Gateway now supports WebSocket proxying, enabling real-time bidirectional communication between clients and backend WebSocket servers. The gateway acts as a transparent proxy, forwarding WebSocket messages in both directions while maintaining connection lifecycle management.

## Features

- ✅ **WebSocket Protocol Support** - Full WebSocket (RFC 6455) implementation
- ✅ **Bidirectional Message Forwarding** - Client ↔ Backend message proxying
- ✅ **Connection Lifecycle Management** - Proper upgrade, ping/pong, and close handling
- ✅ **Protocol Schemes** - Support for `ws://` and `wss://` (secure WebSocket)
- ✅ **Path Routing** - Map external paths to internal backend paths
- ✅ **Multiple Message Types** - Text, Binary, Ping, Pong, and Close frames
- ✅ **Load Balancing Ready** - Backend array support (single backend currently active)

## Configuration

### Basic WebSocket Route

Add a WebSocket route to your `config.json`:

```json
{
  "version": 1,
  "routers": [
    {
      "protocol": "websocket",
      "backends": [
        {
          "host": "ws://localhost",
          "port": 3000,
          "weight": 1
        }
      ],
      "external_path": "/ws/chat",
      "internal_path": "/ws",
      "methods": ["GET"],
      "auth_required": false
    }
  ]
}
```

### Configuration Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `protocol` | string | Yes | Must be `"websocket"` for WebSocket routes |
| `backends` | array | Yes | List of backend WebSocket servers |
| `backends[].host` | string | Yes | Backend URL with `ws://` or `wss://` scheme |
| `backends[].port` | number | Yes | Backend port (1-65535) |
| `backends[].weight` | number | No | Load balancing weight (default: 1) |
| `external_path` | string | Yes | Client-facing WebSocket path |
| `internal_path` | string | Yes | Backend WebSocket path |
| `methods` | array | Yes | Must include `"GET"` for WebSocket upgrade |
| `auth_required` | boolean | No | Enable JWT authentication (default: false) |

### Secure WebSocket (WSS)

For secure WebSocket connections, use the `wss://` scheme:

```json
{
  "protocol": "websocket",
  "backends": [
    {
      "host": "wss://secure-backend.example.com",
      "port": 443,
      "weight": 1
    }
  ],
  "external_path": "/secure/ws",
  "internal_path": "/ws",
  "methods": ["GET"],
  "auth_required": true
}
```

### Path Mapping

The gateway maps external paths to internal backend paths:

```
Client Request:    ws://gateway:5900/ws/chat
Gateway Forwards:  ws://backend:3000/ws
```

This allows you to:
- Version your WebSocket APIs (`/v1/ws`, `/v2/ws`)
- Organize endpoints by feature (`/chat`, `/notifications`, `/updates`)
- Abstract backend infrastructure from clients

## Usage Examples

### Example 1: Simple Chat Application

**Configuration:**
```json
{
  "protocol": "websocket",
  "backends": [
    {
      "host": "ws://localhost",
      "port": 8080,
      "weight": 1
    }
  ],
  "external_path": "/chat",
  "internal_path": "/ws/chat",
  "methods": ["GET"],
  "auth_required": false
}
```

**Client Connection:**
```javascript
const ws = new WebSocket('ws://localhost:5900/chat');

ws.onopen = () => {
  console.log('Connected to chat');
  ws.send(JSON.stringify({ type: 'join', user: 'Alice' }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### Example 2: Authenticated WebSocket

**Configuration:**
```json
{
  "jwt": {
    "secret": "your-secret-key",
    "issuer": "kairos-gateway",
    "audience": "websocket-clients"
  },
  "routers": [
    {
      "protocol": "websocket",
      "backends": [
        {
          "host": "ws://localhost",
          "port": 8080,
          "weight": 1
        }
      ],
      "external_path": "/secure/ws",
      "internal_path": "/ws",
      "methods": ["GET"],
      "auth_required": true
    }
  ]
}
```

**Client Connection (with JWT):**
```javascript
const token = 'your-jwt-token';
const ws = new WebSocket('ws://localhost:5900/secure/ws', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
```

### Example 3: Multiple Backend Support (Future)

```json
{
  "protocol": "websocket",
  "backends": [
    {
      "host": "ws://backend-1",
      "port": 8080,
      "weight": 3
    },
    {
      "host": "ws://backend-2",
      "port": 8080,
      "weight": 2
    },
    {
      "host": "ws://backend-3",
      "port": 8080,
      "weight": 1
    }
  ],
  "external_path": "/ws/load-balanced",
  "internal_path": "/ws",
  "methods": ["GET"],
  "auth_required": false
}
```

**Note:** Load balancing across multiple WebSocket backends is planned for a future release. Currently, the first backend in the array is used.

## Testing Your WebSocket Route

### Using wscat (CLI)

Install wscat:
```bash
npm install -g wscat
```

Connect to your WebSocket route:
```bash
wscat -c "ws://localhost:5900/ws/chat"
```

Send messages:
```
> Hello, WebSocket!
< {"type":"echo","message":"Hello, WebSocket!"}
```

### Using Browser JavaScript

```javascript
const ws = new WebSocket('ws://localhost:5900/ws/chat');

ws.onopen = () => {
  console.log('WebSocket connected');
  ws.send('Hello from browser!');
};

ws.onmessage = (event) => {
  console.log('Server says:', event.data);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('WebSocket disconnected');
};
```

### Test Backend Server

A simple test WebSocket server (using Bun.js):

```typescript
// server.ts
const server = Bun.serve({
  port: 3000,
  fetch(req, server) {
    const url = new URL(req.url);
    
    if (url.pathname === "/ws") {
      const success = server.upgrade(req);
      if (success) return undefined;
    }
    
    return new Response("Not Found", { status: 404 });
  },
  
  websocket: {
    open(ws) {
      console.log("Client connected");
      ws.send(JSON.stringify({ 
        type: "welcome", 
        message: "Connected to WebSocket server" 
      }));
    },
    
    message(ws, message) {
      console.log("Received:", message);
      ws.send(JSON.stringify({
        type: "echo",
        original: message,
        timestamp: new Date().toISOString()
      }));
    },
    
    close(ws) {
      console.log("Client disconnected");
    }
  }
});

console.log(`WebSocket server running on ws://localhost:${server.port}/ws`);
```

Run it:
```bash
bun run server.ts
```

## Message Flow

```
┌─────────┐                 ┌─────────────┐                 ┌─────────┐
│ Client  │                 │   Kairos    │                 │ Backend │
│         │                 │   Gateway   │                 │         │
└────┬────┘                 └──────┬──────┘                 └────┬────┘
     │                             │                             │
     │  HTTP GET /ws/chat          │                             │
     │ Upgrade: websocket          │                             │
     ├────────────────────────────>│                             │
     │                             │                             │
     │                             │  WS Connect ws://backend:3000/ws
     │                             ├────────────────────────────>│
     │                             │                             │
     │                             │  Connection Established     │
     │                             │<────────────────────────────┤
     │                             │                             │
     │  101 Switching Protocols    │                             │
     │<────────────────────────────┤                             │
     │                             │                             │
     │  Text: "Hello"              │                             │
     ├────────────────────────────>│  Text: "Hello"              │
     │                             ├────────────────────────────>│
     │                             │                             │
     │                             │  Text: "Hi there!"          │
     │  Text: "Hi there!"          │<────────────────────────────┤
     │<────────────────────────────┤                             │
     │                             │                             │
     │  Close                      │                             │
     ├────────────────────────────>│  Close                      │
     │                             ├────────────────────────────>│
     │                             │                             │
```

## Architecture

### Components

1. **WebSocketHandler** (`services/websocket.rs`)
   - Manages WebSocket connection lifecycle
   - Handles message forwarding
   - Converts between actix-ws and tokio-tungstenite message types

2. **Route Configuration** (`routes/auth_http.rs`)
   - Detects WebSocket protocol routes
   - Applies JWT authentication if required
   - Routes WebSocket upgrade requests to handler

3. **Backend Validation** (`models/router.rs`)
   - Validates `ws://` and `wss://` schemes
   - Ensures backend configuration correctness
   - Supports both legacy and new backend formats

### Message Types Supported

| Type | Description | Forwarded |
|------|-------------|-----------|
| Text | UTF-8 text messages | ✅ Yes |
| Binary | Raw binary data | ✅ Yes |
| Ping | Connection keepalive | ✅ Yes |
| Pong | Ping response | ✅ Yes |
| Close | Connection termination | ✅ Yes |

## Limitations & Future Enhancements

### Current Limitations

- ⚠️ **Single Backend Only** - Only the first backend in the array is used
- ⚠️ **No Circuit Breaking** - Circuit breaker pattern not yet applied to WebSocket
- ⚠️ **No Health Checks** - Backend health checking not implemented for WebSocket
- ⚠️ **No Message Transformation** - Messages are forwarded as-is

### Planned Features

- 🔜 **Load Balancing** - Distribute connections across multiple backends
- 🔜 **Sticky Sessions** - Keep clients connected to the same backend
- 🔜 **Message Filtering** - Filter/transform messages based on rules
- 🔜 **Rate Limiting** - Per-connection message rate limits
- 🔜 **Metrics** - WebSocket-specific metrics (connections, messages, latency)
- 🔜 **Compression** - Per-message deflate extension support
- 🔜 **Subprotocol Negotiation** - Support for WebSocket subprotocols

## Troubleshooting

### Connection Fails with 502 Bad Gateway

**Problem:** Client receives 502 error when connecting

**Solutions:**
1. Verify backend WebSocket server is running
2. Check backend URL scheme (`ws://` or `wss://`)
3. Confirm backend accepts connections at the internal_path
4. Review gateway logs for connection errors

```bash
RUST_LOG=debug cargo run --bin kairos-gateway
```

### WebSocket Upgrade Fails

**Problem:** Connection doesn't upgrade to WebSocket

**Solutions:**
1. Ensure `methods` includes `"GET"`
2. Verify `protocol` is set to `"websocket"`
3. Check client sends correct upgrade headers
4. Confirm no middleware blocking the upgrade

### Messages Not Forwarded

**Problem:** Messages sent but not received by backend/client

**Solutions:**
1. Check message encoding (text vs binary)
2. Verify both directions are working
3. Review backend WebSocket implementation
4. Check for connection timeouts

### Authentication Issues

**Problem:** Authenticated route rejects connections

**Solutions:**
1. Verify JWT token is valid
2. Check token is sent in `Authorization` header
3. Confirm JWT secret matches configuration
4. Validate token claims (issuer, audience, expiry)

## Performance Considerations

- **Connection Overhead**: Each WebSocket maintains an open TCP connection
- **Message Buffering**: Messages are buffered during forwarding
- **Worker Threads**: Actix-Web spawns multiple workers (default: # of CPU cores)
- **Memory Usage**: ~1-2KB per connection for buffers

## Security Best Practices

1. **Use WSS in Production** - Always use secure WebSocket (`wss://`) in production
2. **Enable Authentication** - Set `auth_required: true` for sensitive endpoints
3. **Rate Limiting** - Plan for message-level rate limiting in your application
4. **Input Validation** - Validate all messages in your backend
5. **Connection Limits** - Monitor and limit concurrent connections
6. **CORS Configuration** - Configure CORS appropriately for WebSocket origins

## Version History

- **v0.2.9 (October 2025)** - Initial WebSocket proxy implementation
  - Basic WebSocket proxying support
  - Text, Binary, Ping, Pong, Close message forwarding
  - JWT authentication support
  - Backend array configuration (single backend active)

## See Also

- [Configuration Guide](https://github.com/DanielSarmiento04/kairos-rs/blob/main/Readme.md#configuration) - General configuration documentation
- [JWT Authentication](https://github.com/DanielSarmiento04/kairos-rs/blob/main/Readme.md#jwt-authentication) - JWT setup and usage
- [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) - WebSocket Protocol Specification
