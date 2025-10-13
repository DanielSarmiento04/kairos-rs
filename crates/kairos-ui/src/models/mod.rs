//! Models module - combines kairos-rs models with UI-specific extensions.

// Re-export core models from kairos-rs backend
pub use kairos_rs::models::router::Router;
pub use kairos_rs::models::settings::Settings;

// UI-specific models
pub mod metrics;
pub mod health;

pub use metrics::{MetricsData, CircuitBreakerMetrics, CircuitBreakerState};
pub use health::{HealthResponse, ReadinessResponse, LivenessResponse};
