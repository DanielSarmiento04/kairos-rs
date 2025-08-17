use serde::{Deserialize, Serialize};
// use crate::models::::Router;

#[derive(Serialize, Deserialize, Debug)]
pub struct Router {
    pub host: String,
    pub port: u16,
    pub external_path: String,
    pub internal_path: String,
    pub methods: Vec<String>,
}

impl Router {
    pub fn validate(&self) -> Result<(), String> {
        if !["http", "https"].contains(&self.host.as_str()) {
            return Err(format!("Invalid host: {}", self.host));
        }

        if !self.external_path.starts_with('/') {
            return Err("External path must start with '/'".to_string());
        }

        if !self.internal_path.starts_with('/') {
            return Err("Internal path must start with '/'".to_string());
        }

        Ok(())
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Settings {
//     pub version: u8,
//     pub routers: Vec<Router>,
// }