//! Service layer implementations for the kairos-rs gateway.
//! 
//! This module contains the service layer that handles the core business logic
//! of the API gateway, including HTTP request forwarding, response processing,
//! and communication with upstream services. The services are designed for
//! high throughput and low latency operation.
//! 
//! # Module Organization
//! 
//! - [`http`] - HTTP request handling and upstream service communication
//! 
//! # Architecture
//! 
//! The service layer sits between the routing layer (which handles incoming requests)
//! and the utility layer (which provides supporting functions). It implements the
//! core gateway logic:
//! 
//! ```text
//! Client → Routes → Services → Utils → Upstream Services
//!   ↑                ↓
//!   └─── Response ←──┘
//! ```
//! 
//! # Key Responsibilities
//! 
//! ## Request Processing
//! - Route matching and validation
//! - HTTP method verification
//! - Header processing and transformation
//! - Request body forwarding
//! 
//! ## Upstream Communication
//! - Connection pooling and management
//! - Timeout handling and circuit breaking
//! - Response streaming and buffering
//! - Error handling and retry logic
//! 
//! ## Performance Features
//! - Async/await based processing for high concurrency
//! - Connection reuse and HTTP/2 support
//! - Optimized header processing
//! - Memory-efficient request/response handling
//! 
//! # Examples
//! 
//! ```rust
//! use kairos_rs::services::http::RouteHandler;
//! use kairos_rs::models::router::Router;
//! use actix_web::{web, HttpRequest};
//! 
//! // Create a route handler
//! let routes = vec![
//!     Router {
//!         host: "http://backend".to_string(),
//!         port: 8080,
//!         external_path: "/api/users/{id}".to_string(),
//!         internal_path: "/v1/user/{id}".to_string(),
//!         methods: vec!["GET".to_string()],
//!     }
//! ];
//! 
//! let handler = RouteHandler::new(routes, 30);
//! 
//! // Handler can be used in Actix Web routes
//! // let response = handler.handle_request(req, body).await?;
//! ```

pub mod circuit_breaker;
pub mod http;
