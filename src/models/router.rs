use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Router {
    pub host: String,
    pub port: u16,
    pub external_path: String,
    pub internal_path: String,
    pub methods: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub version: u8,
    pub routers: Vec<Router>,
}