# Postman Testing Guide for Kairos-rs

This guide provides complete examples for testing Kairos-rs gateway using Postman with the current configuration.

## 🚀 Quick Setup

1. **Start Kairos-rs Gateway:**
   ```bash
   cd kairos-rs
   cargo run
   ```
   Server runs on `http://localhost:5900`

2. **Current Configuration:**
   The gateway is configured with these routes (from `config.json`):

## 📋 Postman Collection Examples

### 1. Public Route - Cat Images (No Auth) 🐱

**Request Details:**
- **Method:** `GET`
- **URL:** `http://localhost:5900/cats/200`
- **Headers:** None required

**Expected Response:**
- Status: `200 OK`
- Content: Cat image (HTTP status cat)
- This route forwards to `https://http.cat/200`

**Postman Setup:**
```
GET http://localhost:5900/cats/200
```

### 2. Protected Route - Cat Images (JWT Required) 🔒🐱

**Request Details:**
- **Method:** `GET` 
- **URL:** `http://localhost:5900/protected/cats/404`
- **Headers:** 
  - `Authorization: Bearer YOUR_JWT_TOKEN`

**Expected Response:**
- Without token: `401 Unauthorized`
- With valid token: `200 OK` + cat image

**Postman Setup:**
```
GET http://localhost:5900/protected/cats/404
Headers:
  Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjk5OTk5OTk5OTksImlzcyI6ImthaXJvcy1nYXRld2F5IiwiYXVkIjoiYXBpLWNsaWVudHMifQ.rJxLfHn8h6lUoFJmKrWOZfP5HnPnEKfP0OGNjEhTJfA
```

### 3. Local Service Route - Identity Registration 🔧

**Request Details:**
- **Method:** `POST` or `GET`
- **URL:** `http://localhost:5900/api/identity/register/v3`
- **Headers:** `Content-Type: application/json`
- **Body (for POST):** 
```json
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "securepassword"
}
```

**Expected Response:**
- Depends on your local service running on `localhost:3000`
- If service is down: Gateway will return appropriate error

**Postman Setup:**
```
POST http://localhost:5900/api/identity/register/v3
Headers:
  Content-Type: application/json
Body (raw JSON):
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "securepassword"
}
```

### 4. External Route - Google Homepage 🌐

**Request Details:**
- **Method:** `GET`
- **URL:** `http://localhost:5900/identity/register/v2`
- **Headers:** None required

**Expected Response:**
- Status: `200 OK`
- Content: Google homepage HTML
- This route forwards to `https://google.com/`

**Postman Setup:**
```
GET http://localhost:5900/identity/register/v2
```

## 🔑 JWT Token Generation

Since the protected routes require JWT authentication, you need a valid token. Here are options:

### Option 1: Use Test Token (Valid until 2033)
```
eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjk5OTk5OTk5OTksImlzcyI6ImthaXJvcy1nYXRld2F5IiwiYXVkIjoiYXBpLWNsaWVudHMifQ.rJxLfHn8h6lUoFJmKrWOZfP5HnPnEKfP0OGNjEhTJfA
```

### Option 2: Generate Your Own Token

Use [jwt.io](https://jwt.io) with these settings:

**Header:**
```json
{
  "typ": "JWT",
  "alg": "HS256"
}
```

**Payload:**
```json
{
  "sub": "1234567890",
  "iat": 1516239022,
  "exp": 9999999999,
  "iss": "kairos-gateway",
  "aud": "api-clients"
}
```

**Secret:** `your-super-secure-jwt-secret-key-must-be-at-least-32-characters-long`

### Option 3: Create Token with curl
```bash
# This would require implementing a token endpoint
# For now, use the provided test token above
```

## 📊 Testing Rate Limiting

The gateway has rate limiting enabled. To test it:

1. **Send multiple rapid requests:**
```bash
for i in {1..10}; do curl http://localhost:5900/cats/200; done
```

2. **Expected behavior:**
- First requests: `200 OK`
- After limit: `429 Too Many Requests`

## 🔍 Health Check Endpoints

**Health Check:**
```
GET http://localhost:5900/health
```
Expected: `200 OK` with status information

**Metrics (Prometheus format):**
```
GET http://localhost:5900/metrics
```
Expected: `200 OK` with Prometheus metrics

## 🧪 Complete Postman Collection JSON

You can import this collection directly into Postman:

```json
{
  "info": {
    "name": "Kairos-rs API Gateway Tests",
    "description": "Test collection for Kairos-rs gateway functionality",
    "version": "1.0.0"
  },
  "item": [
    {
      "name": "Public Cat Image",
      "request": {
        "method": "GET",
        "url": "http://localhost:5900/cats/200",
        "description": "Test public route without authentication"
      }
    },
    {
      "name": "Protected Cat Image",
      "request": {
        "method": "GET",
        "url": "http://localhost:5900/protected/cats/404",
        "header": [
          {
            "key": "Authorization",
            "value": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjk5OTk5OTk5OTksImlzcyI6ImthaXJvcy1nYXRld2F5IiwiYXVkIjoiYXBpLWNsaWVudHMifQ.rJxLfHn8h6lUoFJmKrWOZfP5HnPnEKfP0OGNjEhTJfA"
          }
        ],
        "description": "Test protected route with JWT authentication"
      }
    },
    {
      "name": "Identity Registration POST",
      "request": {
        "method": "POST",
        "url": "http://localhost:5900/api/identity/register/v3",
        "header": [
          {
            "key": "Content-Type",
            "value": "application/json"
          }
        ],
        "body": {
          "mode": "raw",
          "raw": "{\n  \"username\": \"testuser\",\n  \"email\": \"test@example.com\",\n  \"password\": \"securepassword\"\n}"
        },
        "description": "Test local service route with POST data"
      }
    },
    {
      "name": "Google Homepage Route",
      "request": {
        "method": "GET",
        "url": "http://localhost:5900/identity/register/v2",
        "description": "Test external service routing to Google"
      }
    },
    {
      "name": "Health Check",
      "request": {
        "method": "GET",
        "url": "http://localhost:5900/health",
        "description": "Check gateway health status"
      }
    },
    {
      "name": "Metrics Endpoint",
      "request": {
        "method": "GET",
        "url": "http://localhost:5900/metrics",
        "description": "Get Prometheus metrics"
      }
    }
  ]
}
```

## ⚡ Advanced Testing Scenarios

### 1. Test Circuit Breaker
1. Stop your local service on port 3000
2. Make requests to `http://localhost:5900/api/identity/register/v3`
3. After several failures, circuit breaker should open
4. Subsequent requests will fail fast

### 2. Test Rate Limiting
1. Use Postman Runner or Newman to send rapid requests
2. Monitor for `429 Too Many Requests` responses
3. Check `/metrics` endpoint for rate limit statistics

### 3. Test JWT Validation
1. Try requests with no Authorization header
2. Try with malformed JWT tokens
3. Try with expired tokens
4. Try with wrong secret signatures

## 🚨 Troubleshooting

**Common Issues:**

1. **Connection Refused:**
   - Make sure Kairos-rs is running: `cargo run`
   - Check it's listening on port 5900

2. **JWT Authentication Fails:**
   - Verify token is properly formatted
   - Check the secret matches configuration
   - Ensure required claims (sub, exp) are present

3. **Route Not Found:**
   - Verify URL matches exactly the external_path in config
   - Check HTTP method is allowed in configuration

4. **Local Service Errors:**
   - Make sure your local service is running on port 3000
   - Check service is accessible directly

## 🎯 Expected Test Results

| Route | Status | Response |
|-------|--------|----------|
| `/cats/200` | 200 OK | Cat image |
| `/protected/cats/404` (no auth) | 401 Unauthorized | Auth error |
| `/protected/cats/404` (with JWT) | 200 OK | Cat image |
| `/api/identity/register/v3` | Depends on local service | Service response |
| `/identity/register/v2` | 200 OK | Google homepage |
| `/health` | 200 OK | Health status |
| `/metrics` | 200 OK | Prometheus metrics |

Happy testing! 🚀