//! Integration tests for JWT authentication functionality.

use actix_web::{test, App};
use kairos_rs::{
    middleware::auth::{Claims, create_test_token},
    models::{settings::{Settings, JwtSettings}, router::{Router, Backend}},
    routes::auth_http,
    services::http::RouteHandler,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Test configuration for JWT authentication
fn create_test_jwt_config() -> JwtSettings {
    JwtSettings {
        secret: "test-secret-key-that-is-long-enough-for-security-requirements".to_string(),
        issuer: Some("kairos-gateway".to_string()),
        audience: Some("api-clients".to_string()),
        required_claims: vec!["sub".to_string(), "exp".to_string()],
    }
}

/// Test configuration with protected and public routes
fn create_test_settings() -> Settings {
    Settings {
        version: 1,
        jwt: Some(create_test_jwt_config()),
        rate_limit: None,
        routers: vec![
            // Public route - no authentication required
            Router {
                host: Some("http://httpbin.org".to_string()),
                port: Some(80),
                external_path: "/public/test".to_string(),
                internal_path: "/status/200".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "http://httpbin.org".to_string(),
                    port: 80,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
            },
            // Protected route - authentication required
            Router {
                host: Some("http://httpbin.org".to_string()),
                port: Some(80),
                external_path: "/protected/user/{id}".to_string(),
                internal_path: "/json".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: true,
                backends: Some(vec![Backend {
                    host: "http://httpbin.org".to_string(),
                    port: 80,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
            },
        ],
    }
}

#[actix_web::test]
async fn test_public_route_no_auth_required() {
    let settings = create_test_settings();
    let route_handler = RouteHandler::new(settings.routers.clone(), 30);

    let app = test::init_service(
        App::new()
            .configure(|cfg| auth_http::configure_auth_routes(cfg, route_handler, &settings))
    ).await;

    let req = test::TestRequest::get()
        .uri("/public/test")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    println!("Response status: {}", status);
    
    // Let's also try to get the response body to see what error we're getting
    let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    println!("Response body: {}", body_str);
    
    // The request should be processed (though it may fail due to external dependency)
    // We're testing that no authentication error occurs
    // Since this tries to reach an external service, it might fail with 500 or other errors
    // but it should NOT fail with 401 (authentication error)
    assert!(
        status != 401,
        "Expected non-401 status for public route, got 401. This indicates authentication middleware is incorrectly applied to public routes."
    );
}

#[actix_web::test]
async fn test_protected_route_missing_auth() {
    let settings = create_test_settings();
    let route_handler = RouteHandler::new(settings.routers.clone(), 30);

    let app = test::init_service(
        App::new()
            .configure(|cfg| auth_http::configure_auth_routes(cfg, route_handler, &settings))
    ).await;

    let req = test::TestRequest::get()
        .uri("/protected/user/123")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should return 401 Unauthorized due to missing authentication
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_protected_route_invalid_token() {
    let settings = create_test_settings();
    let route_handler = RouteHandler::new(settings.routers.clone(), 30);

    let app = test::init_service(
        App::new()
            .configure(|cfg| auth_http::configure_auth_routes(cfg, route_handler, &settings))
    ).await;

    let req = test::TestRequest::get()
        .uri("/protected/user/123")
        .insert_header(("Authorization", "Bearer invalid-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should return 401 Unauthorized due to invalid token
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_protected_route_valid_token() {
    let settings = create_test_settings();
    let jwt_settings = settings.jwt.as_ref().unwrap();
    let route_handler = RouteHandler::new(settings.routers.clone(), 30);

    // Create a valid JWT token
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let claims = Claims {
        sub: "test-user".to_string(),
        exp: now + 3600, // 1 hour from now
        iat: now,
        iss: jwt_settings.issuer.clone(),
        aud: jwt_settings.audience.clone(),
        roles: None,
    };

    let token = create_test_token(claims, &jwt_settings.secret).unwrap();

    let app = test::init_service(
        App::new()
            .configure(|cfg| auth_http::configure_auth_routes(cfg, route_handler, &settings))
    ).await;

    let req = test::TestRequest::get()
        .uri("/protected/user/123")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should not return 401 Unauthorized (token is valid)
    assert_ne!(resp.status(), 401);
}

#[actix_web::test]
async fn test_jwt_config_validation() {
    // Test with missing JWT configuration when auth is required
    let invalid_settings = Settings {
        version: 1,
        jwt: None,
        rate_limit: None,
        routers: vec![
            Router {
                host: Some("http://example.com".to_string()),
                port: Some(80),
                external_path: "/protected".to_string(),
                internal_path: "/protected".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: true,
                backends: Some(vec![Backend {
                    host: "http://example.com".to_string(),
                    port: 80,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
            }
        ],
    };

    // Should fail validation
    assert!(invalid_settings.validate().is_err());

    // Test with valid JWT configuration
    let valid_settings = create_test_settings();
    assert!(valid_settings.validate().is_ok());
}

#[actix_web::test]
async fn test_jwt_secret_validation() {
    // Test with weak JWT secret
    let weak_secret_settings = Settings {
        version: 1,
        jwt: Some(JwtSettings {
            secret: "short".to_string(), // Too short
            issuer: None,
            audience: None,
            required_claims: vec![],
        }),
        rate_limit: None,
        routers: vec![
            Router {
                host: Some("http://example.com".to_string()),
                port: Some(80),
                external_path: "/protected".to_string(),
                internal_path: "/protected".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: true,
                backends: Some(vec![Backend {
                    host: "http://example.com".to_string(),
                    port: 80,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
            }
        ],
    };

    // Should fail validation due to weak secret
    assert!(weak_secret_settings.validate().is_err());

    // Test with default secret (should also fail)
    let default_secret_settings = Settings {
        version: 1,
        jwt: Some(JwtSettings {
            secret: "please-change-this-secret".to_string(),
            issuer: None,
            audience: None,
            required_claims: vec![],
        }),
        rate_limit: None,
        routers: vec![
            Router {
                host: Some("http://example.com".to_string()),
                port: Some(80),
                external_path: "/protected".to_string(),
                internal_path: "/protected".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: true,
                backends: Some(vec![Backend {
                    host: "http://example.com".to_string(),
                    port: 80,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
            }
        ],
    };

    // Should fail validation due to default secret
    assert!(default_secret_settings.validate().is_err());
}