// yaml configuration

use serde::Deserialize;
use serde_yaml;
use std::fs;


// Import the actix_web crate
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};


#[derive(Debug, Deserialize)]
struct Route {
    domain: String,  // The host
    port: u16,
    protocol: String,
    external_path: String,
    internal_path: String,
    methods: Vec<String>,
}

enum MethodsAvailable {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    CONNECT,
    TRACE,
}

#[derive(Debug, Deserialize)]
struct Config {
    version: String,
    routes: Vec<Route>,
}


// #[actix_web::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    // Read YAML file content
    let yaml_content = fs::read_to_string("config.yml")?;
    
    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content)?;

    println!("Version: {}", config.version);

    let routes = config.routes;
    for route in routes {

        println!("Domain: {}", route.domain);
        println!("Port: {}", route.port);
        println!("Protocol: {}", route.protocol);
        println!("External Path: {}", route.external_path);
        println!("Internal Path: {}", route.internal_path);
        println!("Methods: {:?}", route.methods);

        
        println!();
    }


    Ok(())
}
