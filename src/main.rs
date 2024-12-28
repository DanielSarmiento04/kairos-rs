// yaml configuration
mod yaml_config;
mod redirect_service;

use serde::Deserialize;
use serde_yaml;
use std::fs;
use yaml_config::{Config, Route};
use redirect_service::format_route;


// Import the actix_web crate
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};





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

        let formatted_route = format_route(
            &route.domain,
            route.port,
            &route.protocol,
            &route.internal_path,
        );

        println!("{}", formatted_route);

        
        println!();
    }


    Ok(())
}
