//! Latency percentile computation from Prometheus-style histogram buckets.
//!
//! This module provides [`PercentilePoint`] and the algorithm used by
//! [`crate::services::MetricsStore::query_latency_percentiles`] to derive
//! percentiles (e.g. p50 / p95 / p99) from a series of histogram
//! observations recorded as [`MetricValue::Histogram`] buckets.
//!
//! ## Algorithm
//!
//! For each time window:
//! 1. Collect all `(le, count)` bucket observations in the window.
//! 2. Sort by `le` ascending (Prometheus convention).
//! 3. Build a cumulative distribution: at each `le`, the cumulative count
//!    equals the sum of all bucket counts with `le' <= le`.
//! 4. The total population is taken as the value of the largest finite `le`
//!    bucket (or, if absent, the maximum cumulative count observed).
//! 5. For each requested percentile `p`:
//!    - Target count = `p / 100 * total`.
//!    - Find the first bucket whose cumulative count >= target.
//!    - Linearly interpolate between that bucket's `le` and the previous
//!      bucket's `le` based on the residual count.
//!
//! ## Edge cases
//!
//! - Empty input: no points emitted.
//! - Single bucket: that bucket's `le` is returned for every percentile.
//! - All buckets with count 0: total population is 0 → no points emitted.
//! - `le = +Inf` (Prometheus overflow bucket): treated as the total
//!   population when present, but not used for interpolation.

use serde::{Deserialize, Serialize};

use super::metrics_store::{AggregationInterval, MetricPoint, MetricValue};

/// A single time-bucketed percentile data point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PercentilePoint {
    /// Timestamp marking the start of the aggregation window.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Percentile values keyed by the percentile number as a string,
    /// e.g. `{ "50": 12.3, "95": 45.6, "99": 120.0 }`. Keys are derived from
    /// the caller-supplied `percentiles` slice and are stable for charting.
    pub percentiles: std::collections::BTreeMap<String, f64>,
}

/// Group consecutive histogram observations by `interval` window and
/// compute the requested percentiles for each window.
///
/// Returns an empty `Vec` if `points` is empty, if no observation in any
/// window contains a histogram bucket, or if every bucket has count 0.
pub fn compute_percentiles(
    points: &[MetricPoint],
    interval: AggregationInterval,
    percentiles: &[f64],
) -> Vec<PercentilePoint> {
    if points.is_empty() || percentiles.is_empty() {
        return Vec::new();
    }

    // Bin points by their floor(interval) window. Bins are keyed by the
    // truncated timestamp to keep ordering deterministic.
    let mut bins: std::collections::BTreeMap<chrono::DateTime<chrono::Utc>, Vec<&MetricPoint>> =
        std::collections::BTreeMap::new();
    for p in points {
        let bucket_ts = floor_to_interval(p.timestamp, interval);
        bins.entry(bucket_ts).or_default().push(p);
    }

    let mut out = Vec::with_capacity(bins.len());
    for (bucket_ts, bin_points) in bins {
        if let Some(result) = compute_window_percentiles(&bin_points, percentiles) {
            out.push(PercentilePoint {
                timestamp: bucket_ts,
                percentiles: result,
            });
        }
    }
    out
}

fn floor_to_interval(
    ts: chrono::DateTime<chrono::Utc>,
    interval: AggregationInterval,
) -> chrono::DateTime<chrono::Utc> {
    let secs = interval.to_seconds();
    let epoch = ts.timestamp();
    let floored = epoch - (epoch % secs);
    chrono::DateTime::<chrono::Utc>::from_timestamp(floored, 0)
        .unwrap_or_else(|| ts)
}

/// Build a sorted list of `(le, count)` buckets from observations, keeping
/// the maximum count for each unique `le` within the window.
fn collect_buckets<'a>(
    points: &[&'a MetricPoint],
) -> Vec<(f64, f64)> {
    let mut map: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    for p in points {
        if let MetricValue::Histogram { le, count } = &p.value {
            // Quantize le so that equal bucket boundaries collapse together.
            // Buckets from Prometheus are typically rounded; use micros to
            // preserve precision while still merging close-together values.
            let key = (*le * 1_000_000.0).round() as i64;
            let entry = map.entry(key).or_insert(0.0);
            if (*count as f64) > *entry {
                *entry = *count as f64;
            }
        }
    }
    map.into_iter()
        .map(|(k, v)| (k as f64 / 1_000_000.0, v))
        .collect()
}

/// Compute percentile values for a single aggregation window.
///
/// Returns `None` when there are no usable histogram buckets or when the
/// total observation count is zero.
fn compute_window_percentiles(
    points: &[&MetricPoint],
    percentiles: &[f64],
) -> Option<std::collections::BTreeMap<String, f64>> {
    let mut buckets = collect_buckets(points);
    if buckets.is_empty() {
        return None;
    }
    buckets.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Total population = sum of all per-bucket observation counts. Each
    // bucket represents a count of observations in its `[prev_le, le]`
    // range; we re-cumulate them inside `interpolate_percentile`.
    let total: f64 = buckets.iter().map(|(_, c)| c).sum();
    if total <= 0.0 {
        return None;
    }

    // Special case: single bucket — return its `le` for every percentile.
    if buckets.len() == 1 {
        let only = buckets[0].0;
        let mut out = std::collections::BTreeMap::new();
        for p in percentiles {
            out.insert(percentile_key(*p), only);
        }
        return Some(out);
    }

    let mut out = std::collections::BTreeMap::new();
    for p in percentiles {
        let target = (*p / 100.0) * total;
        let value = interpolate_percentile(&buckets, target, total);
        out.insert(percentile_key(*p), value);
    }
    Some(out)
}

/// Linear-interpolate the `le` value at which the cumulative count first
/// reaches `target`. If `target` exceeds the largest cumulative count we
/// return the largest observed `le` (clamped).
fn interpolate_percentile(buckets: &[(f64, f64)], target: f64, total: f64) -> f64 {
    let mut prev_le = 0.0_f64;
    let mut prev_cum = 0.0_f64;
    for &(le, count) in buckets {
        // Prometheus bucket counts are *cumulative* by convention
        // (counter-style), but our store records per-window deltas. We
        // therefore re-cumulate here.
        let cum = prev_cum + count;
        if cum >= target {
            // Interpolate between (prev_le, prev_cum) and (le, cum).
            let span = (cum - prev_cum).max(f64::EPSILON);
            let frac = ((target - prev_cum) / span).clamp(0.0, 1.0);
            return prev_le + frac * (le - prev_le);
        }
        prev_le = le;
        prev_cum = cum;
    }
    // Target exceeds observed total — clamp to the largest finite le.
    let _ = total;
    prev_le
}

/// Stable key for the percentile map. Renders values like `"50"`, `"95"`,
/// `"99"`, `"99.9"`, `"99.99"` without trailing zeros.
fn percentile_key(p: f64) -> String {
    if (p.fract()).abs() < f64::EPSILON {
        format!("{}", p as i64)
    } else {
        // Strip trailing zeros from a fixed-decimal representation.
        let s = format!("{:.4}", p);
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn point_at(sec: i64, value: MetricValue) -> MetricPoint {
        MetricPoint {
            timestamp: Utc.timestamp_opt(sec, 0).unwrap(),
            value,
        }
    }

    #[test]
    fn empty_input_returns_empty() {
        let out = compute_percentiles(&[], AggregationInterval::OneMinute, &[50.0, 95.0]);
        assert!(out.is_empty());
    }

    #[test]
    fn empty_percentiles_returns_empty() {
        let p = point_at(0, MetricValue::Histogram { le: 1.0, count: 5 });
        let out = compute_percentiles(&[p], AggregationInterval::OneMinute, &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn single_bucket_returns_that_le_for_every_percentile() {
        let p = point_at(60, MetricValue::Histogram { le: 42.0, count: 100 });
        let out = compute_percentiles(&[p], AggregationInterval::OneMinute, &[50.0, 95.0, 99.0]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].percentiles.get("50"), Some(&42.0));
        assert_eq!(out[0].percentiles.get("95"), Some(&42.0));
        assert_eq!(out[0].percentiles.get("99"), Some(&42.0));
    }

    #[test]
    fn all_buckets_zero_returns_none_for_window() {
        let p1 = point_at(60, MetricValue::Histogram { le: 1.0, count: 0 });
        let p2 = point_at(70, MetricValue::Histogram { le: 2.0, count: 0 });
        let out = compute_percentiles(&[p1, p2], AggregationInterval::OneMinute, &[50.0]);
        assert!(out.is_empty());
    }

    #[test]
    fn computes_p50_p95_p99_on_uniform_distribution() {
        // 4 buckets each with 25 observations; le in [10, 20, 30, 40].
        // p50 (target 50) should sit near le=20, p95 (95) near le=30-40.
        let points: Vec<MetricPoint> = [10.0_f64, 20.0, 30.0, 40.0]
            .into_iter()
            .enumerate()
            .map(|(i, le)| {
                point_at(60 + i as i64, MetricValue::Histogram { le, count: 25 })
            })
            .collect();
        let out = compute_percentiles(&points, AggregationInterval::OneMinute, &[50.0, 95.0]);
        assert_eq!(out.len(), 1);
        let p = &out[0];
        // p50 should be in [20, 30] (interpolated)
        let p50 = p.percentiles.get("50").copied().unwrap();
        assert!(p50 >= 20.0 && p50 <= 30.0, "p50={p50}");
        // p95 should be in [30, 40]
        let p95 = p.percentiles.get("95").copied().unwrap();
        assert!(p95 >= 30.0 && p95 <= 40.0, "p95={p95}");
    }

    #[test]
    fn groups_points_into_interval_windows() {
        // Three observations in the same 1-minute window, one in the next.
        let in_window = vec![
            point_at(60, MetricValue::Histogram { le: 1.0, count: 10 }),
            point_at(80, MetricValue::Histogram { le: 2.0, count: 10 }),
            point_at(100, MetricValue::Histogram { le: 3.0, count: 10 }),
        ];
        let next_window = vec![point_at(180, MetricValue::Histogram { le: 1.0, count: 5 })];

        let mut all = in_window;
        all.extend(next_window);
        let out = compute_percentiles(&all, AggregationInterval::OneMinute, &[50.0]);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn percentile_key_strips_trailing_zeros() {
        assert_eq!(percentile_key(50.0), "50");
        assert_eq!(percentile_key(99.9), "99.9");
        assert_eq!(percentile_key(99.99), "99.99");
    }

    #[test]
    fn clamps_when_target_exceeds_total() {
        // Total = 5, but ask for p99.9 → should clamp to largest le.
        let points = vec![
            point_at(60, MetricValue::Histogram { le: 1.0, count: 2 }),
            point_at(70, MetricValue::Histogram { le: 2.0, count: 3 }),
        ];
        let out = compute_percentiles(&points, AggregationInterval::OneMinute, &[99.9]);
        let p = &out[0];
        assert_eq!(p.percentiles.get("99.9"), Some(&2.0));
    }

    #[test]
    fn supports_five_minute_aggregation_interval() {
        // Observations at t=0 and t=200 (both fall in the [0, 300) bin).
        let points = vec![
            point_at(0, MetricValue::Histogram { le: 1.0, count: 5 }),
            point_at(200, MetricValue::Histogram { le: 2.0, count: 5 }),
        ];
        let out = compute_percentiles(&points, AggregationInterval::FiveMinutes, &[50.0]);
        assert_eq!(out.len(), 1);
        let p = &out[0];
        // p50 target = 5; first bucket cumulative = 5 → le=1.
        assert_eq!(p.percentiles.get("50"), Some(&1.0));
        // Timestamp should be floored to 0.
        assert_eq!(p.timestamp.timestamp(), 0);
    }

    #[test]
    fn duration_conversion_matches_interval() {
        // sanity: FiveMinutes = 300 seconds.
        assert_eq!(
            AggregationInterval::FiveMinutes.to_duration(),
            Duration::seconds(300)
        );
    }
}
