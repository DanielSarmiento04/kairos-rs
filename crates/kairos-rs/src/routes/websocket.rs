//! WebSocket connection handling and upgrades for the kairos-rs gateway.
//!
//! This module provides route handlers for WebSocket connections, managing
//! the upgrade process, message forwarding to upstream WebSocket servers,
//! and real-time metrics streaming.

use crate::routes::metrics::MetricsCollector;
use crate::services::metrics_store::MetricsStore;
use crate::services::websocket::WebSocketHandler;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_ws::Message;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;

/// WebSocket metrics subscription configuration.
///
/// Defines which metrics to stream and at what interval.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsSubscription {
    /// List of metric names to subscribe to (empty = all metrics)
    pub metrics: Vec<String>,
    
    /// Update interval in seconds (default: 1 second)
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    
    /// Include historical data (default: false)
    #[serde(default)]
    pub include_history: bool,
}

fn default_interval() -> u64 {
    1
}

/// WebSocket metrics event sent to clients.
///
/// Contains current metrics snapshot or error information.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MetricsEvent {
    /// Current metrics snapshot
    Snapshot {
        timestamp: String,
        requests_total: u64,
        requests_success: u64,
        requests_error: u64,
        active_connections: u64,
        avg_response_time: f64,
        success_rate: f64,
    },
    
    /// Time-series metrics data
    TimeSeries {
        metric_name: String,
        data_points: Vec<TimeSeriesPoint>,
    },
    
    /// Error occurred
    Error {
        message: String,
    },
    
    /// Heartbeat to keep connection alive
    Heartbeat {
        timestamp: String,
    },
}

/// Single time-series data point.
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesPoint {
    pub timestamp: String,
    pub value: f64,
}

/// WebSocket metrics streaming endpoint.
///
/// Provides real-time metrics updates via WebSocket connection.
/// Clients can subscribe to specific metrics and control update frequency.
///
/// # Protocol
///
/// 1. Client connects to `/ws/metrics`
/// 2. Server sends initial snapshot
/// 3. Server sends periodic updates based on subscription
/// 4. Client can send subscription updates to change metrics/interval
///
/// # Message Format
///
/// Subscription (Client → Server):
/// ```json
/// {
///   "metrics": ["requests_total", "response_time_avg"],
///   "interval_seconds": 2,
///   "include_history": false
/// }
/// ```
///
/// Events (Server → Client):
/// ```json
/// {
///   "type": "Snapshot",
///   "timestamp": "2024-11-07T10:30:00Z",
///   "requests_total": 1547,
///   "requests_success": 1520,
///   "requests_error": 27,
///   "active_connections": 15,
///   "avg_response_time": 45.2,
///   "success_rate": 98.25
/// }
/// ```
pub async fn ws_metrics_handler(
    req: HttpRequest,
    stream: web::Payload,
    metrics: web::Data<MetricsCollector>,
    metrics_store: web::Data<MetricsStore>,
) -> Result<HttpResponse, Error> {
    debug!("WebSocket metrics connection requested");
    
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    
    info!("WebSocket metrics client connected");
    
    // Spawn task to handle the WebSocket connection
    actix_web::rt::spawn(async move {
        let mut subscription = MetricsSubscription {
            metrics: vec![],
            interval_seconds: 1,
            include_history: false,
        };
        
        let mut update_interval = interval(Duration::from_secs(subscription.interval_seconds));
        
        // Send initial snapshot
        if let Err(e) = send_metrics_snapshot(&mut session, &metrics).await {
            error!("Failed to send initial snapshot: {}", e);
            return;
        }
        
        loop {
            tokio::select! {
                // Handle incoming messages from client
                Some(msg) = msg_stream.recv() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            debug!("Received subscription update: {}", text);
                            
                            // Parse subscription update
                            match serde_json::from_str::<MetricsSubscription>(&text) {
                                Ok(new_sub) => {
                                    subscription = new_sub.clone();
                                    
                                    // Update interval
                                    update_interval = interval(Duration::from_secs(subscription.interval_seconds));
                                    
                                    // If history requested, send it
                                    if subscription.include_history {
                                        if let Err(e) = send_historical_metrics(
                                            &mut session,
                                            &metrics_store,
                                            &subscription.metrics
                                        ).await {
                                            error!("Failed to send historical metrics: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Invalid subscription format: {}", e);
                                    let _ = send_error(&mut session, &format!("Invalid subscription: {}", e)).await;
                                }
                            }
                        }
                        Ok(Message::Ping(bytes)) => {
                            debug!("Received ping, sending pong");
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Ok(Message::Close(reason)) => {
                            info!("WebSocket metrics client closed: {:?}", reason);
                            let _ = session.close(reason).await;
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                
                // Send periodic updates
                _ = update_interval.tick() => {
                    if let Err(e) = send_metrics_snapshot(&mut session, &metrics).await {
                        error!("Failed to send metrics update: {}", e);
                        break;
                    }
                }
            }
        }
        
        info!("WebSocket metrics connection closed");
    });
    
    Ok(response)
}

/// Sends current metrics snapshot to WebSocket client.
async fn send_metrics_snapshot(
    session: &mut actix_ws::Session,
    metrics: &MetricsCollector,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::atomic::Ordering;
    
    let total_requests = metrics.requests_total.load(Ordering::Relaxed);
    let success_requests = metrics.requests_success.load(Ordering::Relaxed);
    let error_requests = metrics.requests_error.load(Ordering::Relaxed);
    let response_time_sum = metrics.response_time_sum.load(Ordering::Relaxed);
    let active_connections = metrics.active_connections.load(Ordering::Relaxed);
    
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
    
    let event = MetricsEvent::Snapshot {
        timestamp: chrono::Utc::now().to_rfc3339(),
        requests_total: total_requests,
        requests_success: success_requests,
        requests_error: error_requests,
        active_connections,
        avg_response_time,
        success_rate,
    };
    
    let json = serde_json::to_string(&event)?;
    session.text(json).await?;
    
    Ok(())
}

/// Sends historical metrics data to WebSocket client.
async fn send_historical_metrics(
    session: &mut actix_ws::Session,
    metrics_store: &MetricsStore,
    metric_names: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::{Utc, Duration};
    
    let end = Utc::now();
    let start = end - Duration::hours(1); // Last hour
    
    // If no specific metrics requested, get all available metrics
    let metrics_to_fetch = if metric_names.is_empty() {
        metrics_store.list_metrics()
    } else {
        metric_names.to_vec()
    };
    
    for metric_name in metrics_to_fetch {
        let points = metrics_store.query(&metric_name, start, end);
        
        let data_points: Vec<TimeSeriesPoint> = points
            .iter()
            .map(|p| TimeSeriesPoint {
                timestamp: p.timestamp.to_rfc3339(),
                value: match &p.value {
                    crate::services::metrics_store::MetricValue::Counter(v) => *v as f64,
                    crate::services::metrics_store::MetricValue::Gauge(v) => *v,
                    crate::services::metrics_store::MetricValue::Histogram { count, .. } => *count as f64,
                },
            })
            .collect();
        
        let event = MetricsEvent::TimeSeries {
            metric_name: metric_name.clone(),
            data_points,
        };
        
        let json = serde_json::to_string(&event)?;
        session.text(json).await?;
    }
    
    Ok(())
}

/// Sends error message to WebSocket client.
async fn send_error(
    session: &mut actix_ws::Session,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = MetricsEvent::Error {
        message: message.to_string(),
    };
    
    let json = serde_json::to_string(&event)?;
    session.text(json).await?;
    
    Ok(())
}

/// Configures WebSocket routes for the application.
///
/// This sets up WebSocket routes for both metrics streaming and
/// general WebSocket proxying.
///
/// # Parameters
///
/// * `cfg` - Actix Web service configuration
/// * `handler` - WebSocket handler instance for processing connections
///
/// # Routes
///
/// - `/ws/metrics` - Real-time metrics streaming endpoint
/// - `/ws/{tail:.*}` - General WebSocket proxying (catch-all)
///
/// # Examples
///
/// ```rust
/// use actix_web::{App, web};
/// use kairos_rs::routes::websocket::configure_websocket;
/// use kairos_rs::services::websocket::WebSocketHandler;
///
/// let handler = WebSocketHandler::new(30);
/// let app = App::new().configure(|cfg| configure_websocket(cfg, handler));
/// ```
pub fn configure_websocket(cfg: &mut web::ServiceConfig, handler: WebSocketHandler) {
    info!("Configuring WebSocket routes");
    
    // Metrics streaming endpoint
    cfg.route("/ws/metrics", web::get().to(ws_metrics_handler));
    
    // General WebSocket proxying
    cfg.service(
        web::resource("/ws/{tail:.*}")
            .route(web::get().to(move |_req: HttpRequest, _stream: web::Payload| {
                let _handler = handler.clone();
                async move {
                    // Extract backend from route config
                    // This is a simplified version - in production you would:
                    // 1. Match the path to configured WebSocket routes
                    // 2. Select appropriate backend
                    // 3. Handle load balancing if multiple backends
                    
                    // For now, return a simple response indicating WebSocket support
                    Ok::<HttpResponse, Error>(
                        HttpResponse::NotImplemented()
                            .body("WebSocket proxying requires route configuration with protocol: 'websocket'")
                    )
                }
            }))
    );
}
