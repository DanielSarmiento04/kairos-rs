//! Admin authentication middleware for Kairos Gateway control plane.
//!
//! Protects administrative endpoints (`/api/routes`, `/api/config`, `/api/cache`,
//! `/api/metrics`, `/api/admin`, `/ws/admin/*`) with API key or Bearer token validation.

use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Default header name for admin API keys.
pub const DEFAULT_ADMIN_HEADER: &str = "X-Admin-Key";

/// Configuration for Admin Control Plane Authentication.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AdminAuthConfig {
    /// Whether admin authentication is required. Default: false.
    #[serde(default)]
    pub enabled: bool,

    /// List of valid admin API keys.
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Header name to check for the admin API key. Default: "X-Admin-Key".
    #[serde(default = "default_admin_header")]
    pub header_name: String,
}

fn default_admin_header() -> String {
    DEFAULT_ADMIN_HEADER.to_string()
}

impl Default for AdminAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_keys: Vec::new(),
            header_name: default_admin_header(),
        }
    }
}

impl AdminAuthConfig {
    /// Creates a new AdminAuthConfig with specified keys and enabled status.
    pub fn new(enabled: bool, api_keys: Vec<String>) -> Self {
        Self {
            enabled,
            api_keys,
            header_name: default_admin_header(),
        }
    }

    /// Resolves whether admin authentication is active.
    ///
    /// Returns true if `enabled` is true, or if `KAIROS_ADMIN_KEY` environment
    /// variable is set and non-empty.
    pub fn is_active(&self) -> bool {
        if self.enabled {
            return true;
        }
        if let Ok(env_key) = std::env::var("KAIROS_ADMIN_KEY") {
            return !env_key.trim().is_empty();
        }
        false
    }

    /// Validates whether a provided key matches any configured key or the environment variable.
    pub fn is_valid_key(&self, key: &str) -> bool {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Check environment variable
        if let Ok(env_key) = std::env::var("KAIROS_ADMIN_KEY") {
            if !env_key.trim().is_empty() && env_key.trim() == trimmed {
                return true;
            }
        }

        // Check configured API keys
        self.api_keys.iter().any(|k| k.trim() == trimmed)
    }
}

/// Actix-Web middleware transform for Admin Control Plane Authentication.
#[derive(Clone)]
pub struct AdminAuth {
    config: Arc<AdminAuthConfig>,
}

impl AdminAuth {
    /// Creates a new AdminAuth middleware transform.
    pub fn new(config: AdminAuthConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Creates an AdminAuth middleware wrapped in an existing Arc.
    pub fn from_arc(config: Arc<AdminAuthConfig>) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AdminAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminAuthMiddleware {
            service: Arc::new(service),
            config: self.config.clone(),
        })
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Arc<S>,
    config: Arc<AdminAuthConfig>,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // 1. If admin auth is not active, pass through
            if !config.is_active() {
                let res = service.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            // 2. Allow CORS preflight requests (OPTIONS)
            if req.method() == actix_web::http::Method::OPTIONS {
                let res = service.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            // 3. Only protect admin / control plane endpoints
            let path = req.path();
            let is_admin_path = path.starts_with("/api/routes")
                || path.starts_with("/api/config")
                || path.starts_with("/api/cache")
                || path.starts_with("/api/metrics")
                || path.starts_with("/api/admin")
                || path.starts_with("/ws/admin");

            if !is_admin_path {
                let res = service.call(req).await?;
                return Ok(res.map_into_boxed_body());
            }

            // 4. Extract token from header or query string
            let mut extracted_key: Option<String> = None;

            // Check configured header (e.g., X-Admin-Key)
            if let Some(val) = req.headers().get(&config.header_name) {
                if let Ok(str_val) = val.to_str() {
                    extracted_key = Some(str_val.to_string());
                }
            }

            // Check Authorization header: Bearer <key>
            if extracted_key.is_none() {
                if let Some(auth_val) = req.headers().get(actix_web::http::header::AUTHORIZATION) {
                    if let Ok(auth_str) = auth_val.to_str() {
                        if let Some(token) = auth_str.strip_prefix("Bearer ") {
                            extracted_key = Some(token.trim().to_string());
                        } else if let Some(token) = auth_str.strip_prefix("bearer ") {
                            extracted_key = Some(token.trim().to_string());
                        } else {
                            extracted_key = Some(auth_str.trim().to_string());
                        }
                    }
                }
            }

            // Check query parameter (e.g. for WebSockets: ?admin_key=... or ?key=...)
            if extracted_key.is_none() {
                let query = req.query_string();
                if !query.is_empty() {
                    for pair in query.split('&') {
                        let mut parts = pair.splitn(2, '=');
                        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                            if k == "admin_key" || k == "api_key" || k == "key" {
                                extracted_key = Some(v.to_string());
                                break;
                            }
                        }
                    }
                }
            }

            // 5. Validate the extracted key
            match extracted_key {
                Some(key) if config.is_valid_key(&key) => {
                    debug!("Admin authentication successful for path: {}", path);
                    let res = service.call(req).await?;
                    Ok(res.map_into_boxed_body())
                }
                Some(_) => {
                    warn!("Admin authentication failed: invalid key for path: {}", path);
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "success": false,
                                "error": "Unauthorized: invalid admin key",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }))
                            .map_into_boxed_body(),
                    ))
                }
                None => {
                    warn!("Admin authentication failed: missing key for path: {}", path);
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "success": false,
                                "error": "Unauthorized: missing admin key. Provide header 'X-Admin-Key', 'Authorization: Bearer <key>', or query '?admin_key=<key>'",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }))
                            .map_into_boxed_body(),
                    ))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_auth_config_default() {
        let config = AdminAuthConfig::default();
        assert!(!config.enabled);
        assert!(!config.is_active());
        assert_eq!(config.header_name, DEFAULT_ADMIN_HEADER);
    }

    #[test]
    fn test_admin_auth_validation() {
        let config = AdminAuthConfig::new(true, vec!["secret123".to_string(), "master_key".to_string()]);
        assert!(config.is_active());
        assert!(config.is_valid_key("secret123"));
        assert!(config.is_valid_key("master_key"));
        assert!(!config.is_valid_key("wrong_key"));
        assert!(!config.is_valid_key(""));
    }
}
