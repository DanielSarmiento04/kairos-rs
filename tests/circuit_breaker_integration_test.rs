use actix_web::{test, web, App};
use kairos_rs::routes::{http, metrics};
use kairos_rs::services::http::RouteHandler;
use kairos_rs::models::router::Router;
use std::time::Duration;

#[actix_web::test]
async fn test_circuit_breaker_integration() {
    // Create a route handler with a test route pointing to non-existent service
    let routes = vec![
        Router {
            host: "http://non-existent-service".to_string(),
            port: 9999,  // Non-existent port
            external_path: "/api/test".to_string(),
            internal_path: "/test".to_string(),
            methods: vec!["GET".to_string()],
            auth_required: false,
        }
    ];
    let route_handler = RouteHandler::new(routes, 5); // 5 second timeout
    
    // Create metrics collector
    let metrics_collector = metrics::MetricsCollector::default();

    // Create app with full configuration
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(metrics_collector.clone()))
            .configure(metrics::configure_metrics)
            .configure(|cfg| http::configure_route(cfg, route_handler))
    ).await;

    println!("Starting circuit breaker integration test...");

    // Make several requests to trigger circuit breaker
    // The circuit breaker should open after 5 failures (default threshold)
    for i in 1..=7 {
        let req = test::TestRequest::get()
            .uri(&format!("/api/test?attempt={}", i))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        println!("Request {}: {}", i, status);
        
        if i <= 5 {
            // First 5 requests should be upstream errors (502)
            assert_eq!(status, 502, "Expected upstream error for request {}", i);
        } else {
            // After 5 failures, circuit should be open (503)
            assert_eq!(status, 503, "Expected circuit breaker open for request {}", i);
        }
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Check metrics to verify requests were recorded
    let metrics_req = test::TestRequest::get()
        .uri("/metrics")
        .to_request();
    
    let metrics_resp = test::call_service(&app, metrics_req).await;
    assert_eq!(metrics_resp.status(), 200);
    
    let metrics_body = test::read_body(metrics_resp).await;
    let metrics_text = String::from_utf8_lossy(&metrics_body);
    
    println!("Final metrics:\n{}", metrics_text);
    
    // Verify metrics contain our requests
    assert!(metrics_text.contains("kairos_requests_total 7"));
    assert!(metrics_text.contains("kairos_requests_error_total 7")); // All failed
    
    println!("Circuit breaker integration test completed successfully!");
}

#[actix_web::test]
async fn test_multiple_service_circuit_breakers() {
    // Create routes for multiple services to test independent circuit breakers
    let routes = vec![
        Router {
            host: "http://service-a".to_string(),
            port: 8001,
            external_path: "/api/service-a".to_string(),
            internal_path: "/test".to_string(),
            methods: vec!["GET".to_string()],
            auth_required: false,
        },
        Router {
            host: "http://service-b".to_string(),
            port: 8002,
            external_path: "/api/service-b".to_string(),
            internal_path: "/test".to_string(),
            methods: vec!["GET".to_string()],
            auth_required: false,
        }
    ];
    let route_handler = RouteHandler::new(routes, 5);
    
    let metrics_collector = metrics::MetricsCollector::default();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(metrics_collector.clone()))
            .configure(metrics::configure_metrics)
            .configure(|cfg| http::configure_route(cfg, route_handler))
    ).await;

    println!("Testing multiple service circuit breakers...");

    // Trigger failures for service A only
    for i in 1..=6 {
        let req = test::TestRequest::get()
            .uri(&format!("/api/service-a?attempt={}", i))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        println!("Service A request {}: {}", i, resp.status());
    }

    // Service B should still accept requests (circuit not opened)
    let req_b = test::TestRequest::get()
        .uri("/api/service-b")
        .to_request();
    
    let resp_b = test::call_service(&app, req_b).await;
    println!("Service B status: {}", resp_b.status());
    
    // Service B should return 502 (upstream error), not 503 (circuit open)
    assert_eq!(resp_b.status(), 502, "Service B circuit should still be closed");
    
    // Service A should now have circuit open
    let req_a = test::TestRequest::get()
        .uri("/api/service-a")
        .to_request();
    
    let resp_a = test::call_service(&app, req_a).await;
    println!("Service A after circuit open: {}", resp_a.status());
    assert_eq!(resp_a.status(), 503, "Service A circuit should be open");

    println!("Multiple service circuit breaker test completed!");
}