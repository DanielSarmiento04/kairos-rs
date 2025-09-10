mod config;
mod models;
mod logs;
mod routes;
mod services;
mod utils;

use crate::config::settings::load_settings;
use crate::models::settings::{Settings};
use crate::logs::logger::configure_logger;
use crate::services::http::{RouteHandler};
use crate::routes::http;

// use env_logger;
use log::{error, info, warn, trace, debug};
use actix_web::{
    App, HttpServer,
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    configure_logger();

    // Parse YAML into the Config struct
    let config: Settings = load_settings().expect("Failed to load settings");

    info!("Version: {}", config.version);

    config.validate().expect("Invalid configuration");

    let route_handler = RouteHandler::new(config.routers, 30); // 30 second timeout

    info!("Logger initialized");
    warn!("This is a warning message");
    error!("This is an error message");
    debug!("This is a debug message");
    trace!("This is a trace message");
    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            .configure(|cfg| http::configure_route(cfg, route_handler.clone()))
    })
    .bind(("0.0.0.0", 5900))?
    .run()
    .await
}