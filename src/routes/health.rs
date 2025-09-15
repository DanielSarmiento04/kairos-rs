use actix_web::{web, HttpResponse, Result};
use serde_json::json;

/// Health check endpoint
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })))
}

/// Readiness check endpoint (for Kubernetes)
pub async fn readiness_check() -> Result<HttpResponse> {
    // In a real implementation, you might check database connections,
    // external service availability, etc.
    Ok(HttpResponse::Ok().json(json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Liveness check endpoint (for Kubernetes)
pub async fn liveness_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Configure health check routes
pub fn configure_health(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check))
       .route("/ready", web::get().to(readiness_check))
       .route("/live", web::get().to(liveness_check));
}