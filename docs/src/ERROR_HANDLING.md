# Errors & Diagnostics

When Kairos Gateway encounters an issue—whether during routing, validating payloads, or communicating with upstream servers—it replies with a consistent JSON structured response.

This ensures callers and downstream observability tools can programmatically parse and respond to Gateway failures.

## Standard JSON Error Format

All gateway-originated error messages follow this schema:

```json
{
  "error": "Detailed error message",
  "type": "error_type_identifier", 
  "timestamp": "2024-03-15T10:30:00+00:00",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

- `error`: A human-readable description indicating what went wrong.
- `type`: A stable machine-readable identifier categorizing the fault type.
- `timestamp`: The exact UTC timeline of the rejection (RFC 3339 formatted).
- `request_id`: A UUID globally tracking the lifecycle of the transaction. (Very useful when filtering Gateway logs).

## Error Types and HTTP Codes

Kairos Gateway automatically propagates these distinct error codes based on internal failure triggers.

| Error `type` | HTTP Status | Description |
|:---:|:---:|:---|
| `route_not_found` | `404 Not Found` | The requested path did not match any of the configured `external_path` logic within `config.json`. |
| `method_not_allowed` | `405 Method Not Allowed` | Rejection due to caller using an HTTP Method (e.g., `DELETE`) not listed in the router's `methods` array. |
| `bad_request` | `400 Bad Request` | Client request failed validation (e.g., malformed payloads). |
| `timeout` | `504 Gateway Timeout` | The upstream service did not respond within the allocated timeframe parameters. |
| `upstream` | `502 Bad Gateway` | Network errors (Connection refused) or DNS failures targeting the intended upstream host. |
| `config` | `502 Bad Gateway` | The matched route has a systematically invalid internal schema (preventing safe execution). |
| `circuit_open` | `503 Service Unavailable` | A previously unhealthy target triggered the gateway's Circuit Breaker, actively preventing further traffic. |

### Circuit Breakers & Upstream Failures

When utilizing active retries and Load Balancing, Kairos monitors connection stability.

If `type: circuit_open` is surfaced, the target backend `Service {host}:{port} is currently unavailable (circuit breaker open)`.
The gateway halts routing momentarily to let the upstream backend breathe and recuperate. Wait for the circuit breaker to enter the `HalfOpen` state shortly after.

### Debugging using Logs

Ensure `RUST_LOG=info` or `RUST_LOG=debug` is set in your environment prior to launching Kairos.

When an error block is emitted to the client:
1. Copy the emitted `request_id`.
2. Grep your server's standard out: `grep "550e8400-e29b..." /var/log/kairos.log`
3. Track the Gateway internal timeline pinpointing if rejection originated from `RateLimiter`, `AiRouter`, or the `HttpServiceProxy`.
