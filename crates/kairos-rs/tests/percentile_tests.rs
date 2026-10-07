//! Integration tests for `MetricsStore::query_latency_percentiles`.
//!
//! Verifies the end-to-end flow: recording histogram observations into the
//! store, then querying percentiles through the public API. Covers
//! algorithm correctness on synthetic buckets, edge cases (empty, single
//! bucket, all zero counts), percentile boundaries, and time-window
//! grouping across multiple `AggregationInterval` values.
//!
//! See `src/services/percentile.rs` for the algorithm and `src/services/
//! metrics_store.rs` for the storage layer.

use chrono::{Duration, Utc};
use kairos_rs::services::metrics_store::{
    AggregationInterval, MetricsStore, MetricValue,
};

fn bucket_at(store: &MetricsStore, name: &str, le: f64, count: u64) {
    store.record(name, MetricValue::Histogram { le, count });
}

#[test]
fn empty_store_returns_empty_vec() {
    let store = MetricsStore::new(1000, Duration::hours(24));
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now();
    let pts = store.query_latency_percentiles(
        "nonexistent",
        start,
        end,
        &[50.0, 95.0, 99.0],
        AggregationInterval::OneMinute,
    );
    assert!(pts.is_empty());
}

#[test]
fn all_zero_counts_yields_no_windows() {
    let store = MetricsStore::new(1000, Duration::hours(24));
    for le in [1.0, 5.0, 10.0, 50.0, 100.0] {
        bucket_at(&store, "response_time", le, 0);
    }
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now();
    let pts = store.query_latency_percentiles(
        "response_time",
        start,
        end,
        &[50.0, 95.0],
        AggregationInterval::OneMinute,
    );
    // No bucket has a non-zero count → total population is 0 → no windows emitted.
    assert!(pts.is_empty());
}

#[test]
fn single_bucket_emits_one_window_with_that_le() {
    let store = MetricsStore::new(1000, Duration::hours(24));
    bucket_at(&store, "rt", 42.0, 100);

    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "rt",
        start,
        end,
        &[50.0, 95.0, 99.0],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    assert_eq!(p.percentiles.get("50"), Some(&42.0));
    assert_eq!(p.percentiles.get("95"), Some(&42.0));
    assert_eq!(p.percentiles.get("99"), Some(&42.0));
}

#[test]
fn uniform_distribution_yields_p_within_bracket() {
    // Buckets: le=10 (25), le=20 (25), le=30 (25), le=40 (25). Total = 100.
    // p50 target = 50 → first bucket where cum >= 50 is le=20 → p50 ∈ [20, 30].
    // p95 target = 95 → first bucket where cum >= 95 is le=40 → p95 ∈ [30, 40].
    let store = MetricsStore::new(1000, Duration::hours(24));
    for le in [10.0, 20.0, 30.0, 40.0] {
        bucket_at(&store, "latency", le, 25);
    }
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "latency",
        start,
        end,
        &[50.0, 95.0],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    let p50 = p.percentiles.get("50").copied().unwrap();
    let p95 = p.percentiles.get("95").copied().unwrap();
    assert!(
        (20.0..=30.0).contains(&p50),
        "p50 should sit in [20, 30], got {p50}"
    );
    assert!(
        (30.0..=40.0).contains(&p95),
        "p95 should sit in [30, 40], got {p95}"
    );
}

#[test]
fn p99_with_heavy_tail_picks_largest_bucket() {
    // 99 obs in le=10, 1 obs in le=100. Total = 100. Using linear
    // interpolation semantics (each bucket = uniform distribution in
    // [prev_le, le]):
    //   p50 target=50 → cum at le=10 = 99 ≥ 50; frac=(50-0)/99≈0.505; return ≈5.05
    //   p99 target=99 → cum at le=10 = 99 ≥ 99; frac=1.0; return 10.0
    let store = MetricsStore::new(1000, Duration::hours(24));
    bucket_at(&store, "tail", 10.0, 99);
    bucket_at(&store, "tail", 100.0, 1);

    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "tail",
        start,
        end,
        &[50.0, 99.0],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    let p50 = p.percentiles.get("50").copied().unwrap();
    let p99 = p.percentiles.get("99").copied().unwrap();
    // p50 sits in the first bucket (0..10) — interpolated value is ~5.05.
    assert!(
        (4.0..=6.0).contains(&p50),
        "p50 should sit in [4, 6], got {p50}"
    );
    // p99 lands exactly at the boundary of the first bucket.
    assert_eq!(p99, 10.0);
}

#[test]
fn percentiles_extreme_zero_and_hundred() {
    // Single observation: le=5, count=10. Both p0 and p100 should yield 5.
    let store = MetricsStore::new(1000, Duration::hours(24));
    bucket_at(&store, "single", 5.0, 10);

    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "single",
        start,
        end,
        &[0.0, 100.0],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    assert_eq!(p.percentiles.get("0"), Some(&5.0));
    assert_eq!(p.percentiles.get("100"), Some(&5.0));
}

#[test]
fn non_default_percentile_values_round_trip() {
    // Verify that 75, 90, 99.9 all interpolate correctly.
    let store = MetricsStore::new(1000, Duration::hours(24));
    for le in [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0] {
        bucket_at(&store, "fine", le, 10);
    }
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "fine",
        start,
        end,
        &[75.0, 90.0, 99.9],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    // Total = 100. p75 target = 75 → cum at le=80 = 80 ≥ 75, prev = 70 cum=70.
    // Linear interp: 70 + (75-70)/10 * (80-70) = 70 + 5 = 75.
    let p75 = p.percentiles.get("75").copied().unwrap();
    assert!(
        (74.0..=76.0).contains(&p75),
        "p75 should be near 75, got {p75}"
    );
    // p90 target = 90 → cum at le=90 = 90 exactly → le=90.
    let p90 = p.percentiles.get("90").copied().unwrap();
    assert_eq!(p90, 90.0);
    // p99.9 target = 99.9 → cum at le=100 = 100, prev=90 cum=90.
    // Interp: 90 + (99.9-90)/10 * (100-90) = 90 + 9.9 = 99.9.
    let p999 = p.percentiles.get("99.9").copied().unwrap();
    assert!(
        (99.0..=100.0).contains(&p999),
        "p99.9 should be near 99.9, got {p999}"
    );
}

#[test]
fn unknown_metric_returns_empty() {
    let store = MetricsStore::new(1000, Duration::hours(24));
    bucket_at(&store, "registered", 1.0, 5);
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "missing",
        start,
        end,
        &[50.0, 95.0],
        AggregationInterval::OneMinute,
    );
    assert!(pts.is_empty());
}

#[test]
fn mixed_counter_and_histogram_yields_only_histogram_windows() {
    // If the same metric name has both Counter and Histogram observations,
    // the percentile algorithm should ignore the Counter and produce a
    // single window from the histogram data.
    let store = MetricsStore::new(1000, Duration::hours(24));
    store.record("mixed", MetricValue::Counter(42));
    bucket_at(&store, "mixed", 10.0, 5);
    bucket_at(&store, "mixed", 20.0, 5);

    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "mixed",
        start,
        end,
        &[50.0, 95.0],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    // total = 10 (from last bucket). p50 target=5 → cum at le=10 = 5 → le=10.
    assert_eq!(p.percentiles.get("50"), Some(&10.0));
    // p95 target=9.5 → cum at le=10=5, then cum at le=20=10 ≥ 9.5.
    // Interp between (10, 5) and (20, 10): 10 + (9.5-5)/5 * (20-10) = 10 + 9 = 19.
    let p95 = p.percentiles.get("95").copied().unwrap();
    assert!(
        (18.0..=20.0).contains(&p95),
        "p95 should be near 19, got {p95}"
    );
}

#[test]
fn five_minute_interval_aggregates_within_window() {
    // All observations land in the same 5-minute window.
    // (Time-grouping across windows is exercised by the per-bucket floor
    // in the algorithm; here we just verify that the total population
    // is summed correctly when multiple buckets feed one window.)
    let store = MetricsStore::new(1000, Duration::hours(24));
    for le in [10.0, 20.0, 30.0, 40.0] {
        bucket_at(&store, "agg", le, 25);
    }
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "agg",
        start,
        end,
        &[50.0, 95.0, 99.0],
        AggregationInterval::FiveMinutes,
    );
    // All points fall in a single 5-min bin.
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    let p99 = p.percentiles.get("99").copied().unwrap();
    // p99 target = 99 → cum at le=40 = 100 ≥ 99, prev cum=75 (le=30).
    // Interp: 30 + (99-75)/25 * (40-30) = 30 + 9.6 = 39.6.
    assert!(
        (39.0..=40.0).contains(&p99),
        "p99 should be near 39.6, got {p99}"
    );
}

#[test]
fn one_hour_interval_emits_at_most_one_recent_window() {
    let store = MetricsStore::new(1000, Duration::hours(24));
    bucket_at(&store, "hourly", 100.0, 50);
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "hourly",
        start,
        end,
        &[50.0],
        AggregationInterval::OneHour,
    );
    // All observations land in at most one 1-hour window.
    assert!(pts.len() <= 1);
    if !pts.is_empty() {
        assert_eq!(pts[0].percentiles.get("50"), Some(&100.0));
    }
}

#[test]
fn results_are_chronologically_ordered() {
    let store = MetricsStore::new(1000, Duration::hours(24));
    for le in [10.0, 20.0] {
        bucket_at(&store, "ordered", le, 5);
    }
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "ordered",
        start,
        end,
        &[50.0],
        AggregationInterval::OneMinute,
    );
    for w in pts.windows(2) {
        assert!(
            w[0].timestamp <= w[1].timestamp,
            "windows must be returned in chronological order"
        );
    }
}

#[test]
fn percentile_key_renders_integers_without_decimals() {
    // This guards against accidental breaking changes to the key format
    // that the frontend relies on for indexing `percentiles` map.
    let store = MetricsStore::new(1000, Duration::hours(24));
    bucket_at(&store, "key", 1.0, 1);
    let start = Utc::now() - Duration::hours(1);
    let end = Utc::now() + Duration::hours(1);
    let pts = store.query_latency_percentiles(
        "key",
        start,
        end,
        &[50.0, 95.0, 99.0, 99.9],
        AggregationInterval::OneMinute,
    );
    assert_eq!(pts.len(), 1);
    let p = &pts[0];
    // Integer percentiles → "50", "95", "99"; decimal → "99.9".
    assert!(p.percentiles.contains_key("50"));
    assert!(p.percentiles.contains_key("95"));
    assert!(p.percentiles.contains_key("99"));
    assert!(p.percentiles.contains_key("99.9"));
    assert!(!p.percentiles.contains_key("50.0"));
    assert!(!p.percentiles.contains_key("99.90"));
}
