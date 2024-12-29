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

async fn internal_function (internal_url: String) -> &'static str {
    print!("Internal URL: {}", internal_url);
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

        for method in &route.methods {

            println!("Method: {} formate_url {} ", method, formatted_route);

            // handle the method use internal function and pass formatted_route
            match method.as_str() {
                "GET" => {
                    cfg.app_data(web::Data::new(formatted_route.clone()))
                        .service(
                            web::resource(&route.external_path)
                                .route(web::get().to(internal_function))
                        );
                }
                "POST" => {
                    cfg.app_data(web::Data::new(formatted_route.clone()))
                        .service(
                            web::resource(&route.external_path)
                                .route(web::post().to(internal_function))
                        );
                }
                "PUT" => {
                    cfg.service(
                        web::resource(&route.external_path)
                            .route(web::put().to(hello_world))
                    );
                }
                "DELETE" => {
                    cfg.service(
                        web::resource(&route.external_path)
                            .route(web::delete().to(hello_world))
                    );
                }
                "PATCH" => {
                    cfg.service(
                        web::resource(&route.external_path)
                            .route(web::patch().to(hello_world))
                    );
                }
                "HEAD" => {
                    cfg.service(
                        web::resource(&route.external_path)
                            .route(web::head().to(hello_world))
                    );
                }
                "TRACE" => {
                    cfg.service(
                        web::resource(&route.external_path)
                            .route(web::trace().to(hello_world))
                    );
                }
                _ => {
                    cfg.service(
                        web::resource(&route.external_path)
                            .route(web::get().to(hello_world))
                    );
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
