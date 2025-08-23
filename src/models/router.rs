use crate::models::protocol::Protocol;
use serde::{Deserialize, Serialize};

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
        
        // print host, port, external_path, internal_path, methods
        println!("Validating Router: host={}, port={}, external_path={}, internal_path={}, methods={:?}",
            self.host, self.port, self.external_path, self.internal_path, self.methods);  

        // if let Some

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