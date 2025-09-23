//! Prometheus-compatible metrics endpoint for monitoring and observability.
//! 
//! This module provides comprehensive metrics collection and exposure for the
//! Kairos-rs gateway, including request counts, response times, error rates,
//! and system health indicators.

use actix_web::{web, HttpResponse, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Thread-safe metrics collector for comprehensive gateway observability.
/// 
/// The `MetricsCollector` provides atomic counters and gauges for tracking
/// gateway performance, request patterns, and system health. All metrics
/// are thread-safe and designed for high-concurrency environments.
/// 
/// # Metrics Tracked
/// 
/// - **Request Counters**: Total, successful, and failed request counts
/// - **Performance**: Response time tracking and averages
/// - **Concurrency**: Active connection monitoring
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
    /// Application start time for uptime calculations
    pub start_time: Instant,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_success: Arc::new(AtomicU64::new(0)),
            requests_error: Arc::new(AtomicU64::new(0)),
            response_time_sum: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }
}

impl MetricsCollector {
    /// Records the completion of an HTTP request with timing and status information.
    /// 
    /// This method atomically updates multiple metrics to track request patterns
    /// and performance characteristics. It's called automatically by the
    /// RouteHandler for every processed request.
    /// 
    /// # Parameters
    /// 
    /// * `success` - Whether the request completed successfully (2xx status codes)
    /// * `response_time` - Total time taken to process the request
    /// 
    /// # Metrics Updated
    /// 
    /// - Increments `requests_total` counter
    /// - Adds response time to `response_time_sum` for average calculation
    /// - Increments either `requests_success` or `requests_error` based on outcome
    /// 
    /// # Thread Safety
    /// 
    /// Uses relaxed atomic operations for optimal performance in high-concurrency
    /// scenarios. All updates are atomic and consistent.
    pub fn record_request(&self, success: bool, response_time: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.response_time_sum.fetch_add(response_time.as_millis() as u64, Ordering::Relaxed);
        
        if success {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_error.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Increments the active connections counter.
    /// 
    /// Called when a new request begins processing to track concurrent load.
    /// Should be paired with `decrement_connections()` when request completes.
    /// 
    /// # Thread Safety
    /// 
    /// Uses atomic operations safe for concurrent access from multiple threads.
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
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
/// including request statistics, performance metrics, and system health
/// indicators. The output is compatible with Prometheus scraping and
/// standard monitoring infrastructure.
/// 
/// # Parameters
/// 
/// * `metrics` - Shared MetricsCollector instance containing current statistics
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
/// 
/// # Response Format
/// 
/// Returns metrics in Prometheus exposition format:
/// ```text
/// # HELP kairos_requests_total Total number of HTTP requests
/// # TYPE kairos_requests_total counter
/// kairos_requests_total 1547
/// 
/// # HELP kairos_response_time_avg Average response time in milliseconds  
/// # TYPE kairos_response_time_avg gauge
/// kairos_response_time_avg 23.45
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
pub async fn metrics_endpoint(metrics: web::Data<MetricsCollector>) -> Result<HttpResponse> {
    let total_requests = metrics.requests_total.load(Ordering::Relaxed);
    let success_requests = metrics.requests_success.load(Ordering::Relaxed);
    let error_requests = metrics.requests_error.load(Ordering::Relaxed);
    let response_time_sum = metrics.response_time_sum.load(Ordering::Relaxed);
    let active_connections = metrics.active_connections.load(Ordering::Relaxed);
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

# HELP kairos_response_time_avg Average response time in milliseconds
# TYPE kairos_response_time_avg gauge
kairos_response_time_avg {:.2}

# HELP kairos_success_rate Success rate percentage
# TYPE kairos_success_rate gauge
kairos_success_rate {:.2}

# HELP kairos_active_connections Current number of active connections
# TYPE kairos_active_connections gauge
kairos_active_connections {}

# HELP kairos_uptime_seconds Service uptime in seconds
# TYPE kairos_uptime_seconds counter
kairos_uptime_seconds {}
"#,
        total_requests,
        success_requests,
        error_requests,
        avg_response_time,
        success_rate,
        active_connections,
        uptime
    );

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics_text))
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
    cfg.route("/metrics", web::get().to(metrics_endpoint));
}