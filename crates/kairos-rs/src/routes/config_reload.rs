//! Configuration reload API endpoint.
//! 
//! This module provides an HTTP endpoint to trigger manual configuration reload
//! without restarting the gateway service.

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::hot_reload::ConfigManager;

/// Response structure for reload operation
#[derive(Serialize, Deserialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
    pub version: Option<u64>,
    pub timestamp: Option<String>,
}

/// Trigger manual configuration reload
/// 
/// # Endpoint
/// 
/// `POST /api/config/reload`
/// 
/// # Response
/// 
/// Returns success status, version number, and timestamp of the reload.
/// 
/// # Example
/// 
/// ```bash
/// curl -X POST http://localhost:5900/api/config/reload
/// ```
/// 
/// # Errors
/// 
/// Returns error response if:
/// - Configuration file cannot be read
/// - Configuration validation fails
/// - File system errors occur
#[post("/api/config/reload")]
pub async fn reload_config(manager: web::Data<Arc<ConfigManager>>) -> impl Responder {
    match manager.reload_now().await {
        Ok(update) => HttpResponse::Ok().json(ReloadResponse {
            success: true,
            message: "Configuration reloaded successfully".to_string(),
            version: Some(update.version),
            timestamp: Some(update.timestamp.to_rfc3339()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ReloadResponse {
            success: false,
            message: format!("Failed to reload configuration: {}", e),
            version: None,
            timestamp: None,
        }),
    }
}

/// Get current configuration version and status
/// 
/// # Endpoint
/// 
/// `GET /api/config/status`
/// 
/// # Response
/// 
/// Returns current configuration version and last update timestamp.
/// 
/// # Example
/// 
/// ```bash
/// curl http://localhost:5900/api/config/status
/// ```
#[actix_web::get("/api/config/status")]
pub async fn config_status(manager: web::Data<Arc<ConfigManager>>) -> impl Responder {
    let current_config = manager.get_current_config().await;
    
    HttpResponse::Ok().json(ReloadResponse {
        success: true,
        message: "Current configuration status".to_string(),
        version: Some(current_config.version),
        timestamp: Some(current_config.timestamp.to_rfc3339()),
    })
}

/// Configure config reload endpoints
pub fn configure_config_reload(cfg: &mut web::ServiceConfig) {
    cfg.service(reload_config)
        .service(config_status);
}
