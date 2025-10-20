use crate::models::error::GatewayError;
use crate::models::router::Router;
use crate::routes::metrics::MetricsCollector;
use crate::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use crate::services::load_balancer::{LoadBalancer, LoadBalancerFactory};
use crate::utils::path::format_route;
use crate::utils::route_matcher::RouteMatcher;

use actix_web::{
    http::{Method as ActixMethod, StatusCode},
    web, Error as ActixError, HttpRequest, HttpResponse,
};
use log::{debug, info, warn};
use reqwest::{
    header::HeaderMap as ReqwestHeaderMap, header::HeaderName, header::HeaderValue, Client,
    Method as ReqwestMethod,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, timeout, Duration};

/// High-performance HTTP request handler for the kairos-rs gateway.
/// 
/// The `RouteHandler` is responsible for processing incoming HTTP requests,
/// finding matching routes, and forwarding requests to upstream services.
/// It implements connection pooling, timeout management, circuit breaker protection,
/// and efficient header processing for optimal performance and reliability.
/// 
/// # Architecture
/// 
/// ```text
/// Client Request → RouteHandler → Route Matching → Circuit Breaker → Request Forwarding → Upstream Service
///                             ↓                      ↓
///                    Response Processing ← Upstream Response
/// ```
/// 
/// # Key Features
/// 
/// - **Connection Pooling**: Reuses HTTP connections for better performance
/// - **Timeout Management**: Configurable request timeouts prevent hanging requests
/// - **Header Optimization**: Efficient header conversion and filtering
/// - **Route Matching**: Supports both static and dynamic (parameterized) routes
/// - **Thread Safety**: Safe to clone and share across multiple workers
/// 
/// # Performance Optimizations
/// 
/// - Pre-configured HTTP client with connection pooling
/// - Shared route matcher using `Arc` for zero-copy sharing
/// - Optimized header processing that skips problematic headers
/// - Efficient memory management with capacity pre-allocation
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::services::http::RouteHandler;
/// use kairos_rs::models::router::{Router, Backend};
/// 
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
///     }
/// ];
/// 
/// let handler = RouteHandler::new(routes, 30); // 30-second timeout
/// ```
#[derive(Clone)]
pub struct RouteHandler {
    /// HTTP client with connection pooling and optimized settings
    client: Client,
    /// Thread-safe route matcher for path resolution
    route_matcher: Arc<RouteMatcher>,
    /// Request timeout in seconds
    timeout_seconds: u64,
    /// Circuit breakers for upstream services (keyed by host:port)
    circuit_breakers: Arc<HashMap<String, Arc<CircuitBreaker>>>,
    /// Load balancers for each route (keyed by external_path)
    load_balancers: Arc<HashMap<String, Arc<dyn LoadBalancer>>>,
}

impl RouteHandler {
    /// Creates a new HTTP route handler with optimized client configuration.
    /// 
    /// This constructor sets up a high-performance HTTP client with connection
    /// pooling, compiles all route patterns for efficient matching, and initializes
    /// circuit breakers for upstream service protection.
    /// 
    /// # Parameters
    /// 
    /// * `routes` - Vector of router configurations defining request forwarding rules
    /// * `timeout_seconds` - Maximum time in seconds to wait for upstream responses
    /// 
    /// # Returns
    /// 
    /// A new `RouteHandler` instance ready to process requests with full circuit breaker protection
    /// 
    /// # HTTP Client Configuration
    /// 
    /// The internal HTTP client is configured with:
    /// - **Idle Timeout**: 30 seconds to keep connections alive
    /// - **Pool Size**: Up to 32 idle connections per host
    /// - **Connection Reuse**: Automatic connection pooling
    /// 
    /// # Route Compilation
    /// 
    /// All routes are pre-compiled into an optimized matcher that:
    /// - Separates static and dynamic routes for optimal lookup performance
    /// - Compiles regex patterns for parameterized routes
    /// - Validates all route patterns at startup
    /// 
    /// # Circuit Breaker Initialization
    /// 
    /// Circuit breakers are created for each unique upstream service (host:port combination):
    /// - **Failure Threshold**: 5 consecutive failures trigger circuit opening
    /// - **Success Threshold**: 3 consecutive successes close an open circuit
    /// - **Reset Timeout**: 30 seconds before transitioning from open to half-open
    /// - **Service Isolation**: Each upstream service has independent protection
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::http::RouteHandler;
    /// use kairos_rs::models::router::{Router, Backend};
    /// 
    /// let routes = vec![
    ///     Router {
    ///         host: Some("http://auth-service".to_string()),
    ///         port: Some(8080),
    ///         external_path: "/auth/login".to_string(),
    ///         internal_path: "/authenticate".to_string(),
    ///         methods: vec!["POST".to_string()],
    ///         auth_required: false,
    ///         backends: Some(vec![Backend {
    ///             host: "http://auth-service".to_string(),
    ///             port: 8080,
    ///             weight: 1,
    ///             health_check_path: None,
    ///         }]),
    ///         load_balancing_strategy: Default::default(),
    ///         retry: None,
    ///     },
    ///     Router {
    ///         host: Some("http://user-service".to_string()),
    ///         port: Some(8080),
    ///         external_path: "/users/{id}".to_string(),
    ///         internal_path: "/api/v1/user/{id}".to_string(),
    ///         methods: vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
    ///         auth_required: false,
    ///         backends: Some(vec![Backend {
    ///             host: "http://user-service".to_string(),
    ///             port: 8080,
    ///             weight: 1,
    ///             health_check_path: None,
    ///         }]),
    ///         load_balancing_strategy: Default::default(),
    ///         retry: None,
    ///     }
    /// ];
    /// 
    /// let handler = RouteHandler::new(routes, 30);
    /// ```
    /// 
    /// # Panics
    /// 
    /// Panics if:
    /// - HTTP client creation fails (rare, indicates system resource issues)
    /// - Route compilation fails (invalid route patterns in configuration)
    /// 
    /// # Thread Safety
    /// 
    /// The returned handler is safe to clone and share across multiple worker threads.
    /// All internal state is either immutable or thread-safe.
    pub fn new(routes: Vec<Router>, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to create HTTP client");

        let route_matcher = Arc::new(
            RouteMatcher::new(routes.clone()).expect("Failed to create route matcher")
        );

        // Create circuit breakers for each unique backend
        let mut circuit_breakers = HashMap::new();
        let mut load_balancers = HashMap::new();
        
        for route in &routes {
            // Get all backends for this route
            let backends = route.get_backends();
            
            // Create circuit breakers for each backend
            for backend in &backends {
                let service_key = format!("{}:{}", backend.host, backend.port);
                if !circuit_breakers.contains_key(&service_key) {
                    let config = CircuitBreakerConfig::default();
                    let circuit_breaker = CircuitBreaker::new(service_key.clone(), config);
                    circuit_breakers.insert(service_key, circuit_breaker);
                }
            }
            
            // Create load balancer for this route if multiple backends
            if backends.len() > 1 {
                let balancer = LoadBalancerFactory::create(&route.load_balancing_strategy);
                load_balancers.insert(route.external_path.clone(), balancer);
                info!(
                    "Created {:?} load balancer for route {} with {} backends",
                    route.load_balancing_strategy,
                    route.external_path,
                    backends.len()
                );
            }
        }

        Self {
            client,
            route_matcher,
            timeout_seconds,
            circuit_breakers: Arc::new(circuit_breakers),
            load_balancers: Arc::new(load_balancers),
        }
    }

    /// Processes an incoming HTTP request and forwards it to the appropriate upstream service.
    /// 
    /// This is the core request processing method that handles route matching,
    /// method validation, header transformation, circuit breaker protection,
    /// metrics collection, and upstream communication. It implements comprehensive
    /// error handling and timeout management for reliable gateway operation.
    /// 
    /// # Parameters
    /// 
    /// * `req` - The incoming HTTP request with headers, method, and path information
    /// * `body` - The request body as bytes for efficient forwarding
    /// 
    /// # Returns
    /// 
    /// * `Ok(HttpResponse)` - Successfully forwarded request with upstream response
    /// * `Err(ActixError)` - Request processing error (routing, upstream, timeout, circuit open)
    /// 
    /// # Request Processing Flow
    /// 
    /// 1. **Metrics Setup**: Initialize request timing and connection tracking
    /// 2. **Route Resolution**: Matches request path against configured routes
    /// 3. **Method Validation**: Verifies HTTP method is allowed for the route
    /// 4. **Circuit Breaker Check**: Verifies upstream service availability
    /// 5. **Header Processing**: Converts and filters headers for upstream forwarding
    /// 6. **Request Forwarding**: Sends request to upstream service with timeout and circuit protection
    /// 7. **Response Processing**: Converts upstream response back to client format
    /// 8. **Metrics Recording**: Records request timing, success/failure, and connection cleanup
    /// 
    /// # Route Matching
    /// 
    /// Supports both static and dynamic routes:
    /// - Static: `/api/health` → `/internal/health`
    /// - Dynamic: `/users/{id}` → `/v1/user/{id}` (with parameter substitution)
    /// 
    /// # Circuit Breaker Protection
    /// 
    /// Each upstream service is protected by an independent circuit breaker:
    /// - **Fast Failure**: Open circuits return 503 immediately without upstream calls
    /// - **Service Isolation**: Failures in one service don't affect others  
    /// - **Automatic Recovery**: Half-open state tests service recovery
    /// - **Error Tracking**: Tracks consecutive failures and successes
    /// 
    /// # Header Processing
    /// 
    /// - **Forwarded**: Most headers are passed through unchanged
    /// - **Filtered**: Connection-related headers are stripped (`host`, `connection`, etc.)
    /// - **Added**: Default `User-Agent` header if not present
    /// - **Preserved**: Authorization, content-type, and custom headers
    /// 
    /// # Error Handling
    /// 
    /// Returns specific errors for different failure modes:
    /// - **RouteNotFound**: No matching route configuration
    /// - **MethodNotAllowed**: HTTP method not permitted for the route
    /// - **CircuitOpen**: Circuit breaker protecting upstream service (503)
    /// - **Timeout**: Upstream response exceeded configured timeout
    /// - **Upstream**: Connection or response errors from upstream service
    /// - **Config**: Route configuration or processing errors
    /// 
    /// # Timeout Management
    /// 
    /// - **Request Timeout**: Configurable per-instance timeout for upstream requests
    /// - **Connection Pooling**: Automatic connection reuse reduces latency
    /// - **Circuit Breaking**: Failed requests don't block subsequent requests
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use actix_web::{web, HttpRequest, HttpResponse, Error as ActixError};
    /// # use std::sync::Arc;
    /// # 
    /// # struct Router;
    /// # struct RouteHandler;
    /// # impl RouteHandler {
    /// #     fn new(_routes: Vec<Router>, _timeout: u64) -> Arc<Self> {
    /// #         Arc::new(RouteHandler)
    /// #     }
    /// #     async fn handle_request(&self, _req: HttpRequest, _body: web::Bytes) -> Result<HttpResponse, ActixError> {
    /// #         Ok(HttpResponse::Ok().body("OK"))
    /// #     }
    /// # }
    /// 
    /// async fn proxy_handler(
    ///     req: HttpRequest,
    ///     body: web::Bytes,
    ///     handler: web::Data<Arc<RouteHandler>>
    /// ) -> Result<HttpResponse, ActixError> {
    ///     handler.handle_request(req, body).await
    /// }
    /// ```
    /// 
    /// # Performance Characteristics
    /// 
    /// - **Latency**: ~1-5ms overhead for route processing and header conversion
    /// - **Throughput**: Scales with underlying HTTP client connection pool
    /// - **Memory**: Efficient header processing with minimal allocations
    /// - **Concurrency**: Thread-safe with optimistic route matching
    /// 
    /// # Security Considerations
    /// 
    /// - **Header Filtering**: Removes potentially harmful proxy headers
    /// - **Method Validation**: Enforces allowed HTTP methods per route
    /// - **Timeout Protection**: Prevents resource exhaustion from slow upstreams
    /// - **Error Isolation**: Upstream failures don't crash the gateway
    pub async fn handle_request(
        &self,
        req: HttpRequest,
        body: web::Bytes,
    ) -> Result<HttpResponse, ActixError> {
        let start_time = Instant::now();

        // Get metrics collector from app data if available
        let metrics = req.app_data::<web::Data<MetricsCollector>>().cloned();
        
        // Track active connections
        if let Some(ref metrics) = metrics {
            metrics.increment_connections();
        }

        let result = self.handle_request_internal(req, body).await;
        
        // Record metrics
        if let Some(ref metrics) = metrics {
            let duration = start_time.elapsed();
            match &result {
                Ok(resp) => {
                    let success = resp.status().is_success();
                    let status_code = resp.status().as_u16();
                    metrics.record_request(success, duration, status_code, None, None);
                },
                Err(_) => {
                    // For errors, we don't have a specific status code, so use 500
                    metrics.record_request(false, duration, 500, None, None);
                }
            }
            metrics.decrement_connections();
        }

        result
    }

    async fn handle_request_internal(
        &self,
        req: HttpRequest,
        body: web::Bytes,
    ) -> Result<HttpResponse, ActixError> {
        let path = req.path().to_string();
        let method = req.method().clone();

        // Convert Actix method to Reqwest method
        let reqwest_method = self.parse_method(&method);

        // Convert headers
        let reqwest_headers = self.build_headers_optimized(req.headers());

        // Find matching route using the new pattern matching function
        let (route, transformed_internal_path) = self.route_matcher.find_match(&path)
            .map_err(|e| match e {
                crate::utils::route_matcher::RouteMatchError::NoMatch { path } => {
                    GatewayError::RouteNotFound { path }
                }
                _ => GatewayError::Config { 
                    message: e.to_string(), 
                    route: path.clone() 
                }
            })?;

        // Validate method is allowed
        if !route.methods.iter().any(|m| m == method.as_str()) {
            return Err(GatewayError::MethodNotAllowed { 
                method: method.to_string(), 
                path: path.clone() 
            }.into());
        }

        // Get all backends for this route
        let backends = route.get_backends();
        if backends.is_empty() {
            return Err(GatewayError::Config {
                message: "No backends configured for route".to_string(),
                route: path.clone(),
            }.into());
        }

        // Get client IP for IP hash load balancing
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        // Try with retry logic if configured
        let retry_config = route.retry.clone();
        let max_attempts = retry_config.as_ref().map(|c| c.max_retries + 1).unwrap_or(1);

        for attempt in 0..max_attempts {
            // Select backend using load balancing strategy
            let backend = if backends.len() == 1 {
                backends[0].clone()
            } else if let Some(load_balancer) = self.load_balancers.get(&route.external_path) {
                load_balancer
                    .select_backend(&backends, client_ip.as_deref())
                    .ok_or_else(|| GatewayError::Config {
                        message: "Load balancer failed to select backend".to_string(),
                        route: path.clone(),
                    })?
            } else {
                // Fallback to first backend if no load balancer
                backends[0].clone()
            };

            let target_url = format_route(&backend.host, &backend.port, &transformed_internal_path);
            
            if attempt > 0 {
                warn!("Retry attempt {} for {}", attempt, target_url);
            } else {
                debug!("Forwarding request to: {}", target_url);
            }

            // Get circuit breaker for this backend
            let service_key = format!("{}:{}", backend.host, backend.port);
            let circuit_breaker = self.circuit_breakers.get(&service_key)
                .ok_or_else(|| GatewayError::Config { 
                    message: format!("No circuit breaker found for backend: {}", service_key),
                    route: path.clone()
                })?;

            // Prepare request
            let forwarded_req = self
                .client
                .request(reqwest_method.clone(), &target_url)
                .body(body.to_vec())
                .headers(reqwest_headers.clone());

            // Execute request with timeout and circuit breaker protection
            let result = circuit_breaker.call(async {
                match timeout(
                    Duration::from_secs(self.timeout_seconds),
                    forwarded_req.send(),
                ).await {
                    Ok(Ok(resp)) => Ok(resp),
                    Ok(Err(e)) => Err(GatewayError::Upstream { 
                        message: e.to_string(),
                        url: target_url.clone(),
                        status: None,
                    }),
                    Err(_) => Err(GatewayError::Timeout { 
                        timeout: self.timeout_seconds 
                    }),
                }
            }).await;

            match result {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    
                    // Check if we should retry based on status code
                    if let Some(retry_cfg) = &retry_config {
                        if retry_cfg.retry_on_status_codes.contains(&status_code) 
                            && attempt < max_attempts - 1 {
                            warn!(
                                "Retryable status {} from {}, attempt {}/{}",
                                status_code, target_url, attempt + 1, max_attempts
                            );
                            
                            // Exponential backoff
                            let backoff_ms = retry_cfg.calculate_backoff(attempt);
                            sleep(Duration::from_millis(backoff_ms)).await;
                            continue;
                        }
                    }
                    
                    // Success - record and return response
                    if let Some(lb) = self.load_balancers.get(&route.external_path) {
                        lb.record_success(&backend);
                    }
                    
                    // Convert upstream response to HttpResponse
                    let mut builder = HttpResponse::build(
                        StatusCode::from_u16(status_code).unwrap()
                    );

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
                        Ok(bytes) => return Ok(builder.body(bytes)),
                        Err(e) => return Err(GatewayError::Upstream { 
                            message: e.to_string(),
                            url: target_url,
                            status: None,
                        }.into()),
                    }
                }
                Err(CircuitBreakerError::CircuitOpen) => {
                    // Circuit is open, try next backend or fail
                    warn!("Circuit breaker open for {}", service_key);
                    
                    if backends.len() > 1 && attempt < max_attempts - 1 {
                        // Try another backend
                        continue;
                    }
                    
                    return Err(GatewayError::CircuitOpen { 
                        service: service_key 
                    }.into());
                }
                Err(CircuitBreakerError::OperationFailed(gateway_error)) => {
                    // Request failed, record failure
                    if let Some(lb) = self.load_balancers.get(&route.external_path) {
                        lb.record_failure(&backend);
                    }
                    
                    // Check if we should retry
                    if let Some(retry_cfg) = &retry_config {
                        if retry_cfg.retry_on_connection_error && attempt < max_attempts - 1 {
                            warn!(
                                "Connection error to {}, retrying (attempt {}/{})",
                                target_url, attempt + 1, max_attempts
                            );
                            
                            // Exponential backoff
                            let backoff_ms = retry_cfg.calculate_backoff(attempt);
                            sleep(Duration::from_millis(backoff_ms)).await;
                            continue;
                        }
                    }
                    
                    return Err(gateway_error.into());
                }
            }
        }

        // All retries exhausted
        Err(GatewayError::Upstream {
            message: format!("All {} retry attempts exhausted", max_attempts),
            url: path,
            status: None,
        }.into())
    }

    /// Efficiently converts and filters HTTP headers for upstream forwarding.
    /// 
    /// This method transforms Actix Web headers to Reqwest headers while filtering
    /// out problematic headers that could interfere with proper proxying. It
    /// implements optimized header processing with pre-allocated capacity.
    /// 
    /// # Parameters
    /// 
    /// * `original_headers` - The incoming request headers from Actix Web
    /// 
    /// # Returns
    /// 
    /// A `ReqwestHeaderMap` with filtered and converted headers ready for upstream forwarding
    /// 
    /// # Header Processing Rules
    /// 
    /// ## Filtered Headers (Not Forwarded)
    /// - `host` - Will be set by the upstream URL
    /// - `connection` - Connection management headers
    /// - `upgrade` - Protocol upgrade headers  
    /// - `proxy-connection` - Proxy-specific connection headers
    /// 
    /// ## Preserved Headers
    /// - `authorization` - Authentication credentials
    /// - `content-type` - Request body format
    /// - `content-length` - Request body size
    /// - `accept` - Response format preferences
    /// - Custom application headers
    /// 
    /// ## Added Headers
    /// - `user-agent` - Default "kairos-rs/0.2.0" if not present
    /// 
    /// # Performance Optimizations
    /// 
    /// - **Pre-allocation**: Header map capacity matches original size
    /// - **Efficient Lookup**: Skip list uses compile-time constants
    /// - **Zero-Copy**: Header values converted without string allocation
    /// - **Early Exit**: Skip headers that start with problematic prefixes
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use actix_web::http::header::HeaderMap;
    /// # use std::sync::Arc;
    /// # use reqwest::header::HeaderMap as ReqwestHeaderMap;
    /// # 
    /// # struct Router;
    /// # struct RouteHandler;
    /// # impl RouteHandler {
    /// #     fn new(_routes: Vec<Router>, _timeout: u64) -> Arc<Self> {
    /// #         Arc::new(RouteHandler)
    /// #     }
    /// #     fn build_headers_optimized(&self, _headers: &HeaderMap) -> ReqwestHeaderMap {
    /// #         ReqwestHeaderMap::new()
    /// #     }
    /// # }
    /// 
    /// let headers = HeaderMap::new();
    /// // headers would be populated from request
    /// 
    /// let route_handler = RouteHandler::new(vec![], 30);
    /// let filtered_headers = route_handler.build_headers_optimized(&headers);
    /// ```
    /// 
    /// # Security Considerations
    /// 
    /// - **Proxy Headers**: Removes headers that could expose proxy infrastructure
    /// - **Connection Headers**: Prevents connection manipulation attacks
    /// - **Host Header**: Prevents host header injection by regenerating from target URL
    /// 
    /// # Error Handling
    /// 
    /// - Invalid header names or values are silently skipped
    /// - Malformed headers don't cause request failure
    /// - Continues processing remaining headers on individual conversion failures
    fn build_headers_optimized(
        &self,
        original_headers: &actix_web::http::header::HeaderMap,
    ) -> ReqwestHeaderMap {
        let mut reqwest_headers = ReqwestHeaderMap::with_capacity(original_headers.len());
        
        // Skip problematic headers more efficiently
        const SKIP_HEADERS: &[&str] = &["host", "connection", "upgrade", "proxy-connection"];
        
        for (key, value) in original_headers {
            let key_str = key.as_str().to_lowercase();
            if SKIP_HEADERS.iter().any(|&skip| key_str.starts_with(skip)) {
                continue;
            }

            // More efficient header conversion
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(key.as_ref()),
                HeaderValue::from_bytes(value.as_bytes())
            ) {
                reqwest_headers.insert(header_name, header_value);
            }
        }
        
        // Set default User-Agent if not present
        reqwest_headers.entry("user-agent")
            .or_insert_with(|| HeaderValue::from_static("kairos-rs/0.2.0"));
        
        reqwest_headers
    }

    /// Converts Actix Web HTTP method to Reqwest HTTP method.
    /// 
    /// This method provides efficient conversion between HTTP method types
    /// used by different HTTP client libraries. It supports all standard
    /// HTTP methods with a safe fallback for unknown methods.
    /// 
    /// # Parameters
    /// 
    /// * `method` - The HTTP method from Actix Web request
    /// 
    /// # Returns
    /// 
    /// The equivalent Reqwest HTTP method for upstream request
    /// 
    /// # Supported Methods
    /// 
    /// - **GET**: Retrieve data (most common, safest method)
    /// - **POST**: Submit data, create resources
    /// - **PUT**: Update/replace resources
    /// - **DELETE**: Remove resources
    /// - **HEAD**: Get headers only (like GET but no body)
    /// - **OPTIONS**: Get allowed methods/CORS preflight
    /// - **PATCH**: Partial resource updates
    /// - **CONNECT**: Establish tunnel (for HTTPS proxying)
    /// - **TRACE**: Diagnostic method (rarely used)
    /// 
    /// # Fallback Behavior
    /// 
    /// - Unknown methods default to GET for safety
    /// - GET is the safest HTTP method (idempotent, no side effects)
    /// - Prevents potential issues with non-standard methods
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use actix_web::http::Method;
    /// # use reqwest::Method as ReqwestMethod;
    /// # use std::sync::Arc;
    /// # 
    /// # struct Router;
    /// # struct RouteHandler;
    /// # impl RouteHandler {
    /// #     fn new(_routes: Vec<Router>, _timeout: u64) -> Arc<Self> {
    /// #         Arc::new(RouteHandler)
    /// #     }
    /// #     fn parse_method(&self, _method: &Method) -> ReqwestMethod {
    /// #         ReqwestMethod::GET
    /// #     }
    /// # }
    /// 
    /// let handler = RouteHandler::new(vec![], 30);
    /// let reqwest_method = handler.parse_method(&Method::POST);
    /// ```
    /// 
    /// # Performance
    /// 
    /// - **Zero Allocation**: Direct enum-to-enum conversion
    /// - **Compile-Time**: Match statements optimized by compiler
    /// - **Constant Time**: O(1) conversion regardless of method type
    /// 
    /// # Security Considerations
    /// 
    /// - **Safe Fallback**: Unknown methods default to safe GET
    /// - **Method Validation**: Upstream route configurations control allowed methods
    /// - **No Injection**: Direct enum conversion prevents method injection attacks
    fn parse_method(&self, method: &ActixMethod) -> ReqwestMethod {
        match method {
            &ActixMethod::GET => ReqwestMethod::GET,
            &ActixMethod::POST => ReqwestMethod::POST,
            &ActixMethod::PUT => ReqwestMethod::PUT,
            &ActixMethod::DELETE => ReqwestMethod::DELETE,
            &ActixMethod::HEAD => ReqwestMethod::HEAD,
            &ActixMethod::OPTIONS => ReqwestMethod::OPTIONS,
            &ActixMethod::CONNECT => ReqwestMethod::CONNECT,
            &ActixMethod::PATCH => ReqwestMethod::PATCH,
            &ActixMethod::TRACE => ReqwestMethod::TRACE,
            _ => ReqwestMethod::GET, // or another default, or panic! if you want to handle this differently
        }
    }

    /// Gets the current state of all circuit breakers for monitoring.
    /// 
    /// This method provides access to circuit breaker states for monitoring
    /// and observability purposes. It returns a HashMap with service identifiers
    /// as keys and their current circuit breaker state information.
    /// 
    /// # Returns
    /// 
    /// A HashMap where:
    /// - **Key**: Service identifier in format "host:port"
    /// - **Value**: Tuple of (state, failure_count, success_count)
    /// 
    /// # Circuit States
    /// 
    /// - **Closed**: Normal operation, requests are forwarded
    /// - **Open**: Circuit is open, requests fail fast
    /// - **HalfOpen**: Testing recovery, limited requests allowed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::services::http::RouteHandler;
    /// use kairos_rs::services::circuit_breaker::CircuitState;
    /// 
    /// # let handler = RouteHandler::new(vec![], 30);
    /// let states = handler.get_circuit_breaker_states();
    /// for (service, (state, failures, successes)) in states {
    ///     println!("Service: {}, State: {:?}, Failures: {}, Successes: {}", 
    ///              service, state, failures, successes);
    /// }
    /// ```
    /// 
    /// # Use Cases
    /// 
    /// - **Monitoring Dashboards**: Display circuit breaker status
    /// - **Health Checks**: Include circuit state in health endpoints
    /// - **Alerting**: Trigger alerts when circuits open
    /// - **Debugging**: Understand upstream service health
    pub fn get_circuit_breaker_states(&self) -> HashMap<String, (crate::services::circuit_breaker::CircuitState, u64, u64)> {
        self.circuit_breakers
            .iter()
            .map(|(service, breaker)| {
                let state = breaker.get_state();
                let failure_count = breaker.get_failure_count();
                let success_count = breaker.get_success_count();
                (service.clone(), (state, failure_count, success_count))
            })
            .collect()
    }
}
