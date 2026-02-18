# AI-Powered Request Routing Guide

Kairos-rs (v0.3.0+) introduces pioneering AI-driven routing capabilities, allowing the gateway to make intelligent decisions about where to forward requests based on their content, intent, and complexity.

This guide explains how to configure and use the AI orchestration layer.

## Overview

The AI Routing feature uses Large Language Models (LLMs) to analyze incoming HTTP requests (method, path, headers, and body preview) and select the most appropriate backend service from a list of available candidates.

**Key Use Cases:**
- **Content-Based Routing**: Route complex queries to GPT-4 and simple ones to faster/cheaper models.
- **Intent Analysis**: Classify user intent and route to specialized microservices.
- **Smart Load Distribution**: Intelligently distribute load based on request difficulty rather than just connection count.

## Configuration

To enable AI routing, you need to configure two parts:
1. Global AI Settings (Provider & Model)
2. Per-Route AI Policies

### 1. Global AI Settings

Add the `ai` section to your `config.json`. You can support various providers via `rig-core`.

```json
{
  "version": 1,
  "ai": {
    "provider": "openai",
    "model": "gpt-4o",
    "api_key": "sk-..." // Optional: prefer using env vars
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
    "fallback_backend_index": 0
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

## Performance Considerations

- **Latency**: AI calls add latency. Use this feature for heavy or complex operations where the routing decision adds significant value (e.g., choosing between a $0.01 model and a $0.001 model).
- **Caching**: Ensure your AI provider (or an intermediate cache) is performant.
- **Fast Providers**: For real-time routing, consider using fast providers like Groq or lighter models like `gpt-3.5-turbo`.

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
