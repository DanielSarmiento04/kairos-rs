# Release Notes — v0.5.5

> **Focus**: Cache observability — expose HTTP response-cache hit / miss counters
> through Prometheus, the historical metrics store, and the admin cache endpoint
> (PR4a — Cache metrics).

## Highlights

- **Cache hit / miss counters** are now first-class metrics in the gateway.
  `MetricsCollector` exposes `cache_hits_total` and `cache_misses_total` atomic
  counters that are incremented by `RouteHandler` on every cache lookup.
- **Prometheus exposition** (`GET /metrics`) advertises the counters as
  `kairos_cache_hits_total` and `kairos_cache_misses_total`, both typed as
  `counter`, ready for scraping.
- **Historical metrics store** (`MetricsStore::list_metrics()`) now always
  returns `cache_hits_total` and `cache_misses_total`, even before any
  time-series data is recorded. The list backs the `/api/metrics/list`
  endpoint used by the dashboards.
- **Admin endpoint** `GET /api/cache` (already shipped) keeps working and is
  now covered by integration tests. It returns the live in-memory cache
  snapshot (`hits`, `misses`, `entries`, `hit_rate`).
- **Bug fix**: the Prometheus format string in `routes::metrics` had its
  positional arguments misaligned — the cache metric values were being
> printed against the wrong labels. Re-ordered the args so
> `kairos_cache_hits_total` and `kairos_cache_misses_total` now report the
> correct values, and `kairos_uptime_seconds` no longer swallows the
> circuit-breaker string.

## What's New

### Metrics

| Metric                          | Type    | Source                        |
|:--------------------------------|:--------|:------------------------------|
| `kairos_cache_hits_total`       | counter | `RouteHandler` cache lookup   |
| `kairos_cache_misses_total`     | counter | `RouteHandler` cache lookup   |
| `cache_hits_total` *(store)*    | name    | `MetricsStore::list_metrics`  |
| `cache_misses_total` *(store)*  | name    | `MetricsStore::list_metrics`  |

### Code Surface

- `MetricsCollector::cache_snapshot() -> (u64, u64, f64)` returns
  `(hits, misses, hit_rate)` derived from the two atomic counters.
> `hit_rate` is `0.0` when no traffic has been served yet.
- `RouteHandler::handle_request_internal` increments the right counter
> on each cache hit / miss, only when the route is eligible for caching
> (see `should_cache_route` in `services::cache`).

## Tests

Added `crates/kairos-rs/tests/cache_metrics_tests.rs` (6 cases):

1. `test_cache_snapshot_starts_at_zero` — fresh collector reports
   `(0, 0, 0.0)`.
2. `test_cache_snapshot_hit_rate_calculation` — `(3 hits, 1 miss) → 0.75`.
3. `test_metrics_collector_cache_counters_increment` — atomic semantics
   on the new counters.
4. `test_prometheus_exposition_contains_cache_metrics` — `/metrics`
   exposes both `kairos_cache_*_total` lines with the expected TYPE
   and sample values.
5. `test_metrics_store_list_includes_cache_metrics` — `list_metrics()`
   always returns the cache counter names.
6. `test_admin_cache_stats_endpoint_returns_snapshot` — `GET /api/cache`
> returns the in-memory cache snapshot as JSON.

All 6 cases pass on `cargo test -p kairos-rs --test cache_metrics_tests`.

## Notes

>- The wider regression suite (`cargo test -p kairos-rs`) currently fails
> to compile against unrelated test files (`simple_circuit_test`,
> `route_matcher_tests`, `config_*_tests`, ...) because they predate the
> `Router.cache` field added in v0.5.x and still construct `Router`
> without it. These failures are pre-existing and **not** caused by
> PR4a. They will be addressed in a follow-up that updates the legacy
> test fixtures to include `cache: None` (or a default `CacheConfig`).
- No Redis backend yet; `Response caching` is still in Phase 1
> (in-memory only, via `moka`). The Phase 1 markers in
> `services::cache` remain accurate.

