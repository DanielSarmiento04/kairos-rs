mod yaml_config;
mod redirect_service;
mod error_handler;

use actix_web::{
    web, App, HttpRequest, HttpResponse, HttpServer,
    http::{header, StatusCode}, 
    Error as ActixError,
};
use serde_yaml;
use std::fs;
use std::{sync::Arc, collections::HashMap};
use yaml_config::{Config, Route};
use redirect_service::format_route;
use log::{info, error};  // Add logging capabilities
use futures::future::try_join_all;
use tokio::time::{timeout, Duration};
use reqwest::Client;
use error_handler::GatewayError;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

// Route handler structure
#[derive(Clone)]
struct RouteHandler {
    client: Client,
    routes: Arc<Vec<Route>>,
    timeout_seconds: u64,
}

impl RouteHandler {
    fn new(routes: Arc<Vec<Route>>, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            routes,
            timeout_seconds,
        }
    }

    
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();

    // Read and parse config
    let yaml_content = fs::read_to_string("config.yml")
        .map_err(|e| {
            error!("Failed to read config file: {}", e);
            e
        })?;

    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content)
        .map_err(|e| {
            error!("Failed to parse config: {}", e);
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;

    info!("Version: {}", config.version);

    let routes = Arc::new(config.routes);
    let route_handler = RouteHandler::new(routes, 30); // 30 second timeout

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())

    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
