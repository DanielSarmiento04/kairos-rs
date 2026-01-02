//! HTTP route handlers and endpoint definitions for the kairos-rs gateway.
//! 
//! This module contains all HTTP route handlers, endpoint configurations, and
//! request processing logic. It provides the web interface layer that clients
//! interact with, including health checks, proxy routing, and WebSocket support.
//! 
//! # Module Organization
//! 
//! - [`health`] - Health check endpoints for monitoring and Kubernetes probes
//! - [`http`] - HTTP proxy route configuration and request handling
//! - [`websocket`] - WebSocket connection handling and upgrades (future feature)
//! 
//! # Route Architecture
//! 
//! The routing system is designed with these layers:
//! 
//! ```text
//! Client Request → Route Handler → Service Layer → Upstream Service
//!       ↓              ↓              ↓               ↓
//!    HTTP/WS     Path Matching    Business Logic   HTTP Client
//!    Headers     Validation       Error Handling   Connection Pool
//!    Body        Middleware       Logging          Response Processing
//! ```
//! 
//! # Key Features
//! 
//! ## HTTP Proxy Routes
//! - Dynamic route matching with parameter extraction
//! - Configurable payload size limits (1MB default)
//! - JSON payload validation and size constraints
//! - Catch-all route pattern `/{tail:.*}` for flexible routing
//! 
//! ## Health Endpoints
//! - General health checks with service information
//! - Kubernetes readiness and liveness probes
//! - Structured JSON responses with timestamps
//! - High-performance, low-latency responses
//! 
//! ## Security & Validation
//! - Request size validation to prevent DoS attacks
//! - Header validation for security compliance
//! - Suspicious pattern detection in User-Agent strings
//! - Content-Type validation for POST/PUT requests
//! 
//! # Route Configuration
//! 
//! Routes are configured through the main application using Actix Web's
//! service configuration system:
//! 
//! ```rust
//! # use actix_web::{App, web, HttpResponse, Result};
//! # use std::sync::Arc;
//! # 
//! # struct RouteHandler;
//! # impl RouteHandler {
//! #     fn new(_routes: Vec<Router>, _timeout: u64) -> Arc<Self> {
//! #         Arc::new(RouteHandler)
//! #     }
//! # }
//! # 
//! # struct Router;
//! # 
//! # fn configure_health(_cfg: &mut web::ServiceConfig) {}
//! # fn configure_route(_cfg: &mut web::ServiceConfig, _handler: Arc<RouteHandler>) {}
//! 
//! let routes: Vec<Router> = vec![];
//! let handler = RouteHandler::new(routes, 30);
//! 
//! let app = App::new()
//!     .configure(configure_health);
//! ```
//! 
//! # Performance Considerations
//! 
//! - Health endpoints are optimized for sub-millisecond response times
//! - Proxy routes use efficient async/await patterns
//! - Connection pooling is handled at the service layer
//! - Minimal memory allocation in request processing paths
//! 
//! # Future Enhancements
//! 
//! - WebSocket proxy support for real-time applications
//! - GraphQL endpoint proxying
//! - Server-Sent Events (SSE) support
//! - Advanced routing features (rate limiting per route, caching)

pub mod auth_http;
pub mod config_reload;
pub mod health;
pub mod http;
pub mod management;
pub mod metrics;
pub mod websocket;
pub mod websocket_admin;
pub mod ftp;
pub mod dns;
