//! WebSocket metrics integration tests.
//!
//! Tests for WebSocket-specific metrics collection including connection tracking,
//! message counting, and error recording.

use kairos_rs::services::websocket_metrics::{WebSocketMetrics, WebSocketMetricsGlobal};
use std::sync::Arc;

#[test]
fn test_websocket_metrics_lifecycle() {
    let global = Arc::new(WebSocketMetricsGlobal::new());
    let metrics = WebSocketMetrics::new_with_global(
        "/ws/test".to_string(),
        "backend1".to_string(),
        global.clone(),
    );

    assert_eq!(global.get_active_connections(), 1);
    assert_eq!(global.get_connections_total(), 1);

    // Record some messages
    metrics.record_message_sent("text", 100);
    metrics.record_message_received("text", 50);
    metrics.record_message_sent("binary", 1000);

    assert_eq!(global.get_messages_sent(), 2);
    assert_eq!(global.get_messages_received(), 1);
    assert_eq!(global.get_bytes_sent(), 1100);
    assert_eq!(global.get_bytes_received(), 50);

    // Record error
    metrics.record_error("forwarding_error");
    assert_eq!(global.get_connection_errors(), 1);

    // Record ping RTT
    metrics.record_ping_rtt(0.05);

    // Record close
    metrics.record_close("normal");
    assert_eq!(global.get_active_connections(), 0);
}

#[test]
fn test_message_size_tracking() {
    let global = Arc::new(WebSocketMetricsGlobal::new());
    let metrics = WebSocketMetrics::new_with_global(
        "/ws/size_test".to_string(),
        "backend2".to_string(),
        global.clone(),
    );

    // Record messages of various sizes
    metrics.record_message_sent("text", 10);
    metrics.record_message_sent("text", 1000);
    metrics.record_message_sent("binary", 100_000);

    assert_eq!(global.get_messages_sent(), 3);
    assert_eq!(global.get_bytes_sent(), 101_010);

    metrics.record_close("normal");
}

#[test]
fn test_global_metrics() {
    let global = Arc::new(WebSocketMetricsGlobal::new());

    // Create multiple connections
    let metrics1 = WebSocketMetrics::new_with_global(
        "/ws/test1".to_string(),
        "backend1".to_string(),
        global.clone(),
    );
    let metrics2 = WebSocketMetrics::new_with_global(
        "/ws/test2".to_string(),
        "backend2".to_string(),
        global.clone(),
    );

    assert_eq!(global.get_active_connections(), 2);
    assert_eq!(global.get_connections_total(), 2);

    metrics1.record_message_sent("text", 100);
    metrics2.record_message_sent("text", 200);

    assert_eq!(global.get_messages_sent(), 2);
    assert_eq!(global.get_bytes_sent(), 300);

    metrics1.record_close("normal");
    assert_eq!(global.get_active_connections(), 1);

    metrics2.record_close("normal");
    assert_eq!(global.get_active_connections(), 0);
}

#[test]
fn test_connection_cleanup_on_drop() {
    let global = Arc::new(WebSocketMetricsGlobal::new());

    {
        let _metrics = WebSocketMetrics::new_with_global(
            "/ws/drop_test".to_string(),
            "backend1".to_string(),
            global.clone(),
        );
        assert_eq!(global.get_active_connections(), 1);
    } // metrics dropped here

    // Connection should be cleaned up even without explicit close
    assert_eq!(global.get_active_connections(), 0);
}

#[test]
fn test_concurrent_connections() {
    let global = Arc::new(WebSocketMetricsGlobal::new());

    // Simulate multiple concurrent connections
    let mut connections = Vec::new();
    for i in 0..10 {
        let metrics = WebSocketMetrics::new_with_global(
            format!("/ws/conn{}", i),
            format!("backend{}", i % 3),
            global.clone(),
        );
        connections.push(metrics);
    }

    assert_eq!(global.get_active_connections(), 10);
    assert_eq!(global.get_connections_total(), 10);

    // Send messages from all connections
    for metrics in &connections {
        metrics.record_message_sent("text", 100);
        metrics.record_message_received("text", 50);
    }

    assert_eq!(global.get_messages_sent(), 10);
    assert_eq!(global.get_messages_received(), 10);
    assert_eq!(global.get_bytes_sent(), 1000);
    assert_eq!(global.get_bytes_received(), 500);

    // Close half the connections
    for metrics in connections.iter().take(5) {
        metrics.record_close("normal");
    }

    assert_eq!(global.get_active_connections(), 5);

    // Drop remaining connections
    drop(connections);
    assert_eq!(global.get_active_connections(), 0);
}

#[test]
fn test_error_tracking() {
    let global = Arc::new(WebSocketMetricsGlobal::new());
    let metrics = WebSocketMetrics::new_with_global(
        "/ws/error_test".to_string(),
        "backend1".to_string(),
        global.clone(),
    );

    assert_eq!(global.get_connection_errors(), 0);

    // Record various error types
    metrics.record_error("upgrade_failed");
    assert_eq!(global.get_connection_errors(), 1);

    metrics.record_error("backend_unreachable");
    assert_eq!(global.get_connection_errors(), 2);

    metrics.record_error("forwarding_error");
    assert_eq!(global.get_connection_errors(), 3);

    metrics.record_close("error");
}

#[test]
fn test_mixed_message_types() {
    let global = Arc::new(WebSocketMetricsGlobal::new());
    let metrics = WebSocketMetrics::new_with_global(
        "/ws/mixed_test".to_string(),
        "backend1".to_string(),
        global.clone(),
    );

    // Send different message types
    metrics.record_message_sent("text", 100);
    metrics.record_message_sent("binary", 500);
    metrics.record_message_sent("ping", 0);
    metrics.record_message_sent("pong", 0);

    // Receive different message types
    metrics.record_message_received("text", 200);
    metrics.record_message_received("binary", 300);
    metrics.record_message_received("ping", 0);
    metrics.record_message_received("pong", 0);

    assert_eq!(global.get_messages_sent(), 4);
    assert_eq!(global.get_messages_received(), 4);
    assert_eq!(global.get_bytes_sent(), 600);
    assert_eq!(global.get_bytes_received(), 500);

    metrics.record_close("normal");
}
