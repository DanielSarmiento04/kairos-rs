use actix_web::{
    dev::ServiceRequest,
    Error, Result,
};
use log::warn;

/// Validates request size to prevent large payload attacks
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

/// Validates request headers for security issues
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