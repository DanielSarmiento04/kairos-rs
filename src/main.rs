mod error_handler;
mod redirect_service;
mod yaml_config;

use actix_web::{
    http::{Method as ActixMethod, StatusCode},
    web, App, Error as ActixError, HttpRequest, HttpResponse, HttpServer,
};
use error_handler::GatewayError;
use log::{error, info};
use redirect_service::format_route;
use reqwest::{header::HeaderMap as ReqwestHeaderMap, header::HeaderName, header::HeaderValue, Client, Method as ReqwestMethod};
use serde_yaml;
use std::{collections::HashMap, fs, sync::Arc};
use tokio::time::{timeout, Duration};
use yaml_config::{Config, Route};

// Route handler structure
#[derive(Clone)]
struct RouteHandler {
    client: Client,
    route_map: Arc<HashMap<String, Route>>,
    timeout_seconds: u64,
}

impl RouteHandler {
    fn new(routes: Vec<Route>, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to create HTTP client");

        let route_map = Arc::new(
            routes
                .into_iter()
                .map(|route| (route.external_path.clone(), route))
                .collect(),
        );

        Self {
            client,
            route_map,
            timeout_seconds,
        }
    }

    async fn handle_request(
        &self,
        req: HttpRequest,
        body: web::Bytes,
    ) -> Result<HttpResponse, ActixError> {
        let path = req.path().to_string();
        let method = req.method().clone();

        // Convert Actix method to Reqwest method
        let reqwest_method = match method {
            ActixMethod::GET => ReqwestMethod::GET,
            ActixMethod::POST => ReqwestMethod::POST,
            ActixMethod::PUT => ReqwestMethod::PUT,
            ActixMethod::DELETE => ReqwestMethod::DELETE,
            ActixMethod::HEAD => ReqwestMethod::HEAD,
            ActixMethod::OPTIONS => ReqwestMethod::OPTIONS,
            ActixMethod::CONNECT => ReqwestMethod::CONNECT,
            ActixMethod::PATCH => ReqwestMethod::PATCH,
            ActixMethod::TRACE => ReqwestMethod::TRACE,
            _ => return Err(GatewayError::Internal("Unsupported HTTP method".to_string()).into()),
        };

        // Convert headers
        let mut reqwest_headers = ReqwestHeaderMap::new();
        for (key, value) in req.headers() {
            if let Ok(header_name) = HeaderName::from_bytes(key.as_ref()) {
                if let Ok(header_value) = HeaderValue::from_bytes(value.as_bytes()) {
                    reqwest_headers.insert(header_name, header_value);
                }
            }
        }

        // Find matching route
        let route = self
            .route_map
            .get(&path)
            .ok_or_else(|| GatewayError::Config(format!("No route found for path: {}", path)))?;

        // Validate method is allowed
        if !route.methods.iter().any(|m| m == method.as_str()) {
            return Ok(HttpResponse::MethodNotAllowed().finish());
        }

        let target_url = format_route(
            &route.domain,
            route.port,
            &route.protocol,
            &route.internal_path,
        );

        // Forward the request with converted method
        let forwarded_req = self
            .client
            .request(reqwest_method, &target_url)
            .headers(reqwest_headers)
            .body(body);

        // Execute request with timeout
        let response = match timeout(
            Duration::from_secs(self.timeout_seconds),
            forwarded_req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(GatewayError::Upstream(e.to_string()).into()),
            Err(_) => return Err(GatewayError::Timeout.into()),
        };

        // Convert upstream response to HttpResponse
        let mut builder =
            HttpResponse::build(StatusCode::from_u16(response.status().as_u16()).unwrap());

        // Forward headers with proper conversion
        for (key, value) in response.headers() {
            if !key.as_str().starts_with("connection") {
                if let Ok(header_value) =
                    actix_web::http::header::HeaderValue::from_bytes(value.as_bytes())
                {
                    builder.insert_header((key.as_str(), header_value));
                }
            }
        }

        // Handle the response body
        match response.bytes().await {
            Ok(bytes) => Ok(builder.body(bytes)),
            Err(e) => Err(GatewayError::Upstream(e.to_string()).into()),
        }
    }
}

fn configure_route(cfg: &mut web::ServiceConfig, handler: RouteHandler) {
    cfg.app_data(web::PayloadConfig::new(1024 * 1024 * 10)) // 10MB payload limit
        .service(
            web::resource("/{tail:.*}").to(move |req: HttpRequest, body: web::Bytes| {
                let handler: RouteHandler = handler.clone();
                async move { handler.handle_request(req, body).await }
            }),
        );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();

    // Read and parse config
    let yaml_content = fs::read_to_string("config.yml").map_err(|e| {
        error!("Failed to read config file: {}", e);
        e
    })?;

    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content).map_err(|e| {
        error!("Failed to parse config: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;

    info!("Version: {}", config.version);

    config.validate().expect("Invalid configuration");

    let route_handler = RouteHandler::new(config.routes, 30); // 30 second timeout

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            .configure(|cfg| configure_route(cfg, route_handler.clone()))
    })
    .bind(("0.0.0.0", 5900))?
    .run()
    .await
}
