mod config;
mod logs;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use crate::config::settings::load_settings;
use crate::logs::logger::configure_logger;
use crate::middleware::security::security_headers;
use crate::models::settings::Settings;
use crate::routes::{health, http};
use crate::services::http::RouteHandler;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::Logger, App, HttpServer};
use log::{debug, error, info, trace, warn};
use tokio::signal;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    configure_logger();

    // Parse configuration
    let config: Settings = load_settings().expect("Failed to load settings");

    info!("Starting Kairos-rs API Gateway v{}", config.version);

    config.validate().expect("Invalid configuration");

    let route_handler = RouteHandler::new(config.routers, 30); // 30 second timeout

    // Configure rate limiting
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(100) // 100 requests per second
        .burst_size(200) // Allow bursts up to 200 requests
        .finish()
        .unwrap();

    info!("Logger initialized");
    warn!("This is a warning message");
    error!("This is an error message");
    debug!("This is a debug message");
    trace!("This is a trace message");

    // Get server configuration from environment
    let host = std::env::var("KAIROS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("KAIROS_PORT")
        .unwrap_or_else(|_| "5900".to_string())
        .parse::<u16>()
        .unwrap_or(5900);

    info!("Starting server on {}:{}", host, port);

    // Start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Governor::new(&governor_conf))
            .wrap(Logger::new(
                r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#
            ))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(security_headers())
            .configure(health::configure_health)
            .configure(|cfg| http::configure_route(cfg, route_handler.clone()))
    })
    .bind((host.as_str(), port))?
    .run();

    info!("Server started successfully");

    // Graceful shutdown handling
    tokio::select! {
        result = server => {
            match result {
                Ok(_) => info!("Server stopped gracefully"),
                Err(e) => error!("Server error: {}", e),
            }
        }
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, stopping server...");
        }
    }

    Ok(())
}
