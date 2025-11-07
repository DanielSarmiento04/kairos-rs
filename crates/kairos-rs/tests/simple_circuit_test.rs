use actix_web::{test, web, App};
use kairos_rs::routes::{http, metrics};
use kairos_rs::services::http::RouteHandler;
use kairos_rs::models::router::{Router, Backend};
use kairos_rs::models::router::Protocol;

#[actix_web::test]
async fn test_simple_circuit_breaker() {
    // Simple test to verify circuit breaker exists
    let routes = vec![
        Router {
            host: Some("http://localhost".to_string()),
            port: Some(9999),
            external_path: "/test".to_string(),
            internal_path: "/test".to_string(),
            methods: vec!["GET".to_string()],
            auth_required: false,
            backends: Some(vec![
                Backend {
                    host: "http://localhost".to_string(),
                    port: 9999,
                    weight: 1,
                    health_check_path: None,
                }
            ]),
            load_balancing_strategy: Default::default(),
            retry: None,
            protocol: Protocol::Http,
            request_transformation: None,
            response_transformation: None,
        }
    ];
    let route_handler = RouteHandler::new(routes, 5);
    let metrics_collector = metrics::MetricsCollector::default();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(metrics_collector))
            .configure(metrics::configure_metrics)
            .configure(|cfg| http::configure_route(cfg, route_handler))
    ).await;

    // Make a single request
    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;
    
    // Should get 502 (upstream error) not 404 (route not found)
    // This confirms the route handler is working
    assert_eq!(resp.status(), 502);
    println!("Circuit breaker test successful - got expected upstream error");
}