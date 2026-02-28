pub mod config;
pub mod health;
pub mod metrics;
pub mod routes;

/// Base URL for the Kairos Gateway API
pub(crate) const GATEWAY_BASE_URL: &str = "http://localhost:5900";

// Re-export server functions for easier imports
pub use config::*;
pub use health::*;
pub use metrics::*;
pub use routes::*;
