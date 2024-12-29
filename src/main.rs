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

// shared memory
use std::sync::Arc;



async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn internal_function () -> &'static str {
    "Internal function"
}

fn configure_route(cfg: &mut web::ServiceConfig, routes: &[Route])
{
    for route in routes {
        let formatted_route = format_route(
            &route.domain,
            route.port,
            &route.protocol,
            &route.internal_path,
        );

        let methods = route.methods.clone();
        for method in methods {


            match method.as_str() {
                "GET" => {
                    cfg.route(&route.external_path, web::get().to(hello_world));
                },
                "POST" => {
                    cfg.route(&route.external_path, web::post().to(hello_world));
                },
                "PUT" => {
                    cfg.route(&route.external_path, web::put().to(hello_world));
                },
                "DELETE" => {
                    cfg.route(&route.external_path, web::delete().to(hello_world));
                },
                "PATCH" => {
                    cfg.route(&route.external_path, web::patch().to(hello_world));
                },
                "HEAD" => {
                    cfg.route(&route.external_path, web::head().to(hello_world));
                },
                "TRACE" => {
                    cfg.route(&route.external_path, web::trace().to(hello_world));
                },
                _ => {
                    cfg.route(&route.external_path, web::get().to(hello_world));
                }
            }
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{

    // Read YAML file content
    let yaml_content = fs::read_to_string("config.yml").unwrap();
    
    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content).unwrap();

    println!("Version: {}", config.version);

    let routes = config.routes;
    for route in &routes {

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

    let shared_routes = Arc::new(routes);
    
    HttpServer::new(move || {
        App::new()
            .configure(|cfg| {
                configure_route(cfg, &shared_routes)
            })
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await

}
