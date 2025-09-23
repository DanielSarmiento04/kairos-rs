use actix_web::middleware::DefaultHeaders;
use log::{warn, info};

/// Creates security headers middleware for production deployment
pub fn security_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("X-Frame-Options", "DENY"))
        .add(("X-XSS-Protection", "1; mode=block"))
        .add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains"))
        .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
        .add(("Content-Security-Policy", "default-src 'self'"))
        .add(("Permissions-Policy", "geolocation=(), microphone=(), camera=()"))
}

/// Creates configurable CORS headers middleware for cross-origin requests.
/// 
/// This function creates CORS headers based on environment variables for production-safe
/// configuration. It provides sensible defaults for development while allowing
/// fine-grained control in production environments.
/// 
/// # Environment Variables
/// 
/// - `KAIROS_CORS_ORIGIN`: Allowed origins (default: "*" for development)
/// - `KAIROS_CORS_METHODS`: Allowed methods (default: "GET, POST, PUT, DELETE, OPTIONS")
/// - `KAIROS_CORS_HEADERS`: Allowed headers (default: "Content-Type, Authorization")
/// - `KAIROS_CORS_MAX_AGE`: Preflight cache time in seconds (default: "3600")
/// - `KAIROS_CORS_CREDENTIALS`: Allow credentials (default: "false")
/// 
/// # Security Considerations
/// 
/// In production, avoid using "*" for origins and configure specific domains:
/// ```bash
/// export KAIROS_CORS_ORIGIN="https://app.example.com,https://admin.example.com"
/// export KAIROS_CORS_CREDENTIALS="true"
/// ```
/// 
/// # Examples
/// 
/// Development configuration (permissive):
/// ```bash
/// # Uses defaults - allows all origins
/// cargo run
/// ```
/// 
/// Production configuration (restrictive):
/// ```bash
/// export KAIROS_CORS_ORIGIN="https://myapp.com"
/// export KAIROS_CORS_CREDENTIALS="true"
/// cargo run
/// ```
pub fn cors_headers() -> DefaultHeaders {
    // Get CORS configuration from environment variables
    let allowed_origins = std::env::var("KAIROS_CORS_ORIGIN")
        .unwrap_or_else(|_| {
            warn!("KAIROS_CORS_ORIGIN not set, using wildcard (*) - not recommended for production");
            "*".to_string()
        });
    
    let allowed_methods = std::env::var("KAIROS_CORS_METHODS")
        .unwrap_or_else(|_| "GET, POST, PUT, DELETE, OPTIONS".to_string());
    
    let allowed_headers = std::env::var("KAIROS_CORS_HEADERS")
        .unwrap_or_else(|_| "Content-Type, Authorization, X-Requested-With".to_string());
    
    let max_age = std::env::var("KAIROS_CORS_MAX_AGE")
        .unwrap_or_else(|_| "3600".to_string());
    
    let allow_credentials = std::env::var("KAIROS_CORS_CREDENTIALS")
        .unwrap_or_else(|_| "false".to_string());

    info!("CORS configuration: origins={}, methods={}, credentials={}", 
          allowed_origins, allowed_methods, allow_credentials);

    // Warn if using insecure wildcard in production
    if allowed_origins == "*" && allow_credentials == "true" {
        warn!("SECURITY WARNING: CORS configured with wildcard origin (*) and credentials=true - this is a security risk!");
    }

    let mut headers = DefaultHeaders::new()
        .add(("Access-Control-Allow-Origin", allowed_origins))
        .add(("Access-Control-Allow-Methods", allowed_methods))
        .add(("Access-Control-Allow-Headers", allowed_headers))
        .add(("Access-Control-Max-Age", max_age));

    // Only add credentials header if explicitly enabled
    if allow_credentials.to_lowercase() == "true" {
        headers = headers.add(("Access-Control-Allow-Credentials", "true"));
    }

    headers
}