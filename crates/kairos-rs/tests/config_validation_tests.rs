//! Integration tests for configuration validation functionality.
//! 
//! This module contains comprehensive tests for the ConfigValidator and related
//! validation functionality, ensuring proper security checks, performance
//! recommendations, and detailed error reporting.

use kairos_rs::config::validation::{ConfigValidator, ValidationResult};
use kairos_rs::models::settings::Settings;
use kairos_rs::models::router::{Router, Backend};
use kairos_rs::models::router::Protocol;

fn create_test_router(host: &str, external_path: &str, methods: Vec<&str>) -> Router {
    Router {
        host: Some(host.to_string()),
        port: Some(80),
        external_path: external_path.to_string(),
        internal_path: "/test".to_string(),
        methods: methods.iter().map(|s| s.to_string()).collect(),
        auth_required: false,
        backends: Some(vec![Backend {
            host: host.to_string(),
            port: 80,
            weight: 1,
            health_check_path: None,
        }]),
        load_balancing_strategy: Default::default(),
        retry: None,
        protocol: Protocol::Http,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
    }
}

#[test]
fn test_empty_configuration() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("No routers configured")));
}

#[test]
fn test_security_warnings() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("http://example.com", "/api/test", vec!["GET"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.warnings.iter().any(|w| w.contains("Insecure HTTP backend")));
}

#[test]
fn test_performance_warnings() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("https://example.com", "/api/{a}/{b}/{c}/{d}", vec!["GET"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.warnings.iter().any(|w| w.contains("many parameters")));
}

#[test]
fn test_validation_result_new() {
    let result = ValidationResult::new();
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
    assert!(result.recommendations.is_empty());
}

#[test]
fn test_validation_result_add_error() {
    let mut result = ValidationResult::new();
    result.add_error("Test error".to_string());
    
    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0], "Test error");
}

#[test]
fn test_validation_result_add_warning() {
    let mut result = ValidationResult::new();
    result.add_warning("Test warning".to_string());
    
    assert!(result.is_valid); // Warnings don't invalidate
    assert_eq!(result.warnings.len(), 1);
    assert_eq!(result.warnings[0], "Test warning");
}

#[test]
fn test_validation_result_add_recommendation() {
    let mut result = ValidationResult::new();
    result.add_recommendation("Test recommendation".to_string());
    
    assert!(result.is_valid);
    assert_eq!(result.recommendations.len(), 1);
    assert_eq!(result.recommendations[0], "Test recommendation");
}

#[test]
fn test_localhost_http_warnings() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("http://localhost:3000", "/api/test", vec!["GET"]),
            create_test_router("http://127.0.0.1:8080", "/api/local", vec!["POST"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.warnings.iter().any(|w| w.contains("localhost")));
}

#[test]
fn test_overly_permissive_methods() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("https://example.com", "/api/test", 
                              vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.warnings.iter().any(|w| w.contains("many HTTP methods")));
}

#[test]
fn test_path_traversal_detection() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            Router {
                host: Some("https://example.com".to_string()),
                port: Some(443),
                external_path: "/api/../admin".to_string(),
                internal_path: "/test".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "https://example.com".to_string(),
                    port: 443,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
                protocol: Protocol::Http,
                request_transformation: None,
                response_transformation: None,
                ai_policy: None,
            },
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("Path traversal detected")));
}

#[test]
fn test_duplicate_route_paths() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("https://example.com", "/api/test", vec!["GET"]),
            create_test_router("https://other.com", "/api/test", vec!["POST"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|e| e.contains("Duplicate route path")));
}

#[test]
fn test_mixed_http_https_warnings() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("http://example.com", "/api/insecure", vec!["GET"]),
            create_test_router("https://secure.com", "/api/secure", vec!["GET"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    // Should have warnings about HTTP backends but not the "all routes use HTTP" warning
    assert!(result.warnings.iter().any(|w| w.contains("Insecure HTTP backend")));
    assert!(!result.warnings.iter().any(|w| w.contains("All routes use HTTP")));
}

#[test]
fn test_all_http_routes_warning() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("http://example1.com", "/api/test1", vec!["GET"]),
            create_test_router("http://example2.com", "/api/test2", vec!["POST"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.warnings.iter().any(|w| w.contains("All routes use HTTP")));
}

#[test]
fn test_high_dynamic_routes_warning() {
    let mut routers = Vec::new();
    for i in 0..60 {
        routers.push(create_test_router("https://example.com", 
                                       &format!("/api/{{id{}}}", i), vec!["GET"]));
    }
    
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers,
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.warnings.iter().any(|w| w.contains("High number of dynamic routes")));
}

#[test]
fn test_static_route_recommendation() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("https://example.com", "/api/{id}", vec!["GET"]),
            create_test_router("https://example.com", "/api/{user_id}/profile", vec!["GET"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.recommendations.iter().any(|r| r.contains("static routes")));
}

#[test]
fn test_valid_configuration() {
    let settings = Settings {
        jwt: None,
        rate_limit: None,
        version: 1,
        routers: vec![
            create_test_router("https://example.com", "/api/health", vec!["GET"]),
            create_test_router("https://example.com", "/api/users/{id}", vec!["GET", "PUT"]),
        ],
    };
    
    let result = ConfigValidator::validate_comprehensive(&settings);
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}