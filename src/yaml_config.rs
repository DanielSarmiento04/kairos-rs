use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    TRACE,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Route {
    pub domain: String,
    pub port: u16,
    pub protocol: String,
    pub external_path: String,
    pub internal_path: String,
    pub methods: Vec<String>,
}

impl Route {
    pub fn validate(&self) -> Result<(), String> {
        if !["http", "https"].contains(&self.protocol.as_str()) {
            return Err(format!("Invalid protocol: {}", self.protocol));
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

#[derive(Debug, Deserialize)]
pub struct Config {
    pub version: String,
    pub routes: Vec<Route>,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        for route in &self.routes {
            route.validate()?;
        }
        Ok(())
    }
}