//! Models module - UI-specific models that work in both SSR and WASM contexts.

// Core models (WASM-compatible)
pub mod router;
pub mod settings;
pub mod transform;

// UI-specific models
pub mod health;
pub mod metrics;

// Re-exports
pub use health::{HealthResponse, LivenessResponse, ReadinessResponse};
pub use metrics::{
    AggregatedMetric, AggregationInterval, CircuitBreakerMetrics, CircuitBreakerState, MetricPoint,
    MetricValue, MetricsData,
};
pub use router::Router;
pub use settings::{
    AiSettings, CorsConfig, JwtSettings, LimitStrategy, MetricsConfig, RateLimitConfig,
    ServerConfig, Settings, WindowType,
};
