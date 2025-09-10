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

pub fn format_route(host: &str, port: &u16, internal_path: &str) -> String {
    format!(
        "{}{}",
        host,
        // port,
        internal_path
    )
}

/*
    Find matching route paht

    This function is used to return the route that matches the incoming request path.

    It uses regex to match paths with parameters, e.g., /api/{param}/details.

    For example, given the following routes:
    - /api/users/{user_id}
    - /api/products/{product_id}/details
    - /api/orders/{order_id}/items/{item_id}
    - /api/static/path
    - /api/static/path/details
*/
pub fn find_matching_route<'a>(
    routes: &'a HashMap<String, Router>,
    request_path: &str,
) -> Option<&'a Router> {
    for (external_path, router) in routes.iter() {
        // Escape special regex characters in the external_path
        let escaped_path = regex::escape(external_path);

        // Replace escaped parameter placeholders with regex patterns
        let pattern = escaped_path.replace(r"\{[^\}]+\}", r"([^/]+)");

        // Create a regex pattern that matches the entire path
        let regex_pattern = format!("^{}$", pattern);

        // Compile the regex
        if let Ok(re) = Regex::new(&regex_pattern) {
            // Check if the request_path matches the regex
            if re.is_match(request_path) {
                return Some(router);
            }
        }
    }
    None
}
