//! Kairos API Gateway Server
//! 
//! High-performance HTTP API gateway built with Rust and Actix Web.
//! 
//! This binary provides the main server entry point for the Kairos gateway,
//! configuring and starting the HTTP server with all required middleware
//! and routing capabilities.

use kairos_rs::config::settings::load_settings;
use kairos_rs::config::validation::ConfigValidator;  
use kairos_rs::logs::logger::configure_logger;
use kairos_rs::middleware::security::security_headers;
use kairos_rs::middleware::rate_limit::AdvancedRateLimit;
use kairos_rs::models::settings::Settings;
use kairos_rs::routes::{auth_http, health, metrics};
use kairos_rs::services::http::RouteHandler;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::Logger, App, HttpServer};
use log::{error, info};
use tokio::signal;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    configure_logger();

    // Parse configuration
    let config: Settings = load_settings().expect("Failed to load settings");

    info!("Starting Kairos-rs API Gateway v{}", config.version);

    // Comprehensive configuration validation
    let validation_result = ConfigValidator::validate_comprehensive(&config);
    if !validation_result.is_valid {
        error!("Configuration validation failed:");
        for error in &validation_result.errors {
            error!("  - {}", error);
        }
        std::process::exit(1);
    }
    info!("Configuration validated successfully with {} warnings", validation_result.warnings.len());

    let route_handler = RouteHandler::new(config.routers.clone(), 30); // 30 second timeout
    
    // Initialize metrics collector
    let metrics_collector = metrics::MetricsCollector::default();

    // Configure basic rate limiting as fallback
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(100) // 100 requests per second
        .burst_size(200) // Allow bursts up to 200 requests
        .finish()
        .unwrap();


    // Get server configuration from environment
    let host = std::env::var("KAIROS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("KAIROS_PORT")
        .unwrap_or_else(|_| "5900".to_string())
        .parse::<u16>()
        .unwrap_or(5900);

    info!("Starting server on {}:{}", host, port);

    // Create server with appropriate rate limiting middleware
    let server = if let Some(rate_limit_config) = config.rate_limit.clone() {
        info!("Using advanced rate limiting with strategy: {:?}", rate_limit_config.strategy);
        let advanced_rate_limit = AdvancedRateLimit::new(rate_limit_config);
        HttpServer::new(move || {
            App::new()
                .app_data(actix_web::web::Data::new(metrics_collector.clone()))
                .wrap(advanced_rate_limit.clone())
                .wrap(Logger::new(
                    r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#
                ))
                .wrap(actix_web::middleware::Compress::default())
                .wrap(security_headers())
                .configure(health::configure_health)
                .configure(metrics::configure_metrics)
                .configure(|cfg| auth_http::configure_auth_routes(cfg, route_handler.clone(), &config))
        })
        .bind((host.as_str(), port))?
        .run()
    } else {
        info!("Using basic rate limiting (100 req/sec, 200 burst)");
        HttpServer::new(move || {
            App::new()
                .app_data(actix_web::web::Data::new(metrics_collector.clone()))
                .wrap(Governor::new(&governor_conf))
                .wrap(Logger::new(
                    r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#
                ))
                .wrap(actix_web::middleware::Compress::default())
                .wrap(security_headers())
                .configure(health::configure_health)
                .configure(metrics::configure_metrics)
                .configure(|cfg| auth_http::configure_auth_routes(cfg, route_handler.clone(), &config))
        })
        .bind((host.as_str(), port))?
        .run()
    };

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