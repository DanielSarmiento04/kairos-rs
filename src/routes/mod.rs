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
//! use actix_web::{App, web};
//! use kairos_rs::routes::{health, http};
//! use kairos_rs::services::http::RouteHandler;
//! 
//! let handler = RouteHandler::new(routes, 30);
//! 
//! let app = App::new()
//!     .configure(health::configure_health)
//!     .configure(|cfg| http::configure_route(cfg, handler));
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

pub mod health;
pub mod http;
pub mod websocket;
