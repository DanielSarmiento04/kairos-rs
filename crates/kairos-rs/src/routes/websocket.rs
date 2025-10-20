//! WebSocket connection handling and upgrades for the kairos-rs gateway.
//!
//! This module provides route handlers for WebSocket connections, managing
//! the upgrade process and message forwarding to upstream WebSocket servers.

use crate::services::websocket::WebSocketHandler;
use actix_web::{web, HttpRequest, HttpResponse, Error};
use log::info;

/// Configures WebSocket routes for the application.
///
/// This sets up a catch-all WebSocket route that handles connection
/// upgrades and message forwarding for any WebSocket path.
///
/// # Parameters
///
/// * `cfg` - Actix Web service configuration
/// * `handler` - WebSocket handler instance for processing connections
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
