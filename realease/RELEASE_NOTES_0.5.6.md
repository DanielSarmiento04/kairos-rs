# Release Notes — v0.5.6

> **Focus**: Per-route analytics (PR4b — Per-route metrics).
> A new `RouteMetrics` struct tracks requests, latency, error rate, and
> cache hit/miss per route, exposed via Prometheus and a new admin
> endpoint.

## Highlights

- **Per-route metrics breakdown.** `MetricsCollector` now holds a
  `RwLock<HashMap<String, Arc<RouteMetrics>>>` keyed by
  `route.external_path`. The `RouteMetrics` struct mirrors the subset
  of `MetricsCollector` counters that are useful for per-route
  analysis: total / success / error request counts, latency histogram
  buckets (100ms / 500ms / 1s / 5s / +Inf), 4xx / 5xx error split, and
  cache hit / miss counts.
- **Prometheus exposition** (`GET /metrics`) now advertises a
  per-route section with `route="..."` labels. New series:
  - `kairos_route_requests_total{route="..."}` (counter)
  - `kairos_route_requests_success_total{route="..."}` (counter)
  - `kairos_route_requests_error_total{route="..."}` (counter)
  - `kairos_route_response_time_avg{route="..."}` (gauge)
  - `kairos_route_response_time_bucket{route="...",le="..."}` (histogram)
  - `kairos_route_cache_hits_total{route="..."}` (counter)
  - `kairos_route_cache_misses_total{route="..."}` (counter)
- **New admin endpoint** `GET /api/metrics/routes` returns a JSON
  payload of `RouteMetricsSnapshot` objects, sorted by `route_id` for
  stable output. Each snapshot exposes `requests_total`,
  `requests_success`, `requests_error`, `response_time_sum_ms`,
  `avg_response_time_ms`, `success_rate`, `error_rate`, latency
  buckets, `http_4xx_errors`, `http_5xx_errors`,
  `cache_hits_total`, `cache_misses_total`, and `cache_hit_rate`.
- **Wire-up in `RouteHandler::handle_request_internal`**:
  - `record_route_cache_hit(&route.external_path)` on the cache-hit
    branch (matches the existing `cache_hits_total.fetch_add`).
  - `record_route_cache_miss(&route.external_path)` on the cache-miss
    branch (matches the existing `cache_misses_total.fetch_add`).
  - `record_route_request(&route.external_path, success, duration, status_code)`
    on the upstream success path. Per-route duration is captured
    via a new `route_start: Instant` at the top of the function so
    error paths can be added incrementally without breaking the
    success path.

## What's New

### Code Surface

- `MetricsCollector::get_or_create_route(route_id) -> Arc<RouteMetrics>`
  — lazy insertion; same `Arc` returned on subsequent calls.
- `MetricsCollector::record_route_request(route_id, success, response_time, status_code)`
  — mirrors the global `record_request` semantics, keyed by
  `route_id`.
- `MetricsCollector::record_route_cache_hit(route_id)` /
  `record_route_cache_miss(route_id)` — cache counters, per route.
- `MetricsCollector::route_metrics_snapshot() -> Vec<RouteMetricsSnapshot>`
  — sorted by `route_id` for stable JSON output.
- `MetricsCollector::route_prometheus_text() -> String` — Prometheus
  exposition section, or empty string when no route has been seen yet.
- `route_metrics_endpoint` (registered at `/api/metrics/routes`) —
  admin JSON endpoint.
- `escape_prometheus_label` helper — backslash + double-quote
  escaping for `route="..."` label values.

### Data Model

- `RouteMetrics` struct (15 atomic counters + `route_id: String`).
- `RouteMetricsSnapshot` struct (serde::Serialize-friendly).

## Tests

Added `crates/kairos-rs/tests/per_route_metrics_tests.rs` (10 cases,
all pass):

1. `test_route_metrics_starts_empty` — fresh collector has an empty
   `routes` map and an empty Prometheus section.
2. `test_get_or_create_route_is_idempotent` — second call returns
   the same `Arc`; different `route_id` yields a different `Arc`.
3. `test_record_route_request_success` — total / success / sum /
   latency buckets are correctly populated for a 2xx response.
4. `test_record_route_request_4xx` — 4xx increments `http_4xx_errors`
   but not `http_5xx_errors`.
5. `test_record_route_request_5xx` — 5xx increments `http_5xx_errors`
   but not `http_4xx_errors`.
6. `test_record_route_request_infinite_bucket` — a 6s request lands
   in the `+Inf` bucket only.
7. `test_record_route_cache_hit_miss` — cache counters work, request
   counters are untouched.
8. `test_routes_are_isolated` — counters for `/a` and `/b` are
   independent.
9. `test_route_metrics_snapshot_sorted` — snapshots are sorted by
   `route_id` regardless of insertion order.
10. `test_route_prometheus_text_format` — Prometheus exposition
    contains HELP / TYPE / sample lines with the expected labels and
    latency bucket values.

## Notes

- **Error paths in `handle_request_internal`** are not yet wired for
  `record_route_request` (only the upstream success path). The error
  branches (`return Err(GatewayError::Upstream { ... })` and the
  circuit-breaker open branch) still record the global counters in
  `handle_request` but skip the per-route counter. This is a
  known limitation tracked as a follow-up.
- The `route` Prometheus label is escaped for backslashes and double
  quotes so that paths containing those characters do not corrupt the
  exposition format.
- The PR4b per-route breakdown complements the global counters
  exposed in v0.5.5 (PR4a — cache metrics). Together they give
  operators visibility into both gateway-wide health and
  per-route behaviour.
