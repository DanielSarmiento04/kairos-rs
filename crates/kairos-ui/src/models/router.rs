//! Router model for UI - WASM-compatible version.
//!
//! This module provides router configuration models that work in both
//! server-side rendering (SSR) and WebAssembly (WASM) contexts.

use serde::{Deserialize, Serialize};
use crate::models::transform::{RequestTransformation, ResponseTransformation};

/// Protocol type for gateway routes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    #[serde(rename = "websocket")]
    WebSocket,
    Ftp,
    Dns,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Http
    }
}

/// Load balancing strategy for distributing requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
    Weighted,
    IpHash,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Backend server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub health_check_path: Option<String>,
}

fn default_weight() -> u32 {
    1
}

impl Backend {
    pub fn validate(&self) -> Result<(), String> {
        let valid_schemes = ["http://", "https://", "ws://", "wss://"];
        let has_valid_scheme = valid_schemes.iter().any(|scheme| self.host.starts_with(scheme));
        
        if !has_valid_scheme {
            return Err(format!(
                "Backend host must start with http://, https://, ws://, or wss://: {}", 
                self.host
            ));
        }
        
        if self.port == 0 {
            return Err("Backend port must be between 1 and 65535".to_string());
        }
        
        if self.weight == 0 {
            return Err("Backend weight must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

/// Retry configuration for handling transient failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff_ms: u64,
    #[serde(default = "default_max_backoff")]
    pub max_backoff_ms: u64,
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    #[serde(default = "default_retry_status_codes")]
    pub retry_on_status_codes: Vec<u16>,
    #[serde(default = "default_retry_on_connection_error")]
    pub retry_on_connection_error: bool,
}

fn default_max_retries() -> u32 {
    3
}

fn default_initial_backoff() -> u64 {
    100
}

fn default_max_backoff() -> u64 {
    5000
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_retry_status_codes() -> Vec<u16> {
    vec![502, 503, 504]
}

fn default_retry_on_connection_error() -> bool {
    true
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_backoff_ms: default_initial_backoff(),
            max_backoff_ms: default_max_backoff(),
            backoff_multiplier: default_backoff_multiplier(),
            retry_on_status_codes: default_retry_status_codes(),
            retry_on_connection_error: default_retry_on_connection_error(),
        }
    }
}

impl RetryConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_retries > 10 {
            return Err("max_retries should not exceed 10 to prevent excessive delays".to_string());
        }
        
        if self.initial_backoff_ms > self.max_backoff_ms {
            return Err("initial_backoff_ms cannot be greater than max_backoff_ms".to_string());
        }
        
        if self.backoff_multiplier < 1.0 {
            return Err("backoff_multiplier must be >= 1.0".to_string());
        }
        
        Ok(())
    }
}

/// Route configuration model.
///
/// Comprehensive router configuration supporting modern features like
/// load balancing, retry logic, multiple protocols, and authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Router {
    /// Legacy: Single target host (deprecated, use backends instead).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    
    /// Legacy: Target port (deprecated, use backends instead).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    
    /// List of backend servers for load balancing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<Vec<Backend>>,
    
    /// Protocol type (http, websocket, ftp, dns).
    #[serde(default)]
    pub protocol: Protocol,
    
    /// Load balancing strategy.
    #[serde(default)]
    pub load_balancing_strategy: LoadBalancingStrategy,
    
    /// External path pattern that clients use.
    pub external_path: String,
    
    /// Internal path pattern for upstream service.
    pub internal_path: String,
    
    /// Allowed HTTP methods.
    pub methods: Vec<String>,
    
    /// Whether JWT authentication is required.
    #[serde(default)]
    pub auth_required: bool,
    
    /// Retry configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,

    /// Request transformation configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_transformation: Option<RequestTransformation>,
    
    /// Response transformation configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_transformation: Option<ResponseTransformation>,

    /// AI Routing Policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_policy: Option<AiPolicy>,
}

/// AI Routing Strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiRoutingStrategy {
    ContentAnalysis { model: String },
    LatencyPrediction,
    AnomalyDetection,
}

/// AI Policy Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPolicy {
    pub enabled: bool,
    pub strategy: AiRoutingStrategy,
    pub provider: String,
    pub fallback_backend_index: Option<usize>,
}

impl Router {
    /// Create a new router with basic configuration.
    pub fn new_basic(
        host: String,
        port: u16,
        external_path: String,
        internal_path: String,
        methods: Vec<String>,
        auth_required: bool,
    ) -> Self {
        Self {
            host: Some(host),
            port: Some(port),
            backends: None,
            protocol: Protocol::default(),
            load_balancing_strategy: LoadBalancingStrategy::default(),
            external_path,
            internal_path,
            methods,
            auth_required,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
        }
    }
    
    /// Create a new router with load balancing.
    pub fn new_with_backends(
        backends: Vec<Backend>,
        external_path: String,
        internal_path: String,
        methods: Vec<String>,
        load_balancing_strategy: LoadBalancingStrategy,
        auth_required: bool,
    ) -> Self {
        Self {
            host: None,
            port: None,
            backends: Some(backends),
            protocol: Protocol::default(),
            load_balancing_strategy,
            external_path,
            internal_path,
            methods,
            auth_required,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
        }
    }
    
    /// Validate the router configuration.
    pub fn validate(&self) -> Result<(), String> {
        // Validate paths
        if !self.external_path.starts_with('/') {
            return Err("External path must start with '/'".to_string());
        }
        
        if !self.internal_path.starts_with('/') {
            return Err("Internal path must start with '/'".to_string());
        }
        
        // Validate methods
        if self.methods.is_empty() {
            return Err("At least one HTTP method must be specified".to_string());
        }
        
        let valid_methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE"];
        for method in &self.methods {
            if !valid_methods.contains(&method.as_str()) {
                return Err(format!("Invalid HTTP method: {}", method));
            }
        }
        
        // Validate backends or legacy host/port
        if let Some(backends) = &self.backends {
            if backends.is_empty() {
                return Err("At least one backend must be specified".to_string());
            }
            
            for (i, backend) in backends.iter().enumerate() {
                backend.validate().map_err(|e| {
                    format!("Backend {} validation failed: {}", i, e)
                })?;
            }
        } else if let (Some(host), Some(port)) = (&self.host, &self.port) {
            if !host.starts_with("http://") && !host.starts_with("https://") {
                return Err("Host must start with http:// or https://".to_string());
            }
            
            if *port == 0 {
                return Err("Port must be between 1 and 65535".to_string());
            }
        } else {
            return Err("Either backends or host/port must be specified".to_string());
        }
        
        // Validate retry config if present
        if let Some(retry_config) = &self.retry {
            retry_config.validate()?;
        }
        
        Ok(())
    }
}

impl Default for Router {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            backends: None,
            protocol: Protocol::default(),
            load_balancing_strategy: LoadBalancingStrategy::default(),
            external_path: "/".to_string(),
            internal_path: "/".to_string(),
            methods: vec!["GET".to_string()],
            auth_required: false,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
        }
    }
}
