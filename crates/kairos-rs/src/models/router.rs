use serde::{Deserialize, Serialize};

/// Load balancing strategy for distributing requests across multiple backends.
/// 
/// This enum defines different algorithms for selecting which backend server
/// should handle each incoming request. Each strategy has different characteristics
/// and is suitable for different use cases.
/// 
/// # Strategies
/// 
/// - **RoundRobin**: Distributes requests evenly in circular order
/// - **LeastConnections**: Routes to backend with fewest active connections
/// - **Random**: Randomly selects a backend server
/// - **Weighted**: Distributes based on configured weights
/// - **IPHash**: Routes based on client IP hash (sticky sessions)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingStrategy {
    /// Round-robin load balancing (default).
    /// Distributes requests evenly across all backends in order.
    /// Best for: Backends with similar capacity
    RoundRobin,
    
    /// Routes to backend with fewest active connections.
    /// Best for: Backends with varying capacity or long-running requests
    LeastConnections,
    
    /// Randomly selects a backend.
    /// Best for: Simple distribution without state tracking
    Random,
    
    /// Weighted distribution based on backend weights.
    /// Best for: Backends with different capacities
    Weighted,
    
    /// Hash-based routing using client IP for session persistence.
    /// Best for: Applications requiring sticky sessions
    IpHash,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Backend server configuration for a route.
/// 
/// Represents a single backend server that can handle requests for a route.
/// Multiple backends can be configured per route for load balancing and high availability.
/// 
/// # Examples
/// 
/// ```json
/// {
///   "host": "http://backend-1.example.com",
///   "port": 8080,
///   "weight": 2,
///   "health_check_path": "/health"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Backend {
    /// Target host URL including protocol (http:// or https://).
    pub host: String,
    
    /// Target port number for the backend service.
    pub port: u16,
    
    /// Weight for weighted load balancing (default: 1).
    /// Higher weights receive proportionally more traffic.
    /// Only used with LoadBalancingStrategy::Weighted.
    #[serde(default = "default_weight")]
    pub weight: u32,
    
    /// Optional health check path for this backend.
    /// If specified, the gateway will periodically check this endpoint.
    #[serde(default)]
    pub health_check_path: Option<String>,
}

fn default_weight() -> u32 {
    1
}

impl Backend {
    /// Validates backend configuration.
    pub fn validate(&self) -> Result<(), String> {
        if !self.host.starts_with("http://") && !self.host.starts_with("https://") {
            return Err(format!("Backend host must start with http:// or https://: {}", self.host));
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
/// 
/// Defines how the gateway should retry failed requests to backends,
/// including backoff strategies and which errors should trigger retries.
/// 
/// # Examples
/// 
/// ```json
/// {
///   "max_retries": 3,
///   "initial_backoff_ms": 100,
///   "max_backoff_ms": 5000,
///   "backoff_multiplier": 2.0,
///   "retry_on_status_codes": [502, 503, 504]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3).
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    
    /// Initial backoff delay in milliseconds (default: 100ms).
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff_ms: u64,
    
    /// Maximum backoff delay in milliseconds (default: 5000ms).
    #[serde(default = "default_max_backoff")]
    pub max_backoff_ms: u64,
    
    /// Backoff multiplier for exponential backoff (default: 2.0).
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    
    /// HTTP status codes that should trigger a retry.
    /// Common retryable codes: 408, 429, 502, 503, 504
    #[serde(default = "default_retry_status_codes")]
    pub retry_on_status_codes: Vec<u16>,
    
    /// Whether to retry on network/connection errors (default: true).
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
    /// Validates retry configuration.
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
    
    /// Calculates the backoff delay for a given retry attempt.
    pub fn calculate_backoff(&self, attempt: u32) -> u64 {
        let backoff = (self.initial_backoff_ms as f64) 
            * self.backoff_multiplier.powi(attempt as i32);
        backoff.min(self.max_backoff_ms as f64) as u64
    }
}

/// Configuration for HTTP route forwarding in the kairos-rs gateway.
/// 
/// A `Router` defines how external requests are mapped to internal services,
/// including host translation, path transformation, method validation, and
/// optional authentication requirements.
/// 
/// # Examples
/// 
/// Basic route configuration (legacy single backend):
/// ```json
/// {
///   "host": "http://backend-service",
///   "port": 8080,
///   "external_path": "/api/users/{id}",
///   "internal_path": "/v1/user/{id}",
///   "methods": ["GET", "POST", "PUT"],
///   "auth_required": false
/// }
/// ```
/// 
/// Advanced route with load balancing:
/// ```json
/// {
///   "backends": [
///     {"host": "http://backend-1", "port": 8080, "weight": 2},
///     {"host": "http://backend-2", "port": 8080, "weight": 1}
///   ],
///   "load_balancing_strategy": "weighted",
///   "external_path": "/api/users/{id}",
///   "internal_path": "/v1/user/{id}",
///   "methods": ["GET", "POST", "PUT"],
///   "auth_required": false,
///   "retry": {
///     "max_retries": 3,
///     "initial_backoff_ms": 100,
///     "retry_on_status_codes": [502, 503, 504]
///   }
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Router {
    /// Legacy: Single target host URL (deprecated, use backends instead).
    /// This is where the gateway will forward matching requests.
    /// 
    /// # Examples
    /// - `"http://localhost"`
    /// - `"https://api.example.com"`
    /// - `"http://backend-service"` (for container environments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    
    /// Legacy: Target port number (deprecated, use backends instead).
    /// Must be a valid port number between 1 and 65535.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    
    /// List of backend servers for load balancing.
    /// At least one backend must be specified (or use legacy host/port).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<Vec<Backend>>,
    
    /// Load balancing strategy for distributing requests.
    /// Only used when multiple backends are configured.
    #[serde(default)]
    pub load_balancing_strategy: LoadBalancingStrategy,
    
    /// External path pattern that clients will use to access the service.
    /// Supports dynamic parameters using `{parameter_name}` syntax.
    /// Must start with a forward slash (`/`).
    /// 
    /// # Examples
    /// - `"/api/users"` (static path)
    /// - `"/api/users/{id}"` (dynamic parameter)
    /// - `"/api/users/{id}/posts/{post_id}"` (multiple parameters)
    pub external_path: String,
    
    /// Internal path pattern for the upstream service.
    /// Parameters from `external_path` can be used here with the same `{parameter_name}` syntax.
    /// Must start with a forward slash (`/`).
    /// 
    /// # Examples
    /// - `"/v1/user"` (path translation)
    /// - `"/v1/user/{id}"` (parameter forwarding)
    /// - `"/legacy/api/user/{id}"` (path and parameter transformation)
    pub internal_path: String,
    
    /// List of allowed HTTP methods for this route.
    /// Only requests with these methods will be forwarded.
    /// 
    /// # Valid Methods
    /// - `GET`, `POST`, `PUT`, `DELETE`, `HEAD`, `OPTIONS`, `PATCH`, `TRACE`
    /// 
    /// # Examples
    /// - `["GET"]` (read-only endpoint)
    /// - `["POST", "PUT"]` (write operations)
    /// - `["GET", "POST", "PUT", "DELETE"]` (full CRUD)
    pub methods: Vec<String>,

    /// Whether JWT authentication is required for this route.
    /// 
    /// When `true`, the request must include a valid JWT token in the
    /// Authorization header. The token will be validated according to
    /// the global JWT configuration.
    /// 
    /// # Default
    /// 
    /// If not specified, defaults to `false` (no authentication required).
    /// 
    /// # Examples
    /// - `false` for public endpoints (health checks, static content)
    /// - `true` for protected endpoints (user data, admin operations)
    #[serde(default)]
    pub auth_required: bool,
    
    /// Retry configuration for handling transient failures.
    /// If not specified, no retries will be attempted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
}

impl Router {
    /// Validates the router configuration for correctness and security.
    /// 
    /// This method performs comprehensive validation of all router fields:
    /// - Host URL format validation (must include protocol)
    /// - Port range validation (1-65535)
    /// - Path format validation (must start with `/`)
    /// - HTTP method validation (against standard methods)
    /// - Ensures at least one method is specified
    /// - Validates backends if using load balancing
    /// - Validates retry configuration if present
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the configuration is valid
    /// - `Err(String)` with a descriptive error message if validation fails
    /// 
    /// # Examples
    /// 
    /// Legacy configuration:
    /// ```rust
    /// use kairos_rs::models::router::Router;
    /// 
    /// let router = Router {
    ///     host: Some("http://localhost".to_string()),
    ///     port: Some(8080),
    ///     backends: None,
    ///     load_balancing_strategy: Default::default(),
    ///     external_path: "/api/users".to_string(),
    ///     internal_path: "/v1/users".to_string(),
    ///     methods: vec!["GET".to_string(), "POST".to_string()],
    ///     auth_required: false,
    ///     retry: None,
    /// };
    /// 
    /// assert!(router.validate().is_ok());
    /// ```
    /// 
    /// # Errors
    /// 
    /// This method will return an error if:
    /// - Neither host/port nor backends are specified
    /// - Host doesn't start with `http://` or `https://`
    /// - Port is 0 (ports 1-65535 are valid)
    /// - External or internal path doesn't start with `/`
    /// - No HTTP methods are specified
    /// - An invalid HTTP method is provided
    /// - Backend validation fails
    /// - Retry configuration is invalid
    pub fn validate(&self) -> Result<(), String> {
        // Validate paths start with '/'
        if !self.external_path.starts_with('/') {
            return Err("External path must start with '/'".to_string());
        }

        if !self.internal_path.starts_with('/') {
            return Err("Internal path must start with '/'".to_string());
        }

        // Validate HTTP methods
        if self.methods.is_empty() {
            return Err("At least one HTTP method must be specified".to_string());
        }

        let valid_methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE"];
        for method in &self.methods {
            if !valid_methods.contains(&method.as_str()) {
                return Err(format!("Invalid HTTP method: {}", method));
            }
        }

        // Validate backends configuration
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
            // Legacy validation
            log::debug!(
                "Validating legacy Router: host={}, port={}, external_path={}, internal_path={}, methods={:?}",
                host, port, self.external_path, self.internal_path, self.methods
            );

            if !host.starts_with("http://") && !host.starts_with("https://") {
                return Err("Host must start with http:// or https://".to_string());
            }

            if *port == 0 {
                return Err("Port must be between 1 and 65535".to_string());
            }
        } else {
            return Err("Either backends or host/port must be specified".to_string());
        }

        // Validate retry configuration if present
        if let Some(retry_config) = &self.retry {
            retry_config.validate()?;
        }

        Ok(())
    }
    
    /// Returns all backends for this router (handles both legacy and new config).
    pub fn get_backends(&self) -> Vec<Backend> {
        if let Some(backends) = &self.backends {
            backends.clone()
        } else if let (Some(host), Some(port)) = (&self.host, &self.port) {
            // Convert legacy config to backend
            vec![Backend {
                host: host.clone(),
                port: *port,
                weight: 1,
                health_check_path: None,
            }]
        } else {
            vec![]
        }
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Settings {
//     pub version: u8,
//     pub routers: Vec<Router>,
// }
