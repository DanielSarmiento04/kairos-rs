use actix_web::{web, HttpResponse, Result};
use serde_json::json;

/// General health check endpoint providing service status and basic information.
/// 
/// This endpoint provides comprehensive health information including service status,
/// version, current timestamp, and uptime. It's designed for general monitoring
/// and service discovery purposes.
/// 
/// # Response Format
/// 
/// ```json
/// {
///   "status": "healthy",
///   "version": "0.2.1",
///   "timestamp": "2024-03-15T10:30:00Z",
///   "uptime": 3600
/// }
/// ```
/// 
/// # Returns
/// 
/// - `200 OK` with JSON health information
/// 
/// # Use Cases
/// 
/// - General service monitoring
/// - Load balancer health checks
/// - Service discovery registration
/// - Automated testing verification
/// 
/// # Performance
/// 
/// This endpoint has minimal overhead and can handle high request rates.
/// Response time is typically under 1ms.
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

/// Kubernetes readiness probe endpoint indicating service is ready to receive traffic.
/// 
/// This endpoint is specifically designed for Kubernetes readiness probes and indicates
/// whether the gateway is ready to handle incoming requests. In a production environment,
/// this would typically check upstream service connectivity and other dependencies.
/// 
/// # Response Format
/// 
/// ```json
/// {
///   "status": "ready",
///   "timestamp": "2024-03-15T10:30:00Z"
/// }
/// ```
/// 
/// # Returns
/// 
/// - `200 OK` when service is ready to receive traffic
/// - `503 Service Unavailable` when service is not ready (if dependency checks fail)
/// 
/// # Kubernetes Configuration
/// 
/// ```yaml
/// readinessProbe:
///   httpGet:
///     path: /ready
///     port: 5900
///   initialDelaySeconds: 5
///   periodSeconds: 10
/// ```
/// 
/// # Future Enhancements
/// 
/// In a production environment, this endpoint would check:
/// - Database connection availability
/// - Upstream service connectivity
/// - Required configuration presence
/// - Resource availability (memory, disk)
pub async fn readiness_check() -> Result<HttpResponse> {
    // In a real implementation, you might check database connections,
    // external service availability, etc.
    Ok(HttpResponse::Ok().json(json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Kubernetes liveness probe endpoint indicating the service is alive and functioning.
/// 
/// This endpoint is designed for Kubernetes liveness probes and indicates whether
/// the gateway process is alive and should continue running. If this endpoint fails,
/// Kubernetes will restart the pod.
/// 
/// # Response Format
/// 
/// ```json
/// {
///   "status": "alive", 
///   "timestamp": "2024-03-15T10:30:00Z"
/// }
/// ```
/// 
/// # Returns
/// 
/// - `200 OK` when service is alive and functioning
/// 
/// # Kubernetes Configuration
/// 
/// ```yaml
/// livenessProbe:
///   httpGet:
///     path: /live
///     port: 5900
///   initialDelaySeconds: 30
///   periodSeconds: 30
/// ```
/// 
/// # Design Notes
/// 
/// This endpoint should only fail if the application is truly broken and needs
/// to be restarted. It should not fail for temporary issues like downstream
/// service unavailability.
pub async fn liveness_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Configures health check routes for Actix Web service.
/// 
/// This function registers all health-related endpoints with the Actix Web service
/// configuration. It sets up the standard health check endpoints used for monitoring
/// and Kubernetes probes.
/// 
/// # Registered Routes
/// 
/// - `GET /health` - General health check with detailed information
/// - `GET /ready` - Kubernetes readiness probe endpoint
/// - `GET /live` - Kubernetes liveness probe endpoint
/// 
/// # Parameters
/// 
/// * `cfg` - Mutable reference to Actix Web service configuration
/// 
/// # Examples
/// 
/// ```rust
/// use actix_web::{App, web};
/// use kairos_rs::routes::health::configure_health;
/// 
/// let app = App::new().configure(configure_health);
/// ```
/// 
/// # Route Performance
/// 
/// All health endpoints are optimized for:
/// - Minimal response time (< 1ms typical)
/// - Low CPU usage
/// - No external dependencies
/// - High concurrent request handling
pub fn configure_health(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check))
       .route("/ready", web::get().to(readiness_check))
       .route("/live", web::get().to(liveness_check));
}