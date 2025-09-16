//! Security and validation middleware for the kairos-rs gateway.
//! 
//! This module provides comprehensive middleware functions that enhance the
//! security, reliability, and performance of the API gateway. The middleware
//! operates at the request/response level and provides cross-cutting concerns
//! that apply to all routes.
//! 
//! # Module Organization
//! 
//! - [`security`] - Security headers and HTTPS enforcement middleware
//! - [`validation`] - Request validation and security checks middleware
//! 
//! # Middleware Architecture
//! 
//! The middleware system follows Actix Web's middleware pattern and integrates
//! into the request processing pipeline:
//! 
//! ```text
//! Client Request → Middleware Stack → Route Handlers → Response
//!       ↓              ↓                    ↓           ↓
//!   HTTP Headers   Security Headers     Business      Security Headers
//!   Request Body   Size Validation      Logic         Response Processing
//!   Connection     Pattern Detection    Routing       Content Filtering
//! ```
//! 
//! # Security Features
//! 
//! ## Header Security
//! - **Content Security Policy**: Prevents XSS and injection attacks
//! - **HSTS**: Enforces HTTPS connections for enhanced security
//! - **X-Frame-Options**: Prevents clickjacking attacks
//! - **X-Content-Type-Options**: Prevents MIME-type sniffing
//! - **Referrer Policy**: Controls referrer information disclosure
//! 
//! ## Request Validation
//! - **Size Limits**: Prevents memory exhaustion attacks
//! - **Pattern Detection**: Identifies malicious request patterns
//! - **Content-Type Validation**: Ensures appropriate content types
//! - **User-Agent Filtering**: Blocks known malicious tools
//! 
//! # Performance Optimizations
//! 
//! - **Efficient Pattern Matching**: Fast string operations for security checks
//! - **Early Rejection**: Invalid requests rejected before processing
//! - **Minimal Overhead**: Low-latency middleware with optimized algorithms
//! - **Memory Efficient**: Bounded memory usage with size limits
//! 
//! # Usage Examples
//! 
//! ```rust
//! use actix_web::{App, middleware::Logger};
//! use kairos_rs::middleware::{security, validation};
//! 
//! let app = App::new()
//!     .wrap(security::security_headers())
//!     .wrap(Logger::default())
//!     .service(
//!         web::resource("/api/{path:.*}")
//!             .wrap_fn(validation::validate_request_size(1024 * 1024)) // 1MB limit
//!             .wrap_fn(validation::validate_headers())
//!             .route(web::get().to(handler))
//!     );
//! ```
//! 
//! # Configuration
//! 
//! Middleware behavior can be customized through:
//! - Environment variables for security header values
//! - Configuration files for validation thresholds
//! - Runtime parameters for dynamic security policies
//! 
//! # Integration Points
//! 
//! The middleware integrates with:
//! - **Logging System**: Security events and validation failures
//! - **Metrics Collection**: Performance and security metrics
//! - **Error Handling**: Structured error responses for security violations
//! - **Configuration Management**: Dynamic security policy updates

pub mod security;
pub mod validation;