# AI-Powered Request Routing & Streaming Guide

Kairos-rs (v0.4.1+) features an AI-driven routing and orchestration layer powered by `rig-core` 0.44.0. It provides intelligent request classification, multi-provider automated failover chains, and low-latency Server-Sent Events (SSE) streaming proxying.

This guide explains how to configure and use the AI orchestration layer.

## Overview

The AI Routing feature leverages Large Language Models (LLMs) to analyze incoming HTTP requests (method, path, headers, and body preview) and select the most appropriate backend service from a list of available candidates.

**Key Capabilities:**
- **Content-Based Routing**: Route complex queries to frontier models and simpler ones to faster, cheaper endpoints.
- **Intent Analysis**: Classify user intent and route to specialized microservices.
- **Multi-Provider Failover Chains**: Automatically fail over from primary to secondary providers (e.g. OpenAI → Anthropic → Groq) upon rate limits, 5xx errors, or timeouts.
- **Real-Time Token Streaming**: Forward chunked/SSE responses without memory buffering, setting anti-buffering headers and bypassing response caches.

## Configuration

To enable AI routing and orchestration, configure:
1. Global AI Settings (Primary Provider, Default Model, and Global Fallbacks)
2. Per-Route AI Policies (Enabled, Strategy, Provider overrides, Fallback providers, and Streaming toggle)

### 1. Global AI Settings

Add the `ai` section to your `config.json` with primary and optional fallback providers:

```json
{
  "version": 1,
  "ai": {
    "provider": "openai",
    "model": "gpt-4o",
    "api_key": "sk-...",
    "fallback_providers": [
      {
        "provider": "anthropic",
        "model": "claude-3-5-sonnet",
        "api_key": "sk-ant-..."
      },
      {
        "provider": "groq",
        "model": "llama-3.3-70b-versatile"
      }
    ]
  },
  "routers": [...]
}
```

**Supported Providers:**
- `openai` (e.g., `gpt-4o`, `gpt-3.5-turbo`)
- `anthropic` (e.g., `claude-3-opus`, `claude-3-sonnet`)
- `cohere`
- `perplexity`
- `mistral`
- `groq`
- `xai`

**Environment Variables:**
It is recommended to set API keys via environment variables instead of hardcoding them:
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`
- `COHERE_API_KEY`
- ...etc.

### 2. Route Configuration

Enable AI routing for a specific route by adding an `ai_policy`.

```json
{
  "host": "http://main-entry-point",
  "port": 80,
  "external_path": "/api/chat",
  "internal_path": "/v1/chat",
  "methods": ["POST"],
  "ai_policy": {
    "enabled": true,
    "strategy": {
      "content_analysis": {
        "model": "gpt-4o" 
      }
    },
    "provider": "openai",
    "fallback_backend_index": 0,
    "streaming": true,
    "fallback_providers": [
      {
        "provider": "anthropic",
        "model": "claude-3-5-sonnet"
      }
    ]
  },
  "backends": [
    {
      "host": "http://fast-service",
      "port": 8081,
      "weight": 1
    },
    {
      "host": "http://smart-service",
      "port": 8082,
      "weight": 1
    }
  ]
}
```

**How it works:**
1. A request arrives at `/api/chat`.
2. The gateway captures the request content (headers + first 500 chars of body).
3. It sends this context to the AI model with a prompt to select the best backend index.
4. The AI analyzes the request and returns an index (e.g., `1` for "smart-service").
5. The gateway forwards the request to the selected backend.
6. If the AI fails or is too slow, it falls back to `fallback_backend_index` (index 0).

## SSE Streaming & Token Passthrough

When streaming LLM completions or chat interactions (`text/event-stream`, `application/x-ndjson`, or route policy `streaming: true`):
1. **Zero-Buffering**: Upstream chunks are yielded directly via `futures_util::TryStream` without buffering the entire payload into RAM.
2. **Cache Bypass**: Streaming responses bypass the `ResponseCache` entirely to ensure real-time delivery and prevent stale or corrupted partial cached entries.
3. **Anti-Buffering Headers**: Kairos automatically injects downstream proxy headers:
   - `X-Accel-Buffering: no` (disables buffering in Nginx reverse proxies)
   - `Cache-Control: no-cache, no-transform`
   - `X-Streaming: true`
4. **Time To First Token (TTFT)**: Minimized to network RTT without intermediate serialization bottlenecks.

## Performance Considerations

- **Latency**: AI calls add latency (typically 200ms - 1000ms depending on the provider and model). Use this feature for heavy or complex operations where the routing decision adds significant value (e.g., choosing between a $0.01 model and a $0.001 model, or routing a complex database query to a specialized read replica).
- **Cost**: Every AI routing decision consumes tokens. Be mindful of the volume of traffic hitting AI-enabled routes.
- **Fallback**: Always configure a `fallback_backend_index`. If the AI provider is down, times out, or returns an unparseable response, Kairos will instantly route the request to the fallback backend to ensure high availability.
- **Caching**: Ensure your AI provider (or an intermediate cache) is performant.
- **Fast Providers**: For real-time routing, consider using fast providers like Groq or lighter models like `gpt-3.5-turbo`.

## Supported Providers

Kairos Gateway integrates with the `rig-core` library, providing out-of-the-box support for the following AI providers. You can configure the provider globally or override it per-route.

| Provider | Config Value | Environment Variable | Example Models |
| :--- | :--- | :--- | :--- |
| **OpenAI** | `openai` | `OPENAI_API_KEY` | `gpt-4o`, `gpt-3.5-turbo` |
| **Anthropic** | `anthropic` | `ANTHROPIC_API_KEY` | `claude-3-opus`, `claude-3-sonnet` |
| **Cohere** | `cohere` | `COHERE_API_KEY` | `command-r`, `command-r-plus` |
| **Perplexity** | `perplexity` | `PERPLEXITY_API_KEY` | `llama-3-sonar-large-32k-online` |
| **Mistral** | `mistral` | `MISTRAL_API_KEY` | `mistral-large-latest` |
| **Groq** | `groq` | `GROQ_API_KEY` | `llama3-70b-8192` |
| **xAI** | `xai` | `XAI_API_KEY` | `Grok-1` |

*Note: It is highly recommended to use environment variables for API keys rather than hardcoding them in `config.json`.*

## Example: Tiered Support Routing

Scenario: You have a generic support endpoint. You want "urgent" or "premium" tickets to go to a specialized high-priority service, and regular tickets to a standard service.

```json
{
  "external_path": "/api/tickets",
  "ai_policy": {
    "enabled": true,
    "strategy": { "content_analysis": { "model": "gpt-3.5-turbo" } },
    "provider": "openai",
    "fallback_backend_index": 0
  },
  "backends": [
    {
      "host": "http://standard-support",
      "port": 8080,
      "weight": 1
    },
    {
      "host": "http://premium-support",
      "port": 8080,
      "weight": 1
    }
  ]
}
```

The AI prompt explicitly asks the model to classify the request and pick the backend. Kairos-rs handles the prompting engineering internally to ensure the model returns a valid index.
