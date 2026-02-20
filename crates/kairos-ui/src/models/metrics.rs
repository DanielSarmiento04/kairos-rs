//! Metrics data structures for UI display.

use chrono::{DateTime, Utc};
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

/// Time-series metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub value: MetricValue,
}

/// Metric value types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { le: f64, count: u64 },
}

/// Aggregation interval for metrics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AggregationInterval {
    OneMinute,
    FiveMinutes,
    OneHour,
    OneDay,
}

impl std::fmt::Display for AggregationInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OneMinute => write!(f, "1 Minute"),
            Self::FiveMinutes => write!(f, "5 Minutes"),
            Self::OneHour => write!(f, "1 Hour"),
            Self::OneDay => write!(f, "1 Day"),
        }
    }
}

/// Aggregated metric data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

/// System metrics from real-time WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub uptime: u64,
    pub active_connections: usize,
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

    /// Parse Prometheus-formatted metrics text.
    ///
    /// This is a simple parser for demo purposes.
    /// In production, use a proper Prometheus parser library.
    pub fn parse_prometheus(text: &str) -> Result<Self, String> {
        let mut metrics = Self::default();
        let mut success_total = 0u64;

        for line in text.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let name = parts[0];
            let value_str = parts.last().unwrap();
            let value_f64 = value_str.parse::<f64>().unwrap_or(0.0);
            let value_u64 = value_f64 as u64;

            if name == "kairos_requests_total" {
                metrics.requests_total = value_u64;
            } else if name == "kairos_requests_success_total" {
                success_total = value_u64;
            } else if name == "kairos_http_4xx_errors_total" {
                metrics.http_4xx_errors = value_u64;
            } else if name == "kairos_http_5xx_errors_total" {
                metrics.http_5xx_errors = value_u64;
            } else if name == "kairos_timeout_errors_total" {
                metrics.timeout_errors = value_u64;
            } else if name == "kairos_connection_errors_total" {
                metrics.connection_errors = value_u64;
            } else if name == "kairos_response_time_avg" {
                metrics.response_time_avg = value_f64;
            } else if name == "kairos_active_connections" {
                metrics.active_connections = value_u64 as u32;
                metrics.requests_in_flight = value_u64 as u32;
            } else if name == "kairos_peak_connections" {
                metrics.peak_connections = value_u64 as u32;
            } else if name == "kairos_request_bytes_total" {
                metrics.request_bytes_total = value_u64;
            } else if name == "kairos_response_bytes_total" {
                metrics.response_bytes_total = value_u64;
            } else if name == "kairos_success_rate" {
                metrics.success_rate = value_f64;
            } else if name.starts_with("kairos_response_time_bucket") {
                if name.contains("le=\"100\"") {
                    metrics.response_time_bucket_100ms = value_u64;
                } else if name.contains("le=\"500\"") {
                    metrics.response_time_bucket_500ms = value_u64;
                } else if name.contains("le=\"1000\"") {
                    metrics.response_time_bucket_1s = value_u64;
                } else if name.contains("le=\"5000\"") {
                    metrics.response_time_bucket_5s = value_u64;
                } else if name.contains("le=\"+Inf\"") {
                    metrics.response_time_bucket_inf = value_u64;
                }
            } else if name.starts_with("kairos_circuit_breaker_state") {
                // Parse circuit breaker metrics
                // Format: kairos_circuit_breaker_state{service="http://localhost:8081"} 0
                if let Some(start) = name.find("service=\"") {
                    if let Some(_end) = name[start..].find("\",") { // Handle multiple labels if any, or just end quote
                         // Simplified parsing for now, assuming service is the only label or first
                    }
                    let service_start = start + 9;
                    if let Some(service_end) = name[service_start..].find('"') {
                        let service = &name[service_start..service_start + service_end];

                        // Find or create CB metric
                        let mut found = false;
                        for cb in &mut metrics.circuit_breakers {
                            if cb.route == service {
                                cb.state = match value_u64 {
                                    0 => CircuitBreakerState::Closed,
                                    1 => CircuitBreakerState::Open,
                                    2 => CircuitBreakerState::HalfOpen,
                                    _ => CircuitBreakerState::Closed,
                                };
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            metrics.circuit_breakers.push(CircuitBreakerMetrics {
                                route: service.to_string(),
                                state: match value_u64 {
                                    0 => CircuitBreakerState::Closed,
                                    1 => CircuitBreakerState::Open,
                                    2 => CircuitBreakerState::HalfOpen,
                                    _ => CircuitBreakerState::Closed,
                                },
                                failure_count: 0,
                                success_count: 0,
                                last_failure_time: None,
                                next_attempt_time: None,
                            });
                        }
                    }
                }
            } else if name.starts_with("kairos_circuit_breaker_failures") {
                if let Some(start) = name.find("service=\"") {
                    let service_start = start + 9;
                    if let Some(service_end) = name[service_start..].find('"') {
                        let service = &name[service_start..service_start + service_end];
                        for cb in &mut metrics.circuit_breakers {
                            if cb.route == service {
                                cb.failure_count = value_u64 as u32;
                                break;
                            }
                        }
                    }
                }
            } else if name.starts_with("kairos_circuit_breaker_successes") {
                if let Some(start) = name.find("service=\"") {
                    let service_start = start + 9;
                    if let Some(service_end) = name[service_start..].find('"') {
                        let service = &name[service_start..service_start + service_end];
                        for cb in &mut metrics.circuit_breakers {
                            if cb.route == service {
                                cb.success_count = value_u64 as u32;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if metrics.requests_total > 0 {
            metrics.success_rate = (success_total as f64 / metrics.requests_total as f64) * 100.0;
        } else {
            metrics.success_rate = 100.0;
        }

        metrics.data_transferred_bytes = metrics.request_bytes_total + metrics.response_bytes_total;

        Ok(metrics)
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
