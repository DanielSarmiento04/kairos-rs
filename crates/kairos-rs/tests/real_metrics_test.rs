use actix_web::{test, web, App, middleware::Logger};
use kairos_rs::routes::{http, metrics};
use kairos_rs::services::http::RouteHandler;
use kairos_rs::models::router::{Router, Backend};
use kairos_rs::models::router::Protocol;
use std::time::Duration;

#[actix_web::test]
async fn test_real_metrics_collection() {
    // Create a route handler with test routes
    let routes = vec![
        Router {
            host: Some("http://localhost".to_string()),
            port: Some(8080),
            external_path: "/api/test".to_string(),
            internal_path: "/test".to_string(),
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
        }
    ];
    let route_handler = RouteHandler::new(routes, 30);
    
    // Create metrics collector
    let metrics_collector = metrics::MetricsCollector::default();

    // Create app with full configuration like main.rs
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(metrics_collector.clone()))
            .configure(metrics::configure_metrics)  // Configure metrics BEFORE catch-all routes
            .configure(|cfg| http::configure_route(cfg, route_handler))
            .wrap(Logger::default())
    ).await;

    // Make several requests to collect metrics
    for i in 0..5 {
        let req = test::TestRequest::get()
            .uri(&format!("/api/test?request={}", i))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        println!("Request {} status: {}", i, resp.status());
    }

    // Give time for metrics to be recorded
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check metrics endpoint using test framework
    let metrics_req = test::TestRequest::get()
        .uri("/metrics")
        .to_request();
    
    let metrics_resp = test::call_service(&app, metrics_req).await;
    assert_eq!(metrics_resp.status(), 200, "Metrics endpoint should be accessible");
    
    let metrics_body = test::read_body(metrics_resp).await;
    let metrics_text = String::from_utf8_lossy(&metrics_body);
    println!("Metrics response:\n{}", metrics_text);
    
    // Verify the metrics contain our requests
    assert!(metrics_text.contains("kairos_requests_total 5"));
    assert!(metrics_text.contains("kairos_requests_error_total 5")); // All failed due to no upstream
}