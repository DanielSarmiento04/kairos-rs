use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Router {
    pub host: String,
    pub port: u16,
    pub external_path: String,
    pub internal_path: String,
    pub methods: Vec<String>,
}

impl Router {
    pub fn validate(&self) -> Result<(), String> {
        log::debug!(
            "Validating Router: host={}, port={}, external_path={}, internal_path={}, methods={:?}",
            self.host, self.port, self.external_path, self.internal_path, self.methods
        );

        // Validate host format
        if !self.host.starts_with("http://") && !self.host.starts_with("https://") {
            return Err("Host must start with http:// or https://".to_string());
        }

        // Validate port range (u16 max is 65535, so only check lower bound)
        if self.port == 0 {
            return Err("Port must be between 1 and 65535".to_string());
        }

        // Validate paths start with '/'
        if !self.external_path.starts_with('/') {
            return Err("External path must start with '/'".to_string());
        }

        if !self.internal_path.starts_with('/') {
            return Err("Internal path must start with '/'".to_string());
        }

        // Validate HTTP methods
        if self.methods.is_empty() {
            return Err("At least one HTTP method must be specified".to_string());
        }

        let valid_methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE"];
        for method in &self.methods {
            if !valid_methods.contains(&method.as_str()) {
                return Err(format!("Invalid HTTP method: {}", method));
            }
        }

        Ok(())
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Settings {
//     pub version: u8,
//     pub routers: Vec<Router>,
// }
