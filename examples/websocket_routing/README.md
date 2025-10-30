# WebSocket Routing Example

This example demonstrates how to use Kairos Gateway with WebSocket support using Docker.

## Architecture

- **API WebSocket Server** (`api_websocket`): Backend WebSocket server running on port 3000
- **Kairos Gateway** (`gateway`): API Gateway with WebSocket proxy running on port 5900

## Prerequisites

- Docker and Docker Compose
- Bun (for local development)

## Running with Docker Compose

### Option 1: Using Pre-built Multi-Platform Image (Recommended)

After the GitHub Actions workflow builds the multi-platform image:

```sh
docker compose up
```

This will:
1. Build the WebSocket backend server locally
2. Pull the pre-built `kairos-rs:main` gateway image from GitHub Container Registry
3. Start both services with proper networking

**Note**: The multi-platform image supports both AMD64 (Intel/AMD) and ARM64 (Apple Silicon) architectures.

### Option 2: Build Gateway Locally

If you want to build the gateway from source instead of using the pre-built image:

```sh
# From the root of the kairos-rs repository
docker build -t kairos-gateway-local .

# Then modify compose.yml to use the local image:
# image: kairos-gateway-local
```

## Testing the WebSocket Connection

Once the services are running:

```sh
# Using wscat
wscat -c "ws://localhost:5900/ws/chat"

# Or using curl with WebSocket upgrade
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: $(openssl rand -base64 16)" \
  http://localhost:5900/ws/chat
```

## Local Development (Without Docker)

### Install dependencies:
```sh
bun install
```

### Run the WebSocket server:
```sh
bun run dev
```

### Run the gateway separately:
```sh
# From the root of kairos-rs repository
cargo run --bin kairos-gateway -- --config examples/websocket_routing/config.json
```

Then open http://localhost:3000 in your browser.

## Configuration

The `config.json` file defines the WebSocket route:

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

## Troubleshooting

### ARM64 Architecture Error

If you see:
```
no matching manifest for linux/arm64/v8 in the manifest list entries
```

This means the Docker image wasn't built for your architecture. Wait for the GitHub Actions workflow to rebuild with multi-platform support, or build the image locally.

### Connection Refused

Make sure both services are running:
```sh
docker compose ps
```

Check logs:
```sh
docker compose logs gateway
docker compose logs api_websocket
```

## Architecture Diagram

```
Client (Browser/wscat)
    |
    | WebSocket: ws://localhost:5900/ws/chat
    v
Kairos Gateway (Port 5900)
    |
    | Proxies to: ws://api_websocket:3000/ws
    v
WebSocket Backend Server (Port 3000)
```
