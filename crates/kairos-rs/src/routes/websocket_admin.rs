use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use tokio::time::{interval, Duration};
use crate::routes::metrics::MetricsCollector;
use futures_util::StreamExt;
use std::sync::atomic::Ordering;

/// WebSocket handler for real-time admin metrics.
/// 
/// This endpoint establishes a WebSocket connection that pushes real-time
/// metrics updates to the connected client (typically the Admin UI).
/// 
/// # Protocol
/// 
/// - **Server -> Client**: JSON object containing current metrics snapshot (every 1s)
/// - **Client -> Server**: Ping/Pong for keepalive
pub async fn admin_metrics_ws(
    req: HttpRequest,
    stream: web::Payload,
    metrics: web::Data<MetricsCollector>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn a task to push metrics every second
    let metrics = metrics.into_inner();
    let mut session_clone = session.clone();
    
    actix_web::rt::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        
        loop {
            ticker.tick().await;
            
            // Collect snapshot
            let total_requests = metrics.requests_total.load(Ordering::Relaxed);
            let success_requests = metrics.requests_success.load(Ordering::Relaxed);
            
            let success_rate = if total_requests > 0 {
                (success_requests as f64 / total_requests as f64) * 100.0
            } else {
                100.0
            };

            let snapshot = serde_json::json!({
                "requests_total": total_requests,
                "active_connections": metrics.active_connections.load(Ordering::Relaxed),
                "requests_error": metrics.requests_error.load(Ordering::Relaxed),
                "success_rate": success_rate,
                "uptime": metrics.start_time.elapsed().as_secs(),
                "peak_connections": metrics.peak_connections.load(Ordering::Relaxed),
            });
            
            if session_clone.text(snapshot.to_string()).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (e.g., ping/pong or configuration changes)
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() { break; }
                }
                Message::Close(reason) => {
                    let _ = session.close(reason).await;
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(response)
}

/// Configures the admin WebSocket route.
pub fn configure_admin_websocket(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws/admin/metrics", web::get().to(admin_metrics_ws));
}
