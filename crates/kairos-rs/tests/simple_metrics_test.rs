use actix_web::{test, web, App};
use kairos_rs::routes::metrics;
use kairos_rs::models::router::Protocol;

#[actix_web::test]
async fn test_metrics_endpoint_basic() {
    // Create metrics collector
    let metrics_collector = metrics::MetricsCollector::default();

    // Create simple app with just metrics
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(metrics_collector.clone()))
            .configure(metrics::configure_metrics)
    ).await;

    // Test metrics endpoint
    let req = test::TestRequest::get()
        .uri("/metrics")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    println!("Metrics endpoint status: {}", resp.status());
    
    assert_eq!(resp.status(), 200, "Metrics endpoint should be accessible");
    
    let body = test::read_body(resp).await;
    let metrics_text = String::from_utf8_lossy(&body);
    println!("Metrics response:\n{}", metrics_text);
    
    // Basic metrics should be present
    assert!(metrics_text.contains("kairos_requests_total"));
    assert!(metrics_text.contains("kairos_uptime_seconds"));
}