use crate::models::router::Router;
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RouteMatchError {
    #[error("Invalid route pattern: {pattern}")]
    InvalidPattern { pattern: String },
    #[error("Regex compilation failed: {0}")]
    RegexError(String),
    #[error("No matching route found for path: {path}")]
    NoMatch { path: String },
}

/// Compiled route pattern for efficient matching
#[derive(Debug, Clone)]
pub struct CompiledRoute {
    pub router: Router,
    pub regex: Regex,
    pub param_names: Vec<String>,
    pub is_static: bool,
}

/// High-performance route matcher with compiled patterns
#[derive(Debug)]
pub struct RouteMatcher {
    static_routes: HashMap<String, Router>,
    dynamic_routes: Vec<CompiledRoute>,
}

impl RouteMatcher {
    /// Creates a new route matcher with pre-compiled patterns
    pub fn new(routes: Vec<Router>) -> Result<Self, RouteMatchError> {
        let mut static_routes = HashMap::new();
        let mut dynamic_routes = Vec::new();

        for route in routes {
            if route.external_path.contains('{') {
                // Dynamic route
                let compiled = Self::compile_route(route)?;
                dynamic_routes.push(compiled);
            } else {
                // Static route
                static_routes.insert(route.external_path.clone(), route);
            }
        }

        Ok(Self {
            static_routes,
            dynamic_routes,
        })
    }

    /// Finds a matching route and returns the transformed internal path
    pub fn find_match(&self, request_path: &str) -> Result<(Router, String), RouteMatchError> {
        // First, try static routes (O(1) lookup)
        if let Some(route) = self.static_routes.get(request_path) {
            return Ok((route.clone(), route.internal_path.clone()));
        }

        // Then, try dynamic routes
        for compiled_route in &self.dynamic_routes {
            if let Some(captures) = compiled_route.regex.captures(request_path) {
                let transformed_path = self.transform_internal_path(
                    &compiled_route.router.internal_path,
                    &compiled_route.param_names,
                    &captures,
                );
                return Ok((compiled_route.router.clone(), transformed_path));
            }
        }

        Err(RouteMatchError::NoMatch {
            path: request_path.to_string(),
        })
    }

    /// Compiles a route pattern into a regex and extracts parameter names
    fn compile_route(route: Router) -> Result<CompiledRoute, RouteMatchError> {
        let param_names = Self::extract_parameter_names(&route.external_path);
        let regex_pattern = Self::convert_pattern_to_regex(&route.external_path)?;
        
        let regex = Regex::new(&regex_pattern).map_err(|e| RouteMatchError::RegexError(e.to_string()))?;

        Ok(CompiledRoute {
            router: route,
            regex,
            param_names,
            is_static: false,
        })
    }

    /// Converts a route pattern to a regex pattern
    fn convert_pattern_to_regex(pattern: &str) -> Result<String, RouteMatchError> {
        let mut regex_pattern = String::with_capacity(pattern.len() * 2);
        regex_pattern.push('^');
        
        let mut chars = pattern.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    // Find the closing brace and validate parameter name
                    let mut param_name = String::new();
                    let mut found_closing = false;
                    
                    while let Some(inner_ch) = chars.next() {
                        if inner_ch == '}' {
                            found_closing = true;
                            break;
                        }
                        if inner_ch.is_alphanumeric() || inner_ch == '_' {
                            param_name.push(inner_ch);
                        } else {
                            return Err(RouteMatchError::InvalidPattern {
                                pattern: pattern.to_string(),
                            });
                        }
                    }
                    
                    if !found_closing || param_name.is_empty() {
                        return Err(RouteMatchError::InvalidPattern {
                            pattern: pattern.to_string(),
                        });
                    }
                    
                    // Add capture group for parameter
                    regex_pattern.push_str("([^/]+)");
                }
                // Escape special regex characters
                '.' | '?' | '*' | '+' | '^' | '$' | '[' | ']' | '(' | ')' | '|' | '\\' => {
                    regex_pattern.push('\\');
                    regex_pattern.push(ch);
                }
                _ => regex_pattern.push(ch),
            }
        }
        
        regex_pattern.push('$');
        Ok(regex_pattern)
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

    /// Transforms the internal path by replacing parameters with captured values
    fn transform_internal_path(
        &self,
        internal_pattern: &str,
        param_names: &[String],
        captures: &regex::Captures,
    ) -> String {
        let mut result = internal_pattern.to_string();
        
        for (i, param_name) in param_names.iter().enumerate() {
            if let Some(capture) = captures.get(i + 1) {
                let placeholder = format!("{{{}}}", param_name);
                result = result.replace(&placeholder, capture.as_str());
            }
        }
        
        result
    }

    /// Returns the number of static routes
    pub fn static_routes_count(&self) -> usize {
        self.static_routes.len()
    }

    /// Returns the number of dynamic routes
    pub fn dynamic_routes_count(&self) -> usize {
        self.dynamic_routes.len()
    }
}

// Global route matcher instance (lazy initialization)
static ROUTE_MATCHER: OnceLock<RouteMatcher> = OnceLock::new();

/// Initialize the global route matcher
pub fn init_route_matcher(routes: Vec<Router>) -> Result<(), RouteMatchError> {
    let matcher = RouteMatcher::new(routes)?;
    ROUTE_MATCHER.set(matcher).map_err(|_| RouteMatchError::InvalidPattern {
        pattern: "Failed to initialize global route matcher".to_string(),
    })
}

/// Find a matching route using the global matcher
pub fn find_matching_route(request_path: &str) -> Result<(Router, String), RouteMatchError> {
    let matcher = ROUTE_MATCHER.get().ok_or_else(|| RouteMatchError::InvalidPattern {
        pattern: "Route matcher not initialized".to_string(),
    })?;
    
    matcher.find_match(request_path)
}

// Legacy compatibility function
pub fn find_matching_route_legacy(
    request_path: &str,
    routes: &[Router],
) -> Option<(Router, String)> {
    // For backward compatibility, create a temporary matcher
    let matcher = match RouteMatcher::new(routes.to_vec()) {
        Ok(m) => m,
        Err(_) => return None,
    };
    
    matcher.find_match(request_path).ok()
}
