use crate::models::router::Router;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub version: u8,
    pub routers: Vec<Router>,
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        for route in &self.routers {
            route.validate()?;
        }
        Ok(())
    }
}