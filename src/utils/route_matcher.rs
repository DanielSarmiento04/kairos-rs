use crate::models::router::Router;
use ahash::HashMap as AHashMap;
use regex::Regex;
use std::sync::Arc;
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
    pub regex: Arc<Regex>, // Use Arc to share regex across threads
    pub param_names: Vec<String>,
}

/// High-performance route matcher with compiled patterns
#[derive(Debug)]
pub struct RouteMatcher {
    static_routes: AHashMap<String, Router>, // Use ahash for better performance
    dynamic_routes: Vec<CompiledRoute>,
}

impl RouteMatcher {
    /// Creates a new route matcher with pre-compiled patterns
    pub fn new(routes: Vec<Router>) -> Result<Self, RouteMatchError> {
        let mut static_routes = AHashMap::default();
        let mut dynamic_routes = Vec::with_capacity(routes.len());

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

        // Sort dynamic routes by specificity (more parameters = more specific)
        dynamic_routes.sort_by(|a, b| {
            b.param_names.len().cmp(&a.param_names.len())
        });

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
            regex: Arc::new(regex),
            param_names,
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
}
