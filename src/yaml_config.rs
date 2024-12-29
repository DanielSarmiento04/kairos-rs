
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum MethodsAvailable {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    TRACE,
}

#[derive(Debug, Deserialize)]
pub struct Route {
    pub domain: String,  // The host
    pub port: u16,
    pub protocol: String,
    pub external_path: String,
    pub internal_path: String,
    pub methods: Vec<String>,
}



#[derive(Debug, Deserialize)]
pub struct Config {
    pub version: String,
    pub routes: Vec<Route>,
}
