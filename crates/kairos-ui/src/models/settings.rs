//! Settings model for UI - WASM-compatible version.

use serde::{Deserialize, Serialize};
use crate::models::router::Router;

/// JWT configuration settings matching backend structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct JwtSettings {
    pub secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    #[serde(default)]
    pub required_claims: Vec<String>,
}

/// Rate limiting strategy types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum LimitStrategy {
    PerIP,
    PerUser,
    PerRoute,
    PerIPAndRoute,
    PerUserAndRoute,
    Composite { configs: Vec<RateLimitConfig> },
}

impl Default for LimitStrategy {
    fn default() -> Self {
        Self::PerIP
    }
}

/// Time window algorithms for rate limiting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowType {
    FixedWindow,
    SlidingWindow,
    TokenBucket,
}

impl Default for WindowType {
    fn default() -> Self {
        Self::SlidingWindow
    }
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimitConfig {
    pub strategy: LimitStrategy,
    pub requests_per_window: u64,
    #[serde(default = "default_window_duration")]
    pub window_duration_secs: u64,
    pub burst_allowance: u64,
    pub window_type: WindowType,
    pub enable_redis: bool,
    pub redis_key_prefix: String,
}

fn default_window_duration() -> u64 {
    60 // 60 seconds
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            strategy: LimitStrategy::PerIP,
            requests_per_window: 100,
            window_duration_secs: 60,
            burst_allowance: 20,
            window_type: WindowType::SlidingWindow,
            enable_redis: false,
            redis_key_prefix: "kairos_rl".to_string(),
        }
    }
}

/// CORS configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CorsConfig {
    pub enabled: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default)]
    pub allowed_methods: Vec<String>,
    #[serde(default)]
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age_secs: Option<u64>,
}

/// Metrics configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus_endpoint: String,
    pub collect_per_route: bool,
}

/// Server configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub keep_alive_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 5900,
            workers: 4,
            keep_alive_secs: 75,
        }
    }
}

/// Complete gateway configuration matching backend Settings structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub version: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<CorsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfig>,
    pub routers: Vec<Router>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            jwt: None,
            rate_limit: None,
            cors: Some(CorsConfig {
                enabled: true,
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                allowed_headers: vec!["*".to_string()],
                allow_credentials: false,
                max_age_secs: Some(3600),
            }),
            metrics: Some(MetricsConfig {
                enabled: true,
                prometheus_endpoint: "/metrics".to_string(),
                collect_per_route: true,
            }),
            server: Some(ServerConfig::default()),
            routers: Vec::new(),
        }
    }
}

impl Settings {
    /// Validate the settings configuration.
    pub fn validate(&self) -> Result<(), String> {
        // Validate JWT settings if present
        if let Some(ref jwt) = self.jwt {
            if jwt.secret.is_empty() {
                return Err("JWT secret cannot be empty".to_string());
            }
            if jwt.secret.len() < 32 {
                return Err("JWT secret should be at least 32 characters for security".to_string());
            }
            if jwt.secret == "please-change-this-secret" || 
               jwt.secret == "your-super-secure-jwt-secret-key-must-be-at-least-32-characters-long" {
                return Err("JWT secret must be changed from default value".to_string());
            }
        }
        
        // Validate rate limiting settings if present
        if let Some(ref rate_limit) = self.rate_limit {
            if rate_limit.requests_per_window == 0 {
                return Err("Rate limit requests_per_window must be greater than 0".to_string());
            }
            if rate_limit.window_duration_secs == 0 {
                return Err("Rate limit window_duration_secs must be greater than 0".to_string());
            }
        }
        
        // Validate server settings if present
        if let Some(ref server) = self.server {
            if server.host.is_empty() {
                return Err("Server host cannot be empty".to_string());
            }
            if server.port == 0 {
                return Err("Server port must be greater than 0".to_string());
            }
            if server.workers == 0 {
                return Err("Server workers must be greater than 0".to_string());
            }
        }
        
        // Validate all routers
        for router in &self.routers {
            router.validate()?;
        }
        
        Ok(())
    }
}
