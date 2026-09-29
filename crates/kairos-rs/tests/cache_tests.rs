use actix_web::{test, web, App};
use kairos_rs::models::router::{Backend, Protocol, Router};
use kairos_rs::routes::management::{clear_cache, get_cache_stats};
use kairos_rs::services::cache::ResponseCache;
use kairos_rs::services::http::RouteHandler;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_response_cache_put_get_and_expiration() {
    let cache = ResponseCache::new(50, Duration::from_millis(100));

    let headers = vec![("content-type".to_string(), "application/json".to_string())];
    let body = actix_web::web::Bytes::from_static(b"{\"status\":\"cached\"}");

    // Put into cache
    cache.put("GET", "/api/data", 200, &headers, body.clone(), None);

    // Immediate get should hit
    let hit = cache.get("GET", "/api/data");
    assert!(hit.is_some());
    let hit_resp = hit.unwrap();
    assert_eq!(hit_resp.status, 200);
    assert_eq!(hit_resp.body, body);

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.hit_ratio, 100.0);
    assert_eq!(stats.current_entries, 1);

    // Sleep past TTL
    tokio::time::sleep(Duration::from_millis(120)).await;

    // Subsequent get should miss
    let expired = cache.get("GET", "/api/data");
    assert!(expired.is_none());

    let stats2 = cache.stats();
    assert_eq!(stats2.hits, 1);
    assert_eq!(stats2.misses, 1);
    assert_eq!(stats2.hit_ratio, 50.0);
}

#[tokio::test]
async fn test_response_cache_invalidation_and_clear() {
    let cache = ResponseCache::new(100, Duration::from_secs(60));

    let headers = vec![("content-type".to_string(), "application/json".to_string())];
    let body = actix_web::web::Bytes::from_static(b"{\"item\":1}");

    cache.put("GET", "/items/1", 200, &headers, body.clone(), None);
    cache.put("GET", "/items/2", 200, &headers, body.clone(), None);

    assert_eq!(cache.stats().current_entries, 2);

    // Invalidate one item
    let invalidated = cache.invalidate("/items/1");
    assert!(invalidated);
    assert!(cache.get("GET", "/items/1").is_none());
    assert!(cache.get("GET", "/items/2").is_some());

    // Clear all
    cache.clear();
    assert_eq!(cache.stats().current_entries, 0);
    assert!(cache.get("GET", "/items/2").is_none());
}

#[tokio::test]
async fn test_response_cache_capacity_eviction() {
    let max_entries = 10;
    let cache = ResponseCache::new(max_entries, Duration::from_secs(60));
    let headers = vec![];
    let body = actix_web::web::Bytes::from_static(b"ok");

    // Insert more than max capacity
    for i in 0..15 {
        cache.put("GET", &format!("/item/{}", i), 200, &headers, body.clone(), None);
    }

    let stats = cache.stats();
    assert!(stats.current_entries <= max_entries);
    assert!(stats.evictions > 0);
}

#[tokio::test]
async fn test_cache_management_endpoints() {
    let cache = Arc::new(ResponseCache::new(100, Duration::from_secs(60)));
    let headers = vec![("x-custom".to_string(), "val".to_string())];
    cache.put("GET", "/test/item", 200, &headers, actix_web::web::Bytes::from_static(b"test"), None);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .service(get_cache_stats)
            .service(clear_cache),
    )
    .await;

    // Test GET /api/cache/stats
    let req = test::TestRequest::get().uri("/api/cache/stats").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["current_entries"], 1);

    // Test POST /api/cache/clear
    let req_clear = test::TestRequest::post().uri("/api/cache/clear").to_request();
    let resp_clear = test::call_service(&app, req_clear).await;
    assert!(resp_clear.status().is_success());

    assert_eq!(cache.stats().current_entries, 0);
}

#[tokio::test]
async fn test_route_handler_cache_integration() {
    let routes = vec![Router {
        host: Some("http://localhost".to_string()),
        port: Some(8080),
        external_path: "/cached-route".to_string(),
        internal_path: "/internal/cached".to_string(),
        methods: vec!["GET".to_string()],
        auth_required: false,
        backends: Some(vec![Backend {
            host: "http://localhost".to_string(),
            port: 8080,
            weight: 1,
            health_check_path: None,
        }]),
        load_balancing_strategy: Default::default(),
        retry: None,
        protocol: Protocol::Http,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
    }];

    let cache = Arc::new(ResponseCache::new(100, Duration::from_secs(60)));
    let handler = RouteHandler::new(routes, 5).with_cache(cache.clone());

    // Pre-populate cache directly
    let body = actix_web::web::Bytes::from_static(b"{\"cached\":true}");
    let headers = vec![("content-type".to_string(), "application/json".to_string())];
    cache.put("GET", "/cached-route", 200, &headers, body.clone(), None);

    // Call handler with GET request
    let req = test::TestRequest::get().uri("/cached-route").to_http_request();
    let resp = handler.handle_request(req, actix_web::web::Bytes::new()).await;

    assert!(resp.is_ok());
    let http_resp = resp.unwrap();
    assert_eq!(http_resp.status(), actix_web::http::StatusCode::OK);
    assert_eq!(
        http_resp.headers().get("X-Cache").unwrap().to_str().unwrap(),
        "HIT"
    );

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
}
