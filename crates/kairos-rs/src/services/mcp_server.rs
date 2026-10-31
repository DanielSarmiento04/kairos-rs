//! Model Context Protocol (MCP) server implementation.
//!
//! Manages Server-Sent Events (SSE) sessions, dispatches JSON-RPC 2.0 requests,
//! provides native gateway diagnostic tools, and exposes configured routes as tools.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use actix_web::web;
use bytes::Bytes;
use dashmap::DashMap;
use log::{debug, info};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

use crate::config::hot_reload::ConfigWatcher;
use crate::models::mcp::{
    CallToolParams, CallToolResult, ImplementationInfo, InitializeParams, InitializeResult,
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, PromptsCapability,
    PromptsListResult, Resource, ResourcesCapability, ResourcesListResult, ServerCapabilities,
    Tool, ToolsCapability, ToolsListResult, LATEST_PROTOCOL_VERSION,
};
use crate::models::router::Router;
use crate::models::settings::McpConfig;
use crate::routes::management::RouteManager;
use crate::routes::metrics::MetricsCollector;
use crate::services::cache::ResponseCache;
use crate::services::circuit_breaker::CircuitState;
use crate::services::http::RouteHandler;
use crate::services::semantic_cache::SemanticCache;

/// Represents an active MCP client session over SSE.
#[derive(Debug)]
pub struct McpSession {
    pub session_id: String,
    pub created_at: Instant,
    pub last_active: Arc<RwLock<Instant>>,
    pub sender: mpsc::UnboundedSender<String>,
}

/// Core Model Context Protocol (MCP) server for Kairos Gateway.
#[derive(Clone)]
pub struct McpServer {
    config: McpConfig,
    sessions: Arc<DashMap<String, McpSession>>,
    route_manager: Option<Arc<RouteManager>>,
    config_watcher: Option<Arc<ConfigWatcher>>,
    route_handler: Option<Arc<RouteHandler>>,
    metrics: Option<web::Data<MetricsCollector>>,
    response_cache: Option<Arc<ResponseCache>>,
    semantic_cache: Option<Arc<SemanticCache>>,
    start_time: Instant,
}

impl McpServer {
    /// Creates a new MCP server with the given configuration.
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(DashMap::new()),
            route_manager: None,
            config_watcher: None,
            route_handler: None,
            metrics: None,
            response_cache: None,
            semantic_cache: None,
            start_time: Instant::now(),
        }
    }

    pub fn with_route_manager(mut self, route_manager: Arc<RouteManager>) -> Self {
        self.route_manager = Some(route_manager);
        self
    }

    pub fn with_config_watcher(mut self, config_watcher: Arc<ConfigWatcher>) -> Self {
        self.config_watcher = Some(config_watcher);
        self
    }

    pub fn with_route_handler(mut self, route_handler: Arc<RouteHandler>) -> Self {
        self.route_handler = Some(route_handler);
        self
    }

    pub fn with_metrics(mut self, metrics: web::Data<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn with_response_cache(mut self, cache: Arc<ResponseCache>) -> Self {
        self.response_cache = Some(cache);
        self
    }

    pub fn with_semantic_cache(mut self, cache: Arc<SemanticCache>) -> Self {
        self.semantic_cache = Some(cache);
        self
    }

    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    pub fn active_sessions_count(&self) -> usize {
        self.sessions.len()
    }

    /// Creates and registers a new SSE session.
    /// Returns the session ID and receiver channel for SSE events.
    pub fn create_session(&self) -> (String, mpsc::UnboundedReceiver<String>) {
        let session_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        let session = McpSession {
            session_id: session_id.clone(),
            created_at: Instant::now(),
            last_active: Arc::new(RwLock::new(Instant::now())),
            sender: tx.clone(),
        };

        // Emit initial endpoint event per MCP SSE specification
        let endpoint_event = format!("event: endpoint\ndata: /mcp/messages?sessionId={}\n\n", session_id);
        let _ = tx.send(endpoint_event);

        self.sessions.insert(session_id.clone(), session);
        info!("MCP SSE session established: {}", session_id);
        (session_id, rx)
    }

    /// Removes an existing session.
    pub fn remove_session(&self, session_id: &str) {
        if self.sessions.remove(session_id).is_some() {
            info!("MCP session terminated: {}", session_id);
        }
    }

    /// Dispatches an incoming JSON-RPC 2.0 message received via `POST /mcp/messages`.
    pub async fn handle_message(
        &self,
        session_id: &str,
        raw_message: &str,
    ) -> Result<Option<JsonRpcResponse>, JsonRpcError> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| JsonRpcError::invalid_request(format!("Session not found: {}", session_id)))?;

        // Update last activity timestamp
        {
            let mut last_active = session.last_active.write().await;
            *last_active = Instant::now();
        }

        let request: JsonRpcRequest = serde_json::from_str(raw_message).map_err(|e| {
            JsonRpcError::parse_error(format!("Failed to parse JSON-RPC request: {}", e))
        })?;

        debug!("Received MCP JSON-RPC message: method='{}' id={:?}", request.method, request.id);

        if request.is_notification() {
            // Handle notifications (no response message emitted)
            self.handle_notification(&request.method, request.params).await;
            return Ok(None);
        }

        let req_id = request.id.clone();
        let response = match self.dispatch_method(&request.method, request.params).await {
            Ok(result) => JsonRpcResponse::success(req_id, result),
            Err(err) => JsonRpcResponse::error(req_id, err),
        };

        // Push response down the client's SSE channel
        if let Ok(serialized) = serde_json::to_string(&response) {
            let sse_event = format!("event: message\ndata: {}\n\n", serialized);
            let _ = session.sender.send(sse_event);
        }

        Ok(Some(response))
    }

    /// Dispatches JSON-RPC notifications.
    async fn handle_notification(&self, method: &str, _params: Option<serde_json::Value>) {
        match method {
            "notifications/initialized" => {
                debug!("MCP client acknowledged initialization");
            }
            "notifications/cancelled" => {
                debug!("MCP client cancelled ongoing operation");
            }
            other => {
                debug!("Ignored unhandled notification: {}", other);
            }
        }
    }

    /// Dispatches JSON-RPC methods and returns the result or a JSON-RPC error.
    async fn dispatch_method(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        match method {
            "initialize" => {
                let init_params: InitializeParams = if let Some(p) = params {
                    serde_json::from_value(p)
                        .map_err(|e| JsonRpcError::invalid_params(format!("Invalid initialize params: {}", e)))?
                } else {
                    return Err(JsonRpcError::invalid_params("Missing initialize params"));
                };

                info!(
                    "MCP client initializing: '{}' v{}",
                    init_params.client_info.name, init_params.client_info.version
                );

                let result = InitializeResult {
                    protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability {
                            list_changed: Some(true),
                        }),
                        resources: Some(ResourcesCapability {
                            subscribe: Some(false),
                            list_changed: Some(false),
                        }),
                        prompts: Some(PromptsCapability {
                            list_changed: Some(false),
                        }),
                        logging: None,
                    },
                    server_info: ImplementationInfo {
                        name: self.config.server_name.clone(),
                        version: self.config.server_version.clone(),
                    },
                    instructions: Some(
                        "Kairos API Gateway MCP Server. Allows discovering and managing gateway routing, telemetry percentiles, cache states, circuit breakers, and calling backend services directly as tools."
                            .to_string(),
                    ),
                };

                serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
            }

            "ping" => Ok(json!({})),

            "tools/list" => {
                let tools = self.collect_tools().await;
                let result = ToolsListResult {
                    tools,
                    next_cursor: None,
                };
                serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
            }

            "tools/call" => {
                let call_params: CallToolParams = if let Some(p) = params {
                    serde_json::from_value(p)
                        .map_err(|e| JsonRpcError::invalid_params(format!("Invalid call tool params: {}", e)))?
                } else {
                    return Err(JsonRpcError::invalid_params("Missing call tool params"));
                };

                let result = self.execute_tool(&call_params.name, call_params.arguments.unwrap_or(json!({}))).await;
                serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
            }

            "resources/list" => {
                let resources = vec![
                    Resource {
                        uri: "kairos://status".to_string(),
                        name: "Gateway Status".to_string(),
                        description: Some("Current uptime, active connections, and Redis state".to_string()),
                        mime_type: Some("application/json".to_string()),
                    },
                    Resource {
                        uri: "kairos://routes".to_string(),
                        name: "Configured Routes".to_string(),
                        description: Some("Active route table definitions and backend mappings".to_string()),
                        mime_type: Some("application/json".to_string()),
                    },
                    Resource {
                        uri: "kairos://circuit-breakers".to_string(),
                        name: "Circuit Breakers".to_string(),
                        description: Some("Real-time circuit breaker trip states and failure counts".to_string()),
                        mime_type: Some("application/json".to_string()),
                    },
                ];

                let result = ResourcesListResult {
                    resources,
                    next_cursor: None,
                };
                serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
            }

            "resources/read" => {
                let uri = params
                    .as_ref()
                    .and_then(|p| p.get("uri"))
                    .and_then(|u| u.as_str())
                    .ok_or_else(|| JsonRpcError::invalid_params("Missing resource 'uri' parameter"))?;

                let content_str = self.read_resource(uri).await?;
                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": content_str
                    }]
                }))
            }

            "prompts/list" => {
                let prompts = vec![
                    crate::models::mcp::Prompt {
                        name: "diagnose_gateway_health".to_string(),
                        description: Some("Inspect gateway error counters, latency percentiles, and circuit breaker trip states.".to_string()),
                        arguments: None,
                    },
                    crate::models::mcp::Prompt {
                        name: "audit_route_security".to_string(),
                        description: Some("Review active routes for missing authentication or rate limiting policies.".to_string()),
                        arguments: None,
                    },
                ];

                let result = PromptsListResult {
                    prompts,
                    next_cursor: None,
                };
                serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(e.to_string()))
            }

            other => Err(JsonRpcError::method_not_found(other)),
        }
    }

    /// Aggregates all native tools and route-exposed tools.
    pub async fn collect_tools(&self) -> Vec<Tool> {
        let mut tools = Vec::new();

        if self.config.expose_native_tools {
            tools.push(Tool::new(
                "kairos_get_status",
                Some("Returns gateway operational health, uptime, active connections, and Redis status.".to_string()),
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ));

            tools.push(Tool::new(
                "kairos_list_routes",
                Some("Returns all configured HTTP and multi-protocol gateway routes with their backends.".to_string()),
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ));

            tools.push(Tool::new(
                "kairos_get_metrics",
                Some("Retrieves real-time gateway metrics including total requests, error rates, and cache performance.".to_string()),
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ));

            tools.push(Tool::new(
                "kairos_circuit_status",
                Some("Inspects the state of all circuit breakers protecting upstream backends (Closed, Open, HalfOpen).".to_string()),
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ));

            tools.push(Tool::new(
                "kairos_clear_cache",
                Some("Clears HTTP response cache and semantic vector cache. Optionally accepts a regex pattern or prefix.".to_string()),
                json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Optional regex pattern or key prefix to selectively invalidate entries"
                        }
                    }
                }),
            ));

            tools.push(Tool::new(
                "kairos_reload_config",
                Some("Hot-reloads configuration file dynamically without interrupting active client connections.".to_string()),
                json!({
                    "type": "object",
                    "properties": {}
                }),
            ));
        }

        // Collect route-exposed tools
        if let Some(routes) = self.get_configured_routes().await {
            for route in routes {
                if let Some(mcp_tool) = &route.mcp_tool {
                    if mcp_tool.enabled {
                        let tool_name = mcp_tool.name.clone().unwrap_or_else(|| {
                            derive_tool_name_from_route(&route.methods, &route.external_path)
                        });

                        let description = mcp_tool.description.clone().unwrap_or_else(|| {
                            format!(
                                "Forward request to {} via HTTP [{}]",
                                route.external_path,
                                route.methods.join(", ")
                            )
                        });

                        let schema = mcp_tool.input_schema.clone().unwrap_or_else(|| {
                            derive_input_schema_from_path(&route.external_path)
                        });

                        tools.push(Tool::new(tool_name, Some(description), schema));
                    }
                }
            }
        }

        tools
    }

    /// Executes a tool by name (either native tool or route-exposed tool).
    pub async fn execute_tool(&self, name: &str, arguments: serde_json::Value) -> CallToolResult {
        match name {
            "kairos_get_status" => self.tool_get_status().await,
            "kairos_list_routes" => self.tool_list_routes().await,
            "kairos_get_metrics" => self.tool_get_metrics().await,
            "kairos_circuit_status" => self.tool_circuit_status().await,
            "kairos_clear_cache" => self.tool_clear_cache(arguments).await,
            "kairos_reload_config" => self.tool_reload_config().await,
            route_tool_name => {
                // Check if this matches a route-exposed tool
                self.execute_route_tool(route_tool_name, arguments).await
            }
        }
    }

    // ========================================================================
    // Native Tools Implementation
    // ========================================================================

    async fn tool_get_status(&self) -> CallToolResult {
        let uptime_secs = self.start_time.elapsed().as_secs();
        let routes_count = self.get_configured_routes().await.map(|r| r.len()).unwrap_or(0);
        let active_sessions = self.sessions.len();

        let connections = self
            .metrics
            .as_ref()
            .map(|m| m.active_connections.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0);

        let status = json!({
            "status": "healthy",
            "server": self.config.server_name,
            "version": self.config.server_version,
            "uptime_seconds": uptime_secs,
            "active_mcp_sessions": active_sessions,
            "active_client_connections": connections,
            "total_configured_routes": routes_count,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        CallToolResult::text(serde_json::to_string_pretty(&status).unwrap_or_default())
    }

    async fn tool_list_routes(&self) -> CallToolResult {
        if let Some(routes) = self.get_configured_routes().await {
            let simplified: Vec<_> = routes
                .iter()
                .map(|r| {
                    json!({
                        "external_path": r.external_path,
                        "internal_path": r.internal_path,
                        "methods": r.methods,
                        "auth_required": r.auth_required,
                        "backends": r.backends,
                        "mcp_tool_enabled": r.mcp_tool.as_ref().map(|t| t.enabled).unwrap_or(false),
                        "cache_enabled": r.cache.as_ref().map(|c| c.enabled).unwrap_or(false),
                    })
                })
                .collect();

            CallToolResult::text(serde_json::to_string_pretty(&simplified).unwrap_or_default())
        } else {
            CallToolResult::error("Route manager unavailable")
        }
    }

    async fn tool_get_metrics(&self) -> CallToolResult {
        if let Some(metrics) = &self.metrics {
            let total_requests = metrics.requests_total.load(std::sync::atomic::Ordering::Relaxed);
            let total_errors = metrics.requests_error.load(std::sync::atomic::Ordering::Relaxed);
            let connections = metrics.active_connections.load(std::sync::atomic::Ordering::Relaxed);
            let time_sum = metrics.response_time_sum.load(std::sync::atomic::Ordering::Relaxed);
            let avg_response_time = if total_requests > 0 {
                time_sum as f64 / total_requests as f64
            } else {
                0.0
            };
            let cache_hits = metrics.cache_hits_total.load(std::sync::atomic::Ordering::Relaxed);
            let cache_misses = metrics.cache_misses_total.load(std::sync::atomic::Ordering::Relaxed);

            let res = json!({
                "requests_total": total_requests,
                "requests_error": total_errors,
                "active_connections": connections,
                "average_response_time_ms": avg_response_time,
                "cache_hits_total": cache_hits,
                "cache_misses_total": cache_misses,
            });

            CallToolResult::text(serde_json::to_string_pretty(&res).unwrap_or_default())
        } else {
            CallToolResult::error("Metrics collector unavailable")
        }
    }

    async fn tool_circuit_status(&self) -> CallToolResult {
        if let Some(handler) = &self.route_handler {
            let states = handler.get_circuit_breaker_states();
            let mapped: HashMap<String, serde_json::Value> = states
                .into_iter()
                .map(|(service, (state, failures, successes))| {
                    let state_str = match state {
                        CircuitState::Closed => "Closed (Healthy)",
                        CircuitState::Open => "Open (Failing)",
                        CircuitState::HalfOpen => "HalfOpen (Recovering)",
                    };
                    (
                        service,
                        json!({
                            "state": state_str,
                            "failure_count": failures,
                            "success_count": successes
                        }),
                    )
                })
                .collect();

            CallToolResult::text(serde_json::to_string_pretty(&mapped).unwrap_or_default())
        } else {
            CallToolResult::error("Route handler unavailable")
        }
    }

    async fn tool_clear_cache(&self, arguments: serde_json::Value) -> CallToolResult {
        let pattern = arguments.get("pattern").and_then(|v| v.as_str());

        let mut cleared_response_cache = false;
        let mut cleared_semantic_cache = false;

        if let Some(cache) = &self.response_cache {
            if let Some(pat) = pattern {
                cache.invalidate_prefix(pat);
            } else {
                cache.clear();
            }
            cleared_response_cache = true;
        }

        if let Some(sc) = &self.semantic_cache {
            sc.clear();
            cleared_semantic_cache = true;
        }

        if let Some(handler) = &self.route_handler {
            handler.cache_clear().await;
        }

        let res = json!({
            "status": "success",
            "message": "Caches cleared successfully",
            "pattern_applied": pattern,
            "response_cache_cleared": cleared_response_cache,
            "semantic_cache_cleared": cleared_semantic_cache,
        });

        CallToolResult::text(serde_json::to_string_pretty(&res).unwrap_or_default())
    }

    async fn tool_reload_config(&self) -> CallToolResult {
        if let Some(watcher) = &self.config_watcher {
            match watcher.manual_reload().await {
                Ok(update) => {
                    let res = json!({
                        "status": "success",
                        "message": "Configuration reloaded successfully",
                        "version": update.version,
                        "routes_count": update.settings.routers.len(),
                        "timestamp": update.timestamp.to_rfc3339()
                    });
                    CallToolResult::text(serde_json::to_string_pretty(&res).unwrap_or_default())
                }
                Err(e) => CallToolResult::error(format!("Failed to reload configuration: {}", e)),
            }
        } else {
            CallToolResult::error("Config watcher unavailable")
        }
    }

    // ========================================================================
    // Route-Exposed Tool Invocation
    // ========================================================================

    async fn execute_route_tool(&self, tool_name: &str, arguments: serde_json::Value) -> CallToolResult {
        let routes = match self.get_configured_routes().await {
            Some(r) => r,
            None => return CallToolResult::error("Route manager unavailable"),
        };

        // Find route matching this tool name
        let matched_route = routes.into_iter().find(|r| {
            if let Some(tool_cfg) = &r.mcp_tool {
                if tool_cfg.enabled {
                    let derived = derive_tool_name_from_route(&r.methods, &r.external_path);
                    return tool_cfg.name.as_deref() == Some(tool_name) || derived == tool_name;
                }
            }
            false
        });

        let route = match matched_route {
            Some(r) => r,
            None => return CallToolResult::error(format!("Unknown tool: '{}'", tool_name)),
        };

        let method = route.methods.first().cloned().unwrap_or_else(|| "GET".to_string());
        let (resolved_path, remaining_args) = substitute_path_parameters(&route.external_path, &arguments);

        debug!(
            "Executing MCP route tool '{}' -> {} {}",
            tool_name, method, resolved_path
        );

        // Execute via internal RouteHandler
        if let Some(handler) = &self.route_handler {
            // Build an Actix TestRequest to feed into RouteHandler
            let actix_method = actix_web::http::Method::from_bytes(method.as_bytes())
                .unwrap_or(actix_web::http::Method::GET);

            let mut uri = resolved_path;
            let mut body_bytes = Bytes::new();

            if actix_method == actix_web::http::Method::GET || actix_method == actix_web::http::Method::DELETE {
                // If there are remaining arguments, encode them as query parameters
                if let Some(obj) = remaining_args.as_object() {
                    if !obj.is_empty() {
                        let query_pairs: Vec<String> = obj
                            .iter()
                            .map(|(k, v)| {
                                let v_str = if let Some(s) = v.as_str() {
                                    s.to_string()
                                } else {
                                    v.to_string()
                                };
                                format!("{}={}", urlencoding::encode(k), urlencoding::encode(&v_str))
                            })
                            .collect();
                        let separator = if uri.contains('?') { "&" } else { "?" };
                        uri.push_str(separator);
                        uri.push_str(&query_pairs.join("&"));
                    }
                }
            } else {
                // For POST/PUT/PATCH, pass remaining arguments as JSON body
                if !remaining_args.is_null() && remaining_args != json!({}) {
                    body_bytes = Bytes::from(serde_json::to_vec(&remaining_args).unwrap_or_default());
                }
            }

            let req = actix_web::test::TestRequest::default()
                .method(actix_method)
                .uri(&uri)
                .insert_header(("content-type", "application/json"))
                .to_http_request();

            match handler.handle_request(req, body_bytes).await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let body = match actix_web::body::to_bytes(resp.into_body()).await {
                        Ok(b) => String::from_utf8_lossy(&b).to_string(),
                        Err(e) => format!("Failed to read response body: {}", e),
                    };

                    if status >= 400 {
                        CallToolResult::error(format!("HTTP {} from upstream: {}", status, body))
                    } else {
                        CallToolResult::text(body)
                    }
                }
                Err(e) => CallToolResult::error(format!("Gateway forwarding error: {}", e)),
            }
        } else {
            CallToolResult::error("RouteHandler is not attached to MCP server")
        }
    }

    async fn read_resource(&self, uri: &str) -> Result<String, JsonRpcError> {
        match uri {
            "kairos://status" => {
                let status_res = self.tool_get_status().await;
                extract_content_text(status_res)
            }
            "kairos://routes" => {
                let routes_res = self.tool_list_routes().await;
                extract_content_text(routes_res)
            }
            "kairos://circuit-breakers" => {
                let cb_res = self.tool_circuit_status().await;
                extract_content_text(cb_res)
            }
            other => Err(JsonRpcError::invalid_params(format!("Unknown resource URI: '{}'", other))),
        }
    }

    async fn get_configured_routes(&self) -> Option<Vec<Router>> {
        if let Some(rm) = &self.route_manager {
            Some(rm.get_routers().await)
        } else if let Some(cw) = &self.config_watcher {
            Some(cw.get_current_config().await.settings.routers)
        } else {
            None
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Derives a clean snake_case tool name from HTTP method and path pattern.
/// e.g. `["GET"], "/api/users/{id}"` -> `"get_api_users_by_id"`
pub fn derive_tool_name_from_route(methods: &[String], path: &str) -> String {
    let method = methods.first().map(|m| m.to_lowercase()).unwrap_or_else(|| "call".to_string());
    let sanitized_path = path
        .trim_matches('/')
        .replace('/', "_")
        .replace('{', "by_")
        .replace('}', "")
        .replace('-', "_");

    if sanitized_path.is_empty() {
        format!("{}_root", method)
    } else {
        format!("{}_{}", method, sanitized_path)
    }
}

/// Automatically extracts parameters `{name}` from path and creates a JSON Schema.
pub fn derive_input_schema_from_path(path: &str) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for segment in path.split('/') {
        if segment.starts_with('{') && segment.ends_with('}') {
            let param_name = &segment[1..segment.len() - 1];
            properties.insert(
                param_name.to_string(),
                json!({
                    "type": "string",
                    "description": format!("Path parameter '{}'", param_name)
                }),
            );
            required.push(param_name.to_string());
        }
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Substitutes path parameters `{key}` with values from arguments.
/// Returns the resolved path and remaining unused arguments.
pub fn substitute_path_parameters(
    path: &str,
    arguments: &serde_json::Value,
) -> (String, serde_json::Value) {
    let mut resolved_path = path.to_string();
    let mut unused_args = match arguments.as_object() {
        Some(obj) => obj.clone(),
        None => serde_json::Map::new(),
    };

    for segment in path.split('/') {
        if segment.starts_with('{') && segment.ends_with('}') {
            let param_name = &segment[1..segment.len() - 1];
            if let Some(val) = unused_args.remove(param_name) {
                let val_str = if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                };
                resolved_path = resolved_path.replace(segment, &val_str);
            }
        }
    }

    (resolved_path, serde_json::Value::Object(unused_args))
}

fn extract_content_text(res: CallToolResult) -> Result<String, JsonRpcError> {
    for c in res.content {
        if let crate::models::mcp::ToolContent::Text { text } = c {
            return Ok(text);
        }
    }
    Err(JsonRpcError::internal_error("No text content found in resource"))
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::with_capacity(s.len());
        for byte in s.bytes() {
            if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'.' || byte == b'~' {
                encoded.push(byte as char);
            } else {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
        encoded
    }
}
