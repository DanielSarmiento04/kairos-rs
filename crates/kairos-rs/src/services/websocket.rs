//! WebSocket Proxy Service
//!
//! Provides WebSocket protocol support for the Kairos gateway, enabling
//! bidirectional, real-time communication between clients and backend services.

use crate::models::error::GatewayError;
use crate::models::router::Backend;
use crate::services::websocket_metrics::WebSocketMetrics;
use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse, rt as actix_rt};
use actix_ws::Message;
use futures_util::StreamExt;
use log::{debug, error, info};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};
use futures_util::SinkExt;

/// WebSocket proxy handler for upgrading HTTP connections and forwarding messages.
pub struct WebSocketHandler {
    /// Default timeout for WebSocket operations in seconds
    pub(crate) timeout_seconds: u64,
}

impl WebSocketHandler {
    /// Creates a new WebSocket handler with the specified timeout.
    pub fn new(timeout_seconds: u64) -> Self {
        Self { timeout_seconds }
    }

    /// Handles an incoming WebSocket connection upgrade and proxies to backend.
    pub async fn handle_websocket(
        &self,
        req: HttpRequest,
        stream: web::Payload,
        backend: &Backend,
        internal_path: &str,
    ) -> Result<HttpResponse, ActixError> {
        // Build the backend WebSocket URL using the internal_path
        let backend_url = match self.build_backend_url(&backend.host, backend.port, internal_path) {
            Ok(url) => url,
            Err(e) => {
                error!("Failed to build backend URL: {}", e);
                return Ok(HttpResponse::BadGateway().body(format!("Invalid backend URL: {}", e)));
            }
        };

        info!("Upgrading WebSocket connection to backend: {}", backend_url);

        // Initialize metrics for this connection
        let backend_id = format!("{}:{}", backend.host, backend.port);
        let metrics = WebSocketMetrics::new(
            req.path().to_string(),
            backend_id.clone(),
        );

        // Upgrade the client connection to WebSocket
        let (response, mut client_session, mut client_msg_stream) = match actix_ws::handle(&req, stream) {
            Ok(upgrade) => upgrade,
            Err(e) => {
                error!("Failed to upgrade WebSocket connection: {}", e);
                metrics.record_error("upgrade_failed");
                metrics.record_close("upgrade_failed");
                return Err(e);
            }
        };

        // Connect to the backend WebSocket server
        let (backend_ws, _) = match connect_async(&backend_url).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to backend WebSocket: {}", e);
                metrics.record_error("backend_unreachable");
                let _ = client_session.close(Some(actix_ws::CloseReason {
                    code: actix_ws::CloseCode::Error,
                    description: Some(format!("Backend connection failed: {}", e)),
                })).await;
                metrics.record_close("backend_unreachable");
                return Ok(HttpResponse::BadGateway().body(format!("Backend connection failed: {}", e)));
            }
        };

        let (mut backend_write, mut backend_read) = backend_ws.split();

        // Clone metrics for the forwarding tasks
        let metrics_client_to_backend = WebSocketMetrics::new(
            req.path().to_string(),
            backend_id.clone(),
        );
        let metrics_backend_to_client = WebSocketMetrics::new(
            req.path().to_string(),
            backend_id,
        );

        // Spawn task to forward messages from client to backend
        let client_session_clone = client_session.clone();
        actix_rt::spawn(async move {
            while let Some(Ok(msg)) = client_msg_stream.next().await {
                let backend_msg = match &msg {
                    Message::Text(text) => {
                        debug!("Client -> Backend (text): {} bytes", text.len());
                        metrics_client_to_backend.record_message_received("text", text.len());
                        TungsteniteMessage::Text(text.to_string())
                    }
                    Message::Binary(bin) => {
                        debug!("Client -> Backend (binary): {} bytes", bin.len());
                        metrics_client_to_backend.record_message_received("binary", bin.len());
                        TungsteniteMessage::Binary(bin.to_vec())
                    }
                    Message::Ping(bytes) => {
                        debug!("Client -> Backend (ping)");
                        metrics_client_to_backend.record_message_received("ping", bytes.len());
                        TungsteniteMessage::Ping(bytes.to_vec())
                    }
                    Message::Pong(bytes) => {
                        debug!("Client -> Backend (pong)");
                        metrics_client_to_backend.record_message_received("pong", bytes.len());
                        TungsteniteMessage::Pong(bytes.to_vec())
                    }
                    Message::Close(reason) => {
                        info!("Client closed WebSocket: {:?}", reason);
                        let close_reason = reason.as_ref().map(|r| r.description.as_deref().unwrap_or("normal")).unwrap_or("normal");
                        metrics_client_to_backend.record_close(close_reason);
                        let _ = backend_write.close().await;
                        return;
                    }
                    _ => continue,
                };

                if let Err(e) = backend_write.send(backend_msg).await {
                    error!("Failed to forward message to backend: {}", e);
                    metrics_client_to_backend.record_error("forwarding_error");
                    metrics_client_to_backend.record_close("forwarding_error");
                    let _ = client_session_clone.close(None).await;
                    break;
                }
            }
            debug!("Client -> Backend forwarding task finished");
        });

        // Forward messages from backend to client
        actix_rt::spawn(async move {
            while let Some(msg_result) = backend_read.next().await {
                match msg_result {
                    Ok(backend_msg) => {
                        match backend_msg {
                            TungsteniteMessage::Text(text) => {
                                debug!("Backend -> Client (text): {} bytes", text.len());
                                metrics_backend_to_client.record_message_sent("text", text.len());
                                if let Err(e) = client_session.text(text).await {
                                    error!("Failed to forward text to client: {}", e);
                                    metrics_backend_to_client.record_error("forwarding_error");
                                    metrics_backend_to_client.record_close("forwarding_error");
                                    break;
                                }
                            }
                            TungsteniteMessage::Binary(bin) => {
                                debug!("Backend -> Client (binary): {} bytes", bin.len());
                                metrics_backend_to_client.record_message_sent("binary", bin.len());
                                if let Err(e) = client_session.binary(bin).await {
                                    error!("Failed to forward binary to client: {}", e);
                                    metrics_backend_to_client.record_error("forwarding_error");
                                    metrics_backend_to_client.record_close("forwarding_error");
                                    break;
                                }
                            }
                            TungsteniteMessage::Ping(bytes) => {
                                debug!("Backend -> Client (ping)");
                                metrics_backend_to_client.record_message_sent("ping", bytes.len());
                                if let Err(e) = client_session.ping(&bytes).await {
                                    error!("Failed to forward ping to client: {}", e);
                                    metrics_backend_to_client.record_error("forwarding_error");
                                    metrics_backend_to_client.record_close("forwarding_error");
                                    break;
                                }
                            }
                            TungsteniteMessage::Pong(bytes) => {
                                debug!("Backend -> Client (pong)");
                                metrics_backend_to_client.record_message_sent("pong", bytes.len());
                                if let Err(e) = client_session.pong(&bytes).await {
                                    error!("Failed to forward pong to client: {}", e);
                                    metrics_backend_to_client.record_error("forwarding_error");
                                    metrics_backend_to_client.record_close("forwarding_error");
                                    break;
                                }
                            }
                            TungsteniteMessage::Close(reason) => {
                                info!("Backend closed WebSocket: {:?}", reason);
                                let close_desc = reason.as_ref().map(|r| r.reason.to_string()).unwrap_or_else(|| "normal".to_string());
                                metrics_backend_to_client.record_close(&close_desc);
                                
                                let close_reason = reason.map(|r| {
                                    // Convert tungstenite CloseCode to actix_ws CloseCode
                                    let code = match r.code {
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal => actix_ws::CloseCode::Normal,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Away => actix_ws::CloseCode::Away,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Protocol => actix_ws::CloseCode::Protocol,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Unsupported => actix_ws::CloseCode::Unsupported,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Abnormal => actix_ws::CloseCode::Abnormal,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Invalid => actix_ws::CloseCode::Invalid,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Policy => actix_ws::CloseCode::Policy,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Size => actix_ws::CloseCode::Size,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Extension => actix_ws::CloseCode::Extension,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Error => actix_ws::CloseCode::Error,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Restart => actix_ws::CloseCode::Restart,
                                        tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Again => actix_ws::CloseCode::Again,
                                        _ => actix_ws::CloseCode::Error,
                                    };
                                    actix_ws::CloseReason {
                                        code,
                                        description: Some(r.reason.to_string()),
                                    }
                                });
                                let _ = client_session.close(close_reason).await;
                                break;
                            }
                            _ => continue,
                        }
                    }
                    Err(e) => {
                        error!("Error receiving from backend: {}", e);
                        metrics_backend_to_client.record_error("backend_error");
                        metrics_backend_to_client.record_close("backend_error");
                        let _ = client_session.close(Some(actix_ws::CloseReason {
                            code: actix_ws::CloseCode::Error,
                            description: Some(format!("Backend error: {}", e)),
                        })).await;
                        break;
                    }
                }
            }
            debug!("Backend -> Client forwarding task finished");
        });

        Ok(response)
    }

    /// Builds the backend WebSocket URL from the backend configuration
    fn build_backend_url(&self, host: &str, port: u16, path: &str) -> Result<String, GatewayError> {
        // Convert http:// or ws:// schemes appropriately
        let ws_scheme = if host.starts_with("https://") || host.starts_with("wss://") {
            "wss://"
        } else {
            "ws://"
        };

        let host_without_scheme = host
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("wss://")
            .trim_start_matches("ws://");

        Ok(format!("{}{}:{}{}", ws_scheme, host_without_scheme, port, path))
    }

}

impl Clone for WebSocketHandler {
    fn clone(&self) -> Self {
        Self {
            timeout_seconds: self.timeout_seconds,
        }
    }
}
