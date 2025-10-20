//! Integration tests for the Kairos-rs gateway
//! 
//! These tests verify end-to-end functionality including routing, authentication,
//! metrics collection, and error handling.

use actix_web::{test, web, App, HttpResponse, Result};
use kairos_rs::middleware::auth::{JwtAuth, JwtConfig, Claims, create_test_token};
use kairos_rs::models::router::Protocol;
use kairos_rs::routes::{health, metrics};
use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

async fn mock_upstream_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Hello from upstream service",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn failing_upstream_handler() -> Result<HttpResponse> {
    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
        "error": "Upstream service error"
    })))
}

#[actix_web::test]
async fn test_health_endpoints() {
    let app = test::init_service(
        App::new().configure(health::configure_health)
    ).await;

    // Test general health endpoint
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Test readiness endpoint
    let req = test::TestRequest::get().uri("/ready").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Test liveness endpoint
    let req = test::TestRequest::get().uri("/live").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_metrics_endpoint() {
    let metrics_collector = metrics::MetricsCollector::default();
    
    // Record some test metrics
    metrics_collector.record_request(true, Duration::from_millis(100), 200, Some(1024), Some(2048));
    metrics_collector.record_request(false, Duration::from_millis(200), 500, Some(512), Some(0));
    metrics_collector.increment_connections();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(metrics_collector))
            .configure(metrics::configure_metrics)
    ).await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), 200);
    
    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    
    // Verify Prometheus format
    assert!(body_str.contains("kairos_requests_total"));
    assert!(body_str.contains("kairos_requests_success_total"));
    assert!(body_str.contains("kairos_requests_error_total"));
    assert!(body_str.contains("kairos_response_time_avg"));
}

#[actix_web::test]
async fn test_jwt_authentication_success() {
    let secret = "test-secret-key";
    let config = JwtConfig::new(secret.to_string())
        .with_issuer("kairos-rs".to_string())
        .with_audience("api".to_string());
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: "test-user".to_string(),
        exp: now + 3600,
        iat: now,
        iss: Some("kairos-rs".to_string()),
        aud: Some("api".to_string()),
        roles: Some(vec!["user".to_string()]),
    };
    
    let token = create_test_token(claims, secret).unwrap();
    
    let app = test::init_service(
        App::new()
            .wrap(JwtAuth::new(config))
            .route("/protected", web::get().to(mock_upstream_handler))
    ).await;

    let req = test::TestRequest::get()
        .uri("/protected")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_jwt_authentication_failure() {
    let config = JwtConfig::new("test-secret-key".to_string());
    
    let app = test::init_service(
        App::new()
            .wrap(JwtAuth::new(config))
            .route("/protected", web::get().to(mock_upstream_handler))
    ).await;

    // Test missing token
    let req = test::TestRequest::get().uri("/protected").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test invalid token
    let req = test::TestRequest::get()
        .uri("/protected")
        .insert_header(("Authorization", "Bearer invalid-token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test malformed header
    let req = test::TestRequest::get()
        .uri("/protected")
        .insert_header(("Authorization", "InvalidFormat token"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_secs(1),
        reset_timeout: Duration::from_millis(100),
    };
    
    let cb = CircuitBreaker::new("test-service".to_string(), config);
    
    // Test successful operation
    let result = cb.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Test failures that should open the circuit
    let _ = cb.call(async { Err::<i32, &str>("error1") }).await;
    let _ = cb.call(async { Err::<i32, &str>("error2") }).await;
    
    // Circuit should now be open
    let result = cb.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_err());
    
    // Wait for reset timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Should transition to half-open and allow requests
    let result = cb.call(async { Ok::<i32, &str>(42) }).await;
    assert!(result.is_ok());
}

#[actix_web::test]
async fn test_error_handling() {
    let app = test::init_service(
        App::new()
            .route("/error", web::get().to(failing_upstream_handler))
    ).await;

    let req = test::TestRequest::get().uri("/error").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 500);
}

async fn body_reading_handler(body: web::Bytes) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Body received",
        "size": body.len()
    })))
}

#[actix_web::test]
async fn test_request_size_limits() {
    use actix_web::web::PayloadConfig;
    
    let app = test::init_service(
        App::new()
            .app_data(PayloadConfig::new(1024)) // 1KB limit
            .route("/upload", web::post().to(body_reading_handler))
    ).await;

    // Test small payload (should succeed)
    let small_payload = "x".repeat(500);
    let req = test::TestRequest::post()
        .uri("/upload")
        .insert_header(("content-type", "text/plain"))
        .set_payload(small_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Test large payload (should fail)
    let large_payload = "x".repeat(2048);
    let req = test::TestRequest::post()
        .uri("/upload")
        .insert_header(("content-type", "text/plain"))
        .set_payload(large_payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 413); // Payload Too Large
}

#[actix_web::test]
async fn test_cors_headers() {
    use kairos_rs::middleware::security::cors_headers;
    
    let app = test::init_service(
        App::new()
            .wrap(cors_headers())
            .route("/api/test", web::get().to(mock_upstream_handler))
    ).await;

    let req = test::TestRequest::get().uri("/api/test").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), 200);
    
    let headers = resp.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
    assert!(headers.contains_key("access-control-allow-methods"));
}

#[actix_web::test]
async fn test_security_headers() {
    use kairos_rs::middleware::security::security_headers;
    
    let app = test::init_service(
        App::new()
            .wrap(security_headers())
            .route("/api/test", web::get().to(mock_upstream_handler))
    ).await;

    let req = test::TestRequest::get().uri("/api/test").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), 200);
    
    let headers = resp.headers();
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-xss-protection"));
    assert!(headers.contains_key("strict-transport-security"));
}

#[tokio::test]
async fn test_concurrent_requests() {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    
    let metrics_collector = Arc::new(metrics::MetricsCollector::default());
    let semaphore = Arc::new(Semaphore::new(10)); // Limit concurrent requests
    
    let mut handles = Vec::new();
    
    for i in 0..100 {
        let metrics = metrics_collector.clone();
        let permit = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            
            // Simulate request processing
            let start = std::time::Instant::now();
            tokio::time::sleep(Duration::from_millis(10)).await;
            let duration = start.elapsed();
            
            metrics.record_request(
                i % 10 != 0, 
                duration, 
                if i % 10 != 0 { 200 } else { 500 }, 
                Some(1024), 
                Some(2048)
            ); // 90% success rate
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify metrics
    let total_requests = metrics_collector.requests_total.load(std::sync::atomic::Ordering::Relaxed);
    let success_requests = metrics_collector.requests_success.load(std::sync::atomic::Ordering::Relaxed);
    let error_requests = metrics_collector.requests_error.load(std::sync::atomic::Ordering::Relaxed);
    
    assert_eq!(total_requests, 100);
    assert_eq!(success_requests, 90);
    assert_eq!(error_requests, 10);
}