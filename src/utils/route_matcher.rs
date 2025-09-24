use crate::models::router::Router;
use ahash::HashMap as AHashMap;
use regex::Regex;
use std::sync::Arc;
use thiserror::Error;

/// Error types that can occur during route matching operations.
/// 
/// These errors represent issues with route pattern compilation, validation,
/// or matching that prevent the gateway from properly routing requests.
#[derive(Error, Debug, PartialEq)]
pub enum RouteMatchError {
    /// The route pattern contains invalid syntax or unsupported constructs.
    /// 
    /// This occurs when route patterns have malformed parameter syntax,
    /// invalid characters, or other structural issues.
    #[error("Invalid route pattern: {pattern}")]
    InvalidPattern { 
        /// The invalid route pattern that caused the error
        pattern: String 
    },
    
    /// Failed to compile the route pattern into a valid regular expression.
    /// 
    /// This happens when the generated regex is syntactically invalid,
    /// which typically indicates a bug in pattern conversion logic.
    #[error("Regex compilation failed: {0}")]
    RegexError(String),
    
    /// No configured route matches the requested path.
    /// 
    /// This occurs during request processing when the incoming path
    /// doesn't match any static or dynamic route patterns.
    #[error("No matching route found for path: {path}")]
    NoMatch { 
        /// The requested path that couldn't be matched
        path: String 
    },
}

/// A pre-compiled route pattern optimized for high-performance matching.
/// 
/// This structure represents a single dynamic route that has been compiled
/// from a pattern string into a regex for efficient matching. It includes
/// the original router configuration and extracted parameter information.
/// 
/// # Thread Safety
/// 
/// The regex is wrapped in an `Arc` to enable efficient sharing across
/// multiple worker threads without cloning the compiled regex.
/// 
/// # Examples
/// 
/// ```rust
/// // Pattern: "/api/users/{id}/posts/{post_id}"
/// // Regex: "^/api/users/([^/]+)/posts/([^/]+)$"
/// // Param names: ["id", "post_id"]
/// ```
#[derive(Debug, Clone)]
pub struct CompiledRoute {
    /// The original router configuration for this route
    pub router: Router,
    /// Compiled regular expression for path matching (Arc for thread-safe sharing)
    pub regex: Arc<Regex>,
    /// Ordered list of parameter names extracted from the pattern
    pub param_names: Vec<String>,
}

/// High-performance route matcher with optimized lookup strategies.
/// 
/// The `RouteMatcher` provides efficient route resolution by separating static
/// and dynamic routes into different data structures optimized for their use cases:
/// 
/// - **Static routes**: Stored in a hash map for O(1) lookup
/// - **Dynamic routes**: Compiled to regex patterns and sorted by specificity
/// 
/// # Performance Characteristics
/// 
/// - Static route lookup: O(1) average case
/// - Dynamic route lookup: O(n) where n is number of dynamic routes
/// - Memory usage: Optimized with `ahash` and `Arc<Regex>` sharing
/// 
/// # Thread Safety
/// 
/// All fields are immutable after construction, making the matcher safe to
/// share across multiple worker threads without synchronization overhead.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::utils::route_matcher::RouteMatcher;
/// use kairos_rs::models::router::Router;
/// 
/// let routes = vec![
///     Router {
///         host: "http://api".to_string(),
///         port: 8080,
///         external_path: "/users".to_string(),          // Static route
///         internal_path: "/v1/users".to_string(),
///         methods: vec!["GET".to_string()],
///         auth_required: false,
///     },
///     Router {
///         host: "http://api".to_string(),
///         port: 8080,
///         external_path: "/users/{id}".to_string(),     // Dynamic route
///         internal_path: "/v1/user/{id}".to_string(),
///         methods: vec!["GET".to_string()],
///         auth_required: false,
///     },
/// ];
/// 
/// let matcher = RouteMatcher::new(routes).expect("Failed to create matcher");
/// 
/// // Fast static route lookup
/// let (route, path) = matcher.find_match("/users").expect("Route not found");
/// 
/// // Parameter extraction from dynamic routes
/// let (route, path) = matcher.find_match("/users/123").expect("Route not found");
/// assert_eq!(path, "/v1/user/123");
/// ```
#[derive(Debug)]
pub struct RouteMatcher {
    /// Hash map for O(1) static route lookups using ahash for better performance
    static_routes: AHashMap<String, Router>,
    /// Vector of compiled dynamic routes sorted by specificity (most specific first)
    dynamic_routes: Vec<CompiledRoute>,
}

impl RouteMatcher {
    /// Creates a new route matcher with pre-compiled patterns for optimal performance.
    /// 
    /// This constructor analyzes all provided routes and optimizes them for fast lookups:
    /// 1. Separates static routes (no parameters) from dynamic routes (with parameters)
    /// 2. Stores static routes in a hash map for O(1) access
    /// 3. Compiles dynamic routes to regex patterns
    /// 4. Sorts dynamic routes by specificity for consistent matching
    /// 
    /// # Parameters
    /// 
    /// * `routes` - Vector of router configurations to compile
    /// 
    /// # Returns
    /// 
    /// - `Ok(RouteMatcher)` - Successfully created and optimized route matcher
    /// - `Err(RouteMatchError)` - Invalid route pattern that couldn't be compiled
    /// 
    /// # Route Processing Logic
    /// 
    /// **Static Routes**: Patterns without `{param}` syntax are stored directly
    /// in a hash map for fastest possible lookup.
    /// 
    /// **Dynamic Routes**: Patterns containing `{param}` are:
    /// 1. Compiled into regex patterns
    /// 2. Parameter names extracted and stored
    /// 3. Sorted by parameter count (more specific routes first)
    /// 
    /// # Performance Optimizations
    /// 
    /// - Uses `ahash::HashMap` for better hash performance than std HashMap
    /// - Pre-allocates dynamic routes vector with known capacity
    /// - Shares compiled regex via `Arc` for thread-safe access
    /// - Sorts routes for deterministic matching order
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::utils::route_matcher::RouteMatcher;
    /// use kairos_rs::models::router::Router;
    /// 
    /// let routes = vec![
    ///     Router {
    ///         host: "http://localhost".to_string(),
    ///         port: 8080,
    ///         external_path: "/health".to_string(),        // Static
    ///         internal_path: "/status".to_string(),
    ///         methods: vec!["GET".to_string()],
    ///         auth_required: false,
    ///     },
    ///     Router {
    ///         host: "http://localhost".to_string(),
    ///         port: 8080,
    ///         external_path: "/users/{id}".to_string(),    // Dynamic
    ///         internal_path: "/v1/user/{id}".to_string(),
    ///         methods: vec!["GET".to_string()],
    ///         auth_required: false,
    ///     },
    /// ];
    /// 
    /// let matcher = RouteMatcher::new(routes)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns `RouteMatchError::InvalidPattern` if any route contains:
    /// - Malformed parameter syntax (e.g., `{unclosed` or `{empty}`)
    /// - Invalid parameter names (non-alphanumeric characters)
    /// - Regex compilation failures
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

    /// Finds a matching route for the given request path and returns the transformed internal path.
    /// 
    /// This method implements a two-phase matching strategy optimized for performance:
    /// 1. **Static Route Lookup**: O(1) hash map lookup for exact path matches
    /// 2. **Dynamic Route Matching**: Regex matching for parameterized routes
    /// 
    /// # Parameters
    /// 
    /// * `request_path` - The incoming HTTP request path to match against configured routes
    /// 
    /// # Returns
    /// 
    /// - `Ok((Router, String))` - Tuple containing the matched router configuration 
    ///   and the transformed internal path with parameters substituted
    /// - `Err(RouteMatchError::NoMatch)` - No route matches the given path
    /// 
    /// # Matching Algorithm
    /// 
    /// 1. **Static Route Check**: First attempts O(1) lookup in the static routes hash map
    /// 2. **Dynamic Route Check**: If no static match, tries each dynamic route in specificity order
    /// 3. **Parameter Extraction**: For dynamic matches, extracts parameter values from the path
    /// 4. **Path Transformation**: Substitutes parameters into the internal path template
    /// 
    /// # Path Transformation Examples
    /// 
    /// ```text
    /// External: "/api/users/{id}/posts/{post_id}"
    /// Internal: "/v1/user/{id}/post/{post_id}" 
    /// Request:  "/api/users/123/posts/456"
    /// Result:   "/v1/user/123/post/456"
    /// ```
    /// 
    /// # Performance Characteristics
    /// 
    /// - Static routes: O(1) average case
    /// - Dynamic routes: O(n) where n = number of dynamic routes
    /// - Memory: Zero allocations for static routes, minimal for dynamic routes
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use kairos_rs::utils::route_matcher::RouteMatcher;
    /// # use kairos_rs::models::router::Router;
    /// # let routes = vec![
    /// #     Router {
    /// #         host: "http://localhost".to_string(),
    /// #         port: 8080,
    /// #         external_path: "/health".to_string(),
    /// #         internal_path: "/status".to_string(),
    /// #         methods: vec!["GET".to_string()],
    /// #         auth_required: false,
    /// #     },
    /// #     Router {
    /// #         host: "http://localhost".to_string(),
    /// #         port: 8080,
    /// #         external_path: "/users/{id}".to_string(),
    /// #         internal_path: "/v1/user/{id}".to_string(),
    /// #         methods: vec!["GET".to_string()],
    /// #         auth_required: false,
    /// #     }
    /// # ];
    /// # let matcher = RouteMatcher::new(routes)?;
    /// 
    /// // Static route matching
    /// let (route, internal_path) = matcher.find_match("/health")?;
    /// assert_eq!(internal_path, "/status");
    /// 
    /// // Dynamic route with parameter substitution
    /// let (route, internal_path) = matcher.find_match("/users/123")?;
    /// assert_eq!(internal_path, "/v1/user/123");
    /// 
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// 
    /// # Thread Safety
    /// 
    /// This method is safe to call concurrently from multiple threads as it only
    /// reads immutable data structures and doesn't modify the matcher state.
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
