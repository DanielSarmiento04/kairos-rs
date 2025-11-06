//! WebSocket-specific metrics collection and reporting.
//!
//! This module provides detailed metrics for WebSocket connections including:
//! - Active connection count
//! - Message throughput (sent/received)
//! - Message size distribution
//! - Connection duration
//! - Error rates

use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;

/// Global WebSocket metrics tracker
pub struct WebSocketMetricsGlobal {
    /// Total active WebSocket connections
    active_connections: Arc<AtomicI64>,
    /// Total messages sent to clients
    messages_sent: Arc<AtomicU64>,
    /// Total messages received from clients
    messages_received: Arc<AtomicU64>,
    /// Total bytes sent
    bytes_sent: Arc<AtomicU64>,
    /// Total bytes received
    bytes_received: Arc<AtomicU64>,
    /// Total connections established
    connections_total: Arc<AtomicU64>,
    /// Total connection errors
    connection_errors: Arc<AtomicU64>,
}

impl WebSocketMetricsGlobal {
    /// Creates a new global WebSocket metrics tracker
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(AtomicI64::new(0)),
            messages_sent: Arc::new(AtomicU64::new(0)),
            messages_received: Arc::new(AtomicU64::new(0)),
            bytes_sent: Arc::new(AtomicU64::new(0)),
            bytes_received: Arc::new(AtomicU64::new(0)),
            connections_total: Arc::new(AtomicU64::new(0)),
            connection_errors: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Gets the current number of active connections
    pub fn get_active_connections(&self) -> i64 {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Gets the total messages sent
    pub fn get_messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }

    /// Gets the total messages received
    pub fn get_messages_received(&self) -> u64 {
        self.messages_received.load(Ordering::Relaxed)
    }

    /// Gets the total bytes sent
    pub fn get_bytes_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    /// Gets the total bytes received
    pub fn get_bytes_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    /// Gets the total connections established
    pub fn get_connections_total(&self) -> u64 {
        self.connections_total.load(Ordering::Relaxed)
    }

    /// Gets the total connection errors
    pub fn get_connection_errors(&self) -> u64 {
        self.connection_errors.load(Ordering::Relaxed)
    }
}

/// WebSocket metrics tracker for a single connection
pub struct WebSocketMetrics {
    #[allow(dead_code)]
    route: String,
    #[allow(dead_code)]
    backend: String,
    connection_start: std::time::Instant,
    global: Arc<WebSocketMetricsGlobal>,
}

impl WebSocketMetrics {
    /// Creates a new WebSocket metrics tracker
    ///
    /// # Parameters
    ///
    /// * `route` - External route path
    /// * `backend` - Backend server identifier
    ///
    /// # Examples
    ///
    /// ```
    /// use kairos_rs::services::websocket_metrics::{WebSocketMetrics, WebSocketMetricsGlobal};
    /// use std::sync::Arc;
    ///
    /// let global = Arc::new(WebSocketMetricsGlobal::new());
    /// let metrics = WebSocketMetrics::new_with_global("/ws/chat".to_string(), "backend1".to_string(), global);
    /// ```
    pub fn new_with_global(route: String, backend: String, global: Arc<WebSocketMetricsGlobal>) -> Self {
        // Increment active connections
        global.active_connections.fetch_add(1, Ordering::Relaxed);
        // Increment total connections
        global.connections_total.fetch_add(1, Ordering::Relaxed);

        Self {
            route,
            backend,
            connection_start: std::time::Instant::now(),
            global,
        }
    }

    /// Creates a new WebSocket metrics tracker with a new global instance
    pub fn new(route: String, backend: String) -> Self {
        Self::new_with_global(route, backend, Arc::new(WebSocketMetricsGlobal::new()))
    }

    /// Records a message sent to the client
    ///
    /// # Parameters
    ///
    /// * `message_type` - Type of message (text, binary, ping, pong, close)
    /// * `size_bytes` - Size of the message in bytes
    pub fn record_message_sent(&self, _message_type: &str, size_bytes: usize) {
        self.global.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.global.bytes_sent.fetch_add(size_bytes as u64, Ordering::Relaxed);
    }

    /// Records a message received from the client
    ///
    /// # Parameters
    ///
    /// * `message_type` - Type of message (text, binary, ping, pong, close)
    /// * `size_bytes` - Size of the message in bytes
    pub fn record_message_received(&self, _message_type: &str, size_bytes: usize) {
        self.global.messages_received.fetch_add(1, Ordering::Relaxed);
        self.global.bytes_received.fetch_add(size_bytes as u64, Ordering::Relaxed);
    }

    /// Records a connection error
    ///
    /// # Parameters
    ///
    /// * `error_type` - Type of error (upgrade_failed, backend_unreachable, forwarding_error, etc.)
    pub fn record_error(&self, _error_type: &str) {
        self.global.connection_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Records ping/pong round-trip time
    ///
    /// # Parameters
    ///
    /// * `rtt_seconds` - Round-trip time in seconds
    pub fn record_ping_rtt(&self, _rtt_seconds: f64) {
        // For now, we don't track RTT with simple atomics
        // Could be added later with histograms
    }

    /// Records connection closure
    ///
    /// # Parameters
    ///
    /// * `close_reason` - Reason for connection closure (normal, error, timeout, etc.)
    pub fn record_close(&self, _close_reason: &str) {
        let _duration = self.connection_start.elapsed().as_secs_f64();
        
        // Decrement active connections
        self.global.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Drop for WebSocketMetrics {
    fn drop(&mut self) {
        // Ensure we decrement active connections if not explicitly closed
        let current = self.global.active_connections.load(Ordering::Relaxed);
        if current > 0 {
            self.global.active_connections.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
