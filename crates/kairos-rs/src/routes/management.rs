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
use crate::models::settings::Settings;

/// Shared state for route management
#[derive(Clone)]
pub struct RouteManager {
    settings: Arc<RwLock<Settings>>,
    config_path: String,
}

impl RouteManager {
    pub fn new(settings: Settings, config_path: String) -> Self {
        Self {
            settings: Arc::new(RwLock::new(settings)),
            config_path,
        }
    }
    
    /// Saves current settings to disk
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

/// Response structure for route operations
#[derive(Serialize, Deserialize)]
pub struct RouteResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Router>,
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
    
    if let Some(route) = settings.routers.iter().find(|r| r.external_path == external_path) {
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
    if settings.routers.iter().any(|r| r.external_path == route.external_path) {
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
        message: "Route created successfully. Restart required for changes to take effect.".to_string(),
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
    if let Some(existing_route) = settings.routers.iter_mut().find(|r| r.external_path == external_path) {
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
            message: "Route updated successfully. Restart required for changes to take effect.".to_string(),
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
    settings.routers.retain(|r| r.external_path != external_path);
    
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
            message: "Route deleted successfully. Restart required for changes to take effect.".to_string(),
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

/// Configure route management endpoints
pub fn configure_management(cfg: &mut web::ServiceConfig) {
    cfg.service(list_routes)
        .service(get_route)
        .service(create_route)
        .service(update_route)
        .service(delete_route)
        .service(validate_route);
}
