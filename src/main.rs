//! Kairos-rs: High-Performance API Gateway
//! 
//! A modern, high-performance HTTP API gateway built with Rust and Actix Web.
//! Kairos-rs provides intelligent request routing, rate limiting, security headers,
//! and efficient upstream service communication for microservice architectures.
//! 
//! # Features
//! 
//! - **High Performance**: Built with Actix Web for excellent throughput and low latency
//! - **Dynamic Routing**: Support for both static and parameterized route patterns
//! - **Rate Limiting**: Built-in rate limiting with configurable thresholds
//! - **Security**: Comprehensive security headers and HTTPS support
//! - **Observability**: Structured logging and health check endpoints
//! - **Configuration**: Flexible JSON-based configuration with validation
//! 
//! # Architecture
//! 
//! ```text
//! ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
//! │   Client    │───▶│  Kairos-rs   │───▶│  Upstream       │
//! │  Requests   │    │   Gateway    │    │  Services       │
//! └─────────────┘    └──────────────┘    └─────────────────┘
//!                           │
//!                           ▼
//!                    ┌──────────────┐
//!                    │ Configuration│
//!                    │ & Monitoring │
//!                    └──────────────┘
//! ```
//! 
//! # Request Processing Flow
//! 
//! 1. **Incoming Request**: Client sends HTTP request to gateway
//! 2. **Rate Limiting**: Request passes through rate limiting middleware
//! 3. **Route Matching**: Find matching route configuration
//! 4. **Method Validation**: Verify HTTP method is allowed for route
//! 5. **Request Forwarding**: Forward request to upstream service
//! 6. **Response Processing**: Process and return upstream response
//! 
//! # Configuration
//! 
//! The gateway is configured via a JSON file (default: `config.json`):
//! 
//! ```json
//! {
//!   "version": 1,
//!   "routers": [
//!     {
//!       "host": "http://auth-service",
//!       "port": 8080,
//!       "external_path": "/auth/login",
//!       "internal_path": "/authenticate",
//!       "methods": ["POST"]
//!     },
//!     {
//!       "host": "http://user-service", 
//!       "port": 8080,
//!       "external_path": "/users/{id}",
//!       "internal_path": "/api/v1/user/{id}",
//!       "methods": ["GET", "PUT", "DELETE"]
//!     }
//!   ]
//! }
//! ```
//! 
//! # Environment Variables
//! 
//! - `KAIROS_CONFIG_PATH`: Path to configuration file (default: `./config.json`)
//! - `KAIROS_HOST`: Server bind address (default: `0.0.0.0`)
//! - `KAIROS_PORT`: Server port (default: `5900`)
//! - `NO_COLOR`: Disable colored log output
//! 
//! # Health Endpoints
//! 
//! - `GET /health` - General health check
//! - `GET /ready` - Kubernetes readiness probe
//! - `GET /live` - Kubernetes liveness probe
//! 
//! # Performance
//! 
//! - **Static Routes**: O(1) lookup via hash map
//! - **Dynamic Routes**: Compiled regex with parameter extraction
//! - **Connection Pooling**: Automatic HTTP connection reuse
//! - **Memory Efficient**: Zero-copy operations where possible
//! 
//! # Examples
//! 
//! Starting the gateway:
//! ```bash
//! # With default configuration
//! ./kairos-rs
//! 
//! # With custom configuration
//! KAIROS_CONFIG_PATH=/etc/kairos/config.json ./kairos-rs
//! 
//! # Custom host and port
//! KAIROS_HOST=127.0.0.1 KAIROS_PORT=8080 ./kairos-rs
//! ```

mod config;
mod logs;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use crate::config::{settings::load_settings, validation::ConfigValidator};
use crate::logs::logger::configure_logger;
use crate::middleware::security::security_headers;
use crate::middleware::rate_limit::AdvancedRateLimit;
use crate::models::settings::Settings;
use crate::routes::{auth_http, health, metrics};
use crate::services::http::RouteHandler;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::Logger, App, HttpServer};
use log::{debug, error, info, trace, warn};
use tokio::signal;

/// Application entry point for the kairos-rs API gateway.
/// 
/// This async main function initializes and starts the HTTP gateway server with
/// all required middleware, routing, and upstream service configuration. It handles
/// the complete application lifecycle from startup to graceful shutdown.
/// 
/// # Initialization Process
/// 
/// 1. **Logger Setup**: Initialize structured logging with optional color output
/// 2. **Configuration Loading**: Load and validate JSON configuration file
/// 3. **Route Compilation**: Pre-compile all route patterns for optimal performance
/// 4. **Middleware Setup**: Configure rate limiting, security headers, and logging
/// 5. **Server Binding**: Bind HTTP server to configured host and port
/// 6. **Graceful Shutdown**: Handle SIGINT/SIGTERM for clean shutdown
/// 
/// # Server Configuration
/// 
/// The HTTP server is configured with:
/// - **Rate Limiting**: 100 requests/second with 200 request burst capacity
/// - **Security Headers**: Comprehensive security header middleware
/// - **Compression**: Automatic response compression
/// - **Request Logging**: Detailed request/response logging
/// 
/// # Middleware Stack
/// 
/// Request processing flows through these middleware layers:
/// ```text
/// Request → Rate Limiter → Logger → Compression → Security Headers → Routes
/// ```
/// 
/// # Route Configuration
/// 
/// The server configures these route groups:
/// - **Health Routes**: `/health`, `/ready`, `/live` for monitoring
/// - **Proxy Routes**: Dynamic routes based on configuration file
/// 
/// # Error Handling
/// 
/// The application handles various startup errors:
/// - Configuration file loading/parsing errors
/// - Route compilation failures  
/// - Network binding errors
/// - Resource initialization failures
/// 
/// # Graceful Shutdown
/// 
/// The server supports graceful shutdown via:
/// - SIGINT (Ctrl+C) signal handling
/// - Tokio select! for concurrent server and signal monitoring
/// - Proper resource cleanup on shutdown
/// 
/// # Performance Notes
/// 
/// - Uses Actix Web's multi-threaded runtime for high concurrency
/// - Connection pooling for upstream services (30s idle timeout, 32 connections/host)
/// - Pre-compiled route patterns for O(1) static route lookup
/// - Efficient header processing with minimal allocations
/// 
/// # Examples
/// 
/// Successful startup logs:
/// ```text
/// INFO Starting Kairos-rs API Gateway v0.2.1
/// INFO Logger initialized  
/// INFO Starting server on 0.0.0.0:5900
/// INFO Server started successfully
/// ```
/// 
/// # Returns
/// 
/// - `Ok(())` - Server shut down gracefully
/// - `Err(std::io::Error)` - Server binding or startup error
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
