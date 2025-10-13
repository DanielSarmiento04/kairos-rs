//! Settings model for UI - WASM-compatible version.

use serde::{Deserialize, Serialize};

/// JWT configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtSettings {
    pub secret: String,
    pub expiration_hours: u64,
    pub issuer: String,
    pub audience: String,
}

/// Gateway configuration settings.
/// 
/// This is a simplified version of the backend Settings model
/// that works in both SSR and WASM contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub server_port: u16,
    pub server_host: String,
    pub log_level: String,
    pub jwt: Option<JwtSettings>,
    pub enable_cors: bool,
    pub enable_metrics: bool,
    pub enable_health_checks: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_port: 5900,
            server_host: "127.0.0.1".to_string(),
            log_level: "info".to_string(),
            jwt: None,
            enable_cors: true,
            enable_metrics: true,
            enable_health_checks: true,
        }
    }
}

impl Settings {
    /// Validate the settings configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.server_host.is_empty() {
            return Err("Server host cannot be empty".to_string());
        }
        
        if self.server_port == 0 {
            return Err("Server port must be greater than 0".to_string());
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!(
                "Invalid log level '{}'. Must be one of: {}",
                self.log_level,
                valid_log_levels.join(", ")
            ));
        }
        
        if let Some(ref jwt) = self.jwt {
            if jwt.secret.is_empty() {
                return Err("JWT secret cannot be empty".to_string());
            }
            if jwt.issuer.is_empty() {
                return Err("JWT issuer cannot be empty".to_string());
            }
            if jwt.audience.is_empty() {
                return Err("JWT audience cannot be empty".to_string());
            }
            if jwt.expiration_hours == 0 {
                return Err("JWT expiration hours must be greater than 0".to_string());
            }
        }
        
        Ok(())
    }
}
