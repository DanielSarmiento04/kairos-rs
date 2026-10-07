# SSL Offloading & TLS Termination Guide

This guide explains how to properly configure SSL/TLS offloading and termination when deploying **Kairos Gateway** in production.

---

## Architecture Overview

Kairos Gateway is designed as an application-layer API gateway and AI routing orchestrator. In production architectures, **SSL/TLS termination is recommended to be handled at the network edge** by a dedicated reverse proxy or cloud load balancer (such as **Nginx**, **Caddy**, **Traefik**, **AWS ALB**, **Cloudflare**, or a **Kubernetes Ingress Controller**).

```text
               PUBLIC INTERNET (HTTPS / TLS 1.3)
                             │
                             ▼
     ┌────────────────────────────────────────────────┐
     │   Edge Reverse Proxy / Load Balancer           │
     │   (Nginx / Caddy / Cloudflare / AWS ALB)       │
     │   • TLS Termination & Automated ACME Certs     │
     │   • HTTP/2 & HTTP/3 ALPN Negotiation           │
     │   • Edge DDoS & SYN Flood Protection           │
     └────────────────────────────────────────────────┘
                             │
                             │  Internal Network (Plain HTTP / Low Latency)
                             │  Headers: X-Forwarded-For, X-Forwarded-Proto: https
                             ▼
     ┌────────────────────────────────────────────────┐
     │   Kairos-rs Gateway (:5900)                    │
     │   • AI Content-Aware Routing                   │
     │   • In-Memory Response Caching (ResponseCache) │
     │   • Request/Response Transformation            │
     │   • Circuit Breakers & Upstream Load Balancing │
     └────────────────────────────────────────────────┘
                             │
                             ▼
     ┌────────────────────────────────────────────────┐
     │   Upstream AI Providers & Backend Services     │
     │   (OpenAI, Anthropic, Internal Microservices)  │
     └────────────────────────────────────────────────┘
```

### Why External SSL Offloading?

1. **CPU & Cryptographic Efficiency**: Cryptographic handshakes (AES-GCM, ChaCha20-Poly1305) consume CPU cycles. Terminating TLS at the edge preserves Kairos-rs CPU and memory strictly for route matching, response caching, and AI orchestration.
2. **Automated Certificate Lifecycle**: Edge proxies natively automate Let's Encrypt / ZeroSSL renewal via the ACME protocol with zero downtime.
3. **Internal Network Latency**: Connections between your edge proxy and Kairos-rs reside in a private Docker bridge, Kubernetes pod mesh, or VPC with persistent HTTP keep-alive connections.
4. **WebSocket Compatibility**: Edge proxies seamlessly handle WebSocket connection upgrades for the real-time telemetry stream (`/ws/admin/metrics`).

---

## Native HTTPS vs. Reverse Proxy

| Capability | Kairos-rs Gateway | Edge Reverse Proxy (Nginx / Caddy) |
| :--- | :--- | :--- |
| **Incoming Inbound TLS** | Plain HTTP listener (`:5900`) | Native TLS 1.2 / TLS 1.3 Termination |
| **Outbound Upstream HTTPS** | Native (via `reqwest` + system TLS certs) | Transparent pass-through or re-encryption |
| **Automatic ACME (Let's Encrypt)** | Not natively integrated | Fully automated (zero-config in Caddy) |
| **HSTS Headers** | Built-in (`Strict-Transport-Security`) | Supported |
| **Real IP Preservation** | Extracts via `realip_remote_addr()` | Injects `X-Forwarded-For` and `X-Real-IP` |

---

## Production Configurations

### Option 1: Caddy (Recommended for Simplicity & HTTP/3)

Caddy provides automatic HTTPS, HTTP/3 (QUIC), and transparent WebSocket proxying out of the box.

Create a `Caddyfile`:

```caddy
gateway.yourdomain.com {
    # Automatic TLS with HTTP/3 support
    encode zstd gzip

    # Proxy all traffic to Kairos Gateway
    reverse_proxy localhost:5900 {
        header_up X-Forwarded-Proto {scheme}
        header_up X-Forwarded-Host {host}
        header_up X-Real-IP {remote_host}
    }
}
```

---

### Option 2: Nginx (Enterprise High-Throughput)

Nginx offers fine-grained control over TLS ciphers, session caching, and buffer tuning.

Create `/etc/nginx/conf.d/kairos.conf`:

```nginx
# Upstream definition for Kairos Gateway
upstream kairos_backend {
    server 127.0.0.1:5900;
    keepalive 64; # Maintain persistent connection pool
}

# WebSocket upgrade map
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    listen [::]:80;
    server_name gateway.yourdomain.com;
    return 301 https://$host$request_uri;
}

# HTTPS Server Block
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name gateway.yourdomain.com;

    # SSL Certificates
    ssl_certificate /etc/letsencrypt/live/gateway.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/gateway.yourdomain.com/privkey.pem;

    # Modern TLS Tuning
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_prefer_server_ciphers off;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;

    # Client payload size (must accommodate your largest model prompts)
    client_max_body_size 50M;

    location / {
        proxy_pass http://kairos_backend;
        proxy_http_version 1.1;

        # WebSocket support (Required for /ws/admin/metrics live telemetry)
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        # Client Identity Headers for Rate Limiting & IP Hash Load Balancing
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts for heavy LLM inference
        proxy_connect_timeout 10s;
        proxy_send_timeout 300s;
        proxy_read_timeout 300s;
        proxy_buffering off; # Low latency for streaming token responses
    }
}
```

---

### Option 3: Production Docker Compose Setup

Run the edge proxy and Kairos Gateway together in an isolated private bridge network:

```yaml
version: '3.8'

services:
  edge-proxy:
    image: caddy:alpine
    restart: always
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - gateway
    networks:
      - internal_net

  gateway:
    image: ghcr.io/danielsarmiento04/kairos-rs:latest
    restart: always
    environment:
      - KAIROS_HOST=0.0.0.0
      - KAIROS_PORT=5900
      - KAIROS_CONFIG_PATH=/app/config.json
    volumes:
      - ./config.json:/app/config.json:rw
    # Do NOT expose port 5900 to the public host
    expose:
      - "5900"
    networks:
      - internal_net

networks:
  internal_net:
    driver: bridge

volumes:
  caddy_data:
  caddy_config:
```

---

## Best Practices & Gotchas

1. **Forward `X-Real-IP` and `X-Forwarded-For`**:
   Kairos-rs rate limiting (`actix-governor`) and the `ip_hash` load balancing algorithm extract the client IP using `req.connection_info().realip_remote_addr()`. Ensuring your proxy forwards `X-Real-IP` or `X-Forwarded-For` guarantees that rate limiting applies to the actual client rather than the proxy.
2. **Disable Proxy Buffering for Streaming AI Prompts (`proxy_buffering off`)**:
   When using streaming LLM completions (`text/event-stream`), reverse proxy buffering can delay output tokens. Disabling buffering ensures tokens reach downstream clients with zero artificial lag.
3. **Configure Extended Timeouts for AI Inference**:
   Heavy LLM inference or multi-agent tasks can take 30 to 120+ seconds. Set `proxy_read_timeout` to at least 120s–300s to prevent premature `504 Gateway Timeout` errors.
4. **WebSocket Headers for Metrics**:
   The admin UI connects to `/ws/admin/metrics` for real-time telemetry. Always ensure the `Upgrade` and `Connection` headers are correctly mapped in your proxy.
