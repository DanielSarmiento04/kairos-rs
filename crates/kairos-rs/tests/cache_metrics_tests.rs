//! Integration tests for the cache metrics exposure (PR4a — Cache metrics).
//!
//! Validates that:
//! - `MetricsCollector` exposes `cache_hits_total` / `cache_misses_total`
//!   counters and a `cache_snapshot()` helper that derives hit rate.
//! - The Prometheus `/metrics` endpoint advertises `kairos_cache_hits_total`
//!   and `kairos_cache_misses_total`.
//! - The historical `MetricsStore::list_metrics()` always returns
//!   `cache_hits_total` and `cache_misses_total`, even before any
//!   time-series data is recorded.
//! - The admin endpoint `GET /api/cache` (registered in
//!   `routes::management::configure_management`) returns a JSON payload
//!   with the in-memory cache snapshot.

use actix_web::{test, web, App};
use kairos_rs::routes::{management, metrics};
use kairos_rs::services::metrics_store::MetricsStore;
use std::sync::atomic::Ordering;

/// Fresh `MetricsCollector` starts with zero cache counters and a
/// well-defined hit rate of `0.0`.
#[actix_web::test]
async fn test_cache_snapshot_starts_at_zero() {
    let m = metrics::MetricsCollector::default();
    let (hits, misses, hit_rate) = m.cache_snapshot();
    assert_eq!(hits, 0, "hits must start at 0");
    assert_eq!(misses, 0, "misses must start at 0");
    assert!(
        (hit_rate - 0.0).abs() < 1e-9,
        "hit_rate must be 0.0 with no traffic, got {hit_rate}"
    );
}

/// `hit_rate = hits / (hits + misses)` when there is traffic.
#[actix_web::test]
async fn test_cache_snapshot_hit_rate_calculation() {
    let m = metrics::MetricsCollector::default();
    // 3 hits, 1 miss => 0.75
    m.cache_hits_total.fetch_add(3, Ordering::Relaxed);
    m.cache_misses_total.fetch_add(1, Ordering::Relaxed);
    let (hits, misses, hit_rate) = m.cache_snapshot();
    assert_eq!(hits, 3);
    assert_eq!(misses, 1);
    assert!(
        (hit_rate - 0.75).abs() < 1e-9,
        "hit_rate must be 0.75, got {hit_rate}"
    );
}

/// Counter increments behave like atomic counters (concurrent-safe).
#[actix_web::test]
async fn test_metrics_collector_cache_counters_increment() {
    let m = metrics::MetricsCollector::default();
    m.cache_hits_total.fetch_add(5, Ordering::Relaxed);
    m.cache_misses_total.fetch_add(2, Ordering::Relaxed);
    assert_eq!(m.cache_hits_total.load(Ordering::Relaxed), 5);
    assert_eq!(m.cache_misses_total.load(Ordering::Relaxed), 2);
}

/// Prometheus exposition includes `kairos_cache_hits_total` and
/// `kairos_cache_misses_total` with the right TYPE and sample values.
#[actix_web::test]
async fn test_prometheus_exposition_contains_cache_metrics() {
    let m = metrics::MetricsCollector::default();
    m.cache_hits_total.fetch_add(7, Ordering::Relaxed);
    m.cache_misses_total.fetch_add(3, Ordering::Relaxed);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(m))
            .configure(metrics::configure_metrics),
    )
    .await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();

    // HELP and TYPE lines
    assert!(
        body_str.contains("# HELP kairos_cache_hits_total"),
        "missing HELP for kairos_cache_hits_total"
    );
    assert!(
        body_str.contains("# TYPE kairos_cache_hits_total counter"),
        "kairos_cache_hits_total must be a counter"
    );
    assert!(
        body_str.contains("# HELP kairos_cache_misses_total"),
        "missing HELP for kairos_cache_misses_total"
    );
    assert!(
        body_str.contains("# TYPE kairos_cache_misses_total counter"),
        "kairos_cache_misses_total must be a counter"
    );

    // Sample line values
    assert!(
        body_str.contains("kairos_cache_hits_total 7"),
        "kairos_cache_hits_total sample must equal 7"
    );
    assert!(
        body_str.contains("kairos_cache_misses_total 3"),
        "kairos_cache_misses_total sample must equal 3"
    );
}

/// `MetricsStore::list_metrics()` must always include the cache counter
/// names, even with zero recorded time-series data.
#[actix_web::test]
async fn test_metrics_store_list_includes_cache_metrics() {
    use chrono::Duration;
    let store = MetricsStore::new(100, Duration::hours(1));
    let names = store.list_metrics();
    assert!(
        names.iter().any(|n| n == "cache_hits_total"),
        "list_metrics must include cache_hits_total, got {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "cache_misses_total"),
        "list_metrics must include cache_misses_total, got {names:?}"
    );
}

/// `GET /api/cache` returns a JSON object with `success: true` and the
/// in-memory cache snapshot (`hits`, `misses`, `entries`, `hit_rate`).
#[actix_web::test]
async fn test_admin_cache_stats_endpoint_returns_snapshot() {
    use kairos_rs::models::router::{CacheConfig, Protocol, Router};
    use kairos_rs::services::http::RouteHandler;

    // Build a RouteHandler with a route that opts in to caching so the
    // in-memory cache is constructed.
    let routes = vec![Router {
        host: Some("http://127.0.0.1".into()),
        port: Some(80),
        backends: None,
        protocol: Protocol::Http,
        load_balancing_strategy: Default::default(),
        external_path: "/cached".into(),
        internal_path: "/".into(),
        methods: vec!["GET".into()],
        auth_required: false,
        retry: None,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
        cache: Some(CacheConfig {
            enabled: true,
            ttl_secs: 60,
            max_size: 16,
        }),
    }];
    let handler = RouteHandler::new(routes, 30);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(handler))
            .configure(management::configure_management),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/cache").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body = test::read_body(resp).await;
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["success"], serde_json::Value::Bool(true));
    let stats = &payload["stats"];
    assert!(stats.is_object(), "stats must be a JSON object, got {stats}");
    assert_eq!(stats["hits"], 0);
    assert_eq!(stats["misses"], 0);
    assert_eq!(stats["entries"], 0);
    assert!(
        (stats["hit_rate"].as_f64().unwrap() - 0.0).abs() < 1e-9,
        "fresh cache must have hit_rate 0.0"
    );
}
