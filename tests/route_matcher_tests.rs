use ben::models::router::Router;
use ben::utils::route_matcher::{RouteMatcher, RouteMatchError};

/// Helper function to create test routes
fn create_test_routes() -> Vec<Router> {
    vec![
        Router {
            host: "http://localhost".to_string(),
            port: 3000,
            external_path: "/api/identity/register/v3".to_string(),
            internal_path: "/api/identity/register".to_string(),
            methods: vec!["POST".to_string(), "GET".to_string()],
        },
        Router {
            host: "https://google.com".to_string(),
            port: 443,
            external_path: "/identity/register/v2".to_string(),
            internal_path: "/".to_string(),
            methods: vec!["POST".to_string(), "GET".to_string()],
        },
        Router {
            host: "https://http.cat".to_string(),
            port: 443,
            external_path: "/cats/{id}".to_string(),
            internal_path: "/{id}".to_string(),
            methods: vec!["GET".to_string()],
        },
        Router {
            host: "http://api.example.com".to_string(),
            port: 80,
            external_path: "/api/users/{user_id}".to_string(),
            internal_path: "/users/{user_id}".to_string(),
            methods: vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
        },
        Router {
            host: "http://api.example.com".to_string(),
            port: 80,
            external_path: "/api/products/{product_id}/details".to_string(),
            internal_path: "/products/{product_id}/info".to_string(),
            methods: vec!["GET".to_string()],
        },
        Router {
            host: "http://api.example.com".to_string(),
            port: 80,
            external_path: "/api/orders/{order_id}/items/{item_id}".to_string(),
            internal_path: "/orders/{order_id}/items/{item_id}".to_string(),
            methods: vec!["GET".to_string(), "PUT".to_string()],
        },
        Router {
            host: "http://static.example.com".to_string(),
            port: 80,
            external_path: "/api/static/path".to_string(),
            internal_path: "/static".to_string(),
            methods: vec!["GET".to_string()],
        },
        Router {
            host: "http://static.example.com".to_string(),
            port: 80,
            external_path: "/api/static/path/details".to_string(),
            internal_path: "/static/details".to_string(),
            methods: vec!["GET".to_string()],
        },
    ]
}

/// Create a route matcher for testing
fn create_route_matcher() -> RouteMatcher {
    RouteMatcher::new(create_test_routes()).expect("Failed to create route matcher")
}

#[cfg(test)]
mod route_matcher_tests {
    use super::*;

    #[test]
    fn test_new_route_matcher_creation() {
        let routes = create_test_routes();
        let matcher = RouteMatcher::new(routes).unwrap();
        
        // Static routes:
        // - "/api/identity/register/v3"
        // - "/identity/register/v2" 
        // - "/api/static/path"
        // - "/api/static/path/details"
        // 
        // Dynamic routes:
        // - "/cats/{id}"
        // - "/api/users/{user_id}"
        // - "/api/products/{product_id}/details"
        // - "/api/orders/{order_id}/items/{item_id}"
        
        assert_eq!(matcher.static_routes_count(), 4); // 4 static routes
        assert_eq!(matcher.dynamic_routes_count(), 4); // 4 dynamic routes
    }

    #[test]
    fn test_static_route_matching() {
        let matcher = create_route_matcher();

        // Test exact match for static route
        let result = matcher.find_match("/api/identity/register/v3");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/identity/register/v3");
        assert_eq!(internal_path, "/api/identity/register");
        assert_eq!(route.host, "http://localhost");
    }

    #[test]
    fn test_single_parameter_replacement() {
        let matcher = create_route_matcher();

        // Test single parameter route
        let result = matcher.find_match("/cats/200");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/cats/{id}");
        assert_eq!(internal_path, "/200");
        assert_eq!(route.host, "https://http.cat");

        // Test user ID route
        let result = matcher.find_match("/api/users/123");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/users/{user_id}");
        assert_eq!(internal_path, "/users/123");
    }

    #[test]
    fn test_multiple_parameter_replacement() {
        let matcher = create_route_matcher();

        // Test multiple parameters
        let result = matcher.find_match("/api/orders/123/items/456");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/orders/{order_id}/items/{item_id}");
        assert_eq!(internal_path, "/orders/123/items/456");
    }

    #[test]
    fn test_product_details_route() {
        let matcher = create_route_matcher();

        // Test product details route
        let result = matcher.find_match("/api/products/abc123/details");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/products/{product_id}/details");
        assert_eq!(internal_path, "/products/abc123/info");
    }

    #[test]
    fn test_non_matching_routes() {
        let matcher = create_route_matcher();

        // Test non-matching route
        let result = matcher.find_match("/api/nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RouteMatchError::NoMatch { .. }));

        // Test partial match that shouldn't work
        let result = matcher.find_match("/api/users");
        assert!(result.is_err());

        // Test route with extra segments
        let result = matcher.find_match("/api/users/123/extra");
        assert!(result.is_err());
    }

    #[test]
    fn test_static_routes_priority() {
        let matcher = create_route_matcher();

        // Static routes should be matched before dynamic ones
        let result = matcher.find_match("/api/static/path");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/static/path");
        assert_eq!(internal_path, "/static");

        let result = matcher.find_match("/api/static/path/details");
        assert!(result.is_ok());
        
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/static/path/details");
        assert_eq!(internal_path, "/static/details");
    }

    #[test]
    fn test_edge_cases() {
        let matcher = create_route_matcher();

        // Test empty path
        let result = matcher.find_match("");
        assert!(result.is_err());

        // Test root path
        let result = matcher.find_match("/");
        assert!(result.is_err());

        // Test path with special characters in parameter
        let result = matcher.find_match("/cats/test-123_abc");
        assert!(result.is_ok());
        let (_, internal_path) = result.unwrap();
        assert_eq!(internal_path, "/test-123_abc");

        // Test path with encoded characters
        let result = matcher.find_match("/cats/test%20space");
        assert!(result.is_ok());
        let (_, internal_path) = result.unwrap();
        assert_eq!(internal_path, "/test%20space");
    }

    #[test]
    fn test_invalid_route_patterns() {
        let invalid_routes = vec![
            Router {
                host: "http://localhost".to_string(),
                port: 3000,
                external_path: "/api/users/{user_id".to_string(), // Missing closing brace
                internal_path: "/users/{user_id}".to_string(),
                methods: vec!["GET".to_string()],
            },
            Router {
                host: "http://localhost".to_string(),
                port: 3000,
                external_path: "/api/users/{user id}".to_string(), // Space in parameter name
                internal_path: "/users/{user_id}".to_string(),
                methods: vec!["GET".to_string()],
            },
            Router {
                host: "http://localhost".to_string(),
                port: 3000,
                external_path: "/api/users/{}".to_string(), // Empty parameter name
                internal_path: "/users/{}".to_string(),
                methods: vec!["GET".to_string()],
            },
        ];

        for invalid_route in invalid_routes {
            let result = RouteMatcher::new(vec![invalid_route]);
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), RouteMatchError::InvalidPattern { .. }));
        }
    }

    #[test]
    fn test_error_types() {
        let matcher = create_route_matcher();

        // Test NoMatch error
        let result = matcher.find_match("/nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            RouteMatchError::NoMatch { path } => {
                assert_eq!(path, "/nonexistent");
            }
            _ => panic!("Expected NoMatch error"),
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_static_route_performance() {
        let matcher = create_route_matcher();
        let iterations = 10_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = matcher.find_match("/api/static/path");
        }
        let duration = start.elapsed();

        println!("Static route matching: {} iterations in {:?}", iterations, duration);
        assert!(duration.as_millis() < 100); // Should be very fast
    }

    #[test]
    fn test_dynamic_route_performance() {
        let matcher = create_route_matcher();
        let iterations = 10_000;

        let start = Instant::now();
        for i in 0..iterations {
            let _ = matcher.find_match(&format!("/cats/{}", i));
        }
        let duration = start.elapsed();

        println!("Dynamic route matching: {} iterations in {:?}", iterations, duration);
        assert!(duration.as_millis() < 500); // Should still be reasonably fast
    }

    #[test]
    fn test_complex_pattern_performance() {
        let matcher = create_route_matcher();
        let iterations = 10_000;

        let start = Instant::now();
        for i in 0..iterations {
            let _ = matcher.find_match(&format!("/api/orders/{}/items/{}", i, i * 2));
        }
        let duration = start.elapsed();

        println!("Complex pattern matching: {} iterations in {:?}", iterations, duration);
        assert!(duration.as_millis() < 1000);
    }
}
