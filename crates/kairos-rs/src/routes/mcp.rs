//! Model Context Protocol (MCP) HTTP and SSE routes.
//!
//! Provides the standard MCP endpoints `/mcp/sse` and `/mcp/messages`, as well as
//! management endpoints `/api/mcp/status`, `/api/mcp/tools`, and `/api/mcp/tools/call`.

use actix_web::{get, post, web, HttpResponse, Responder};
use bytes::Bytes;
use serde::Deserialize;

use crate::models::mcp::CallToolParams;
use crate::services::mcp_server::McpServer;

#[derive(Debug, Deserialize)]
pub struct McpQuery {
    #[serde(alias = "sessionId", alias = "session_id")]
    pub session_id: Option<String>,
}

/// GET /mcp/sse - Establishes an SSE stream for MCP clients (Claude Desktop, IDEs, Agents).
#[get("/mcp/sse")]
pub async fn mcp_sse_endpoint(server: web::Data<McpServer>) -> impl Responder {
    if !server.config().enabled {
        return HttpResponse::NotFound().body("MCP Server is disabled");
    }

    let (session_id, rx) = server.create_session();

    // Stream lines from the unbounded receiver to the HTTP client
    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|msg| (Ok::<_, actix_web::Error>(Bytes::from(msg)), rx))
    });

    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, "text/event-stream"))
        .insert_header((actix_web::http::header::CACHE_CONTROL, "no-cache, no-transform"))
        .insert_header((actix_web::http::header::CONNECTION, "keep-alive"))
        .insert_header(("X-Accel-Buffering", "no"))
        .insert_header(("X-Mcp-Session-Id", session_id))
        .streaming(stream)
}

/// POST /mcp/messages - Receives JSON-RPC 2.0 requests from MCP clients.
#[post("/mcp/messages")]
pub async fn mcp_messages_endpoint(
    query: web::Query<McpQuery>,
    body: String,
    server: web::Data<McpServer>,
) -> impl Responder {
    if !server.config().enabled {
        return HttpResponse::NotFound().body("MCP Server is disabled");
    }

    let session_id = match &query.session_id {
        Some(id) if !id.is_empty() => id,
        _ => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32600,
                    "message": "Missing required query parameter: sessionId"
                }
            }));
        }
    };

    match server.handle_message(session_id, &body).await {
        Ok(Some(response)) => HttpResponse::Accepted().json(response),
        Ok(None) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::Ok().json(crate::models::mcp::JsonRpcResponse::error(None, err)),
    }
}

/// GET /api/mcp/status - Gateway management endpoint returning MCP server operational metrics.
#[get("/api/mcp/status")]
pub async fn mcp_status_endpoint(server: web::Data<McpServer>) -> impl Responder {
    let tools = server.collect_tools().await;
    HttpResponse::Ok().json(serde_json::json!({
        "enabled": server.config().enabled,
        "server_name": server.config().server_name,
        "server_version": server.config().server_version,
        "auth_required": server.config().auth_required,
        "active_sessions": server.active_sessions_count(),
        "total_tools": tools.len(),
    }))
}

/// GET /api/mcp/tools - Gateway management endpoint returning the aggregated tool catalog.
#[get("/api/mcp/tools")]
pub async fn mcp_tools_list_endpoint(server: web::Data<McpServer>) -> impl Responder {
    let tools = server.collect_tools().await;
    HttpResponse::Ok().json(serde_json::json!({
        "tools": tools,
        "count": tools.len(),
    }))
}

/// POST /api/mcp/tools/call - Gateway management endpoint allowing Web UI to test calling tools.
#[post("/api/mcp/tools/call")]
pub async fn mcp_tool_call_endpoint(
    params: web::Json<CallToolParams>,
    server: web::Data<McpServer>,
) -> impl Responder {
    let result = server.execute_tool(&params.name, params.arguments.clone().unwrap_or_default()).await;
    HttpResponse::Ok().json(result)
}

/// Configures all MCP routes in the Actix web application.
pub fn configure_mcp(cfg: &mut web::ServiceConfig) {
    cfg.service(mcp_sse_endpoint)
        .service(mcp_messages_endpoint)
        .service(mcp_status_endpoint)
        .service(mcp_tools_list_endpoint)
        .service(mcp_tool_call_endpoint);
}
