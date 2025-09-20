//! Prometheus-compatible metrics endpoint for monitoring and observability.
//! 
//! This module provides comprehensive metrics collection and exposure for the
//! Kairos-rs gateway, including request counts, response times, error rates,
//! and system health indicators.

use actix_web::{web, HttpResponse, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Global metrics collector for the gateway
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    pub requests_total: Arc<AtomicU64>,
    pub requests_success: Arc<AtomicU64>,
    pub requests_error: Arc<AtomicU64>,
    pub response_time_sum: Arc<AtomicU64>,
    pub active_connections: Arc<AtomicU64>,
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
    pub fn record_request(&self, success: bool, response_time: Duration) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.response_time_sum.fetch_add(response_time.as_millis() as u64, Ordering::Relaxed);
        
        if success {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_error.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Prometheus-compatible metrics endpoint
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

pub fn configure_metrics(cfg: &mut web::ServiceConfig) {
    cfg.route("/metrics", web::get().to(metrics_endpoint));
}