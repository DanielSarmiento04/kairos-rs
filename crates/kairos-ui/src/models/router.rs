//! Router model for UI - WASM-compatible version.

use serde::{Deserialize, Serialize};

/// Router authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterAuth {
    pub required: bool,
}

/// Route configuration model.
/// 
/// This is a simplified version of the backend Router model
/// that works in both SSR and WASM contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Router {
    pub host: String,
    pub port: u16,
    pub external_path: String,
    pub internal_path: String,
    pub methods: Vec<String>,
    pub auth: RouterAuth,
}

impl Router {
    /// Validate the router configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }
        
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }
        
        if self.external_path.is_empty() {
            return Err("External path cannot be empty".to_string());
        }
        
        if self.internal_path.is_empty() {
            return Err("Internal path cannot be empty".to_string());
        }
        
        if self.methods.is_empty() {
            return Err("At least one HTTP method must be specified".to_string());
        }
        
        Ok(())
    }
}

impl Router {
    /// Create a new router configuration.
    pub fn new(
        host: String,
        port: u16,
        external_path: String,
        internal_path: String,
        methods: Vec<String>,
        auth_required: bool,
    ) -> Self {
        Self {
            host,
            port,
            external_path,
            internal_path,
            methods,
            auth: RouterAuth {
                required: auth_required,
            },
        }
    }
}
