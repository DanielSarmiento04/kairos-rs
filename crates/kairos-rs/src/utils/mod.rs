//! Utility functions and helper modules for the kairos-rs gateway.
//! 
//! This module provides essential utilities that support the core gateway functionality,
//! including URL manipulation, route matching algorithms, and other helper functions.
//! These utilities are designed for high performance and are used extensively
//! throughout the gateway's request processing pipeline.
//! 
//! # Module Organization
//! 
//! - [`path`] - URL and path manipulation utilities for request forwarding
//! - [`route_matcher`] - High-performance route matching with regex compilation
//! 
//! # Performance Focus
//! 
//! All utilities in this module are optimized for:
//! - **Zero-Copy Operations**: Minimize memory allocations where possible
//! - **Efficient Algorithms**: Use optimal data structures and algorithms
//! - **Thread Safety**: Support concurrent access without synchronization overhead
//! - **Memory Efficiency**: Pre-allocate buffers and reuse resources
//! 
//! # Key Features
//! 
//! ## Route Matching
//! - O(1) static route lookups using hash maps
//! - Compiled regex patterns for dynamic routes  
//! - Parameter extraction and path transformation
//! - Sorted matching for consistent behavior
//! 
//! ## Path Processing
//! - URL construction and manipulation
//! - Safe path handling with validation
//! - Efficient string operations
//! 
//! # Examples
//! 
//! ```rust
//! use kairos_rs::utils::{path::format_route, route_matcher::RouteMatcher};
//! use kairos_rs::models::router::{Router, Backend, Protocol};
//! 
//! // URL formatting
//! let url = format_route("http://backend", &8080, "/api/users/123");
//! assert_eq!(url, "http://backend:8080/api/users/123");
//! 
//! // Route matching with proper route configuration
//! let routes = vec![
//!     Router {
//!         host: Some("http://localhost".to_string()),
//!         port: Some(8080),
//!         external_path: "/users/{id}".to_string(),
//!         internal_path: "/v1/user/{id}".to_string(),
//!         methods: vec!["GET".to_string()],
//!         auth_required: false,
//!         backends: Some(vec![Backend {
//!             host: "http://localhost".to_string(),
//!             port: 8080,
//!             weight: 1,
//!             health_check_path: None,
//!         }]),
//!         load_balancing_strategy: Default::default(),
//!         retry: None,
//!         protocol: Protocol::Http,
//!         request_transformation: None,
//!         response_transformation: None,
//!     }
//! ];
//! let matcher = RouteMatcher::new(routes)?;
//! let (route, internal_path) = matcher.find_match("/users/123")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod path;
pub mod route_matcher;
