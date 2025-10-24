//! DNS proxy route handling for the kairos-rs gateway.
//!
//! This module provides HTTP endpoints that proxy DNS queries to
//! backend DNS servers, with caching support for improved performance.

use crate::models::router::Backend;
use crate::services::dns::DnsHandler;
use actix_web::{web, HttpResponse, Error};
use log::info;
use serde::{Deserialize, Serialize};

/// Request body for DNS query operations.
#[derive(Deserialize)]
pub struct DnsQueryRequest {
    /// DNS query as hex-encoded bytes
    pub query: String,
}

/// Response for DNS query operations.
#[derive(Serialize)]
pub struct DnsQueryResponse {
    /// DNS response as hex-encoded bytes
    pub response: String,
    /// Whether the response was served from cache
    pub cached: bool,
}

/// Cache statistics response.
#[derive(Serialize)]
pub struct DnsCacheStats {
    /// Number of entries in cache
    pub size: usize,
}

/// Handles DNS query forwarding requests.
///
/// Accepts DNS queries as hex-encoded bytes and forwards them to
/// the configured DNS backend server.
///
/// # Request
///
/// ```json
/// POST /dns/query
/// {
///   "query": "00010100000100000000000003777777076578616d706c6503636f6d0000010001"
/// }
/// ```
///
/// # Response
///
/// ```json
/// {
///   "response": "00018180000100010000000003777777076578616d706c6503636f6d0000010001...",
///   "cached": false
/// }
/// ```
pub async fn handle_dns_query(
    body: web::Json<DnsQueryRequest>,
    handler: web::Data<DnsHandler>,
    backend: web::Data<Backend>,
) -> Result<HttpResponse, Error> {
    info!("DNS query request received");

    // Decode hex query
    let query_bytes = hex::decode(&body.query)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid hex query: {}", e)))?;

    // Forward query
    let response_bytes = handler
        .forward_query(&query_bytes, &backend)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Encode response as hex
    let response_hex = hex::encode(&response_bytes);

    Ok(HttpResponse::Ok().json(DnsQueryResponse {
        response: response_hex,
        cached: false, // TODO: Track if response was from cache
    }))
}

/// Handles cache cleanup requests.
///
/// Clears expired entries from the DNS cache to free up memory.
///
/// # Request
///
/// ```text
/// POST /dns/cache/cleanup
/// ```
///
/// # Response
///
/// ```json
/// {
///   "success": true,
///   "message": "Cache cleanup completed"
/// }
/// ```
pub async fn handle_dns_cache_cleanup(
    handler: web::Data<DnsHandler>,
) -> Result<HttpResponse, Error> {
    info!("DNS cache cleanup requested");

    // Trigger cache cleanup
    handler.clear_expired().await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Cache cleanup completed"
    })))
}

/// Handles cache statistics requests.
///
/// Returns information about the current DNS cache state.
///
/// # Request
///
/// ```text
/// GET /dns/cache/stats
/// ```
///
/// # Response
///
/// ```json
/// {
///   "size": 42
/// }
/// ```
pub async fn handle_dns_cache_stats(
    handler: web::Data<DnsHandler>,
) -> Result<HttpResponse, Error> {
    let size = handler.cache_size().await;

    Ok(HttpResponse::Ok().json(DnsCacheStats { size }))
}

/// Configures DNS proxy routes for the application.
///
/// Sets up HTTP endpoints that proxy DNS operations:
/// - POST /dns/query - Forward DNS query
/// - POST /dns/cache/cleanup - Cleanup expired cache entries
/// - GET /dns/cache/stats - Get cache statistics
///
/// # Parameters
///
/// * `cfg` - Actix Web service configuration
///
/// # Examples
///
/// ```rust
/// use actix_web::{App, web};
/// use kairos_rs::routes::dns::configure_dns;
/// use kairos_rs::services::dns::DnsHandler;
/// use kairos_rs::models::router::Backend;
///
/// let handler = DnsHandler::new(5);
/// let backend = Backend {
///     host: "8.8.8.8".to_string(),
///     port: 53,
///     weight: 1,
///     health_check_path: None,
/// };
///
/// let app = App::new()
///     .app_data(web::Data::new(handler))
///     .app_data(web::Data::new(backend))
///     .configure(configure_dns);
/// ```
pub fn configure_dns(cfg: &mut web::ServiceConfig) {
    info!("Configuring DNS proxy routes");
    
    cfg.service(
        web::scope("/dns")
            .route("/query", web::post().to(handle_dns_query))
            .service(
                web::scope("/cache")
                    .route("/cleanup", web::post().to(handle_dns_cache_cleanup))
                    .route("/stats", web::get().to(handle_dns_cache_stats))
            )
    );
}
