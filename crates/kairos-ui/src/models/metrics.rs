//! Metrics data structures for UI display.

use serde::{Deserialize, Serialize};

/// Metrics data structure parsed from Prometheus format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub requests_total: u64,
    pub success_rate: f64,
    pub response_time_avg: f64,
    pub active_connections: u32,
    pub peak_connections: u32,
    pub http_4xx_errors: u64,
    pub http_5xx_errors: u64,
    pub timeout_errors: u64,
    pub connection_errors: u64,
    pub response_time_bucket_100ms: u64,
    pub response_time_bucket_500ms: u64,
    pub response_time_bucket_1s: u64,
    pub response_time_bucket_5s: u64,
    pub response_time_bucket_inf: u64,
    pub requests_in_flight: u32,
    pub data_transferred_bytes: u64,
    pub request_bytes_total: u64,
    pub response_bytes_total: u64,
    pub circuit_breakers: Vec<CircuitBreakerMetrics>,
}

impl MetricsData {
    /// Format bytes into human-readable format.
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;
        
        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

impl Default for MetricsData {
    fn default() -> Self {
        Self {
            requests_total: 0,
            success_rate: 0.0,
            response_time_avg: 0.0,
            active_connections: 0,
            peak_connections: 0,
            http_4xx_errors: 0,
            http_5xx_errors: 0,
            timeout_errors: 0,
            connection_errors: 0,
            response_time_bucket_100ms: 0,
            response_time_bucket_500ms: 0,
            response_time_bucket_1s: 0,
            response_time_bucket_5s: 0,
            response_time_bucket_inf: 0,
            requests_in_flight: 0,
            data_transferred_bytes: 0,
            request_bytes_total: 0,
            response_bytes_total: 0,
            circuit_breakers: Vec::new(),
        }
    }
}

/// Circuit breaker state enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreakerState {
    /// Check if the circuit breaker is in a healthy state.
    pub fn is_healthy(&self) -> bool {
        matches!(self, CircuitBreakerState::Closed)
    }
}

/// Circuit breaker metrics for a specific route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub route: String,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<String>,
    pub next_attempt_time: Option<String>,
}
