use std::sync::Arc;

use actix_web::{test, web, App};
use kairos_rs::models::mcp::{CallToolParams, LATEST_PROTOCOL_VERSION};
use kairos_rs::models::router::{McpToolConfig, Protocol, Router};
use kairos_rs::models::settings::{McpConfig, Settings};
use kairos_rs::routes::management::RouteManager;
use kairos_rs::routes::mcp;
use kairos_rs::services::mcp_server::McpServer;

fn create_test_settings(routers: Vec<Router>) -> Settings {
    Settings {
        version: 1,
        jwt: None,
        rate_limit: None,
        ai: None,
        compression: None,
        admin_auth: None,
        redis: None,
        mcp: Some(McpConfig {
            enabled: true,
            server_name: "kairos-test".to_string(),
            server_version: "0.4.3".to_string(),
            auth_required: false,
            expose_native_tools: true,
            session_timeout_seconds: 3600,
        }),
        routers,
    }
}

#[tokio::test]
async fn test_mcp_direct_lifecycle() {
    let server = McpServer::new(McpConfig {
        enabled: true,
        server_name: "kairos-test".to_string(),
        server_version: "0.4.3".to_string(),
        auth_required: false,
        expose_native_tools: true,
        session_timeout_seconds: 3600,
    });

    // 1. Session creation
    let (session_id, mut rx) = server.create_session();
    assert!(!session_id.is_empty());
    assert_eq!(server.active_sessions_count(), 1);

    // Verify initial endpoint announcement per MCP SSE spec
    let initial_event = rx.recv().await.expect("Expected initial endpoint event");
    assert!(initial_event.contains("event: endpoint"));
    assert!(initial_event.contains(&format!("data: /mcp/messages?sessionId={}", session_id)));

    // 2. Handshake: initialize
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": LATEST_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "claude-desktop-test",
                "version": "1.0.0"
            }
        }
    });

    let resp = server
        .handle_message(&session_id, &init_req.to_string())
        .await
        .expect("handle_message failed")
        .expect("expected response");

    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_none());
    let init_result = resp.result.expect("result expected");
    assert_eq!(init_result["protocolVersion"], LATEST_PROTOCOL_VERSION);
    assert_eq!(init_result["serverInfo"]["name"], "kairos-test");
    assert!(init_result["capabilities"]["tools"].is_object());
    assert!(init_result["capabilities"]["resources"].is_object());
    assert!(init_result["capabilities"]["prompts"].is_object());

    // 3. Ping
    let ping_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "ping"
    });
    let ping_resp = server
        .handle_message(&session_id, &ping_req.to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ping_resp.jsonrpc, "2.0");
    assert!(ping_resp.result.is_some());

    // 4. tools/list
    let list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list"
    });
    let list_resp = server
        .handle_message(&session_id, &list_req.to_string())
        .await
        .unwrap()
        .unwrap();
    let tools_val = list_resp.result.expect("tools result expected");
    let tools = tools_val["tools"].as_array().expect("tools array expected");
    assert!(tools.len() >= 6);

    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(tool_names.contains(&"kairos_get_status"));
    assert!(tool_names.contains(&"kairos_list_routes"));
    assert!(tool_names.contains(&"kairos_get_metrics"));
    assert!(tool_names.contains(&"kairos_circuit_status"));
    assert!(tool_names.contains(&"kairos_clear_cache"));
    assert!(tool_names.contains(&"kairos_reload_config"));

    // 5. tools/call: kairos_get_status
    let call_status_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "kairos_get_status",
            "arguments": {}
        }
    });
    let call_status_resp = server
        .handle_message(&session_id, &call_status_req.to_string())
        .await
        .unwrap()
        .unwrap();
    let call_res = call_status_resp.result.expect("call result expected");
    assert_eq!(call_res["isError"], false);
    let content_text = call_res["content"][0]["text"].as_str().unwrap();
    assert!(content_text.contains("kairos-test"));
    assert!(content_text.contains("healthy"));

    // 6. resources/list and resources/read
    let res_list_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "resources/list"
    });
    let res_list_resp = server
        .handle_message(&session_id, &res_list_req.to_string())
        .await
        .unwrap()
        .unwrap();
    let res_list = res_list_resp.result.expect("resources list expected");
    let resources = res_list["resources"].as_array().unwrap();
    assert!(resources.iter().any(|r| r["uri"] == "kairos://status"));

    let res_read_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "resources/read",
        "params": {
            "uri": "kairos://status"
        }
    });
    let res_read_resp = server
        .handle_message(&session_id, &res_read_req.to_string())
        .await
        .unwrap()
        .unwrap();
    let read_contents = res_read_resp.result.expect("read result expected");
    assert!(read_contents["contents"][0]["text"].as_str().unwrap().contains("healthy"));

    // 7. prompts/list
    let prompts_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "prompts/list"
    });
    let prompts_resp = server
        .handle_message(&session_id, &prompts_req.to_string())
        .await
        .unwrap()
        .unwrap();
    let prompts_list = prompts_resp.result.expect("prompts list expected");
    let prompts = prompts_list["prompts"].as_array().unwrap();
    assert!(prompts.iter().any(|p| p["name"] == "diagnose_gateway_health"));
}

#[tokio::test]
async fn test_mcp_declarative_route_exposure() {
    let router_with_mcp = Router {
        host: Some("http://localhost".to_string()),
        port: Some(8080),
        backends: None,
        protocol: Protocol::Http,
        load_balancing_strategy: Default::default(),
        external_path: "/api/users/{id}".to_string(),
        internal_path: "/internal/users/{id}".to_string(),
        methods: vec!["GET".to_string()],
        auth_required: false,
        retry: None,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
        cache: None,
        mcp_tool: Some(McpToolConfig {
            enabled: true,
            name: Some("lookup_user".to_string()),
            description: Some("Look up user profile by identifier".to_string()),
            input_schema: None,
        }),
    };

    let router_auto_named = Router {
        host: Some("http://localhost".to_string()),
        port: Some(8081),
        backends: None,
        protocol: Protocol::Http,
        load_balancing_strategy: Default::default(),
        external_path: "/api/orders/summary".to_string(),
        internal_path: "/internal/orders/summary".to_string(),
        methods: vec!["POST".to_string()],
        auth_required: false,
        retry: None,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
        cache: None,
        mcp_tool: Some(McpToolConfig {
            enabled: true,
            name: None, // Auto-derive name from path
            description: None,
            input_schema: None,
        }),
    };

    let settings = create_test_settings(vec![router_with_mcp, router_auto_named]);
    let route_manager = Arc::new(RouteManager::new(settings, "dummy_config.json".to_string()));

    let server = McpServer::new(McpConfig::default())
        .with_route_manager(route_manager);

    let tools = server.collect_tools().await;

    // Check custom-named tool lookup_user
    let lookup_tool = tools
        .iter()
        .find(|t| t.name == "lookup_user")
        .expect("lookup_user tool must be exposed");
    assert_eq!(
        lookup_tool.description.as_deref(),
        Some("Look up user profile by identifier")
    );
    assert_eq!(lookup_tool.input_schema["type"], "object");
    assert!(lookup_tool.input_schema["properties"]["id"].is_object());
    assert_eq!(lookup_tool.input_schema["required"][0], "id");

    // Check auto-named tool
    let auto_tool = tools
        .iter()
        .find(|t| t.name == "post_api_orders_summary")
        .expect("auto-derived tool name must exist");
    assert_eq!(auto_tool.input_schema["type"], "object");
}

#[actix_web::test]
async fn test_mcp_actix_endpoints() {
    let settings = create_test_settings(vec![]);
    let mcp_cfg = settings.mcp.clone().unwrap();
    let server = McpServer::new(mcp_cfg);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(server.clone()))
            .configure(mcp::configure_mcp),
    )
    .await;

    // 1. GET /api/mcp/status
    let req = test::TestRequest::get().uri("/api/mcp/status").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["server_name"], "kairos-test");
    assert!(body["total_tools"].as_u64().unwrap() >= 6);

    // 2. GET /api/mcp/tools
    let req = test::TestRequest::get().uri("/api/mcp/tools").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["count"].as_u64().unwrap() >= 6);
    assert!(body["tools"].as_array().is_some());

    // 3. POST /api/mcp/tools/call
    let call_params = CallToolParams {
        name: "kairos_get_status".to_string(),
        arguments: Some(serde_json::json!({})),
    };
    let req = test::TestRequest::post()
        .uri("/api/mcp/tools/call")
        .set_json(&call_params)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["isError"], false);
    assert!(body["content"][0]["text"].as_str().unwrap().contains("healthy"));

    // 4. GET /mcp/sse
    let req = test::TestRequest::get().uri("/mcp/sse").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .unwrap(),
        "text/event-stream"
    );
    let session_header = resp
        .headers()
        .get("X-Mcp-Session-Id")
        .expect("X-Mcp-Session-Id must be set");
    let session_id = session_header.to_str().unwrap().to_string();

    // 5. POST /mcp/messages without sessionId -> 400 Bad Request
    let req = test::TestRequest::post()
        .uri("/mcp/messages")
        .set_payload("{}")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // 6. POST /mcp/messages with valid sessionId and JSON-RPC ping
    let ping_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-1",
        "method": "ping"
    });
    let req = test::TestRequest::post()
        .uri(&format!("/mcp/messages?sessionId={}", session_id))
        .set_payload(ping_payload.to_string())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 202); // 202 Accepted per spec
}

#[actix_web::test]
async fn test_mcp_disabled_returns_not_found() {
    let mut mcp_cfg = McpConfig::default();
    mcp_cfg.enabled = false;
    let server = McpServer::new(mcp_cfg);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(server))
            .configure(mcp::configure_mcp),
    )
    .await;

    let req = test::TestRequest::get().uri("/mcp/sse").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let req = test::TestRequest::post()
        .uri("/mcp/messages?sessionId=dummy")
        .set_payload("{}")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_rmcp_model_interop() {
    let kairos_tool = kairos_rs::models::mcp::Tool::new(
        "test_tool",
        Some("A test tool description".to_string()),
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"]
        }),
    );

    // Convert Kairos Tool -> rmcp Tool
    let rmcp_tool: kairos_rs::rmcp::model::Tool = kairos_tool.clone().into();
    assert_eq!(rmcp_tool.name.as_ref(), "test_tool");
    assert_eq!(rmcp_tool.description.as_deref(), Some("A test tool description"));
    assert!(rmcp_tool.input_schema.contains_key("properties"));

    // Convert rmcp Tool -> Kairos Tool
    let converted_back: kairos_rs::models::mcp::Tool = rmcp_tool.into();
    assert_eq!(converted_back.name, "test_tool");
    assert_eq!(converted_back.description.as_deref(), Some("A test tool description"));
    assert_eq!(converted_back.input_schema["properties"]["query"]["type"], "string");
}
