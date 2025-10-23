//! Authentication-aware HTTP routing for the Kairos-rs gateway.
//! 
//! This module provides enhanced routing capabilities that integrate JWT authentication
//! with the existing HTTP routing system. It automatically applies JWT middleware
//! to routes that require authentication while leaving public routes unprotected.

use crate::middleware::auth::{JwtAuth, JwtConfig};
use crate::models::settings::Settings;
use crate::models::router::Protocol;
use crate::services::http::RouteHandler;
use crate::services::websocket::WebSocketHandler;
use actix_web::{web, HttpRequest, HttpResponse, Error as ActixError};
use std::sync::Arc;
use log::{debug, warn};

/// Configures routes with optional JWT authentication based on route settings.
/// 
/// This function sets up both authenticated and public routes based on the configuration.
/// Routes requiring authentication will have JWT middleware applied automatically.
pub fn configure_auth_routes(cfg: &mut web::ServiceConfig, handler: RouteHandler, settings: &Settings) {
    let handler = Arc::new(handler);
    let ws_handler = WebSocketHandler::new(30);
    
    // Configure public routes first (no authentication required)
    for router in &settings.routers {
        if !router.auth_required {
            let path = router.external_path.clone();
            let methods = router.methods.clone();
            let protocol = router.protocol.clone();
            
            // Handle WebSocket routes separately
            if matches!(protocol, Protocol::WebSocket) {
                debug!("Configuring WebSocket route: {}", path);
                let ws_handler_clone = ws_handler.clone();
                let router_clone = router.clone();
                
                cfg.route(&path, web::get().to(move |req: HttpRequest, stream: web::Payload| {
                    let ws_handler = ws_handler_clone.clone();
                    let router = router_clone.clone();
                    async move {
                        handle_websocket_route(req, stream, ws_handler, router).await
                    }
                }));
                continue;
            }
            
            for method in methods {
                let handler_clone = handler.clone();
                let path_clone = path.clone();
                
                match method.to_uppercase().as_str() {
                    "GET" => {
                        cfg.route(&path_clone, web::get().to(move |req: HttpRequest, body: web::Bytes| {
                            let handler = handler_clone.clone();
                            async move {
                                handler.handle_request(req, body).await
                            }
                        }));
                    }
                    "POST" => {
                        cfg.route(&path_clone, web::post().to(move |req: HttpRequest, body: web::Bytes| {
                            let handler = handler_clone.clone();
                            async move {
                                handler.handle_request(req, body).await
                            }
                        }));
                    }
                    "PUT" => {
                        cfg.route(&path_clone, web::put().to(move |req: HttpRequest, body: web::Bytes| {
                            let handler = handler_clone.clone();
                            async move {
                                handler.handle_request(req, body).await
                            }
                        }));
                    }
                    "DELETE" => {
                        cfg.route(&path_clone, web::delete().to(move |req: HttpRequest, body: web::Bytes| {
                            let handler = handler_clone.clone();
                            async move {
                                handler.handle_request(req, body).await
                            }
                        }));
                    }
                    _ => {} // Skip unsupported methods
                }
            }
        }
    }
    
    // Configure authenticated routes with JWT middleware
    if let Some(jwt_settings) = &settings.jwt {
        let jwt_config = JwtConfig::new(jwt_settings.secret.clone())
            .with_issuer(jwt_settings.issuer.clone().unwrap_or_default())
            .with_audience(jwt_settings.audience.clone().unwrap_or_default());
        
        for router in &settings.routers {
            if router.auth_required {
                let path = router.external_path.clone();
                let methods = router.methods.clone();
                let handler_clone = handler.clone();
                
                for method in methods {
                    let handler_for_method = handler_clone.clone();
                    let path_for_method = path.clone();
                    let jwt_middleware = JwtAuth::new(jwt_config.clone());
                    
                    match method.to_uppercase().as_str() {
                        "GET" => {
                            cfg.service(
                                web::resource(&path_for_method)
                                    .wrap(jwt_middleware)
                                    .route(web::get().to(move |req: HttpRequest, body: web::Bytes| {
                                        let handler = handler_for_method.clone();
                                        async move {
                                            handler.handle_request(req, body).await
                                        }
                                    }))
                            );
                        }
                        "POST" => {
                            cfg.service(
                                web::resource(&path_for_method)
                                    .wrap(jwt_middleware)
                                    .route(web::post().to(move |req: HttpRequest, body: web::Bytes| {
                                        let handler = handler_for_method.clone();
                                        async move {
                                            handler.handle_request(req, body).await
                                        }
                                    }))
                            );
                        }
                        "PUT" => {
                            cfg.service(
                                web::resource(&path_for_method)
                                    .wrap(jwt_middleware)
                                    .route(web::put().to(move |req: HttpRequest, body: web::Bytes| {
                                        let handler = handler_for_method.clone();
                                        async move {
                                            handler.handle_request(req, body).await
                                        }
                                    }))
                            );
                        }
                        "DELETE" => {
                            cfg.service(
                                web::resource(&path_for_method)
                                    .wrap(jwt_middleware)
                                    .route(web::delete().to(move |req: HttpRequest, body: web::Bytes| {
                                        let handler = handler_for_method.clone();
                                        async move {
                                            handler.handle_request(req, body).await
                                        }
                                    }))
                            );
                        }
                        _ => {} // Skip unsupported methods
                    }
                }
            }
        }
    }
    
    // Fallback catch-all route for any unmatched requests
    let fallback_handler = handler.clone();
    cfg.default_service(web::route().to(move |req: HttpRequest, body: web::Bytes| {
        let handler = fallback_handler.clone();
        async move {
            handler.handle_request(req, body).await
        }
    }));
}

/// Handles WebSocket route upgrade and proxying
async fn handle_websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    ws_handler: WebSocketHandler,
    router: crate::models::router::Router,
) -> Result<HttpResponse, ActixError> {
    // Get the first backend for this router
    let backends = router.get_backends();
    if backends.is_empty() {
        warn!("No backends configured for WebSocket route: {}", router.external_path);
        return Ok(HttpResponse::BadGateway().body("No backends configured"));
    }
    
    let backend = &backends[0]; // For now, use first backend (TODO: load balancing)
    
    debug!("Handling WebSocket connection to backend: {}:{}", backend.host, backend.port);
    
    // Use the internal_path from the router configuration
    let internal_path = &router.internal_path;
    
    ws_handler.handle_websocket(req, stream, backend, internal_path).await
}