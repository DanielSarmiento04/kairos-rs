use kairos_rs::middleware::rate_limit::{
    AdvancedRateLimit, LimitStrategy, RateLimitConfig, WindowType,
};
use kairos_rs::models::settings::RedisConfig;
use kairos_rs::services::cache::ResponseCache;
use std::time::Duration;

#[test]
fn test_redis_config_defaults_and_serde() {
    let default_cfg = RedisConfig::default();
    assert!(default_cfg.enabled);
    assert_eq!(default_cfg.url, "redis://127.0.0.1:6379");
    assert_eq!(default_cfg.pool_size, 16);
    assert_eq!(default_cfg.timeout_ms, 1500);
    assert!(default_cfg.fallback_to_memory);

    let json = serde_json::to_string(&default_cfg).expect("Serialization failed");
    let deserialized: RedisConfig =
        serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(deserialized.url, "redis://127.0.0.1:6379");
    assert_eq!(deserialized.pool_size, 16);
    assert!(deserialized.fallback_to_memory);
}

#[tokio::test]
async fn test_rate_limit_redis_fallback_to_memory() {
    // Configure rate limit with Redis enabled
    let config = RateLimitConfig {
        strategy: LimitStrategy::PerIP,
        requests_per_window: 2,
        window_duration: Duration::from_secs(60),
        burst_allowance: 0,
        window_type: WindowType::SlidingWindow,
        enable_redis: true,
        redis_key_prefix: "test_rl".to_string(),
    };

    let rate_limiter = AdvancedRateLimit::new(config);

    // Test service wrapping with AdvancedRateLimit middleware
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .wrap(rate_limiter)
            .default_service(actix_web::web::to(|| async {
                actix_web::HttpResponse::Ok().body("OK")
            })),
    )
    .await;

    // First request should be allowed (200)
    let req1 = actix_web::test::TestRequest::get().uri("/test").to_request();
    let resp1 = actix_web::test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), actix_web::http::StatusCode::OK);

    // Second request should be allowed (200)
    let req2 = actix_web::test::TestRequest::get().uri("/test").to_request();
    let resp2 = actix_web::test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), actix_web::http::StatusCode::OK);

    // Third request should be rate limited (429 Too Many Requests)
    let req3 = actix_web::test::TestRequest::get().uri("/test").to_request();
    let resp3_res = actix_web::test::try_call_service(&app, req3).await;
    let status = match resp3_res {
        Ok(resp) => resp.status(),
        Err(err) => err.error_response().status(),
    };
    assert_eq!(status, actix_web::http::StatusCode::TOO_MANY_REQUESTS);
}

#[test]
fn test_response_cache_with_redis_builder() {
    let cache = ResponseCache::new(100, Duration::from_secs(30));
    assert_eq!(cache.stats().current_entries, 0);

    // Verify cache store and get
    let headers = vec![("Content-Type".to_string(), "text/plain".to_string())];
    cache.put(
        "GET",
        "/test/resource",
        200,
        &headers,
        bytes::Bytes::from_static(b"Hello Cache"),
        None,
    );

    assert_eq!(cache.stats().current_entries, 1);
    let hit = cache.get("GET", "/test/resource");
    assert!(hit.is_some());
    assert_eq!(hit.unwrap().body, bytes::Bytes::from_static(b"Hello Cache"));

    // Invalidation
    cache.invalidate("/test/resource");
    assert!(cache.get("GET", "/test/resource").is_none());
}
