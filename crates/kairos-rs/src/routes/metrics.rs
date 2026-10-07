//! Prometheus-compatible metrics endpoint for monitoring and observability.
//! 
//! This module provides comprehensive metrics collection and exposure for the
//! Kairos-rs gateway, including request counts, response times, error rates,
//! histograms, memory usage, per-route statistics, and system health indicators.

/// Retrieves latency percentile time-series from histogram buckets.
///
/// For each `interval`-sized window in `[start, end]`, computes the
/// requested percentiles (p50/p95/p99 by default) by interpolating the
/// cumulative distribution of `Histogram` buckets.
///
/// # Query Parameters
///
/// * `name` - Metric name (must contain histogram observations)
/// * `start` - Start timestamp (ISO 8601)
/// * `end` - End timestamp (ISO 8601)
/// * `interval` - Aggregation interval (`one_minute`, `five_minutes`, ...)
/// * `percentiles` - Optional comma-separated list, e.g. `50,95,99.9`.
///   Defaults to `50,95,99`.
///
/// # Returns
///
/// JSON array of [`crate::services::percentile::PercentilePoint`] in
/// chronological order. Empty array when the metric has no histogram data.
pub async fn get_latency_percentiles(
    store: web::Data<MetricsStore>,
    query: web::Query<LatencyPercentileQuery>,
) -> Result<HttpResponse> {
    let pcts = query
        .percentiles
        .clone()
        .unwrap_or_else(|| vec![50.0, 95.0, 99.0]);
    let data = store.query_latency_percentiles(
        &query.name,
        query.start,
        query.end,
        &pcts,
        query.interval,
    );
    Ok(HttpResponse::Ok().json(data))
}
use actix_web::{web, HttpResponse, Result};
use crate::services::http::RouteHandler;
use crate::services::metrics_store::{MetricsStore, AggregationInterval};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Thread-safe metrics collector for comprehensive gateway observability.
/// 
/// The `MetricsCollector` provides atomic counters, gauges, and histograms for tracking
/// gateway performance, request patterns, memory usage, per-route statistics, and system 
/// health. All metrics are thread-safe and designed for high-concurrency environments.
/// 
/// # Metrics Tracked
/// 
/// - **Request Counters**: Total, successful, and failed request counts
/// - **Performance**: Response time tracking, histograms, and percentiles
/// - **Concurrency**: Active connection monitoring and peak tracking
/// - **Memory**: Memory usage and garbage collection statistics
/// - **Per-Route**: Individual route performance metrics
/// - **Error Tracking**: Detailed error categorization and rates
/// - **Uptime**: Service start time and duration tracking
/// 
/// # Thread Safety
/// 
/// All metrics use atomic operations for lock-free updates from multiple
/// worker threads. The collector can be safely cloned and shared across
/// the entire application.
/// 
/// # Usage
/// 
/// The collector is typically initialized once at application startup
/// and shared via Actix Web's application data:
/// 
/// ```rust
/// use actix_web::{web, App};
/// use kairos_rs::routes::metrics::MetricsCollector;
/// 
/// # fn example() {
/// let metrics = MetricsCollector::default();
/// let app = App::new()
///     .app_data(web::Data::new(metrics.clone()))
///     .configure(kairos_rs::routes::metrics::configure_metrics);
/// # }
/// ```
/// 
/// # Prometheus Compatibility
/// 
/// All metrics are exported in Prometheus format via the `/metrics` endpoint,
/// making them compatible with standard monitoring and alerting infrastructure.
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Total number of HTTP requests processed (counter)
    pub requests_total: Arc<AtomicU64>,
    /// Number of successful HTTP requests (2xx status codes)
    pub requests_success: Arc<AtomicU64>,
    /// Number of failed HTTP requests (4xx, 5xx status codes)
    pub requests_error: Arc<AtomicU64>,
    /// Sum of all response times in milliseconds for average calculation
    pub response_time_sum: Arc<AtomicU64>,
    /// Current number of active HTTP connections being processed
    pub active_connections: Arc<AtomicU64>,
    /// Peak number of concurrent connections observed
    pub peak_connections: Arc<AtomicU64>,
    /// Total bytes of requests processed
    pub request_bytes_total: Arc<AtomicU64>,
    /// Total bytes of responses sent
    pub response_bytes_total: Arc<AtomicU64>,
    /// Number of requests with response time < 100ms
    pub response_time_bucket_100ms: Arc<AtomicU64>,
    /// Number of requests with response time < 500ms
    pub response_time_bucket_500ms: Arc<AtomicU64>,
    /// Number of requests with response time < 1000ms
    pub response_time_bucket_1s: Arc<AtomicU64>,
    /// Number of requests with response time < 5000ms
    pub response_time_bucket_5s: Arc<AtomicU64>,
    /// Number of requests with response time >= 5000ms
    pub response_time_bucket_inf: Arc<AtomicU64>,
    /// Number of 4xx client errors
    pub http_4xx_errors: Arc<AtomicU64>,
    /// Number of 5xx server errors
    pub http_5xx_errors: Arc<AtomicU64>,
    /// Number of timeout errors
    pub timeout_errors: Arc<AtomicU64>,
    /// Number of connection errors
    pub connection_errors: Arc<AtomicU64>,
    /// Number of cache lookups that returned a stored response (counter)
    pub cache_hits_total: Arc<AtomicU64>,
    /// Number of cache lookups that fell through to the upstream (counter)
    pub cache_misses_total: Arc<AtomicU64>,
    /// Application start time for uptime calculations
    pub start_time: Instant,
    /// Per-route metrics, keyed by route external path (e.g. "/users/{id}").
    /// Created on first request to a given route, never removed (bounded by
    /// the number of configured routes in practice). Stored as `Arc` so the
    /// `RouteMetrics` can be shared between the collector and the prometheus
    /// exposition without copying atomic counters.
    pub routes: Arc<RwLock<HashMap<String, Arc<RouteMetrics>>>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_success: Arc::new(AtomicU64::new(0)),
            requests_error: Arc::new(AtomicU64::new(0)),
            response_time_sum: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
            peak_connections: Arc::new(AtomicU64::new(0)),
            request_bytes_total: Arc::new(AtomicU64::new(0)),
            response_bytes_total: Arc::new(AtomicU64::new(0)),
            response_time_bucket_100ms: Arc::new(AtomicU64::new(0)),
            response_time_bucket_500ms: Arc::new(AtomicU64::new(0)),
            response_time_bucket_1s: Arc::new(AtomicU64::new(0)),
            response_time_bucket_5s: Arc::new(AtomicU64::new(0)),
            response_time_bucket_inf: Arc::new(AtomicU64::new(0)),
            http_4xx_errors: Arc::new(AtomicU64::new(0)),
            http_5xx_errors: Arc::new(AtomicU64::new(0)),
            timeout_errors: Arc::new(AtomicU64::new(0)),
            connection_errors: Arc::new(AtomicU64::new(0)),
            cache_hits_total: Arc::new(AtomicU64::new(0)),
            cache_misses_total: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}


// ============== PR4b: Per-route analytics ==============

/// Per-route metrics for one matched route (keyed by `external_path`).
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

    pub fn record_route_cache_hit(&self, route_id: &str) {
        let rm = self.get_or_create_route(route_id);
        rm.cache_hits_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_route_cache_miss(&self, route_id: &str) {
        let rm = self.get_or_create_route(route_id);
        rm.cache_misses_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn route_metrics_snapshot(&self) -> Vec<RouteMetricsSnapshot> {
        let map = self.routes.read().unwrap_or_else(|e| e.into_inner());
        let mut out: Vec<RouteMetricsSnapshot> = map
            .values()
            .map(|rm| RouteMetricsSnapshot::from_metrics(rm.as_ref()))
            .collect();
        out.sort_by(|a, b| a.route_id.cmp(&b.route_id));
        out
    }

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

fn escape_prometheus_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Admin endpoint that returns the per-route metrics breakdown as JSON.
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

impl MetricsCollector {
    /// Read current cache counters.
    ///
    /// Returns `(hits, misses, hit_rate)` where `hit_rate = hits / (hits + misses)`,
    /// or `0.0` if no traffic has been served yet. All values are read with
    /// `Ordering::Relaxed` since exact ordering between hit/miss counters is
    /// not required for the snapshot.
    pub fn cache_snapshot(&self) -> (u64, u64, f64) {
        let hits = self.cache_hits_total.load(Ordering::Relaxed);
        let misses = self.cache_misses_total.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }

    /// Records the completion of an HTTP request with detailed timing and status information.
    /// 
    /// This method atomically updates multiple metrics to track request patterns,
    /// performance characteristics, error categorization, and histogram buckets.
    /// It's called automatically by the RouteHandler for every processed request.
    /// 
    /// # Parameters
    /// 
    /// * `success` - Whether the request completed successfully (2xx status codes)
    /// * `response_time` - Total time taken to process the request
    /// * `status_code` - HTTP status code for error categorization
    /// * `request_bytes` - Size of the request in bytes (optional)
    /// * `response_bytes` - Size of the response in bytes (optional)
    /// 
    /// # Metrics Updated
    /// 
    /// - Increments `requests_total` counter
    /// - Updates response time histogram buckets
    /// - Categorizes errors by type (4xx, 5xx, timeout, connection)
    /// - Tracks data transfer volumes
    /// - Updates average response time calculation
    /// 
    /// # Thread Safety
    /// 
    /// Uses relaxed atomic operations for optimal performance in high-concurrency
    /// scenarios. All updates are atomic and consistent.
    pub fn record_request(
        &self, 
        success: bool, 
        response_time: Duration, 
        status_code: u16,
        request_bytes: Option<u64>,
        response_bytes: Option<u64>
    ) {
        // Basic counters
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.response_time_sum.fetch_add(response_time.as_millis() as u64, Ordering::Relaxed);
        
        // Track data transfer
        if let Some(bytes) = request_bytes {
            self.request_bytes_total.fetch_add(bytes, Ordering::Relaxed);
        }
        if let Some(bytes) = response_bytes {
            self.response_bytes_total.fetch_add(bytes, Ordering::Relaxed);
        }
        
        // Update histogram buckets based on response time
        let response_time_ms = response_time.as_millis() as u64;
        if response_time_ms <= 100 {
            self.response_time_bucket_100ms.fetch_add(1, Ordering::Relaxed);
        }
        if response_time_ms <= 500 {
            self.response_time_bucket_500ms.fetch_add(1, Ordering::Relaxed);
        }
        if response_time_ms <= 1000 {
            self.response_time_bucket_1s.fetch_add(1, Ordering::Relaxed);
        }
        if response_time_ms <= 5000 {
            self.response_time_bucket_5s.fetch_add(1, Ordering::Relaxed);
        } else {
            self.response_time_bucket_inf.fetch_add(1, Ordering::Relaxed);
        }
        
        // Categorize success/error and track specific error types
        if success {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_error.fetch_add(1, Ordering::Relaxed);
            
            // Categorize errors by status code
            match status_code {
                400..=499 => { self.http_4xx_errors.fetch_add(1, Ordering::Relaxed); },
                500..=599 => { self.http_5xx_errors.fetch_add(1, Ordering::Relaxed); },
                _ => {} // Other error types handled separately
            }
        }
    }
    
    /// Records a timeout error for requests that exceed the configured timeout.
    /// 
    /// This method is used to track timeout-related failures which help identify
    /// upstream service performance issues or network latency problems.
    /// 
    /// # Thread Safety
    /// 
    /// Uses atomic operations safe for concurrent access from multiple threads.
    #[allow(dead_code)] // Used for specific error tracking
    pub fn record_timeout_error(&self) {
        self.timeout_errors.fetch_add(1, Ordering::Relaxed);
        self.requests_error.fetch_add(1, Ordering::Relaxed);
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Records a connection error for requests that fail to establish connections.
    /// 
    /// This method tracks infrastructure-level failures separate from application
    /// errors to help distinguish between upstream service issues and gateway problems.
    /// 
    /// # Thread Safety
    /// 
    /// Uses atomic operations safe for concurrent access from multiple threads.
    #[allow(dead_code)] // Used for specific error tracking
    pub fn record_connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::Relaxed);
        self.requests_error.fetch_add(1, Ordering::Relaxed);
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Increments the active connections counter and updates peak if necessary.
    /// 
    /// Called when a new request begins processing to track concurrent load.
    /// Should be paired with `decrement_connections()` when request completes.
    /// Also tracks peak concurrent connections for capacity planning.
    /// 
    /// # Thread Safety
    /// 
    /// Uses atomic operations safe for concurrent access from multiple threads.
    pub fn increment_connections(&self) {
        let current = self.active_connections.fetch_add(1, Ordering::Relaxed) + 1;
        
        // Update peak connections if current exceeds previous peak
        let mut peak = self.peak_connections.load(Ordering::Relaxed);
        while current > peak {
            match self.peak_connections.compare_exchange_weak(
                peak, 
                current, 
                Ordering::Relaxed, 
                Ordering::Relaxed
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }
    
    /// Decrements the active connections counter.
    /// 
    /// Called when request processing completes to accurately track concurrent load.
    /// Must be called exactly once for each `increment_connections()` call.
    /// 
    /// # Thread Safety
    /// 
    /// Uses atomic operations safe for concurrent access from multiple threads.
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

/// HTTP endpoint that exposes gateway metrics in Prometheus format.
/// 
/// This endpoint provides comprehensive monitoring data for the gateway,
/// including request statistics, performance metrics, circuit breaker states,
/// and system health indicators. The output is compatible with Prometheus 
/// scraping and standard monitoring infrastructure.
/// 
/// # Parameters
/// 
/// * `metrics` - Shared MetricsCollector instance containing current statistics
/// * `route_handler` - Optional RouteHandler for circuit breaker state information
/// 
/// # Returns
/// 
/// * `Ok(HttpResponse)` - Prometheus-formatted metrics as plain text
/// * `Err(ActixError)` - Internal error (rare, indicates system issues)
/// 
/// # Metrics Exposed
/// 
/// - **kairos_requests_total**: Total HTTP requests processed (counter)
/// - **kairos_requests_success_total**: Successful requests (counter)
/// - **kairos_requests_error_total**: Failed requests (counter)
/// - **kairos_response_time_avg**: Average response time in milliseconds (gauge)
/// - **kairos_success_rate**: Success rate as percentage (gauge)
/// - **kairos_active_connections**: Current active connections (gauge)
/// - **kairos_uptime_seconds**: Service uptime in seconds (counter)
/// - **kairos_circuit_breaker_state**: Circuit breaker state by service (gauge)
/// - **kairos_circuit_breaker_failures**: Circuit breaker failure count (counter)
/// - **kairos_circuit_breaker_successes**: Circuit breaker success count (counter)
/// 
/// # Response Format
/// 
/// Returns metrics in Prometheus exposition format:
/// ```text
/// # HELP kairos_requests_total Total number of HTTP requests
/// # TYPE kairos_requests_total counter
/// kairos_requests_total 1547
/// 
/// # HELP kairos_circuit_breaker_state Circuit breaker state (0=Closed, 1=Open, 2=HalfOpen)
/// # TYPE kairos_circuit_breaker_state gauge
/// kairos_circuit_breaker_state{service="api.example.com:443"} 0
/// ```
/// 
/// # Performance Characteristics
/// 
/// - **Lightweight**: Uses atomic loads with minimal computation
/// - **Real-time**: Reflects current system state without caching
/// - **Non-blocking**: Does not interfere with request processing
/// 
/// # Monitoring Integration
/// 
/// This endpoint can be scraped by:
/// - Prometheus monitoring system
/// - Grafana dashboards  
/// - Custom monitoring tools
/// - Health check systems
pub async fn metrics_endpoint(
    metrics: web::Data<MetricsCollector>, 
    route_handler: Option<web::Data<RouteHandler>>
) -> Result<HttpResponse> {
    let total_requests = metrics.requests_total.load(Ordering::Relaxed);
    let success_requests = metrics.requests_success.load(Ordering::Relaxed);
    let error_requests = metrics.requests_error.load(Ordering::Relaxed);
    let response_time_sum = metrics.response_time_sum.load(Ordering::Relaxed);
    let active_connections = metrics.active_connections.load(Ordering::Relaxed);
    let peak_connections = metrics.peak_connections.load(Ordering::Relaxed);
    let request_bytes = metrics.request_bytes_total.load(Ordering::Relaxed);
    let response_bytes = metrics.response_bytes_total.load(Ordering::Relaxed);
    let bucket_100ms = metrics.response_time_bucket_100ms.load(Ordering::Relaxed);
    let bucket_500ms = metrics.response_time_bucket_500ms.load(Ordering::Relaxed);
    let bucket_1s = metrics.response_time_bucket_1s.load(Ordering::Relaxed);
    let bucket_5s = metrics.response_time_bucket_5s.load(Ordering::Relaxed);
    let bucket_inf = metrics.response_time_bucket_inf.load(Ordering::Relaxed);
    let http_4xx_errors = metrics.http_4xx_errors.load(Ordering::Relaxed);
    let http_5xx_errors = metrics.http_5xx_errors.load(Ordering::Relaxed);
    let timeout_errors = metrics.timeout_errors.load(Ordering::Relaxed);
    let connection_errors = metrics.connection_errors.load(Ordering::Relaxed);
    let cache_hits_total = metrics.cache_hits_total.load(Ordering::Relaxed);
    let cache_misses_total = metrics.cache_misses_total.load(Ordering::Relaxed);
    let uptime = metrics.start_time.elapsed().as_secs();
    
    let avg_response_time = if total_requests > 0 {
        response_time_sum as f64 / total_requests as f64
    } else {
        0.0
    };
    
    let success_rate = if total_requests > 0 {
        (success_requests as f64 / total_requests as f64) * 100.0
    } else {
        100.0
    };

    // Generate circuit breaker metrics if route handler is available
    let mut circuit_breaker_metrics = String::new();
    if let Some(handler) = route_handler {
        let cb_states = handler.get_circuit_breaker_states();
        
        if !cb_states.is_empty() {
            circuit_breaker_metrics.push_str("\n# HELP kairos_circuit_breaker_state Circuit breaker state (0=Closed, 1=Open, 2=HalfOpen)\n");
            circuit_breaker_metrics.push_str("# TYPE kairos_circuit_breaker_state gauge\n");
            
            circuit_breaker_metrics.push_str("\n# HELP kairos_circuit_breaker_failures Circuit breaker failure count\n");
            circuit_breaker_metrics.push_str("# TYPE kairos_circuit_breaker_failures counter\n");
            
            circuit_breaker_metrics.push_str("\n# HELP kairos_circuit_breaker_successes Circuit breaker success count\n");
            circuit_breaker_metrics.push_str("# TYPE kairos_circuit_breaker_successes counter\n");
            
            for (service, (state, failures, successes)) in cb_states {
                let state_value = match state {
                    crate::services::circuit_breaker::CircuitState::Closed => 0,
                    crate::services::circuit_breaker::CircuitState::Open => 1,
                    crate::services::circuit_breaker::CircuitState::HalfOpen => 2,
                };
                
                circuit_breaker_metrics.push_str(&format!(
                    "kairos_circuit_breaker_state{{service=\"{}\"}} {}\n",
                    service, state_value
                ));
                circuit_breaker_metrics.push_str(&format!(
                    "kairos_circuit_breaker_failures{{service=\"{}\"}} {}\n",
                    service, failures
                ));
                circuit_breaker_metrics.push_str(&format!(
                    "kairos_circuit_breaker_successes{{service=\"{}\"}} {}\n",
                    service, successes
                ));
            }
        }
    }

    // PR4b: per-route Prometheus section (empty string when no route has been seen yet).
    let route_metrics_text = metrics.route_prometheus_text();

    let metrics_text = format!(
        r#"# HELP kairos_requests_total Total number of HTTP requests
# TYPE kairos_requests_total counter
kairos_requests_total {}

# HELP kairos_requests_success_total Total number of successful HTTP requests
# TYPE kairos_requests_success_total counter
kairos_requests_success_total {}

# HELP kairos_requests_error_total Total number of failed HTTP requests
# TYPE kairos_requests_error_total counter
kairos_requests_error_total {}

# HELP kairos_http_4xx_errors_total Total number of 4xx client errors
# TYPE kairos_http_4xx_errors_total counter
kairos_http_4xx_errors_total {}

# HELP kairos_http_5xx_errors_total Total number of 5xx server errors
# TYPE kairos_http_5xx_errors_total counter
kairos_http_5xx_errors_total {}

# HELP kairos_timeout_errors_total Total number of timeout errors
# TYPE kairos_timeout_errors_total counter
kairos_timeout_errors_total {}

# HELP kairos_connection_errors_total Total number of connection errors
# TYPE kairos_connection_errors_total counter
kairos_connection_errors_total {}

# HELP kairos_response_time_avg Average response time in milliseconds
# TYPE kairos_response_time_avg gauge
kairos_response_time_avg {:.2}

# HELP kairos_response_time_bucket Response time histogram buckets
# TYPE kairos_response_time_bucket histogram
kairos_response_time_bucket{{le="100"}} {}
kairos_response_time_bucket{{le="500"}} {}
kairos_response_time_bucket{{le="1000"}} {}
kairos_response_time_bucket{{le="5000"}} {}
kairos_response_time_bucket{{le="+Inf"}} {}

# HELP kairos_request_bytes_total Total bytes received in requests
# TYPE kairos_request_bytes_total counter
kairos_request_bytes_total {}

# HELP kairos_response_bytes_total Total bytes sent in responses
# TYPE kairos_response_bytes_total counter
kairos_response_bytes_total {}

# HELP kairos_success_rate Success rate percentage
# TYPE kairos_success_rate gauge
kairos_success_rate {:.2}

# HELP kairos_active_connections Current number of active connections
# TYPE kairos_active_connections gauge
kairos_active_connections {}

# HELP kairos_peak_connections Peak number of concurrent connections
# TYPE kairos_peak_connections gauge
kairos_peak_connections {}

# HELP kairos_cache_hits_total Total cache lookups that returned a stored response.
# TYPE kairos_cache_hits_total counter
kairos_cache_hits_total {}

# HELP kairos_cache_misses_total Total cache lookups that fell through to the upstream.
# TYPE kairos_cache_misses_total counter
kairos_cache_misses_total {}

# HELP kairos_uptime_seconds Service uptime in seconds
# TYPE kairos_uptime_seconds counter
kairos_uptime_seconds {}{}{}
"#,
        total_requests,
        success_requests,
        error_requests,
        http_4xx_errors,
        http_5xx_errors,
        timeout_errors,
        connection_errors,
        avg_response_time,
        bucket_100ms,
        bucket_500ms,
        bucket_1s,
        bucket_5s,
        bucket_inf,
        request_bytes,
        response_bytes,
        success_rate,
        active_connections,
        peak_connections,
        cache_hits_total,
        cache_misses_total,
        uptime,
        route_metrics_text,
        circuit_breaker_metrics
    );

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_text))
}

/// Query parameters for historical metrics.
#[derive(Debug, Deserialize)]
pub struct HistoricalMetricsQuery {
    /// Name of the metric to query
    pub name: String,
    /// Start timestamp (ISO 8601)
    pub start: DateTime<Utc>,
    /// End timestamp (ISO 8601)
    pub end: DateTime<Utc>,
    /// Optional aggregation interval
    pub interval: Option<AggregationInterval>,
}

/// Lists all available metrics tracked by the historical store.
///
/// # Returns
///
/// JSON list of metric names.
pub async fn list_metrics(store: web::Data<MetricsStore>) -> Result<HttpResponse> {
    let metrics = store.list_metrics();
    Ok(HttpResponse::Ok().json(metrics))
}

/// Retrieves historical metrics data based on time range and optional aggregation.
///
/// # Query Parameters
///
/// * `name` - Name of the metric to query
/// * `start` - Start timestamp (ISO 8601)
/// * `end` - End timestamp (ISO 8601)
/// * `interval` - Optional aggregation interval (one_minute, five_minutes, one_hour, one_day)
///
/// # Returns
///
/// JSON response containing either raw data points or aggregated statistics.
pub async fn get_historical_metrics(
    store: web::Data<MetricsStore>,
    query: web::Query<HistoricalMetricsQuery>,
) -> Result<HttpResponse> {
    if let Some(interval) = query.interval {
        let data = store.query_aggregated(&query.name, query.start, query.end, interval);
        Ok(HttpResponse::Ok().json(data))
    } else {
        let data = store.query(&query.name, query.start, query.end);
        Ok(HttpResponse::Ok().json(data))
    }
}

/// Query parameters for latency percentile retrieval.
///
/// Used by the `/api/metrics/latency/percentiles` endpoint to derive
/// p50/p95/p99 (or any custom subset) from histogram observations stored
/// in the metrics store.
#[derive(Debug, Deserialize)]
pub struct LatencyPercentileQuery {
    /// Metric name. Must contain `MetricValue::Histogram` observations.
    pub name: String,
    /// Start of the time range (inclusive, ISO 8601).
    pub start: DateTime<Utc>,
    /// End of the time range (inclusive, ISO 8601).
    pub end: DateTime<Utc>,
    /// Window size used to bucket observations before computing the per-window CDF.
    pub interval: AggregationInterval,
    /// Percentile values to compute, e.g. `[50.0, 95.0, 99.0]`.
    /// Defaults to `[50.0, 95.0, 99.0]` when omitted.
    pub percentiles: Option<Vec<f64>>,
}
/// Configures the metrics endpoint route for Actix Web application.
/// 
/// This function registers the `/metrics` endpoint that exposes Prometheus-compatible
/// metrics for monitoring and observability. It should be called during application
/// setup to enable metrics collection.
/// 
/// # Parameters
/// 
/// * `cfg` - Mutable reference to Actix Web service configuration
/// 
/// # Route Configuration
/// 
/// - **Path**: `/metrics`
/// - **Method**: GET only
/// - **Handler**: `metrics_endpoint` function
/// - **Response**: Prometheus exposition format (text/plain)
/// 
/// # Usage
/// 
/// ```rust
/// use actix_web::{App, web};
/// use kairos_rs::routes::metrics;
/// 
/// # fn example() {
/// # let metrics_collector = kairos_rs::routes::metrics::MetricsCollector::default();
/// let app = App::new()
///     .app_data(web::Data::new(metrics_collector))
///     .configure(metrics::configure_metrics);
/// # }
/// ```
/// 
/// # Integration
/// 
/// Must be used alongside a shared `MetricsCollector` instance in application data.
/// The endpoint automatically accesses the collector to provide real-time metrics.
pub fn configure_metrics(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(metrics_endpoint))
       .route("/api/metrics/list", web::get().to(list_metrics))
       .route("/api/metrics/history", web::get().to(get_historical_metrics))
       .route("/api/metrics/latency/percentiles", web::get().to(get_latency_percentiles))
       .route("/api/metrics/routes", web::get().to(route_metrics_endpoint));
}
