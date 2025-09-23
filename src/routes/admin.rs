//! Administrative API routes protected by JWT authentication.
//! 
//! This module provides administrative endpoints that require JWT authentication
//! for accessing sensitive gateway operations like configuration management,
//! metrics, and system status information.

use crate::middleware::auth::{JwtAuth, JwtConfig};
use actix_web::{web, HttpResponse, Result, HttpRequest};
use log::{info, warn};
use serde_json::json;

/// Configuration information response for admin endpoints.
/// 
/// Provides structured information about the current gateway configuration
/// without exposing sensitive details like upstream URLs or secrets.
pub async fn admin_status(req: HttpRequest) -> Result<HttpResponse> {
    // Log admin access for audit purposes
    let connection_info = req.connection_info();
    let client_ip = connection_info.peer_addr().unwrap_or("unknown");
    info!("Admin status accessed from: {}", client_ip);

    // Get runtime information
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "kairos-rs-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": uptime,
        "authenticated": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "features": {
            "jwt_auth": true,
            "rate_limiting": true,
            "circuit_breaker": true,
            "metrics": true,
            "health_checks": true
        }
    })))
}

/// Configuration reload endpoint for administrative operations.
/// 
/// Provides a way to trigger configuration validation and report any issues
/// without requiring a full service restart.
pub async fn admin_config_check(req: HttpRequest) -> Result<HttpResponse> {
    let connection_info = req.connection_info();
    let client_ip = connection_info.peer_addr().unwrap_or("unknown");
    warn!("Configuration check requested from: {}", client_ip);

    // In a production system, this would reload and validate configuration
    // For now, we'll return the current configuration status
    Ok(HttpResponse::Ok().json(json!({
        "message": "Configuration validation completed",
        "status": "valid",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "warnings": [
            "This is a demonstration endpoint",
            "In production, this would perform actual config validation"
        ]
    })))
}

/// Configures JWT-protected administrative routes.
/// 
/// This function sets up admin endpoints that require valid JWT authentication.
/// The JWT secret can be configured via the `KAIROS_JWT_SECRET` environment variable.
/// 
/// # Environment Variables
/// 
/// - `KAIROS_JWT_SECRET`: Secret key for JWT validation (default: "dev-secret-change-in-production")
/// 
/// # Routes Configured
/// 
/// - `GET /admin/status` - System status and feature information
/// - `POST /admin/config/check` - Configuration validation endpoint
/// 
/// # Security
/// 
/// All admin routes require a valid JWT token in the Authorization header:
/// ```
/// Authorization: Bearer <jwt-token>
/// ```
pub fn configure_admin(cfg: &mut web::ServiceConfig) {
    // Get JWT secret from environment, with a default for development
    let jwt_secret = std::env::var("KAIROS_JWT_SECRET")
        .unwrap_or_else(|_| {
            warn!("KAIROS_JWT_SECRET not set, using default secret for development");
            "dev-secret-change-in-production".to_string()
        });

    let jwt_config = JwtConfig::new(jwt_secret);
    let jwt_auth = JwtAuth::new(jwt_config);

    cfg.service(
        web::scope("/admin")
            .wrap(jwt_auth)
            .route("/status", web::get().to(admin_status))
            .route("/config/check", web::post().to(admin_config_check))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::{Claims, create_test_token};
    use actix_web::{test, App};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[actix_web::test]
    async fn test_admin_status_with_auth() {
        let secret = "test-secret";
        let jwt_config = JwtConfig::new(secret.to_string());
        let jwt_auth = JwtAuth::new(jwt_config);

        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/admin")
                        .wrap(jwt_auth)
                        .route("/status", web::get().to(admin_status))
                )
        ).await;

        // Create a valid JWT token
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let claims = Claims {
            sub: "admin".to_string(),
            exp: now + 3600, // 1 hour from now
            iat: now,
            iss: None,
            aud: None,
            roles: None,
        };
        let token = create_test_token(claims, secret).unwrap();

        let req = test::TestRequest::get()
            .uri("/admin/status")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_admin_status_without_auth() {
        let secret = "test-secret";
        let jwt_config = JwtConfig::new(secret.to_string());
        let jwt_auth = JwtAuth::new(jwt_config);

        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/admin")
                        .wrap(jwt_auth)
                        .route("/status", web::get().to(admin_status))
                )
        ).await;

        let req = test::TestRequest::get()
            .uri("/admin/status")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401); // Unauthorized
    }
}