use crate::services::http::RouteHandler;
use actix_web::{web, HttpRequest};

/// Configures the main HTTP proxy route for the kairos-rs gateway.
/// 
/// This function sets up the primary route handler that processes all incoming
/// HTTP requests and forwards them to appropriate upstream services based on
/// the configured routing rules. It includes payload size limits and catch-all
/// routing for maximum flexibility.
/// 
/// # Parameters
/// 
/// * `cfg` - Mutable reference to Actix Web service configuration
/// * `handler` - Route handler instance that processes proxy requests
/// 
/// # Configuration Details
/// 
/// The function configures:
/// - **Payload Limits**: 1MB maximum for both raw payloads and JSON
/// - **Catch-All Route**: `/{tail:.*}` pattern matches any incoming path
/// - **Handler Integration**: Connects route processing to business logic
/// 
/// # Route Pattern
/// 
/// The catch-all pattern `/{tail:.*}` captures any incoming path and forwards
/// it to the route handler for processing. This allows the gateway to:
/// - Handle any URL structure from clients
/// - Dynamically match against configured route patterns
/// - Transform paths based on routing configuration
/// 
/// # Payload Size Limits
/// 
/// Security limits are enforced to prevent resource exhaustion:
/// - **Raw Payload**: 1MB maximum for any request body
/// - **JSON Payload**: 1MB maximum for JSON-formatted requests
/// - **Protection**: Guards against memory exhaustion attacks
/// 
/// # Request Processing Flow
/// 
/// 1. **Route Matching**: Incoming request matched against `/{tail:.*}`
/// 2. **Size Validation**: Payload size checked against configured limits
/// 3. **Handler Dispatch**: Request forwarded to RouteHandler for processing
/// 4. **Async Processing**: Handler processes request asynchronously
/// 5. **Response Return**: Processed response returned to client
/// 
/// # Examples
/// 
/// ```rust
/// use actix_web::{App, web};
/// use kairos_rs::routes::http::configure_route;
/// use kairos_rs::services::http::RouteHandler;
/// use kairos_rs::models::router::{Router, Backend, Protocol};
/// 
/// // Create route handler with configuration
/// let routes = vec![
///     Router {
///         host: Some("http://backend".to_string()),
///         port: Some(8080),
///         external_path: "/api/users/{id}".to_string(),
///         internal_path: "/v1/user/{id}".to_string(),
///         methods: vec!["GET".to_string(), "PUT".to_string()],
///         auth_required: false,
///         backends: Some(vec![Backend {
///             host: "http://backend".to_string(),
///             port: 8080,
///             weight: 1,
///             health_check_path: None,
///         }]),
///         load_balancing_strategy: Default::default(),
///         retry: None,
///         protocol: Protocol::Http,
///     }
/// ];
/// let handler = RouteHandler::new(routes, 30);
/// 
/// // Configure the application with proxy routes
/// let app = App::new().configure(|cfg| configure_route(cfg, handler));
/// ```
/// 
/// # Performance Characteristics
/// 
/// - **Memory Efficient**: 1MB payload limit prevents excessive memory use
/// - **Async Processing**: Non-blocking request handling for high concurrency
/// - **Single Route**: Minimal routing overhead with catch-all pattern
/// - **Handler Reuse**: Cloned handler instances share underlying resources
/// 
/// # Security Features
/// 
/// - **Size Limits**: Prevents large payload DoS attacks
/// - **Input Validation**: JSON parsing limits and validation
/// - **Resource Protection**: Guards against memory exhaustion
/// 
/// # Thread Safety
/// 
/// The route handler is cloned for each request, but shares underlying
/// resources (HTTP client, route matcher) safely across threads.
#[allow(dead_code)] // Public API for custom route configuration
pub fn configure_route(cfg: &mut web::ServiceConfig, handler: RouteHandler) {
    cfg.app_data(web::PayloadConfig::new(1024 * 1024)) // 1MB payload limit (reduced from 10MB)
        .app_data(web::JsonConfig::default().limit(1024 * 1024)) // 1MB JSON limit
        .service(
            web::resource("/{tail:.*}").to(move |req: HttpRequest, body: web::Bytes| {
                let handler: RouteHandler = handler.clone();
                async move { handler.handle_request(req, body).await }
            }),
        );
}
