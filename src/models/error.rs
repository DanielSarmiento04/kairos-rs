use actix_web::HttpResponse;
use serde_json::json;

/// Error types for the kairos-rs API gateway.
/// 
/// This enum represents all possible error conditions that can occur during
/// request processing, from configuration issues to upstream service failures.
/// Each error type provides structured information for proper HTTP response
/// generation and logging.
/// 
/// # Error Categories
/// 
/// - **Timeout**: Request processing exceeded configured time limits
/// - **Config**: Route configuration validation or parsing errors  
/// - **Upstream**: Errors from target services (connection, HTTP errors)
/// - **RouteNotFound**: No matching route configuration found
/// - **MethodNotAllowed**: HTTP method not allowed for the matched route
/// - **BadRequest**: Client request validation failures
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::models::error::GatewayError;
/// 
/// // Create a timeout error
/// let error = GatewayError::Timeout { timeout: 30 };
/// 
/// // Create an upstream service error
/// let error = GatewayError::Upstream {
///     message: "Connection refused".to_string(),
///     url: "http://backend:8080/api".to_string(),
///     status: Some(503),
/// };
/// ```
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// Request processing exceeded the configured timeout duration.
    /// 
    /// This occurs when upstream services are slow to respond or become
    /// unresponsive. The timeout value is configurable per gateway instance.
    #[error("Request timeout after {timeout}s")]
    Timeout { 
        /// Timeout duration in seconds that was exceeded
        timeout: u64 
    },
    
    /// Route configuration is invalid or malformed.
    /// 
    /// This error indicates issues with route definitions, such as invalid
    /// URL patterns, missing required fields, or configuration syntax errors.
    #[error("Invalid route configuration: {message} (route: {route})")]
    Config { 
        /// Detailed error message describing the configuration issue
        message: String, 
        /// The route pattern that caused the error
        route: String 
    },
    
    /// Error occurred when communicating with upstream services.
    /// 
    /// This includes network errors, HTTP errors from target services,
    /// or any issues during request forwarding.
    #[error("Upstream service error: {message} (url: {url}, status: {status:?})")]
    Upstream { 
        /// Detailed error message from the upstream service or client
        message: String, 
        /// The target URL that failed
        url: String, 
        /// HTTP status code returned by upstream service (if available)
        status: Option<u16>,
    },
    
    /// No route configuration matches the requested path.
    /// 
    /// This occurs when a client requests a path that doesn't match any
    /// configured route patterns in the gateway.
    #[error("Route not found: {path}")]
    RouteNotFound { 
        /// The requested path that couldn't be matched
        path: String 
    },
    
    /// HTTP method not allowed for the matched route.
    /// 
    /// This occurs when a client uses an HTTP method that isn't listed
    /// in the route's allowed methods configuration.
    #[error("Method not allowed: {method} on {path}")]
    MethodNotAllowed { 
        /// The HTTP method that was attempted
        method: String, 
        /// The path where the method was not allowed
        path: String 
    },
    
    /// Client request validation failed.
    /// 
    /// This covers various client-side errors such as malformed requests,
    /// invalid headers, or other request validation failures.
    #[error("Invalid request: {reason}")]
    BadRequest { 
        /// Specific reason why the request was invalid
        reason: String 
    },

    /// Circuit breaker is open, protecting upstream service.
    /// 
    /// This occurs when an upstream service has experienced too many failures
    /// and the circuit breaker has opened to prevent further requests.
    #[error("Circuit breaker is open for service: {service}")]
    CircuitOpen {
        /// The upstream service identifier (host:port)
        service: String
    },
}

impl actix_web::error::ResponseError for GatewayError {
    /// Converts a `GatewayError` into an appropriate HTTP response.
    /// 
    /// This implementation provides structured error responses with consistent
    /// formatting across all error types. Each response includes:
    /// - Appropriate HTTP status code
    /// - Structured JSON error message
    /// - Error type classification
    /// - RFC3339 timestamp
    /// - Unique request ID for tracing
    /// 
    /// # HTTP Status Code Mapping
    /// 
    /// - `Timeout` → 504 Gateway Timeout
    /// - `Config` → 502 Bad Gateway  
    /// - `Upstream` → 502 Bad Gateway
    /// - `CircuitOpen` → 503 Service Unavailable
    /// - `RouteNotFound` → 404 Not Found
    /// - `MethodNotAllowed` → 405 Method Not Allowed
    /// - `BadRequest` → 400 Bad Request
    /// 
    /// # Response Format
    /// 
    /// ```json
    /// {
    ///   "error": "Detailed error message",
    ///   "type": "error_type_identifier", 
    ///   "timestamp": "2024-03-15T10:30:00Z",
    ///   "request_id": "550e8400-e29b-41d4-a716-446655440000"
    /// }
    /// ```
    /// 
    /// # Examples
    /// 
    /// Timeout error response:
    /// ```json
    /// {
    ///   "error": "Request timeout after 30s",
    ///   "type": "timeout",
    ///   "timestamp": "2024-03-15T10:30:00Z", 
    ///   "request_id": "550e8400-e29b-41d4-a716-446655440000"
    /// }
    /// ```
    fn error_response(&self) -> HttpResponse {
        let (status, error_type, error_message) = match self {
            GatewayError::Timeout { timeout } => (
                actix_web::http::StatusCode::GATEWAY_TIMEOUT,
                "timeout",
                format!("Request timeout after {}s", timeout)
            ),
            GatewayError::Config { message, route } => (
                actix_web::http::StatusCode::BAD_GATEWAY,
                "config",
                format!("Configuration error for route {}: {}", route, message)
            ),
            GatewayError::Upstream { message, url, status } => (
                actix_web::http::StatusCode::BAD_GATEWAY,
                "upstream",
                format!("Upstream error for {}: {} (status: {:?})", url, message, status)
            ),
            GatewayError::RouteNotFound { path } => (
                actix_web::http::StatusCode::NOT_FOUND,
                "route_not_found",
                format!("No route found for path: {}", path)
            ),
            GatewayError::MethodNotAllowed { method, path } => (
                actix_web::http::StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                format!("Method {} not allowed for path: {}", method, path)
            ),
            GatewayError::BadRequest { reason } => (
                actix_web::http::StatusCode::BAD_REQUEST,
                "bad_request",
                reason.clone()
            ),
            GatewayError::CircuitOpen { service } => (
                actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
                "circuit_open",
                format!("Service {} is currently unavailable (circuit breaker open)", service)
            ),
        };
        
        HttpResponse::build(status).json(json!({
            "error": error_message,
            "type": error_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "request_id": uuid::Uuid::new_v4().to_string()
        }))
    }
}
