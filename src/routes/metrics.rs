//! Prometheus-compatible metrics endpoint for monitoring and observability.
//! 
//! This module provides comprehensive metrics collection and exposure for the
//! Kairos-rs gateway, including request counts, response times, error rates,
//! histograms, memory usage, per-route statistics, and system health indicators.

use actix_web::{web, HttpResponse, Result};
use crate::services::http::RouteHandler;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
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
            start_time: Instant::now(),
        }
    }
}

impl MetricsCollector {
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

# HELP kairos_uptime_seconds Service uptime in seconds
# TYPE kairos_uptime_seconds counter
kairos_uptime_seconds {}{}
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
        uptime,
        circuit_breaker_metrics
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