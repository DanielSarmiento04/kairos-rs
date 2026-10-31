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
use kairos_rs::middleware::compression::{self as compression_mw, CompressionConfig};
use kairos_rs::middleware::rate_limit::AdvancedRateLimit;
use kairos_rs::middleware::security::security_headers;
use kairos_rs::models::settings::Settings;
use kairos_rs::routes::{auth_http, config_reload, health, management, mcp, metrics, websocket, websocket_admin};
use kairos_rs::services::http::RouteHandler;
use kairos_rs::services::inflight::InFlightTracker;
use kairos_rs::services::mcp_server::McpServer;
use kairos_rs::services::metrics_store::MetricsStore;
use kairos_rs::services::websocket::WebSocketHandler;

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware::Logger, App, HttpServer};
use chrono::Duration;
use log::{error, info, warn};
use tokio::signal;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    configure_logger();

    // Parse configuration
    let config: Settings = load_settings().expect("Failed to load settings");
    let compression_cfg: CompressionConfig = config.compression.clone().unwrap_or_default();
    if let Err(e) = compression_cfg.validate() {
        error!("Invalid compression config in settings: {}", e);
        std::process::exit(1);
    }
    if compression_cfg.enabled {
        info!(
            "Response compression enabled (algorithms={:?}, level={}, min_size={} bytes)",
            compression_cfg.algorithms, compression_cfg.level, compression_cfg.min_size
        );
    } else {
        info!("Response compression disabled by config");
    }

    info!("Starting Kairos-rs API Gateway v{}", env!("CARGO_PKG_VERSION"));

    // Comprehensive configuration validation
    let validation_result = ConfigValidator::validate_comprehensive(&config);
    if !validation_result.is_valid {
        error!("Configuration validation failed:");
        for error in &validation_result.errors {
            error!("  - {}", error);
        }
        std::process::exit(1);
    }
    info!(
        "Configuration validated successfully with {} warnings",
        validation_result.warnings.len()
    );

    let mut route_handler = RouteHandler::new(config.routers.clone(), 30); // 30 second timeout

    // Initialize distributed Redis backend if configured
    let redis_service: Option<std::sync::Arc<kairos_rs::services::redis_service::RedisService>> =
        if let Some(redis_cfg) = config.redis.clone() {
            if redis_cfg.enabled {
                match kairos_rs::services::redis_service::RedisService::new(redis_cfg).await {
                    Ok(svc) => {
                        info!("Distributed Redis backend initialized");
                        Some(std::sync::Arc::new(svc))
                    }
                    Err(e) => {
                        warn!("Failed to initialize Redis backend ({}): falling back to in-memory mode", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

    // Initialize in-memory / distributed response cache (5,000 max entries, 60s default TTL)
    let mut response_cache_builder = kairos_rs::services::cache::ResponseCache::new(
        5_000,
        std::time::Duration::from_secs(60),
    );
    if let Some(ref redis_svc) = redis_service {
        response_cache_builder = response_cache_builder.with_redis(redis_svc.clone());
    }
    let response_cache = std::sync::Arc::new(response_cache_builder);
    let inflight_tracker = std::sync::Arc::new(InFlightTracker::new());

    // Initialize Semantic Cache for AI/LLM requests
    let semantic_cache_cfg = config
        .ai
        .as_ref()
        .and_then(|a| a.semantic_cache.clone())
        .unwrap_or_default();
    let semantic_cache = std::sync::Arc::new(
        kairos_rs::services::semantic_cache::SemanticCache::new(semantic_cache_cfg),
    );

    route_handler = route_handler
        .with_cache(response_cache.clone())
        .with_semantic_cache(semantic_cache.clone())
        .with_inflight_tracker(inflight_tracker);

    // Initialize AI Service if configured
    if let Some(ai_settings) = config.ai.clone() {
        use kairos_rs::services::ai::AiService;
        let ai_service = AiService::new(ai_settings);
        route_handler = route_handler.with_ai_service(ai_service);
        info!("AI Service initialized successfully");
    }

    // Initialize metrics collector
    let metrics_collector = metrics::MetricsCollector::default();

    // Initialize historical metrics store (10,000 points max, 24 hour retention)
    let metrics_store = MetricsStore::new(10_000, Duration::hours(24));

    // Initialize WebSocket handler
    let websocket_handler = WebSocketHandler::new(30);

    // Spawn background task for historical metrics collection
    let history_collector = metrics_collector.clone();
    let history_store = metrics_store.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;

            // Collect metrics
            let requests = history_collector
                .requests_total
                .load(std::sync::atomic::Ordering::Relaxed);
            let errors = history_collector
                .requests_error
                .load(std::sync::atomic::Ordering::Relaxed);
            let active = history_collector
                .active_connections
                .load(std::sync::atomic::Ordering::Relaxed);

            // Calculate average response time
            let total_req = history_collector
                .requests_total
                .load(std::sync::atomic::Ordering::Relaxed);
            let total_time = history_collector
                .response_time_sum
                .load(std::sync::atomic::Ordering::Relaxed);
            let avg_latency = if total_req > 0 {
                total_time as f64 / total_req as f64
            } else {
                0.0
            };

            // Record to store
            use kairos_rs::services::metrics_store::MetricValue;
            history_store.record("requests_total", MetricValue::Counter(requests));
            history_store.record("requests_error", MetricValue::Counter(errors));
            history_store.record("active_connections", MetricValue::Gauge(active as f64));
            history_store.record("response_time_avg", MetricValue::Gauge(avg_latency));
        }
    });

    // Get config path for route management
    let config_path =
        std::env::var("KAIROS_CONFIG_PATH").unwrap_or_else(|_| "config.json".to_string());

    // Initialize route manager for dynamic configuration
    let config_path_clone = config_path.clone();
    let route_manager = management::RouteManager::new(config.clone(), config_path);

    // Initialize ConfigManager for hot reload
    use kairos_rs::config::hot_reload::ConfigManager;
    let config_manager = std::sync::Arc::new(ConfigManager::new(config.clone(), config_path_clone));

    // Spawn watcher
    let config_manager_watcher = config_manager.clone();
    tokio::spawn(async move {
        config_manager_watcher.start().await;
    });

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

    let admin_auth_cfg = config.admin_auth.clone().unwrap_or_default();
    let admin_auth = kairos_rs::middleware::admin_auth::AdminAuth::new(admin_auth_cfg);

    // Initialize Model Context Protocol (MCP) server
    let mcp_config = config.mcp.clone().unwrap_or_default();
    let mcp_server = McpServer::new(mcp_config)
        .with_route_manager(std::sync::Arc::new(route_manager.clone()))
        .with_route_handler(std::sync::Arc::new(route_handler.clone()))
        .with_metrics(actix_web::web::Data::new(metrics_collector.clone()))
        .with_response_cache(response_cache.clone())
        .with_semantic_cache(semantic_cache.clone());
    info!(
        "MCP Server initialized successfully (enabled={})",
        mcp_server.config().enabled
    );

    // Create server with appropriate rate limiting middleware
    let server = if let Some(rate_limit_config) = config.rate_limit.clone() {
        info!(
            "Using advanced rate limiting with strategy: {:?}",
            rate_limit_config.strategy
        );
        let mut advanced_rate_limit = AdvancedRateLimit::new(rate_limit_config);
        if let Some(ref redis_svc) = redis_service {
            advanced_rate_limit = advanced_rate_limit.with_redis(redis_svc.clone());
        }
        let redis_data = redis_service.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(actix_web::web::Data::new(metrics_collector.clone()))
                .app_data(actix_web::web::Data::new(metrics_store.clone()))
                .app_data(actix_web::web::Data::new(route_manager.clone()))
                .app_data(actix_web::web::Data::new(route_handler.clone()))
                .app_data(actix_web::web::Data::new(config_manager.clone()))
                .app_data(actix_web::web::Data::new(response_cache.clone()))
                .app_data(actix_web::web::Data::new(semantic_cache.clone()))
                .app_data(actix_web::web::Data::new(redis_service.clone()))
                .app_data(actix_web::web::Data::new(redis_data.clone()))
                .app_data(actix_web::web::Data::new(mcp_server.clone()))
                .wrap(admin_auth.clone())
                .wrap(advanced_rate_limit.clone())
                .wrap(Logger::new(
                    r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
                ))
                .wrap(security_headers())
                .wrap(
                    compression_mw::build(&compression_cfg)
                        .map(|(c, _)| c)
                        .unwrap_or_else(actix_web::middleware::Compress::default),
                )
                .wrap(
                    compression_mw::build(&compression_cfg)
                        .map(|(_, h)| h)
                        .unwrap_or_else(actix_web::middleware::DefaultHeaders::new),
                )
                .configure(health::configure_health)
                .configure(metrics::configure_metrics)
                .configure(websocket_admin::configure_admin_websocket)
                .configure(management::configure_management)
                .configure(mcp::configure_mcp)
                .configure(config_reload::configure_config_reload)
                .configure(|cfg| websocket::configure_websocket(cfg, websocket_handler.clone()))
                .configure(|cfg| {
                    auth_http::configure_auth_routes(cfg, route_handler.clone(), &config)
                })
        })
        .bind((host.as_str(), port))?
        .run()
    } else {
        info!("Using basic rate limiting (100 req/sec, 200 burst)");
        HttpServer::new(move || {
            App::new()
                .app_data(actix_web::web::Data::new(metrics_collector.clone()))
                .app_data(actix_web::web::Data::new(metrics_store.clone()))
                .app_data(actix_web::web::Data::new(route_manager.clone()))
                .app_data(actix_web::web::Data::new(route_handler.clone()))
                .app_data(actix_web::web::Data::new(config_manager.clone()))
                .app_data(actix_web::web::Data::new(response_cache.clone()))
                .app_data(actix_web::web::Data::new(semantic_cache.clone()))
                .app_data(actix_web::web::Data::new(redis_service.clone()))
                .app_data(actix_web::web::Data::new(mcp_server.clone()))
                .wrap(admin_auth.clone())
                .wrap(Governor::new(&governor_conf))
                .wrap(Logger::new(
                    r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
                ))
                .wrap(security_headers())
                .wrap(
                    compression_mw::build(&compression_cfg)
                        .map(|(c, _)| c)
                        .unwrap_or_else(actix_web::middleware::Compress::default),
                )
                .wrap(
                    compression_mw::build(&compression_cfg)
                        .map(|(_, h)| h)
                        .unwrap_or_else(actix_web::middleware::DefaultHeaders::new),
                )
                .configure(health::configure_health)
                .configure(metrics::configure_metrics)
                .configure(websocket_admin::configure_admin_websocket)
                .configure(management::configure_management)
                .configure(mcp::configure_mcp)
                .configure(config_reload::configure_config_reload)
                .configure(|cfg| websocket::configure_websocket(cfg, websocket_handler.clone()))
                .configure(|cfg| {
                    auth_http::configure_auth_routes(cfg, route_handler.clone(), &config)
                })
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
