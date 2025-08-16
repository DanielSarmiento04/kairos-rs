use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Router {
    host: String,
    port: u16,
    external_path: String,
    internal_path: String,
    methods: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    version: u8,
    routers: Vec<Router>,
}