use actix_web::{test, web, App, HttpResponse, Responder};
use kairos_rs::middleware::admin_auth::{AdminAuth, AdminAuthConfig};

async fn dummy_routes() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "routes": [] }))
}

async fn dummy_public_health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy" }))
}

async fn dummy_verify() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "authenticated": true }))
}

#[actix_web::test]
async fn test_admin_auth_disabled_allows_all() {
    let config = AdminAuthConfig::new(false, vec!["my_key".to_string()]);
    let app = test::init_service(
        App::new()
            .wrap(AdminAuth::new(config))
            .route("/api/routes", web::get().to(dummy_routes))
            .route("/health", web::get().to(dummy_public_health)),
    )
    .await;

    // Both should succeed because admin auth is disabled
    let req = test::TestRequest::get().uri("/api/routes").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_admin_auth_enabled_blocks_unauthorized() {
    let config = AdminAuthConfig::new(true, vec!["super_secret_admin_key".to_string()]);
    let app = test::init_service(
        App::new()
            .wrap(AdminAuth::new(config))
            .route("/api/routes", web::get().to(dummy_routes))
            .route("/health", web::get().to(dummy_public_health))
            .route("/api/admin/verify", web::get().to(dummy_verify)),
    )
    .await;

    // Public health check should pass without credentials
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Protected route without key should return 401
    let req = test::TestRequest::get().uri("/api/routes").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Protected route with invalid key should return 401
    let req = test::TestRequest::get()
        .uri("/api/routes")
        .insert_header(("X-Admin-Key", "wrong_key"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Protected route with valid X-Admin-Key should succeed
    let req = test::TestRequest::get()
        .uri("/api/routes")
        .insert_header(("X-Admin-Key", "super_secret_admin_key"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Protected route with valid Authorization: Bearer <key> should succeed
    let req = test::TestRequest::get()
        .uri("/api/routes")
        .insert_header(("Authorization", "Bearer super_secret_admin_key"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Protected route with query parameter ?admin_key=<key> should succeed (WebSocket compatibility)
    let req = test::TestRequest::get()
        .uri("/api/routes?admin_key=super_secret_admin_key")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // CORS preflight OPTIONS request should pass through without credentials
    let req = test::TestRequest::default()
        .method(actix_web::http::Method::OPTIONS)
        .uri("/api/routes")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), 401);
}
