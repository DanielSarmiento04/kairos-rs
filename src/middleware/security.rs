use actix_web::middleware::DefaultHeaders;

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

/// Creates CORS headers for development (should be configured properly for production)
pub fn cors_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("Access-Control-Allow-Origin", "*"))
        .add(("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"))
        .add(("Access-Control-Allow-Headers", "Content-Type, Authorization"))
}