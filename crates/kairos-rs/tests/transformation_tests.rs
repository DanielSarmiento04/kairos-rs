use kairos_rs::middleware::transform::*;
use actix_web::http::{header::{HeaderMap, HeaderValue, HOST, AUTHORIZATION, COOKIE, CONTENT_TYPE, USER_AGENT, SERVER}, StatusCode};
use std::collections::HashMap;

#[test]
fn test_request_header_add() {
    let config = RequestTransformation {
        headers: vec![
            HeaderTransformation {
                action: TransformAction::Add,
                name: "X-Gateway".to_string(),
                value: Some("kairos-rs".to_string()),
                pattern: None,
                replacement: None,
            },
            HeaderTransformation {
                action: TransformAction::Add,
                name: "X-Request-ID".to_string(),
                value: Some("123456".to_string()),
                pattern: None,
                replacement: None,
            },
        ],
        path: None,
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    let mut headers = HeaderMap::new();
    
    transformer.transform_headers(&mut headers);
    
    assert_eq!(headers.get("X-Gateway").unwrap().to_str().unwrap(), "kairos-rs");
    assert_eq!(headers.get("X-Request-ID").unwrap().to_str().unwrap(), "123456");
}

#[test]
fn test_request_header_set_override() {
    let config = RequestTransformation {
        headers: vec![HeaderTransformation {
            action: TransformAction::Set,
            name: "Host".to_string(),
            value: Some("backend.local".to_string()),
            pattern: None,
            replacement: None,
        }],
        path: None,
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("original.host"));
    
    transformer.transform_headers(&mut headers);
    
    assert_eq!(headers.get(HOST).unwrap().to_str().unwrap(), "backend.local");
}

#[test]
fn test_request_header_add_no_override() {
    let config = RequestTransformation {
        headers: vec![HeaderTransformation {
            action: TransformAction::Add,
            name: "Host".to_string(),
            value: Some("backend.local".to_string()),
            pattern: None,
            replacement: None,
        }],
        path: None,
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    let mut headers = HeaderMap::new();
    headers.insert(HOST, HeaderValue::from_static("original.host"));
    
    transformer.transform_headers(&mut headers);
    
    // Should NOT override existing header
    assert_eq!(headers.get(HOST).unwrap().to_str().unwrap(), "original.host");
}

#[test]
fn test_request_header_remove() {
    let config = RequestTransformation {
        headers: vec![
            HeaderTransformation {
                action: TransformAction::Remove,
                name: "Authorization".to_string(),
                value: None,
                pattern: None,
                replacement: None,
            },
            HeaderTransformation {
                action: TransformAction::Remove,
                name: "Cookie".to_string(),
                value: None,
                pattern: None,
                replacement: None,
            },
        ],
        path: None,
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer token"));
    headers.insert(COOKIE, HeaderValue::from_static("session=abc123"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    transformer.transform_headers(&mut headers);
    
    assert!(headers.get(AUTHORIZATION).is_none());
    assert!(headers.get(COOKIE).is_none());
    assert!(headers.get(CONTENT_TYPE).is_some());
}

#[test]
fn test_request_header_replace_regex() {
    let config = RequestTransformation {
        headers: vec![HeaderTransformation {
            action: TransformAction::Replace,
            name: "User-Agent".to_string(),
            value: None,
            pattern: Some(r"Mozilla/(\d+\.\d+)".to_string()),
            replacement: Some("KairosGateway/$1".to_string()),
        }],
        path: None,
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (compatible)"));
    
    transformer.transform_headers(&mut headers);
    
    assert_eq!(headers.get(USER_AGENT).unwrap().to_str().unwrap(), "KairosGateway/5.0 (compatible)");
}

#[test]
fn test_path_rewrite_simple() {
    let config = RequestTransformation {
        headers: vec![],
        path: Some(PathTransformation {
            pattern: r"^/api/v1/(.+)$".to_string(),
            replacement: "/v2/$1".to_string(),
        }),
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    
    assert_eq!(transformer.transform_path("/api/v1/users"), "/v2/users");
    assert_eq!(transformer.transform_path("/api/v1/posts/123"), "/v2/posts/123");
    assert_eq!(transformer.transform_path("/other/path"), "/other/path"); // No match
}

#[test]
fn test_path_rewrite_remove_prefix() {
    let config = RequestTransformation {
        headers: vec![],
        path: Some(PathTransformation {
            pattern: r"^/gateway/(.+)$".to_string(),
            replacement: "/$1".to_string(),
        }),
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    
    assert_eq!(transformer.transform_path("/gateway/api/users"), "/api/users");
    assert_eq!(transformer.transform_path("/gateway/health"), "/health");
}

#[test]
fn test_path_rewrite_add_prefix() {
    let config = RequestTransformation {
        headers: vec![],
        path: Some(PathTransformation {
            pattern: r"^/(.+)$".to_string(),
            replacement: "/backend/$1".to_string(),
        }),
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    
    assert_eq!(transformer.transform_path("/users"), "/backend/users");
    assert_eq!(transformer.transform_path("/api/posts"), "/backend/api/posts");
}

#[test]
fn test_path_rewrite_complex_pattern() {
    let config = RequestTransformation {
        headers: vec![],
        path: Some(PathTransformation {
            pattern: r"^/api/v(\d+)/(\w+)/(.+)$".to_string(),
            replacement: "/v$1/api/$2/$3".to_string(),
        }),
        query_params: vec![],
    };

    let transformer = RequestTransformer::new(config);
    
    assert_eq!(
        transformer.transform_path("/api/v1/users/123"),
        "/v1/api/users/123"
    );
    assert_eq!(
        transformer.transform_path("/api/v2/posts/abc"),
        "/v2/api/posts/abc"
    );
}

#[test]
fn test_query_param_add() {
    let config = RequestTransformation {
        headers: vec![],
        path: None,
        query_params: vec![
            QueryTransformation {
                action: TransformAction::Add,
                name: "api_key".to_string(),
                value: Some("secret123".to_string()),
            },
            QueryTransformation {
                action: TransformAction::Add,
                name: "version".to_string(),
                value: Some("v1".to_string()),
            },
        ],
    };

    let transformer = RequestTransformer::new(config);
    let mut params = HashMap::new();
    
    transformer.transform_query_params(&mut params);
    
    assert_eq!(params.get("api_key").unwrap(), "secret123");
    assert_eq!(params.get("version").unwrap(), "v1");
}

#[test]
fn test_query_param_set() {
    let config = RequestTransformation {
        headers: vec![],
        path: None,
        query_params: vec![QueryTransformation {
            action: TransformAction::Set,
            name: "format".to_string(),
            value: Some("json".to_string()),
        }],
    };

    let transformer = RequestTransformer::new(config);
    let mut params = HashMap::new();
    params.insert("format".to_string(), "xml".to_string());
    
    transformer.transform_query_params(&mut params);
    
    assert_eq!(params.get("format").unwrap(), "json");
}

#[test]
fn test_query_param_remove() {
    let config = RequestTransformation {
        headers: vec![],
        path: None,
        query_params: vec![
            QueryTransformation {
                action: TransformAction::Remove,
                name: "debug".to_string(),
                value: None,
            },
            QueryTransformation {
                action: TransformAction::Remove,
                name: "internal".to_string(),
                value: None,
            },
        ],
    };

    let transformer = RequestTransformer::new(config);
    let mut params = HashMap::new();
    params.insert("debug".to_string(), "true".to_string());
    params.insert("internal".to_string(), "yes".to_string());
    params.insert("user_id".to_string(), "123".to_string());
    
    transformer.transform_query_params(&mut params);
    
    assert!(params.get("debug").is_none());
    assert!(params.get("internal").is_none());
    assert!(params.get("user_id").is_some());
}

#[test]
fn test_response_header_transformation() {
    let config = ResponseTransformation {
        headers: vec![
            HeaderTransformation {
                action: TransformAction::Add,
                name: "X-Powered-By".to_string(),
                value: Some("Kairos Gateway".to_string()),
                pattern: None,
                replacement: None,
            },
            HeaderTransformation {
                action: TransformAction::Remove,
                name: "Server".to_string(),
                value: None,
                pattern: None,
                replacement: None,
            },
        ],
        status_code_mappings: vec![],
    };

    let transformer = ResponseTransformer::new(config);
    let mut headers = HeaderMap::new();
    headers.insert(SERVER, HeaderValue::from_static("Apache/2.4"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    transformer.transform_headers(&mut headers);
    
    assert_eq!(headers.get("X-Powered-By").unwrap().to_str().unwrap(), "Kairos Gateway");
    assert!(headers.get(SERVER).is_none());
    assert!(headers.get(CONTENT_TYPE).is_some());
}

#[test]
fn test_response_status_code_mapping() {
    let config = ResponseTransformation {
        headers: vec![],
        status_code_mappings: vec![
            StatusCodeMapping {
                from: StatusCode::NOT_FOUND,
                to: StatusCode::OK,
                condition: None,
            },
            StatusCodeMapping {
                from: StatusCode::BAD_GATEWAY,
                to: StatusCode::SERVICE_UNAVAILABLE,
                condition: None,
            },
        ],
    };

    let transformer = ResponseTransformer::new(config);
    
    assert_eq!(transformer.transform_status_code(StatusCode::NOT_FOUND, "/test"), StatusCode::OK);
    assert_eq!(transformer.transform_status_code(StatusCode::BAD_GATEWAY, "/test"), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(transformer.transform_status_code(StatusCode::OK, "/test"), StatusCode::OK); // No mapping
}

#[test]
fn test_combined_request_transformation() {
    let config = RequestTransformation {
        headers: vec![
            HeaderTransformation {
                action: TransformAction::Add,
                name: "X-Forwarded-By".to_string(),
                value: Some("kairos-gateway".to_string()),
                pattern: None,
                replacement: None,
            },
            HeaderTransformation {
                action: TransformAction::Remove,
                name: "Cookie".to_string(),
                value: None,
                pattern: None,
                replacement: None,
            },
        ],
        path: Some(PathTransformation {
            pattern: r"^/public/(.+)$".to_string(),
            replacement: "/api/v1/$1".to_string(),
        }),
        query_params: vec![QueryTransformation {
            action: TransformAction::Add,
            name: "source".to_string(),
            value: Some("gateway".to_string()),
        }],
    };

    let transformer = RequestTransformer::new(config);
    
    // Test headers
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_static("session=abc"));
    transformer.transform_headers(&mut headers);
    assert!(headers.get("X-Forwarded-By").is_some());
    assert!(headers.get(COOKIE).is_none());
    
    // Test path
    assert_eq!(transformer.transform_path("/public/users"), "/api/v1/users");
    
    // Test query params
    let mut params = HashMap::new();
    transformer.transform_query_params(&mut params);
    assert_eq!(params.get("source").unwrap(), "gateway");
}

#[test]
fn test_serialization_deserialization() {
    let config = RequestTransformation {
        headers: vec![HeaderTransformation {
            action: TransformAction::Add,
            name: "X-Custom".to_string(),
            value: Some("value".to_string()),
            pattern: None,
            replacement: None,
        }],
        path: Some(PathTransformation {
            pattern: r"^/api/(.+)$".to_string(),
            replacement: "/$1".to_string(),
        }),
        query_params: vec![],
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RequestTransformation = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.headers.len(), 1);
    assert_eq!(deserialized.headers[0].name, "X-Custom");
    assert!(deserialized.path.is_some());
}
