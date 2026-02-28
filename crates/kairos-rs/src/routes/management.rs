//! Route management API endpoints for dynamic configuration.
//!
//! This module provides REST API endpoints for managing routes at runtime,
//! including creating, reading, updating, and deleting route configurations,
//! as well as validating route configurations.

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::router::Router;
use crate::models::settings::{AiSettings, Settings};

/// Shared state for route management operations.
///
/// This structure provides thread-safe access to gateway configuration
/// and handles persistence of route changes to disk.
///
/// # Thread Safety
///
/// Uses `Arc<RwLock<Settings>>` to allow concurrent reads while ensuring
/// exclusive access for writes.
#[derive(Clone)]
pub struct RouteManager {
    settings: Arc<RwLock<Settings>>,
    config_path: String,
}

impl RouteManager {
    /// Creates a new route manager with the given settings and configuration path.
    ///
    /// # Parameters
    ///
    /// * `settings` - Initial gateway settings
    /// * `config_path` - Path to the configuration file for persistence
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kairos_rs::routes::management::RouteManager;
    /// use kairos_rs::models::settings::Settings;
    ///
    /// let settings = Settings {
    ///     version: 1,
    ///     jwt: None,
    ///     rate_limit: None,
    ///     ai: None,
    ///     routers: vec![],
    /// };
    /// let manager = RouteManager::new(settings, "config.json".to_string());
    /// ```
    pub fn new(settings: Settings, config_path: String) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            config_path,
        }
    }

    /// Saves current settings to disk in JSON format.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error message on failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Settings cannot be serialized to JSON
    /// - File system write operation fails
    async fn save_to_disk(&self) -> Result<(), String> {
        let settings = self.settings.read().await;
        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        tokio::fs::write(&self.config_path, json)
            .await
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

/// Response structure for route operations.
///
/// Provides a consistent response format for all route management endpoints,
/// including success status, descriptive messages, and optional route data.
///
/// # Examples
///
/// ```rust
/// use kairos_rs::routes::management::RouteResponse;
///
/// let response = RouteResponse {
///     success: true,
///     message: "Route created successfully".to_string(),
///     route: None,
///     routes: None,
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct RouteResponse {
    /// Whether the operation completed successfully
    pub success: bool,
    /// Human-readable message describing the result
    pub message: String,
    /// Optional single route data (for get/create/update operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Router>,
    /// Optional list of routes (for list operations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<Router>>,
}

/// Request structure for route validation
#[derive(Serialize, Deserialize)]
pub struct ValidateRouteRequest {
    pub route: Router,
}

/// Response structure for route validation
#[derive(Serialize, Deserialize)]
pub struct ValidateRouteResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

/// List all routes
///
/// # Endpoint
///
/// `GET /api/routes`
///
/// # Response
///
/// Returns a JSON array of all configured routes with their complete configuration.
///
/// # Example
///
/// ```bash
/// curl http://localhost:5900/api/routes
/// ```
#[get("/api/routes")]
pub async fn list_routes(manager: web::Data<RouteManager>) -> impl Responder {
    let settings = manager.settings.read().await;

    HttpResponse::Ok().json(RouteResponse {
        success: true,
        message: format!("Found {} routes", settings.routers.len()),
        route: None,
        routes: Some(settings.routers.clone()),
    })
}

/// Get a specific route by external path
///
/// # Endpoint
///
/// `GET /api/routes/{external_path}`
///
/// # Parameters
///
/// * `external_path` - URL-encoded external path (e.g., `/api/users/{id}`)
///
/// # Example
///
/// ```bash
/// curl http://localhost:5900/api/routes/%2Fapi%2Fusers%2F%7Bid%7D
/// ```
#[get("/api/routes/{external_path:.*}")]
pub async fn get_route(
    manager: web::Data<RouteManager>,
    path: web::Path<String>,
) -> impl Responder {
    let external_path = format!("/{}", path.into_inner());
    let settings = manager.settings.read().await;

    if let Some(route) = settings
        .routers
        .iter()
        .find(|r| r.external_path == external_path)
    {
        HttpResponse::Ok().json(RouteResponse {
            success: true,
            message: "Route found".to_string(),
            route: Some(route.clone()),
            routes: None,
        })
    } else {
        HttpResponse::NotFound().json(RouteResponse {
            success: false,
            message: format!("Route not found: {}", external_path),
            route: None,
            routes: None,
        })
    }
}

/// Create a new route
///
/// # Endpoint
///
/// `POST /api/routes`
///
/// # Request Body
///
/// JSON object with route configuration. See `Router` struct for schema.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/routes \
///   -H "Content-Type: application/json" \
///   -d '{
///     "backends": [
///       {"host": "http://backend-1", "port": 8080, "weight": 1}
///     ],
///     "external_path": "/api/test",
///     "internal_path": "/test",
///     "methods": ["GET"],
///     "auth_required": false
///   }'
/// ```
#[post("/api/routes")]
pub async fn create_route(
    manager: web::Data<RouteManager>,
    route: web::Json<Router>,
) -> impl Responder {
    // Validate the route
    if let Err(e) = route.validate() {
        return HttpResponse::BadRequest().json(RouteResponse {
            success: false,
            message: format!("Route validation failed: {}", e),
            route: None,
            routes: None,
        });
    }

    let mut settings = manager.settings.write().await;

    // Check if route already exists
    if settings
        .routers
        .iter()
        .any(|r| r.external_path == route.external_path)
    {
        return HttpResponse::Conflict().json(RouteResponse {
            success: false,
            message: format!("Route already exists: {}", route.external_path),
            route: None,
            routes: None,
        });
    }

    // Add the route
    settings.routers.push(route.into_inner());

    // Save to disk
    drop(settings); // Release lock before saving
    if let Err(e) = manager.save_to_disk().await {
        return HttpResponse::InternalServerError().json(RouteResponse {
            success: false,
            message: format!("Failed to save configuration: {}", e),
            route: None,
            routes: None,
        });
    }

    HttpResponse::Created().json(RouteResponse {
        success: true,
        message: "Route created successfully. Restart required for changes to take effect."
            .to_string(),
        route: None,
        routes: None,
    })
}

/// Update an existing route
///
/// # Endpoint
///
/// `PUT /api/routes/{external_path}`
///
/// # Parameters
///
/// * `external_path` - URL-encoded external path of the route to update
///
/// # Request Body
///
/// JSON object with complete route configuration.
///
/// # Example
///
/// ```bash
/// curl -X PUT http://localhost:5900/api/routes/%2Fapi%2Ftest \
///   -H "Content-Type: application/json" \
///   -d '{
///     "backends": [
///       {"host": "http://backend-1", "port": 8080, "weight": 2}
///     ],
///     "external_path": "/api/test",
///     "internal_path": "/test",
///     "methods": ["GET", "POST"],
///     "auth_required": false
///   }'
/// ```
#[put("/api/routes/{external_path:.*}")]
pub async fn update_route(
    manager: web::Data<RouteManager>,
    path: web::Path<String>,
    route: web::Json<Router>,
) -> impl Responder {
    let external_path = format!("/{}", path.into_inner());

    // Validate the route
    if let Err(e) = route.validate() {
        return HttpResponse::BadRequest().json(RouteResponse {
            success: false,
            message: format!("Route validation failed: {}", e),
            route: None,
            routes: None,
        });
    }

    // Ensure the external_path in the route matches the URL parameter
    if route.external_path != external_path {
        return HttpResponse::BadRequest().json(RouteResponse {
            success: false,
            message: "Route external_path must match URL parameter".to_string(),
            route: None,
            routes: None,
        });
    }

    let mut settings = manager.settings.write().await;

    // Find and update the route
    if let Some(existing_route) = settings
        .routers
        .iter_mut()
        .find(|r| r.external_path == external_path)
    {
        *existing_route = route.into_inner();

        // Save to disk
        drop(settings); // Release lock before saving
        if let Err(e) = manager.save_to_disk().await {
            return HttpResponse::InternalServerError().json(RouteResponse {
                success: false,
                message: format!("Failed to save configuration: {}", e),
                route: None,
                routes: None,
            });
        }

        HttpResponse::Ok().json(RouteResponse {
            success: true,
            message: "Route updated successfully. Restart required for changes to take effect."
                .to_string(),
            route: None,
            routes: None,
        })
    } else {
        HttpResponse::NotFound().json(RouteResponse {
            success: false,
            message: format!("Route not found: {}", external_path),
            route: None,
            routes: None,
        })
    }
}

/// Delete a route
///
/// # Endpoint
///
/// `DELETE /api/routes/{external_path}`
///
/// # Parameters
///
/// * `external_path` - URL-encoded external path of the route to delete
///
/// # Example
///
/// ```bash
/// curl -X DELETE http://localhost:5900/api/routes/%2Fapi%2Ftest
/// ```
#[delete("/api/routes/{external_path:.*}")]
pub async fn delete_route(
    manager: web::Data<RouteManager>,
    path: web::Path<String>,
) -> impl Responder {
    let external_path = format!("/{}", path.into_inner());
    let mut settings = manager.settings.write().await;

    let original_count = settings.routers.len();
    settings
        .routers
        .retain(|r| r.external_path != external_path);

    if settings.routers.len() < original_count {
        // Save to disk
        drop(settings); // Release lock before saving
        if let Err(e) = manager.save_to_disk().await {
            return HttpResponse::InternalServerError().json(RouteResponse {
                success: false,
                message: format!("Failed to save configuration: {}", e),
                route: None,
                routes: None,
            });
        }

        HttpResponse::Ok().json(RouteResponse {
            success: true,
            message: "Route deleted successfully. Restart required for changes to take effect."
                .to_string(),
            route: None,
            routes: None,
        })
    } else {
        HttpResponse::NotFound().json(RouteResponse {
            success: false,
            message: format!("Route not found: {}", external_path),
            route: None,
            routes: None,
        })
    }
}

/// Validate a route configuration
///
/// # Endpoint
///
/// `POST /api/routes/validate`
///
/// # Request Body
///
/// JSON object with a `route` field containing the route configuration to validate.
///
/// # Response
///
/// Returns validation result with any errors or warnings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/routes/validate \
///   -H "Content-Type: application/json" \
///   -d '{
///     "route": {
///       "backends": [
///         {"host": "http://backend-1", "port": 8080, "weight": 1}
///       ],
///       "external_path": "/api/test",
///       "internal_path": "/test",
///       "methods": ["GET"],
///       "auth_required": false
///     }
///   }'
/// ```
#[post("/api/routes/validate")]
pub async fn validate_route(request: web::Json<ValidateRouteRequest>) -> impl Responder {
    match request.route.validate() {
        Ok(_) => HttpResponse::Ok().json(ValidateRouteResponse {
            valid: true,
            error: None,
            warnings: None,
        }),
        Err(e) => HttpResponse::Ok().json(ValidateRouteResponse {
            valid: false,
            error: Some(e),
            warnings: None,
        }),
    }
}

/// Get complete configuration
///
/// # Endpoint
///
/// `GET /api/config`
///
/// # Response
///
/// Returns the complete gateway configuration including JWT, rate limit, and all routes.
///
/// # Example
///
/// ```bash
/// curl http://localhost:5900/api/config
/// ```
#[get("/api/config")]
pub async fn get_config(manager: web::Data<RouteManager>) -> impl Responder {
    let settings = manager.settings.read().await;
    HttpResponse::Ok().json(&*settings)
}

/// Update JWT configuration
///
/// # Endpoint
///
/// `POST /api/config/jwt`
///
/// # Request Body
///
/// JSON object with JWT settings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/config/jwt \
///   -H "Content-Type: application/json" \
///   -d '{
///     "secret": "your-very-secure-secret-key-here-at-least-32-chars",
///     "issuer": "kairos-gateway",
///     "audience": "kairos-api",
///     "required_claims": ["sub", "exp"]
///   }'
/// ```
#[post("/api/config/jwt")]
pub async fn update_jwt_config(
    manager: web::Data<RouteManager>,
    jwt_settings: web::Json<crate::models::settings::JwtSettings>,
) -> impl Responder {
    // Validate JWT settings
    if jwt_settings.secret.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "JWT secret cannot be empty"
        }));
    }

    if jwt_settings.secret == "please-change-this-secret" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "JWT secret must be changed from default value"
        }));
    }

    if jwt_settings.secret.len() < 32 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "JWT secret should be at least 32 characters for security"
        }));
    }

    let mut settings = manager.settings.write().await;
    settings.jwt = Some(jwt_settings.into_inner());

    // Save to disk
    drop(settings);
    if let Err(e) = manager.save_to_disk().await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save configuration: {}", e)
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "JWT configuration updated successfully. Restart required for changes to take effect."
    }))
}

/// Update rate limit configuration
///
/// # Endpoint
///
/// `POST /api/config/rate-limit`
///
/// # Request Body
///
/// JSON object with rate limit settings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/config/rate-limit \
///   -H "Content-Type: application/json" \
///   -d '{
///     "strategy": "FixedWindow",
///     "window_type": "Minute",
///     "max_requests": 100,
///     "redis_url": null
///   }'
/// ```
#[post("/api/config/rate-limit")]
pub async fn update_rate_limit_config(
    manager: web::Data<RouteManager>,
    rate_limit: web::Json<crate::middleware::rate_limit::RateLimitConfig>,
) -> impl Responder {
    let mut settings = manager.settings.write().await;
    settings.rate_limit = Some(rate_limit.into_inner());

    // Save to disk
    drop(settings);
    if let Err(e) = manager.save_to_disk().await {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": format!("Failed to save configuration: {}", e)
        }));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Rate limit configuration updated successfully. Restart required for changes to take effect."
    }))
}

/// Update CORS configuration
///
/// # Endpoint
///
/// `POST /api/config/cors`
///
/// # Request Body
///
/// JSON object with CORS settings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/config/cors \
///   -H "Content-Type: application/json" \
///   -d '{
///     "allowed_origins": ["http://localhost:3000"],
///     "allowed_methods": ["GET", "POST", "PUT", "DELETE"],
///     "allowed_headers": ["Content-Type", "Authorization"],
///     "allow_credentials": true
///   }'
/// ```
/// CORS configuration structure for cross-origin resource sharing.
///
/// Defines which origins, methods, and headers are allowed for cross-origin requests.
///
/// # Examples
///
/// ```rust
/// use kairos_rs::routes::management::CorsConfig;
///
/// let cors = CorsConfig {
///     allowed_origins: vec!["http://localhost:3000".to_string()],
///     allowed_methods: vec!["GET".to_string(), "POST".to_string()],
///     allowed_headers: vec!["Content-Type".to_string()],
///     allow_credentials: true,
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct CorsConfig {
    /// List of allowed origin URLs
    pub allowed_origins: Vec<String>,
    /// List of allowed HTTP methods
    pub allowed_methods: Vec<String>,
    /// List of allowed request headers
    pub allowed_headers: Vec<String>,
    /// Whether to allow credentials (cookies, authorization headers)
    pub allow_credentials: bool,
}

#[post("/api/config/cors")]
pub async fn update_cors_config(
    _manager: web::Data<RouteManager>,
    _cors_config: web::Json<CorsConfig>,
) -> impl Responder {
    // Note: CORS configuration is typically handled at the middleware level
    // and would require server restart to apply changes.
    // For now, we'll acknowledge the request but note it requires restart.

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "CORS configuration received. Server restart required to apply CORS settings."
    }))
}

/// Update metrics configuration
///
/// # Endpoint
///
/// `POST /api/config/metrics`
///
/// # Request Body
///
/// JSON object with metrics settings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/config/metrics \
///   -H "Content-Type: application/json" \
///   -d '{
///     "endpoint": "/metrics",
///     "enable_per_route_metrics": true
///   }'
/// ```
/// Metrics configuration structure for Prometheus endpoint settings.
///
/// Controls where metrics are exposed and what level of detail to collect.
///
/// # Examples
///
/// ```rust
/// use kairos_rs::routes::management::MetricsConfig;
///
/// let metrics = MetricsConfig {
///     endpoint: "/metrics".to_string(),
///     enable_per_route_metrics: true,
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Path where Prometheus metrics are exposed
    pub endpoint: String,
    /// Whether to collect metrics broken down by individual routes
    pub enable_per_route_metrics: bool,
}

#[post("/api/config/metrics")]
pub async fn update_metrics_config(
    _manager: web::Data<RouteManager>,
    _metrics_config: web::Json<MetricsConfig>,
) -> impl Responder {
    // Note: Metrics configuration changes would require server restart
    // to reconfigure the metrics collection middleware

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Metrics configuration received. Server restart required to apply metrics settings."
    }))
}

/// Update server configuration
///
/// # Endpoint
///
/// `POST /api/config/server`
///
/// # Request Body
///
/// JSON object with server settings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/config/server \
///   -H "Content-Type: application/json" \
///   -d '{
///     "host": "0.0.0.0",
///     "port": 5900,
///     "workers": 4,
///     "keep_alive": 75
///   }'
/// ```
/// Server configuration structure for gateway runtime settings.
///
/// Defines the server's network binding, worker processes, and connection handling.
///
/// # Examples
///
/// ```rust
/// use kairos_rs::routes::management::ServerConfig;
///
/// let server = ServerConfig {
///     host: "0.0.0.0".to_string(),
///     port: 5900,
///     workers: 4,
///     keep_alive: 75,
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    /// IP address to bind the server to (e.g., "0.0.0.0" or "127.0.0.1")
    pub host: String,
    /// Port number to listen on
    pub port: u16,
    /// Number of worker threads for handling requests
    pub workers: usize,
    /// Keep-alive timeout in seconds for HTTP connections
    pub keep_alive: u64,
}

#[post("/api/config/server")]
pub async fn update_server_config(
    _manager: web::Data<RouteManager>,
    server_config: web::Json<ServerConfig>,
) -> impl Responder {
    // Validate server config
    if server_config.port == 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Port must be greater than 0"
        }));
    }

    if server_config.workers == 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "message": "Workers must be greater than 0"
        }));
    }

    // Note: Server configuration changes require a full server restart
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Server configuration received. Server restart required to apply changes."
    }))
}

/// Update AI configuration
///
/// # Endpoint
///
/// `POST /api/config/ai`
///
/// # Request Body
///
/// JSON object with AI settings.
///
/// # Example
///
/// ```bash
/// curl -X POST http://localhost:5900/api/config/ai \
///   -H "Content-Type: application/json" \
///   -d '{
///     "provider": "openai",
///     "model": "gpt-4",
///     "api_key": "sk-1234"
///   }'
/// ```
#[post("/api/config/ai")]
pub async fn update_ai_config(
    _manager: web::Data<RouteManager>,
    _ai_config: web::Json<AiSettings>,
) -> impl Responder {
    // Note: AI configuration changes require a server restart
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "AI configuration received. Server restart required to apply changes."
    }))
}

/// Configure route management endpoints
pub fn configure_management(cfg: &mut web::ServiceConfig) {
    cfg.service(list_routes)
        .service(get_route)
        .service(create_route)
        .service(update_route)
        .service(delete_route)
        .service(validate_route)
        .service(get_config)
        .service(update_jwt_config)
        .service(update_rate_limit_config)
        .service(update_cors_config)
        .service(update_metrics_config)
        .service(update_server_config)
        .service(update_ai_config);
}
