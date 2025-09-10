use crate::models::router::Router;
use regex::Regex;
use std::collections::HashMap;

/*
    {
        "host": "https://http.cat",
        "port": 443,
        "external_path": "/{path}",
        "internal_path": "/api/{path}",
        "methods": [
            "GET"
        ]
    },
    {
        "host": "https://google.com",
        "port": 443,
        "external_path": "/identity/register/v2",
        "internal_path": "/api/identity/register/v2",
        "methods": [
            "POST",
            "GET"
        ]
    },
*/

pub fn format_route(host: &str, _port: &u16, internal_path: &str) -> String {
    format!(
        "{}{}",
        host,
        // port,
        internal_path
    )
}

/*
    Find matching route path

    This function is used to return the route that matches the incoming request path.

    It uses regex to match paths with parameters, e.g., /api/{param}/details.

    For example, given the following routes:
    - /api/users/{user_id}
    - /api/products/{product_id}/details
    - /api/orders/{order_id}/items/{item_id}
    - /api/static/path
    - /api/static/path/details
*/
pub fn find_matching_route(
    request_path: &str,
    routes: &[Router],
) -> Option<(Router, String)> {
    // First, try to find exact matches (static routes)
    for route in routes {
        if route.external_path == request_path {
            return Some((route.clone(), route.internal_path.clone()));
        }
    }

    // Then, try pattern matching for dynamic routes
    for route in routes {
        if let Some(internal_path) = match_dynamic_route(request_path, &route.external_path, &route.internal_path) {
            return Some((route.clone(), internal_path));
        }
    }

    None
}

/// Matches a dynamic route pattern and replaces parameters in the internal path
fn match_dynamic_route(
    request_path: &str,
    external_pattern: &str,
    internal_pattern: &str,
) -> Option<String> {
    // Check if the external pattern contains parameters
    if !external_pattern.contains('{') {
        return None; // This is a static route, already handled above
    }

    // Convert the external pattern to a regex pattern
    let regex_pattern = convert_pattern_to_regex(external_pattern);
    
    // Create regex from pattern
    let regex = match Regex::new(&regex_pattern) {
        Ok(r) => r,
        Err(_) => return None,
    };

    // Try to match the request path
    let captures = regex.captures(request_path)?;
    
    // Extract parameter names from the external pattern
    let param_names = extract_parameter_names(external_pattern);
    
    // Build a map of parameter values
    let mut param_values = HashMap::new();
    for (i, param_name) in param_names.iter().enumerate() {
        if let Some(capture) = captures.get(i + 1) {
            param_values.insert(param_name.clone(), capture.as_str().to_string());
        }
    }

    // Replace parameters in the internal pattern
    Some(replace_parameters(internal_pattern, &param_values))
}

/// Converts a route pattern like "/api/users/{user_id}" to a regex pattern
fn convert_pattern_to_regex(pattern: &str) -> String {
    let mut regex_pattern = "^".to_string();
    let mut chars = pattern.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                // Find the closing brace
                let mut param_name = String::new();
                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '}' {
                        break;
                    }
                    param_name.push(inner_ch);
                }
                // Add a capture group for this parameter
                regex_pattern.push_str("([^/]+)");
            }
            '/' => regex_pattern.push('/'),
            '.' => regex_pattern.push_str("\\."),
            '?' => regex_pattern.push_str("\\?"),
            '*' => regex_pattern.push_str("\\*"),
            '+' => regex_pattern.push_str("\\+"),
            '^' => regex_pattern.push_str("\\^"),
            '$' => regex_pattern.push_str("\\$"),
            '[' => regex_pattern.push_str("\\["),
            ']' => regex_pattern.push_str("\\]"),
            '(' => regex_pattern.push_str("\\("),
            ')' => regex_pattern.push_str("\\)"),
            '|' => regex_pattern.push_str("\\|"),
            '\\' => regex_pattern.push_str("\\\\"),
            _ => regex_pattern.push(ch),
        }
    }
    
    regex_pattern.push('$');
    regex_pattern
}

/// Extracts parameter names from a route pattern
fn extract_parameter_names(pattern: &str) -> Vec<String> {
    let mut param_names = Vec::new();
    let mut chars = pattern.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut param_name = String::new();
            while let Some(inner_ch) = chars.next() {
                if inner_ch == '}' {
                    break;
                }
                param_name.push(inner_ch);
            }
            if !param_name.is_empty() {
                param_names.push(param_name);
            }
        }
    }
    
    param_names
}

/// Replaces parameter placeholders in a path with actual values
fn replace_parameters(pattern: &str, param_values: &HashMap<String, String>) -> String {
    let mut result = pattern.to_string();
    
    for (param_name, param_value) in param_values {
        let placeholder = format!("{{{}}}", param_name);
        result = result.replace(&placeholder, param_value);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_static_route_matching() {
        let routes = create_test_routes();

        // Test exact match for static route
        let result = find_matching_route("/api/identity/register/v3", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/identity/register/v3");
        assert_eq!(internal_path, "/api/identity/register");
        assert_eq!(route.host, "http://localhost");
    }

    #[test]
    fn test_single_parameter_replacement() {
        let routes = create_test_routes();

        // Test single parameter route
        let result = find_matching_route("/cats/200", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/cats/{id}");
        assert_eq!(internal_path, "/200");
        assert_eq!(route.host, "https://http.cat");

        // Test user ID route
        let result = find_matching_route("/api/users/123", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/users/{user_id}");
        assert_eq!(internal_path, "/users/123");
    }

    #[test]
    fn test_multiple_parameter_replacement() {
        let routes = create_test_routes();

        // Test multiple parameters
        let result = find_matching_route("/api/orders/123/items/456", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/orders/{order_id}/items/{item_id}");
        assert_eq!(internal_path, "/orders/123/items/456");
    }

    #[test]
    fn test_product_details_route() {
        let routes = create_test_routes();

        // Test product details route
        let result = find_matching_route("/api/products/abc123/details", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/products/{product_id}/details");
        assert_eq!(internal_path, "/products/abc123/info");
    }

    #[test]
    fn test_non_matching_routes() {
        let routes = create_test_routes();

        // Test non-matching route
        let result = find_matching_route("/api/nonexistent", &routes);
        assert!(result.is_none());

        // Test partial match that shouldn't work
        let result = find_matching_route("/api/users", &routes);
        assert!(result.is_none());

        // Test route with extra segments
        let result = find_matching_route("/api/users/123/extra", &routes);
        assert!(result.is_none());
    }

    #[test]
    fn test_static_routes_priority() {
        let routes = create_test_routes();

        // Static routes should be matched before dynamic ones
        let result = find_matching_route("/api/static/path", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/static/path");
        assert_eq!(internal_path, "/static");

        let result = find_matching_route("/api/static/path/details", &routes);
        assert!(result.is_some());
        let (route, internal_path) = result.unwrap();
        assert_eq!(route.external_path, "/api/static/path/details");
        assert_eq!(internal_path, "/static/details");
    }

    #[test]
    fn test_convert_pattern_to_regex() {
        assert_eq!(convert_pattern_to_regex("/api/users/{id}"), "^/api/users/([^/]+)$");
        assert_eq!(convert_pattern_to_regex("/api/orders/{order_id}/items/{item_id}"), "^/api/orders/([^/]+)/items/([^/]+)$");
        assert_eq!(convert_pattern_to_regex("/api/static/path"), "^/api/static/path$");
        assert_eq!(convert_pattern_to_regex("/api/special.file"), "^/api/special\\.file$");
    }

    #[test]
    fn test_extract_parameter_names() {
        assert_eq!(extract_parameter_names("/api/users/{id}"), vec!["id"]);
        assert_eq!(extract_parameter_names("/api/orders/{order_id}/items/{item_id}"), vec!["order_id", "item_id"]);
        assert_eq!(extract_parameter_names("/api/static/path"), Vec::<String>::new());
        assert_eq!(extract_parameter_names("/api/{param1}/something/{param2}/end"), vec!["param1", "param2"]);
    }

    #[test]
    fn test_replace_parameters() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        assert_eq!(replace_parameters("/users/{id}", &params), "/users/123");

        let mut params = HashMap::new();
        params.insert("order_id".to_string(), "456".to_string());
        params.insert("item_id".to_string(), "789".to_string());
        assert_eq!(replace_parameters("/orders/{order_id}/items/{item_id}", &params), "/orders/456/items/789");

        // Test with no parameters
        let params = HashMap::new();
        assert_eq!(replace_parameters("/static/path", &params), "/static/path");
    }

    #[test]
    fn test_edge_cases() {
        let routes = create_test_routes();

        // Test empty path
        let result = find_matching_route("", &routes);
        assert!(result.is_none());

        // Test root path
        let result = find_matching_route("/", &routes);
        assert!(result.is_none());

        // Test path with special characters in parameter
        let result = find_matching_route("/cats/test-123_abc", &routes);
        assert!(result.is_some());
        let (_, internal_path) = result.unwrap();
        assert_eq!(internal_path, "/test-123_abc");

        // Test path with encoded characters
        let result = find_matching_route("/cats/test%20space", &routes);
        assert!(result.is_some());
        let (_, internal_path) = result.unwrap();
        assert_eq!(internal_path, "/test%20space");
    }

    #[test]
    fn test_match_dynamic_route() {
        // Test static route (should return None since it's handled separately)
        let result = match_dynamic_route("/api/static", "/api/static", "/static");
        assert!(result.is_none());

        // Test dynamic route
        let result = match_dynamic_route("/api/users/123", "/api/users/{id}", "/users/{id}");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "/users/123");

        // Test non-matching dynamic route
        let result = match_dynamic_route("/api/products", "/api/users/{id}", "/users/{id}");
        assert!(result.is_none());
    }
}