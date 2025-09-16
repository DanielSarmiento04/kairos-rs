use actix_web::{
    dev::ServiceRequest,
    Error, Result,
};
use log::warn;

/// Creates a request size validation middleware to prevent large payload attacks.
/// 
/// This function returns a middleware closure that validates incoming request
/// payload sizes against a configurable maximum. It helps protect the gateway
/// from memory exhaustion attacks and ensures resource usage stays within
/// acceptable bounds.
/// 
/// # Parameters
/// 
/// * `max_size` - Maximum allowed payload size in bytes
/// 
/// # Returns
/// 
/// A middleware function that can be used with Actix Web's `wrap_fn` method
/// 
/// # Security Features
/// 
/// - **DoS Protection**: Prevents memory exhaustion from large payloads
/// - **Early Rejection**: Rejects oversized requests before processing
/// - **Detailed Logging**: Records blocked requests with client information
/// - **Configurable Limits**: Allows different size limits per endpoint
/// 
/// # Validation Process
/// 
/// 1. **Header Inspection**: Checks `Content-Length` header if present
/// 2. **Size Comparison**: Compares declared size against maximum allowed
/// 3. **Early Rejection**: Returns `413 Payload Too Large` for oversized requests
/// 4. **Logging**: Records security violations with client IP and size information
/// 
/// # Examples
/// 
/// ```rust
/// use actix_web::{web, App};
/// use kairos_rs::middleware::validation::validate_request_size;
/// 
/// let app = App::new()
///     .service(
///         web::resource("/upload")
///             .wrap_fn(validate_request_size(10 * 1024 * 1024)) // 10MB limit
///             .route(web::post().to(upload_handler))
///     )
///     .service(
///         web::resource("/api/{path:.*}")
///             .wrap_fn(validate_request_size(1024 * 1024)) // 1MB limit
///             .route(web::post().to(api_handler))
///     );
/// ```
/// 
/// # Error Responses
/// 
/// Returns `413 Payload Too Large` with message "Request payload too large"
/// when the payload exceeds the configured maximum size.
/// 
/// # Performance
/// 
/// - **Minimal Overhead**: Simple header parsing and integer comparison
/// - **Early Exit**: Rejects oversized requests before body processing
/// - **Memory Efficient**: No payload reading required for validation
/// 
/// # Thread Safety
/// 
/// The returned middleware is safe to use across multiple threads and
/// can be applied to multiple routes simultaneously.
pub fn validate_request_size(max_size: usize) -> impl Fn(&ServiceRequest) -> Result<(), Error> {
    move |req: &ServiceRequest| {
        if let Some(content_length) = req.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    if length > max_size {
                        warn!(
                            "Request payload too large: {} bytes (max: {} bytes) from {}",
                            length,
                            max_size,
                            req.connection_info().peer_addr().unwrap_or("unknown")
                        );
                        return Err(actix_web::error::ErrorPayloadTooLarge(
                            "Request payload too large"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Creates a request header validation middleware for enhanced security.
/// 
/// This function returns a middleware closure that validates incoming request
/// headers for security threats and compliance with acceptable patterns. It
/// helps protect the gateway from various attack vectors and ensures only
/// legitimate requests are processed.
/// 
/// # Returns
/// 
/// A middleware function that can be used with Actix Web's `wrap_fn` method
/// 
/// # Security Checks
/// 
/// ## User-Agent Validation
/// - **Malicious Tool Detection**: Blocks known security scanning tools
/// - **Pattern Matching**: Identifies suspicious User-Agent strings
/// - **Tool Blacklist**: Blocks sqlmap, nikto, nmap, masscan, burp suite
/// - **Case-Insensitive**: Handles various case combinations of tool names
/// 
/// ## Content-Type Validation
/// - **Method-Specific**: Validates Content-Type for POST/PUT requests
/// - **Whitelist Approach**: Only allows predefined content types
/// - **Security Focus**: Prevents content-type confusion attacks
/// - **Media Type Support**: Supports common legitimate content types
/// 
/// # Allowed Content Types
/// 
/// For POST/PUT requests, these content types are allowed:
/// - `application/json` - JSON API requests
/// - `application/x-www-form-urlencoded` - Form submissions
/// - `text/plain` - Plain text content
/// - `multipart/form-data` - File uploads and forms
/// 
/// # Validation Process
/// 
/// 1. **User-Agent Check**: Examines User-Agent header for malicious patterns
/// 2. **Method Inspection**: Determines if Content-Type validation is needed
/// 3. **Content-Type Validation**: Validates Content-Type against whitelist
/// 4. **Security Logging**: Records all security violations with details
/// 5. **Error Response**: Returns appropriate HTTP error for violations
/// 
/// # Examples
/// 
/// ```rust
/// use actix_web::{web, App};
/// use kairos_rs::middleware::validation::validate_headers;
/// 
/// let app = App::new()
///     .service(
///         web::resource("/api/{path:.*}")
///             .wrap_fn(validate_headers())
///             .route(web::post().to(api_handler))
///             .route(web::get().to(api_handler))
///     );
/// ```
/// 
/// # Error Responses
/// 
/// - **403 Forbidden**: For suspicious User-Agent patterns
/// - **415 Unsupported Media Type**: For invalid Content-Type headers
/// 
/// # Performance
/// 
/// - **Fast Pattern Matching**: Efficient string operations for validation
/// - **Early Rejection**: Blocks malicious requests before processing
/// - **Selective Validation**: Only validates relevant headers based on method
/// 
/// # Security Benefits
/// 
/// - **Automated Tool Blocking**: Prevents common security scanning tools
/// - **Content-Type Attacks**: Prevents MIME confusion and upload attacks
/// - **Request Filtering**: Reduces attack surface by blocking suspicious requests
/// - **Audit Trail**: Comprehensive logging of security events
/// 
/// # Thread Safety
/// 
/// The returned middleware is safe to use across multiple threads and
/// requests simultaneously.
pub fn validate_headers() -> impl Fn(&ServiceRequest) -> Result<(), Error> {
    |req: &ServiceRequest| {
        // Check for suspicious User-Agent patterns
        if let Some(user_agent) = req.headers().get("user-agent") {
            if let Ok(ua_str) = user_agent.to_str() {
                // Block known malicious patterns
                let suspicious_patterns = ["sqlmap", "nikto", "nmap", "masscan", "burp"];
                for pattern in &suspicious_patterns {
                    if ua_str.to_lowercase().contains(pattern) {
                        warn!("Suspicious User-Agent detected: {} from {}", 
                            ua_str, 
                            req.connection_info().peer_addr().unwrap_or("unknown")
                        );
                        return Err(actix_web::error::ErrorForbidden("Forbidden"));
                    }
                }
            }
        }

        // Validate Content-Type for POST/PUT requests
        let method = req.method();
        if method == actix_web::http::Method::POST || method == actix_web::http::Method::PUT {
            if let Some(content_type) = req.headers().get("content-type") {
                if let Ok(ct_str) = content_type.to_str() {
                    // Allow only specific content types
                    let allowed_types = [
                        "application/json",
                        "application/x-www-form-urlencoded",
                        "text/plain",
                        "multipart/form-data"
                    ];
                    
                    if !allowed_types.iter().any(|&allowed| ct_str.starts_with(allowed)) {
                        warn!("Unsupported Content-Type: {} from {}", 
                            ct_str,
                            req.connection_info().peer_addr().unwrap_or("unknown")
                        );
                        return Err(actix_web::error::ErrorUnsupportedMediaType(
                            "Unsupported Content-Type"
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}