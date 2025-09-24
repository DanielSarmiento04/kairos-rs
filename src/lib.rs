//! # Kairos-rs API Gateway
//! 
//! A high-performance, async HTTP API gateway built with Rust and Actix Web.
//! Kairos-rs provides intelligent request routing, rate limiting, security features,
//! and efficient upstream service communication for modern microservice architectures.
//! 
//! ## Quick Start
//! 
//! ```rust
//! # use std::fs;
//! # // Create a temporary config file for testing
//! # let config_content = r#"{"version": 1, "routers": []}"#;
//! # fs::write("./config.json", config_content).unwrap();
//! use kairos_rs::{
//!     config::settings::load_settings,
//!     services::http::RouteHandler,
//!     models::router::Router,
//! };
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = load_settings()?;
//!     config.validate()?;
//!     
//!     // Create route handler
//!     let handler = RouteHandler::new(config.routers, 30);
//!     
//!     // Handler is now ready to process requests
//!     Ok(())
//! }
//! # // Clean up
//! # fs::remove_file("./config.json").ok();
//! ```
//! 
//! ## Core Features
//! 
//! ### High-Performance Routing
//! - **Static Routes**: O(1) hash map lookup for exact path matches
//! - **Dynamic Routes**: Compiled regex patterns with parameter extraction
//! - **Method Validation**: Configurable HTTP method restrictions per route
//! - **Path Transformation**: Flexible mapping from external to internal paths
//! 
//! ### Security & Reliability
//! - **Rate Limiting**: Built-in rate limiting with configurable thresholds
//! - **Security Headers**: Comprehensive security header middleware
//! - **Request Timeout**: Configurable upstream request timeouts
//! - **Input Validation**: Comprehensive configuration and request validation
//! 
//! ### Observability
//! - **Structured Logging**: Detailed request/response logging with timestamps
//! - **Health Endpoints**: Kubernetes-compatible health, readiness, and liveness checks
//! - **Error Tracking**: Structured error responses with unique request IDs
//! - **Performance Metrics**: Built-in request timing and throughput monitoring
//! 
//! ## Architecture Overview
//! 
//! ```text
//! ┌─────────────┐   ┌─────────────────┐   ┌──────────────────┐
//! │   Client    │──▶│   Kairos-rs     │──▶│   Upstream       │
//! │  (Browser,  │   │   Gateway       │   │   Services       │
//! │   Mobile,   │   │                 │   │ (Microservices,  │
//! │   API)      │   │  ┌──────────────┤   │  APIs, etc.)     │
//! └─────────────┘   │  │ Rate Limiter ││   └──────────────────┘
//!                   │  ├──────────────┤│
//!                   │  │ Route Matcher││
//!                   │  ├──────────────┤│
//!                   │  │ Load Balancer││
//!                   │  ├──────────────┤│
//!                   │  │ Circuit Break││
//!                   │  └──────────────┘│
//!                   └─────────────────┘
//! ```
//! 
//! ## Module Organization
//! 
//! - **[`config`]** - Configuration management and file loading
//! - **[`models`]** - Data models, domain types, and validation logic
//! - **[`services`]** - Business logic and upstream service communication
//! - **[`routes`]** - HTTP route definitions and handlers
//! - **[`middleware`]** - Security, logging, and request processing middleware
//! - **[`utils`]** - Utility functions and helper modules
//! - **[`logs`]** - Logging configuration and structured output
//! 
//! ## Configuration Example
//! 
//! ```json
//! {
//!   "version": 1,
//!   "routers": [
//!     {
//!       "host": "http://auth-service",
//!       "port": 8080,
//!       "external_path": "/auth/login",
//!       "internal_path": "/api/v1/authenticate",
//!       "methods": ["POST"]
//!     },
//!     {
//!       "host": "http://user-service",
//!       "port": 8080,
//!       "external_path": "/users/{id}",
//!       "internal_path": "/api/v1/user/{id}",
//!       "methods": ["GET", "PUT", "DELETE"]
//!     },
//!     {
//!       "host": "http://content-service",
//!       "port": 8080,
//!       "external_path": "/users/{user_id}/posts/{post_id}",
//!       "internal_path": "/api/v1/post/{post_id}?user={user_id}",
//!       "methods": ["GET", "PUT", "DELETE"]
//!     }
//!   ]
//! }
//! ```
//! 
//! ## Environment Variables
//! 
//! - `KAIROS_CONFIG_PATH`: Configuration file path (default: `./config.json`)
//! - `KAIROS_HOST`: Server bind address (default: `0.0.0.0`)
//! - `KAIROS_PORT`: Server port (default: `5900`)
//! - `NO_COLOR`: Disable colored log output
//! 
//! ## Performance Characteristics
//! 
//! - **Throughput**: 50,000+ requests/second on modern hardware
//! - **Latency**: Sub-millisecond routing overhead
//! - **Memory**: Efficient memory usage with connection pooling
//! - **Concurrency**: Full async/await support with Tokio runtime
//! 
//! ## Use Cases
//! 
//! - **API Gateway**: Central entry point for microservice architectures
//! - **Reverse Proxy**: Load balancing and request forwarding
//! - **Path Transformation**: Mapping legacy URLs to modern APIs  
//! - **Rate Limiting**: Protecting backend services from overload
//! - **Security Layer**: Adding security headers and request validation

pub mod config;
pub mod logs;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;
