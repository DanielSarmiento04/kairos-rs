//! Models module - UI-specific models that work in both SSR and WASM contexts.

// Core models (WASM-compatible)
pub mod router;
pub mod settings;
pub mod transform;

// UI-specific models
pub mod metrics;
pub mod health;

// Re-exports
pub use router::Router;
pub use settings::{
    Settings, JwtSettings, RateLimitConfig, LimitStrategy, WindowType,
    CorsConfig, MetricsConfig, ServerConfig
};
pub use metrics::{
    MetricsData, CircuitBreakerMetrics, CircuitBreakerState,
    MetricPoint, MetricValue, AggregatedMetric, AggregationInterval
};
pub use health::{HealthResponse, ReadinessResponse, LivenessResponse};
