use bytes::Bytes;
use kairos_rs::models::router::{AiPolicy, AiRoutingStrategy, Backend, Protocol, Router};
use kairos_rs::services::http::RouteHandler;
use kairos_rs::services::semantic_cache::{
    cosine_similarity, extract_prompt, vectorize_local, SemanticCache, SemanticCacheConfig,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_semantic_cache_lifecycle_and_similarity() {
    let config = SemanticCacheConfig {
        enabled: true,
        similarity_threshold: 0.80,
        ttl_seconds: 3600,
        embedding_provider: "local".to_string(),
        embedding_model: "default".to_string(),
        max_entries: 3,
    };
    let cache = SemanticCache::new(config);

    // Initial stats
    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.current_entries, 0);
    assert_eq!(stats.hit_ratio, 0.0);

    // Insert entry 1
    let prompt1 = "How do I sort a vector in Rust?";
    let vec1 = vectorize_local(prompt1, 128);
    cache.insert(
        prompt1.to_string(),
        vec1.clone(),
        200,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        Bytes::from_static(b"{\"answer\":\"Use vec.sort()\"}"),
        None,
    );

    assert_eq!(cache.stats().current_entries, 1);

    // Exact query lookup -> HIT
    let hit1 = cache.lookup(&vec1, 0.80);
    assert!(hit1.is_some());
    let (entry1, sim1) = hit1.unwrap();
    assert_eq!(entry1.status, 200);
    assert!(sim1 >= 0.99);

    // Paraphrased query lookup -> HIT
    let prompt1_var = "how do I sort a vector in rust?";
    let vec1_var = vectorize_local(prompt1_var, 128);
    let hit_var = cache.lookup(&vec1_var, 0.80);
    assert!(hit_var.is_some());
    let (_, sim_var) = hit_var.unwrap();
    assert!(sim_var >= 0.90);

    // Unrelated query lookup -> MISS
    let prompt_unrelated = "What is the capital of France?";
    let vec_unrelated = vectorize_local(prompt_unrelated, 128);
    let miss = cache.lookup(&vec_unrelated, 0.80);
    assert!(miss.is_none());

    // Verify stats after lookups
    let stats = cache.stats();
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.misses, 1);
    assert!((stats.hit_ratio - 66.666).abs() < 1.0);

    // Capacity eviction test (max_entries: 3)
    let prompt2 = "How to open a file in Rust?";
    let vec2 = vectorize_local(prompt2, 128);
    cache.insert(prompt2.to_string(), vec2, 200, vec![], Bytes::from_static(b"File::open"), None);

    let prompt3 = "How to parse JSON in Rust?";
    let vec3 = vectorize_local(prompt3, 128);
    cache.insert(prompt3.to_string(), vec3, 200, vec![], Bytes::from_static(b"serde_json"), None);

    assert_eq!(cache.stats().current_entries, 3);

    // Inserting 4th item should evict oldest
    let prompt4 = "How to write async functions in Rust?";
    let vec4 = vectorize_local(prompt4, 128);
    cache.insert(prompt4.to_string(), vec4, 200, vec![], Bytes::from_static(b"async fn"), None);

    assert_eq!(cache.stats().current_entries, 3);
    assert!(cache.stats().evictions >= 1);

    // Clear cache
    cache.clear();
    assert_eq!(cache.stats().current_entries, 0);
}

#[tokio::test]
async fn test_semantic_cache_expiration() {
    let config = SemanticCacheConfig {
        enabled: true,
        similarity_threshold: 0.85,
        ttl_seconds: 1, // 1 second TTL
        ..Default::default()
    };
    let cache = SemanticCache::new(config);

    let prompt = "Quick expiring prompt";
    let vec = vectorize_local(prompt, 128);
    cache.insert(
        prompt.to_string(),
        vec.clone(),
        200,
        vec![],
        Bytes::from_static(b"expiring"),
        Some(Duration::from_millis(50)),
    );

    // Immediate lookup -> HIT
    assert!(cache.lookup(&vec, 0.85).is_some());

    // Sleep past TTL
    tokio::time::sleep(Duration::from_millis(70)).await;

    // Lookup after expiration -> MISS
    assert!(cache.lookup(&vec, 0.85).is_none());
}

#[tokio::test]
async fn test_route_handler_semantic_cache_fast_path() {
    let cache_cfg = SemanticCacheConfig {
        enabled: true,
        similarity_threshold: 0.80,
        ttl_seconds: 3600,
        embedding_provider: "local".to_string(),
        embedding_model: "default".to_string(),
        max_entries: 50,
    };
    let semantic_cache = Arc::new(SemanticCache::new(cache_cfg.clone()));

    // Pre-populate semantic cache with a known answer
    let cached_prompt = "How do I reverse a string in Rust?";
    let cached_vec = vectorize_local(cached_prompt, 128);
    semantic_cache.insert(
        cached_prompt.to_string(),
        cached_vec,
        200,
        vec![("Content-Type".to_string(), "application/json".to_string())],
        Bytes::from_static(b"{\"answer\":\"s.chars().rev().collect()\"}"),
        None,
    );

    let route = Router {
        host: Some("http://127.0.0.1".to_string()),
        port: Some(8080),
        external_path: "/api/chat".to_string(),
        internal_path: "/v1/chat".to_string(),
        methods: vec!["POST".to_string()],
        auth_required: false,
        backends: Some(vec![Backend {
            host: "http://127.0.0.1".to_string(),
            port: 8080,
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
            provider: Some("local".to_string()),
            fallback_backend_index: None,
            fallback_providers: Vec::new(),
            streaming: Some(false),
            semantic_cache: Some(cache_cfg),
        }),
        cache: None,
    };

    let handler = RouteHandler::new(vec![route], 5).with_semantic_cache(semantic_cache.clone());

    // Query with paraphrase: should hit semantic cache fast-path and NOT attempt upstream connection!
    let prompt_variation = r#"{"messages": [{"role": "user", "content": "how do I reverse a string in rust?"}]}"#;
    let req = actix_web::test::TestRequest::post()
        .uri("/api/chat")
        .insert_header(("content-type", "application/json"))
        .to_http_request();

    let resp = handler
        .handle_request(req, Bytes::from(prompt_variation))
        .await
        .expect("Fast-path cache hit should not fail");

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    assert_eq!(
        resp.headers().get("X-Cache").and_then(|v| v.to_str().ok()),
        Some("HIT")
    );
    assert_eq!(
        resp.headers().get("X-Semantic-Cache").and_then(|v| v.to_str().ok()),
        Some("HIT")
    );
    assert!(resp.headers().contains_key("X-Cache-Similarity"));

    // Verify response body matches pre-populated cache
    let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, Bytes::from_static(b"{\"answer\":\"s.chars().rev().collect()\"}"));

    // Verify stats
    assert_eq!(semantic_cache.stats().hits, 1);
}

#[test]
fn test_prompt_extraction_variations() {
    // 1. OpenAI chat messages
    let json1 = br#"{"messages": [{"role": "system", "content": "You are helpful"}, {"role": "user", "content": "Explain async/await"}]}"#;
    assert_eq!(extract_prompt(json1), Some("Explain async/await".to_string()));

    // 2. OpenAI single prompt
    let json2 = br#"{"prompt": "Generate a random number"}"#;
    assert_eq!(extract_prompt(json2), Some("Generate a random number".to_string()));

    // 3. Prompt array
    let json3 = br#"{"prompt": ["line 1", "line 2"]}"#;
    assert_eq!(extract_prompt(json3), Some("line 2".to_string()));

    // 4. Query field
    let json4 = br#"{"query": "Rust pattern matching"}"#;
    assert_eq!(extract_prompt(json4), Some("Rust pattern matching".to_string()));

    // 5. Input field
    let json5 = br#"{"input": "Search terms"}"#;
    assert_eq!(extract_prompt(json5), Some("Search terms".to_string()));

    // 6. Empty / invalid
    assert_eq!(extract_prompt(b""), None);
}

#[test]
fn test_cosine_similarity_edge_cases() {
    let empty: Vec<f32> = vec![];
    assert_eq!(cosine_similarity(&empty, &empty), 0.0);

    let zero = vec![0.0, 0.0, 0.0];
    let non_zero = vec![1.0, 2.0, 3.0];
    assert_eq!(cosine_similarity(&zero, &non_zero), 0.0);

    let diff_len_a = vec![1.0, 2.0];
    let diff_len_b = vec![1.0, 2.0, 3.0];
    assert_eq!(cosine_similarity(&diff_len_a, &diff_len_b), 0.0);
}
