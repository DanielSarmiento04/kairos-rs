//! Integration tests for the per-route analytics (PR4b — Per-route metrics).
//!
//! Validates that:
//! - `MetricsCollector::record_route_request` updates the per-route counters
//!   (total / success / error / latency buckets / 4xx / 5xx) for the
//!   `route_id` it is given.
//! - `MetricsCollector::record_route_cache_hit` / `record_route_cache_miss`
//!   increment the per-route cache counters without touching other routes.
//! - `MetricsCollector::route_metrics_snapshot` returns snapshots sorted
//!   by `route_id` for stable JSON output.
//! - `MetricsCollector::route_prometheus_text` emits Prometheus exposition
//!   format with `route` labels and a HELP/TYPE preamble for each metric.
//! - Two different `route_id`s are isolated from each other: counters for
//!   one route do not leak into another.

use kairos_rs::routes::metrics::MetricsCollector;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// Fresh `MetricsCollector` has an empty `routes` map and an empty snapshot.
#[actix_web::test]
async fn test_route_metrics_starts_empty() {
    let m = MetricsCollector::default();
    let snapshots = m.route_metrics_snapshot();
    assert!(
        snapshots.is_empty(),
        "fresh collector must have no per-route metrics, got {snapshots:?}"
    );
    assert_eq!(
        m.route_prometheus_text(),
        "",
        "fresh collector must emit an empty Prometheus section"
    );
}

/// `get_or_create_route` lazily inserts a `RouteMetrics` and returns the
/// same `Arc` on subsequent calls.
#[actix_web::test]
async fn test_get_or_create_route_is_idempotent() {
    let m = MetricsCollector::default();
    let a = m.get_or_create_route("/users/{id}");
    let b = m.get_or_create_route("/users/{id}");
    assert!(Arc::ptr_eq(&a, &b), "second call must return the same Arc");
    let c = m.get_or_create_route("/orders");
    assert!(
        !Arc::ptr_eq(&a, &c),
        "different route_id must yield a different Arc"
    );
    let snapshots = m.route_metrics_snapshot();
    assert_eq!(snapshots.len(), 2, "two distinct routes tracked");
}

/// `record_route_request` increments total + success + sum + the right
/// latency bucket (100ms) for a 2xx response.
#[actix_web::test]
async fn test_record_route_request_success() {
    let m = MetricsCollector::default();
    m.record_route_request("/health", true, Duration::from_millis(50), 200);
    let rm = m.get_or_create_route("/health");
    assert_eq!(rm.requests_total.load(Ordering::Relaxed), 1);
    assert_eq!(rm.requests_success.load(Ordering::Relaxed), 1);
    assert_eq!(rm.requests_error.load(Ordering::Relaxed), 0);
    assert_eq!(rm.response_time_sum.load(Ordering::Relaxed), 50);
    assert_eq!(rm.response_time_bucket_100ms.load(Ordering::Relaxed), 1);
    assert_eq!(rm.response_time_bucket_500ms.load(Ordering::Relaxed), 1);
    assert_eq!(rm.response_time_bucket_1s.load(Ordering::Relaxed), 1);
    assert_eq!(rm.response_time_bucket_5s.load(Ordering::Relaxed), 1);
    assert_eq!(rm.response_time_bucket_inf.load(Ordering::Relaxed), 0);
    assert_eq!(rm.http_4xx_errors.load(Ordering::Relaxed), 0);
    assert_eq!(rm.http_5xx_errors.load(Ordering::Relaxed), 0);
}

/// A 4xx response increments `requests_error` and `http_4xx_errors` but
/// not `http_5xx_errors`.
#[actix_web::test]
async fn test_record_route_request_4xx() {
    let m = MetricsCollector::default();
    m.record_route_request("/api", false, Duration::from_millis(10), 404);
    let rm = m.get_or_create_route("/api");
    assert_eq!(rm.requests_total.load(Ordering::Relaxed), 1);
    assert_eq!(rm.requests_error.load(Ordering::Relaxed), 1);
    assert_eq!(rm.http_4xx_errors.load(Ordering::Relaxed), 1);
    assert_eq!(rm.http_5xx_errors.load(Ordering::Relaxed), 0);
}

/// A 5xx response increments `requests_error` and `http_5xx_errors` but
/// not `http_4xx_errors`.
#[actix_web::test]
async fn test_record_route_request_5xx() {
    let m = MetricsCollector::default();
    m.record_route_request("/api", false, Duration::from_millis(75), 502);
    let rm = m.get_or_create_route("/api");
    assert_eq!(rm.requests_total.load(Ordering::Relaxed), 1);
    assert_eq!(rm.requests_error.load(Ordering::Relaxed), 1);
    assert_eq!(rm.http_4xx_errors.load(Ordering::Relaxed), 0);
    assert_eq!(rm.http_5xx_errors.load(Ordering::Relaxed), 1);
}

/// Latency bucket boundaries: a request that takes >5000ms goes into
/// the `+Inf` bucket but not the others.
#[actix_web::test]
async fn test_record_route_request_infinite_bucket() {
    let m = MetricsCollector::default();
    m.record_route_request("/slow", true, Duration::from_millis(6000), 200);
    let rm = m.get_or_create_route("/slow");
    assert_eq!(rm.response_time_bucket_100ms.load(Ordering::Relaxed), 0);
    assert_eq!(rm.response_time_bucket_500ms.load(Ordering::Relaxed), 0);
    assert_eq!(rm.response_time_bucket_1s.load(Ordering::Relaxed), 0);
    assert_eq!(rm.response_time_bucket_5s.load(Ordering::Relaxed), 0);
    assert_eq!(rm.response_time_bucket_inf.load(Ordering::Relaxed), 1);
}

/// `record_route_cache_hit` / `record_route_cache_miss` increment the
/// per-route cache counters and nothing else.
#[actix_web::test]
async fn test_record_route_cache_hit_miss() {
    let m = MetricsCollector::default();
    m.record_route_cache_hit("/cached");
    m.record_route_cache_hit("/cached");
    m.record_route_cache_miss("/cached");
    let rm = m.get_or_create_route("/cached");
    assert_eq!(rm.cache_hits_total.load(Ordering::Relaxed), 2);
    assert_eq!(rm.cache_misses_total.load(Ordering::Relaxed), 1);
    assert_eq!(
        rm.requests_total.load(Ordering::Relaxed),
        0,
        "cache hit/miss must not touch request counters"
    );
}

/// Counters for two distinct routes are isolated from each other.
#[actix_web::test]
async fn test_routes_are_isolated() {
    let m = MetricsCollector::default();
    m.record_route_request("/a", true, Duration::from_millis(10), 200);
    m.record_route_request("/a", true, Duration::from_millis(20), 200);
    m.record_route_request("/b", false, Duration::from_millis(30), 500);
    let a = m.get_or_create_route("/a");
    let b = m.get_or_create_route("/b");
    assert_eq!(a.requests_total.load(Ordering::Relaxed), 2);
    assert_eq!(a.requests_success.load(Ordering::Relaxed), 2);
    assert_eq!(b.requests_total.load(Ordering::Relaxed), 1);
    assert_eq!(b.requests_error.load(Ordering::Relaxed), 1);
    assert_eq!(b.http_5xx_errors.load(Ordering::Relaxed), 1);
}

/// `route_metrics_snapshot` returns snapshots sorted by `route_id` for
/// stable JSON output, regardless of insertion order.
#[actix_web::test]
async fn test_route_metrics_snapshot_sorted() {
    let m = MetricsCollector::default();
    m.record_route_request("/zeta", true, Duration::from_millis(1), 200);
    m.record_route_request("/alpha", true, Duration::from_millis(1), 200);
    m.record_route_request("/mu", true, Duration::from_millis(1), 200);
    let snapshots = m.route_metrics_snapshot();
    let ids: Vec<&str> = snapshots.iter().map(|s| s.route_id.as_str()).collect();
    assert_eq!(ids, vec!["/alpha", "/mu", "/zeta"]);
}

/// `route_prometheus_text` emits HELP/TYPE lines and per-route sample
/// lines with the correct `route="..."` labels.
#[actix_web::test]
async fn test_route_prometheus_text_format() {
    let m = MetricsCollector::default();
    m.record_route_request("/users/{id}", true, Duration::from_millis(80), 200);
    m.record_route_cache_hit("/users/{id}");
    m.record_route_cache_miss("/users/{id}");
    let text = m.route_prometheus_text();
    assert!(text.contains("# HELP kairos_route_requests_total"));
    assert!(text.contains("# TYPE kairos_route_requests_total counter"));
    assert!(text.contains("kairos_route_requests_total{route=\"/users/{id}\"} 1"));
    assert!(text.contains("# TYPE kairos_route_cache_hits_total counter"));
    assert!(text.contains("kairos_route_cache_hits_total{route=\"/users/{id}\"} 1"));
    assert!(text.contains("kairos_route_cache_misses_total{route=\"/users/{id}\"} 1"));
    // Histogram bucket lines include the `route` label. The 80ms request
    // falls into the cumulative `le="100"`, `le="500"`, `le="1000"`,
    // `le="5000"` buckets, but NOT into the `+Inf` bucket.
    assert!(text.contains("kairos_route_response_time_bucket{route=\"/users/{id}\",le=\"100\"} 1"));
    assert!(text.contains("kairos_route_response_time_bucket{route=\"/users/{id}\",le=\"500\"} 1"));
    assert!(text.contains("kairos_route_response_time_bucket{route=\"/users/{id}\",le=\"1000\"} 1"));
    assert!(text.contains("kairos_route_response_time_bucket{route=\"/users/{id}\",le=\"5000\"} 1"));
    assert!(text.contains("kairos_route_response_time_bucket{route=\"/users/{id}\",le=\"+Inf\"} 0"));
}
