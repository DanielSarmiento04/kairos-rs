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

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("200")
}


#[actix_web::main]
async fn main() -> std::io::Result<()>{

    // Read YAML file content
    let yaml_content = fs::read_to_string("config.yml")?;
    
    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content).unwrap();

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
        println!("Route formate {}", formatted_route);
        
        println!();
    }

    HttpServer::new(|| {
        App::new().service(
            web::scope("/").service(hello)
        )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await


}
