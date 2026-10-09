use actix_web::{web, web::Bytes, App, HttpResponse, HttpServer};
use futures_util::stream;
use kairos_rs::models::router::{
    AiPolicy, AiRoutingStrategy, Backend, Protocol, Router,
};
use kairos_rs::models::settings::{AiProviderConfig, AiSettings};
use kairos_rs::services::ai::AiService;
use kairos_rs::services::cache::ResponseCache;
use kairos_rs::services::http::RouteHandler;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_sse_streaming_passthrough() {
    // 1. Start mock upstream server serving an SSE stream
    let server = HttpServer::new(|| {
        App::new().route(
            "/v1/chat/stream",
            web::post().to(|| async {
                let chunks = vec![
                    Ok::<_, actix_web::Error>(Bytes::from_static(b"data: {\"token\":\"Hello\"}\n\n")),
                    Ok::<_, actix_web::Error>(Bytes::from_static(b"data: {\"token\":\" world\"}\n\n")),
                    Ok::<_, actix_web::Error>(Bytes::from_static(b"data: [DONE]\n\n")),
                ];
                let stream = stream::iter(chunks);
                HttpResponse::Ok()
                    .content_type("text/event-stream")
                    .streaming(stream)
            }),
        )
    })
    .bind("127.0.0.1:0")
    .expect("Failed to bind mock upstream server");

    let port = server.addrs()[0].port();
    let server_handle = tokio::spawn(server.run());

    // Allow mock server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 2. Configure Kairos RouteHandler pointing to mock server
    let routes = vec![Router {
        host: Some("http://127.0.0.1".to_string()),
        port: Some(port),
        external_path: "/api/chat/stream".to_string(),
        internal_path: "/v1/chat/stream".to_string(),
        methods: vec!["POST".to_string()],
        auth_required: false,
        backends: Some(vec![Backend {
            host: "http://127.0.0.1".to_string(),
            port,
            weight: 1,
            health_check_path: None,
        }]),
        load_balancing_strategy: Default::default(),
        retry: None,
        protocol: Protocol::Http,
        request_transformation: None,
        response_transformation: None,
        ai_policy: Some(AiPolicy {
            enabled: true,
            strategy: AiRoutingStrategy::ContentAnalysis { model: None },
            provider: None,
            fallback_backend_index: None,
            fallback_providers: Vec::new(),
            streaming: Some(true),
        }),
    }];

    let cache = Arc::new(ResponseCache::new(50, Duration::from_secs(60)));
    let handler = RouteHandler::new(routes, 5).with_cache(cache.clone());

    // 3. Send client request
    let req = actix_web::test::TestRequest::post()
        .uri("/api/chat/stream")
        .insert_header(("content-type", "application/json"))
        .to_http_request();

    let resp_res = handler
        .handle_request(req, Bytes::from_static(b"{\"prompt\":\"hi\"}"))
        .await;

    assert!(resp_res.is_ok(), "Request handling failed");
    let resp = resp_res.unwrap();

    // 4. Verify streaming headers
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    assert_eq!(
        resp.headers().get("X-Streaming").map(|v| v.to_str().unwrap()),
        Some("true")
    );
    assert_eq!(
        resp.headers().get("X-Accel-Buffering").map(|v| v.to_str().unwrap()),
        Some("no")
    );
    assert!(
        resp.headers()
            .get("Cache-Control")
            .map(|v| v.to_str().unwrap())
            .unwrap()
            .contains("no-cache")
    );

    // 5. Verify ResponseCache was bypassed for streaming
    assert!(
        resp.headers().get("X-Cache").is_none(),
        "X-Cache header should not be present on streaming responses"
    );
    assert!(
        cache.get("POST", "/api/chat/stream").is_none(),
        "Streaming response should not be saved into ResponseCache"
    );

    // 6. Read stream body
    let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);

    assert!(body_str.contains("data: {\"token\":\"Hello\"}\n\n"));
    assert!(body_str.contains("data: {\"token\":\" world\"}\n\n"));
    assert!(body_str.contains("data: [DONE]\n\n"));

    // Cleanup
    server_handle.abort();
}

#[tokio::test]
async fn test_ai_multi_provider_failover_structure() {
    let settings = AiSettings {
        provider: "unsupported_primary".to_string(),
        model: "model-1".to_string(),
        api_key: Some("key-1".to_string()),
        fallback_providers: vec![
            AiProviderConfig {
                provider: "unsupported_fallback_1".to_string(),
                model: "model-2".to_string(),
                api_key: Some("key-2".to_string()),
            },
            AiProviderConfig {
                provider: "unsupported_fallback_2".to_string(),
                model: "model-3".to_string(),
                api_key: None,
            },
        ],
    };

    let service = AiService::new(settings);

    // Attempting a prompt with unsupported providers will try primary, then fallbacks in sequence
    let result = service.ask("Hello").await;
    assert!(result.is_err());

    // Verify error reflects that all providers failed
    match result.unwrap_err() {
        kairos_rs::services::ai::AiServiceError::UnsupportedProvider(p) => {
            assert_eq!(p, "unsupported_primary");
        }
        other => panic!("Expected UnsupportedProvider, got {:?}", other),
    }
}

#[test]
fn test_ai_policy_serialization_with_streaming_and_fallbacks() {
    let policy = AiPolicy {
        enabled: true,
        strategy: AiRoutingStrategy::ContentAnalysis { model: None },
        provider: Some("openai".to_string()),
        fallback_backend_index: Some(1),
        fallback_providers: vec![
            AiProviderConfig {
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                api_key: None,
            },
            AiProviderConfig {
                provider: "groq".to_string(),
                model: "llama-3.3-70b-versatile".to_string(),
                api_key: None,
            },
        ],
        streaming: Some(true),
    };

    let serialized = serde_json::to_string(&policy).expect("Failed to serialize AiPolicy");
    let deserialized: AiPolicy =
        serde_json::from_str(&serialized).expect("Failed to deserialize AiPolicy");

    assert!(deserialized.enabled);
    assert_eq!(deserialized.fallback_providers.len(), 2);
    assert_eq!(deserialized.fallback_providers[0].provider, "anthropic");
    assert_eq!(deserialized.fallback_providers[1].provider, "groq");
    assert_eq!(deserialized.streaming, Some(true));
}
