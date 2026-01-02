//! Data models and domain types for the kairos-rs API gateway.
//! 
//! This module contains all the core data structures used throughout the gateway,
//! including configuration models, error types, and domain entities. These models
//! provide type safety, serialization support, and validation logic for the
//! gateway's operation.
//! 
//! # Module Organization
//! 
//! - [`error`] - Gateway-specific error types with HTTP response mapping
//! - [`router`] - Route configuration and validation logic  
//! - [`settings`] - Application configuration and settings management
//! 
//! # Design Principles
//! 
//! All models in this module follow these principles:
//! - **Type Safety**: Leverage Rust's type system to prevent runtime errors
//! - **Validation**: Include comprehensive validation logic for data integrity
//! - **Serialization**: Support JSON serialization/deserialization via serde
//! - **Documentation**: Extensive documentation with examples for all public types
//! 
//! # Examples
//! 
//! ```rust
//! use kairos_rs::models::{router::{Router, Backend, Protocol}, settings::Settings, error::GatewayError};
//! 
//! // Create a route configuration
//! let route = Router {
//!     host: Some("http://backend".to_string()),
//!     port: Some(8080),
//!     external_path: "/api/users/{id}".to_string(),
//!     internal_path: "/v1/user/{id}".to_string(),
//!     methods: vec!["GET".to_string(), "PUT".to_string()],
//!     auth_required: false,
//!     backends: Some(vec![Backend {
//!         host: "http://backend".to_string(),
//!         port: 8080,
//!         weight: 1,
//!         health_check_path: None,
//!     }]),
//!     load_balancing_strategy: Default::default(),
//!     retry: None,
//!     protocol: Protocol::Http,
//!     request_transformation: None,
//!     response_transformation: None,
//!     ai_policy: None,
//! };
//! 
//! // Validate the configuration
//! route.validate().expect("Invalid route configuration");
//! 
//! // Handle gateway errors
//! let error = GatewayError::RouteNotFound { 
//!     path: "/unknown/path".to_string() 
//! };
//! ```

pub mod error;
pub mod router;
pub mod settings;
