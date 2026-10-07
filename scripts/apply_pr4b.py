#!/usr/bin/env python3
"""Apply PR4b — Per-route analytics modifications to metrics.rs in one shot.

This script reads the original metrics.rs (HEAD) and applies all
modifications atomically: imports, struct field, Default init,
struct RouteMetrics + impl + struct RouteMetricsSnapshot + impl,
impl MetricsCollector methods, escape_prometheus_label helper,
route_metrics_endpoint admin function, configure_metrics update,
and the Prometheus format! injection in metrics_endpoint.
"""

import subprocess
import sys
from pathlib import Path

REPO = Path("/Users/vongthai/workspace/kairos-rs")
TARGET = REPO / "crates/kairos-rs/src/routes/metrics.rs"
REL = str(TARGET.relative_to(REPO))

# 1. Read original content from HEAD
result = subprocess.run(
    ["git", "show", f"HEAD:{REL}"],
    cwd=REPO, capture_output=True, text=True, check=True,
)
content = result.stdout
print(f"Loaded {len(content.splitlines())} lines from HEAD")

# 2. Imports: Arc -> {Arc, RwLock} + add HashMap
old_imports = "use std::sync::Arc;\nuse std::time::{Duration, Instant};"
new_imports = (
    "use std::sync::{Arc, RwLock};\n"
    "use std::collections::HashMap;\n"
    "use std::time::{Duration, Instant};"
)
assert old_imports in content, "imports anchor not found"
content = content.replace(old_imports, new_imports, 1)

# 3. Add `routes` field to struct MetricsCollector (after start_time)
old_field = "    /// Application start time for uptime calculations\n    pub start_time: Instant,\n}"
new_field = (
    "    /// Application start time for uptime calculations\n"
    "    pub start_time: Instant,\n"
    "    /// Per-route metrics, keyed by route external path (e.g. \"/users/{id}\").\n"
    "    /// Created on first request to a given route, never removed (bounded by\n"
    "    /// the number of configured routes in practice). Stored as `Arc` so the\n"
    "    /// `RouteMetrics` can be shared between the collector and the prometheus\n"
    "    /// exposition without copying atomic counters.\n"
    "    pub routes: Arc<RwLock<HashMap<String, Arc<RouteMetrics>>>>,\n"
    "}"
)
assert old_field in content, "struct field anchor not found"
content = content.replace(old_field, new_field, 1)

# 4. Add `routes` init in Default impl
old_init = (
    "            start_time: Instant::now(),\n"
    "        }\n"
    "    }\n"
    "}"
)
new_init = (
    "            start_time: Instant::now(),\n"
    "            routes: Arc::new(RwLock::new(HashMap::new())),\n"
    "        }\n"
    "    }\n"
    "}"
)
assert old_init in content, "default init anchor not found"
content = content.replace(old_init, new_init, 1)

# 5. Insert PR4b code before `impl MetricsCollector {` (the cache_snapshot block)
ANCHOR = "impl MetricsCollector {\n    /// Read current cache counters."
assert ANCHOR in content, "PR4b anchor not found"

PR4B_INSERT = r"""
// ============== PR4b: Per-route analytics ==============

/// Per-route metrics for one matched route (keyed by `external_path`).
///
/// Mirrors the subset of `MetricsCollector` counters that are useful for
/// per-route breakdown. Counters are `Arc<AtomicU64>` so the struct is
/// cheap to clone and the same counters can be read by the prometheus
/// exposition path and the admin endpoint without taking a lock.
#[derive(Debug)]
pub struct RouteMetrics {
    pub route_id: String,
    pub requests_total: Arc<AtomicU64>,
    pub requests_success: Arc<AtomicU64>,
    pub requests_error: Arc<AtomicU64>,
    pub response_time_sum: Arc<AtomicU64>,
    pub response_time_bucket_100ms: Arc<AtomicU64>,
    pub response_time_bucket_500ms: Arc<AtomicU64>,
    pub response_time_bucket_1s: Arc<AtomicU64>,
    pub response_time_bucket_5s: Arc<AtomicU64>,
    pub response_time_bucket_inf: Arc<AtomicU64>,
    pub http_4xx_errors: Arc<AtomicU64>,
    pub http_5xx_errors: Arc<AtomicU64>,
    pub cache_hits_total: Arc<AtomicU64>,
    pub cache_misses_total: Arc<AtomicU64>,
}

impl RouteMetrics {
    /// Build a fresh `RouteMetrics` for the given route id with all
    /// counters initialised to zero.
    pub fn new(route_id: impl Into<String>) -> Self {
        Self {
            route_id: route_id.into(),
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_success: Arc::new(AtomicU64::new(0)),
            requests_error: Arc::new(AtomicU64::new(0)),
            response_time_sum: Arc::new(AtomicU64::new(0)),
            response_time_bucket_100ms: Arc::new(AtomicU64::new(0)),
            response_time_bucket_500ms: Arc::new(AtomicU64::new(0)),
            response_time_bucket_1s: Arc::new(AtomicU64::new(0)),
            response_time_bucket_5s: Arc::new(AtomicU64::new(0)),
            response_time_bucket_inf: Arc::new(AtomicU64::new(0)),
            http_4xx_errors: Arc::new(AtomicU64::new(0)),
            http_5xx_errors: Arc::new(AtomicU64::new(0)),
            cache_hits_total: Arc::new(AtomicU64::new(0)),
            cache_misses_total: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// Read-only snapshot of per-route counters, safe to serialise to JSON.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RouteMetricsSnapshot {
    pub route_id: String,
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub response_time_sum_ms: u64,
    pub avg_response_time_ms: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub bucket_100ms: u64,
    pub bucket_500ms: u64,
    pub bucket_1s: u64,
    pub bucket_5s: u64,
    pub bucket_inf: u64,
    pub http_4xx_errors: u64,
    pub http_5xx_errors: u64,
    pub cache_hits_total: u64,
    pub cache_misses_total: u64,
    pub cache_hit_rate: f64,
}

impl RouteMetricsSnapshot {
    /// Build a snapshot by reading all atomic counters with
    /// `Ordering::Relaxed`. Order of hit/miss increments does not
    /// matter for the derived rates, so relaxed loads are sufficient.
    pub fn from_metrics(rm: &RouteMetrics) -> Self {
        let total = rm.requests_total.load(Ordering::Relaxed);
        let success = rm.requests_success.load(Ordering::Relaxed);
        let error = rm.requests_error.load(Ordering::Relaxed);
        let sum = rm.response_time_sum.load(Ordering::Relaxed);
        let hits = rm.cache_hits_total.load(Ordering::Relaxed);
        let misses = rm.cache_misses_total.load(Ordering::Relaxed);
        let avg = if total > 0 { sum as f64 / total as f64 } else { 0.0 };
        let success_rate = if total > 0 { (success as f64 / total as f64) * 100.0 } else { 100.0 };
        let error_rate = if total > 0 { (error as f64 / total as f64) * 100.0 } else { 0.0 };
        let cache_total = hits + misses;
        let cache_hit_rate = if cache_total > 0 { hits as f64 / cache_total as f64 } else { 0.0 };
        Self {
            route_id: rm.route_id.clone(),
            requests_total: total,
            requests_success: success,
            requests_error: error,
            response_time_sum_ms: sum,
            avg_response_time_ms: avg,
            success_rate,
            error_rate,
            bucket_100ms: rm.response_time_bucket_100ms.load(Ordering::Relaxed),
            bucket_500ms: rm.response_time_bucket_500ms.load(Ordering::Relaxed),
            bucket_1s: rm.response_time_bucket_1s.load(Ordering::Relaxed),
            bucket_5s: rm.response_time_bucket_5s.load(Ordering::Relaxed),
            bucket_inf: rm.response_time_bucket_inf.load(Ordering::Relaxed),
            http_4xx_errors: rm.http_4xx_errors.load(Ordering::Relaxed),
            http_5xx_errors: rm.http_5xx_errors.load(Ordering::Relaxed),
            cache_hits_total: hits,
            cache_misses_total: misses,
            cache_hit_rate,
        }
    }
}

impl MetricsCollector {
    /// Look up (or lazily create) the per-route `RouteMetrics` for
    /// `route_id`. The `RwLock` is released before returning the
    /// `Arc<RouteMetrics>`, so callers can use the counters without
    /// holding any map-level lock.
    pub fn get_or_create_route(&self, route_id: &str) -> Arc<RouteMetrics> {
        {
            let map = self.routes.read().unwrap_or_else(|e| e.into_inner());
            if let Some(rm) = map.get(route_id) {
                return rm.clone();
            }
        }
        let mut map = self.routes.write().unwrap_or_else(|e| e.into_inner());
        map.entry(route_id.to_string())
            .or_insert_with(|| Arc::new(RouteMetrics::new(route_id)))
            .clone()
    }

    /// Record the completion of an HTTP request for a specific route.
    /// Mirrors the global `record_request` semantics but writes to the
    /// per-route counters in the `routes` map.
    pub fn record_route_request(
        &self,
        route_id: &str,
        success: bool,
        response_time: Duration,
        status_code: u16,
    ) {
        let rm = self.get_or_create_route(route_id);
        rm.requests_total.fetch_add(1, Ordering::Relaxed);
        rm.response_time_sum.fetch_add(response_time.as_millis() as u64, Ordering::Relaxed);
        let ms = response_time.as_millis() as u64;
        if ms <= 100 { rm.response_time_bucket_100ms.fetch_add(1, Ordering::Relaxed); }
        if ms <= 500 { rm.response_time_bucket_500ms.fetch_add(1, Ordering::Relaxed); }
        if ms <= 1000 { rm.response_time_bucket_1s.fetch_add(1, Ordering::Relaxed); }
        if ms <= 5000 { rm.response_time_bucket_5s.fetch_add(1, Ordering::Relaxed); }
        else { rm.response_time_bucket_inf.fetch_add(1, Ordering::Relaxed); }
        if success {
            rm.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            rm.requests_error.fetch_add(1, Ordering::Relaxed);
            match status_code {
                400..=499 => { rm.http_4xx_errors.fetch_add(1, Ordering::Relaxed); }
                500..=599 => { rm.http_5xx_errors.fetch_add(1, Ordering::Relaxed); }
                _ => {}
            }
        }
    }

    /// Increment the cache-hit counter for the given route.
    pub fn record_route_cache_hit(&self, route_id: &str) {
        let rm = self.get_or_create_route(route_id);
        rm.cache_hits_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the cache-miss counter for the given route.
    pub fn record_route_cache_miss(&self, route_id: &str) {
        let rm = self.get_or_create_route(route_id);
        rm.cache_misses_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot every per-route `RouteMetrics` into a `RouteMetricsSnapshot`
    /// and return them sorted by `route_id` for stable JSON output.
    pub fn route_metrics_snapshot(&self) -> Vec<RouteMetricsSnapshot> {
        let map = self.routes.read().unwrap_or_else(|e| e.into_inner());
        let mut out: Vec<RouteMetricsSnapshot> = map
            .values()
            .map(|rm| RouteMetricsSnapshot::from_metrics(rm.as_ref()))
            .collect();
        out.sort_by(|a, b| a.route_id.cmp(&b.route_id));
        out
    }

    /// Build the Prometheus text-format section for per-route metrics.
    /// Returns an empty string when no route has been seen yet.
    pub fn route_prometheus_text(&self) -> String {
        let map = self.routes.read().unwrap_or_else(|e| e.into_inner());
        if map.is_empty() { return String::new(); }
        let mut entries: Vec<(&String, &Arc<RouteMetrics>)> = map.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut out = String::new();
        out.push_str("\n# HELP kairos_route_requests_total Total HTTP requests served per route.\n");
        out.push_str("# TYPE kairos_route_requests_total counter\n");
        for (id, rm) in &entries {
            out.push_str(&format!(
                "kairos_route_requests_total{{route=\"{}\"}} {}\n",
                escape_prometheus_label(id),
                rm.requests_total.load(Ordering::Relaxed)
            ));
        }
        out.push_str("\n# HELP kairos_route_requests_success_total Successful (2xx) HTTP requests per route.\n");
        out.push_str("# TYPE kairos_route_requests_success_total counter\n");
        for (id, rm) in &entries {
            out.push_str(&format!(
                "kairos_route_requests_success_total{{route=\"{}\"}} {}\n",
                escape_prometheus_label(id),
                rm.requests_success.load(Ordering::Relaxed)
            ));
        }
        out.push_str("\n# HELP kairos_route_requests_error_total Failed (4xx/5xx) HTTP requests per route.\n");
        out.push_str("# TYPE kairos_route_requests_error_total counter\n");
        for (id, rm) in &entries {
            out.push_str(&format!(
                "kairos_route_requests_error_total{{route=\"{}\"}} {}\n",
                escape_prometheus_label(id),
                rm.requests_error.load(Ordering::Relaxed)
            ));
        }
        out.push_str("\n# HELP kairos_route_response_time_avg Average response time in milliseconds per route.\n");
        out.push_str("# TYPE kairos_route_response_time_avg gauge\n");
        for (id, rm) in &entries {
            let total = rm.requests_total.load(Ordering::Relaxed);
            let sum = rm.response_time_sum.load(Ordering::Relaxed);
            let avg = if total > 0 { sum as f64 / total as f64 } else { 0.0 };
            out.push_str(&format!(
                "kairos_route_response_time_avg{{route=\"{}\"}} {:.2}\n",
                escape_prometheus_label(id),
                avg
            ));
        }
        out.push_str("\n# HELP kairos_route_response_time_bucket Response time histogram buckets per route.\n");
        out.push_str("# TYPE kairos_route_response_time_bucket histogram\n");
        for (id, rm) in &entries {
            let label = escape_prometheus_label(id);
            out.push_str(&format!(
                "kairos_route_response_time_bucket{{route=\"{}\",le=\"100\"}} {}\n",
                label,
                rm.response_time_bucket_100ms.load(Ordering::Relaxed)
            ));
            out.push_str(&format!(
                "kairos_route_response_time_bucket{{route=\"{}\",le=\"500\"}} {}\n",
                label,
                rm.response_time_bucket_500ms.load(Ordering::Relaxed)
            ));
            out.push_str(&format!(
                "kairos_route_response_time_bucket{{route=\"{}\",le=\"1000\"}} {}\n",
                label,
                rm.response_time_bucket_1s.load(Ordering::Relaxed)
            ));
            out.push_str(&format!(
                "kairos_route_response_time_bucket{{route=\"{}\",le=\"5000\"}} {}\n",
                label,
                rm.response_time_bucket_5s.load(Ordering::Relaxed)
            ));
            out.push_str(&format!(
                "kairos_route_response_time_bucket{{route=\"{}\",le=\"+Inf\"}} {}\n",
                label,
                rm.response_time_bucket_inf.load(Ordering::Relaxed)
            ));
        }
        out.push_str("\n# HELP kairos_route_cache_hits_total Cache hits per route.\n");
        out.push_str("# TYPE kairos_route_cache_hits_total counter\n");
        for (id, rm) in &entries {
            out.push_str(&format!(
                "kairos_route_cache_hits_total{{route=\"{}\"}} {}\n",
                escape_prometheus_label(id),
                rm.cache_hits_total.load(Ordering::Relaxed)
            ));
        }
        out.push_str("\n# HELP kairos_route_cache_misses_total Cache misses per route.\n");
        out.push_str("# TYPE kairos_route_cache_misses_total counter\n");
        for (id, rm) in &entries {
            out.push_str(&format!(
                "kairos_route_cache_misses_total{{route=\"{}\"}} {}\n",
                escape_prometheus_label(id),
                rm.cache_misses_total.load(Ordering::Relaxed)
            ));
        }
        out
    }
}

/// Escape a Prometheus label value: backslashes first, then double
/// quotes. Used by `route_prometheus_text` so route ids containing
/// those characters don't corrupt the exposition format.
fn escape_prometheus_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Admin endpoint that returns the per-route metrics breakdown as JSON.
///
/// # Request
///
/// ```text
/// GET /api/metrics/routes
/// ```
///
/// # Response
///
/// ```json
/// {
///   "success": true,
///   "count": 1,
///   "routes": [
///     { "route_id": "/users/{id}", "requests_total": 42, "...": "..." }
///   ]
/// }
/// ```
pub async fn route_metrics_endpoint(
    metrics: web::Data<MetricsCollector>,
) -> Result<HttpResponse> {
    let routes = metrics.route_metrics_snapshot();
    let count = routes.len();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "count": count,
        "routes": routes,
    })))
}
"""

content = content.replace(ANCHOR, PR4B_INSERT + "\n" + ANCHOR, 1)

# 6. Register the new admin endpoint in configure_metrics
old_cfg = (
    '       .route("/api/metrics/latency/percentiles", web::get().to(get_latency_percentiles));\n'
    "}\n"
)
new_cfg = (
    '       .route("/api/metrics/latency/percentiles", web::get().to(get_latency_percentiles))\n'
    '       .route("/api/metrics/routes", web::get().to(route_metrics_endpoint));\n'
    "}\n"
)
assert old_cfg in content, "configure_metrics anchor not found"
content = content.replace(old_cfg, new_cfg, 1)

# 7. Inject per-route Prometheus section into metrics_endpoint
# Add a `{}` placeholder right before the closing `"#` of the format! raw string
# (right after `kairos_uptime_seconds {}{}`). This adds one extra `{}` to the
# format! invocation.
old_fmt_close = "kairos_uptime_seconds {}{}\n\"#,\n        total_requests,"
new_fmt_close = "kairos_uptime_seconds {}{}{}\n\"#,\n        total_requests,"
assert old_fmt_close in content, "format close anchor not found"
content = content.replace(old_fmt_close, new_fmt_close, 1)



# 9. Build the per-route section before the format! invocation. The simplest
# place is right after the `let circuit_breaker_metrics = String::new();` block
# ends, just before the format! macro. We insert two lines.
old_cb_end = (
    "            }\n"
    "        }\n"
    "    }\n"
    "\n"
    "    let metrics_text = format!("
)
new_cb_end = (
    "            }\n"
    "        }\n"
    "    }\n"
    "\n"
    "    // PR4b: per-route Prometheus section (empty when no route has been seen).\n"
    "    let route_metrics_text = metrics.route_prometheus_text();\n"
    "\n"
    "    let metrics_text = format!("
)
assert old_cb_end in content, "circuit_breaker end anchor not found"
content = content.replace(old_cb_end, new_cb_end, 1)

# Write back
TARGET.write_text(content)
print(f"Wrote {len(content.splitlines())} lines to {TARGET}")
