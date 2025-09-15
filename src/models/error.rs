use actix_web::HttpResponse;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Request timeout after {timeout}s")]
    Timeout { timeout: u64 },
    
    #[error("Invalid route configuration: {message} (route: {route})")]
    Config { message: String, route: String },
    
    #[error("Upstream service error: {message} (url: {url}, status: {status:?})")]
    Upstream { 
        message: String, 
        url: String, 
        status: Option<u16>,
    },
    
    #[error("Route not found: {path}")]
    RouteNotFound { path: String },
    
    #[error("Method not allowed: {method} on {path}")]
    MethodNotAllowed { method: String, path: String },
    
    #[error("Invalid request: {reason}")]
    BadRequest { reason: String },
}

impl actix_web::error::ResponseError for GatewayError {
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
        };
        
        HttpResponse::build(status).json(json!({
            "error": error_message,
            "type": error_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "request_id": uuid::Uuid::new_v4().to_string()
        }))
    }
}
