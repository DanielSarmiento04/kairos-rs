use actix_web::http::{header::{HeaderMap, HeaderName, HeaderValue}, StatusCode};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Transformation action type for request/response modification.
/// 
/// Defines the type of transformation to apply to headers, paths, or other request/response components.
/// 
/// # Actions
/// 
/// - **Add**: Add a new header or parameter (won't override existing)
/// - **Set**: Set a header or parameter (overrides if exists)
/// - **Remove**: Remove a header or parameter
/// - **Replace**: Replace using regex pattern matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransformAction {
    /// Add a value without overriding existing
    Add,
    /// Set a value (override if exists)
    Set,
    /// Remove a value
    Remove,
    /// Replace using regex pattern
    Replace,
}

/// Header transformation rule.
///
/// Defines how to transform HTTP headers in requests or responses.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::{HeaderTransformation, TransformAction};
/// 
/// // Add a custom header
/// let add_header = HeaderTransformation {
///     action: TransformAction::Add,
///     name: "X-Custom-Header".to_string(),
///     value: Some("custom-value".to_string()),
///     pattern: None,
///     replacement: None,
/// };
/// 
/// // Remove authorization header
/// let remove_auth = HeaderTransformation {
///     action: TransformAction::Remove,
///     name: "Authorization".to_string(),
///     value: None,
///     pattern: None,
///     replacement: None,
/// };
/// 
/// // Replace header value using regex
/// let replace_header = HeaderTransformation {
///     action: TransformAction::Replace,
///     name: "User-Agent".to_string(),
///     value: None,
///     pattern: Some(r"(\d+\.\d+)".to_string()),
///     replacement: Some("v$1-proxy".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderTransformation {
    /// Action to perform
    pub action: TransformAction,
    
    /// Header name
    pub name: String,
    
    /// Value to set/add (required for Add and Set)
    pub value: Option<String>,
    
    /// Regex pattern for Replace action
    pub pattern: Option<String>,
    
    /// Replacement template for Replace action
    pub replacement: Option<String>,
}

/// Path transformation rule.
///
/// Defines how to rewrite request paths before forwarding to backend.
/// Supports regex-based pattern matching with capture groups and template-based replacement.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::PathTransformation;
/// 
/// // Rewrite /api/v1/users/:id to /users/:id
/// let path_rewrite = PathTransformation {
///     pattern: r"^/api/v1/(.+)$".to_string(),
///     replacement: "/$1".to_string(),
/// };
/// 
/// // Add prefix to all paths
/// let add_prefix = PathTransformation {
///     pattern: r"^(.+)$".to_string(),
///     replacement: "/v2$1".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathTransformation {
    /// Regex pattern to match against the path
    pub pattern: String,
    
    /// Replacement template with capture group support ($1, $2, etc.)
    pub replacement: String,
}

/// Query parameter transformation rule.
///
/// Defines how to transform URL query parameters before forwarding requests.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::{QueryTransformation, TransformAction};
/// 
/// // Add API key parameter
/// let add_key = QueryTransformation {
///     action: TransformAction::Add,
///     name: "api_key".to_string(),
///     value: Some("secret123".to_string()),
/// };
/// 
/// // Remove debug parameter
/// let remove_debug = QueryTransformation {
///     action: TransformAction::Remove,
///     name: "debug".to_string(),
///     value: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTransformation {
    /// Action to perform
    pub action: TransformAction,
    
    /// Parameter name
    pub name: String,
    
    /// Value to set/add (required for Add and Set)
    pub value: Option<String>,
}

/// Request transformation configuration.
///
/// Complete configuration for transforming incoming requests before forwarding to backends.
/// Supports header manipulation, path rewriting, and query parameter transformation.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::{RequestTransformation, HeaderTransformation, PathTransformation, TransformAction};
/// 
/// let transform = RequestTransformation {
///     headers: vec![
///         HeaderTransformation {
///             action: TransformAction::Add,
///             name: "X-Forwarded-By".to_string(),
///             value: Some("kairos-gateway".to_string()),
///             pattern: None,
///             replacement: None,
///         }
///     ],
///     path: Some(PathTransformation {
///         pattern: r"^/api/v1/(.+)$".to_string(),
///         replacement: "/$1".to_string(),
///     }),
///     query_params: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestTransformation {
    /// Header transformations to apply
    #[serde(default)]
    pub headers: Vec<HeaderTransformation>,
    
    /// Optional path transformation
    pub path: Option<PathTransformation>,
    
    /// Query parameter transformations
    #[serde(default)]
    pub query_params: Vec<QueryTransformation>,
}

/// Response transformation configuration.
///
/// Configuration for transforming backend responses before sending to clients.
/// Supports header manipulation, status code mapping, and body transformation.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::{ResponseTransformation, HeaderTransformation, StatusCodeMapping, TransformAction};
/// use actix_web::http::StatusCode;
/// 
/// let transform = ResponseTransformation {
///     headers: vec![
///         HeaderTransformation {
///             action: TransformAction::Remove,
///             name: "Server".to_string(),
///             value: None,
///             pattern: None,
///             replacement: None,
///         }
///     ],
///     status_code_mappings: vec![
///         StatusCodeMapping {
///             from: StatusCode::NOT_FOUND,
///             to: StatusCode::OK,
///             condition: Some("path == '/health'".to_string()),
///         }
///     ],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseTransformation {
    /// Header transformations to apply
    #[serde(default)]
    pub headers: Vec<HeaderTransformation>,
    
    /// Status code mappings
    #[serde(default)]
    pub status_code_mappings: Vec<StatusCodeMapping>,
}

/// Status code mapping for response transformation.
///
/// Maps specific status codes to different codes based on optional conditions.
/// Useful for normalizing error responses or masking backend errors.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::StatusCodeMapping;
/// use actix_web::http::StatusCode;
/// 
/// // Map 404 to 200 for health checks
/// let mapping = StatusCodeMapping {
///     from: StatusCode::NOT_FOUND,
///     to: StatusCode::OK,
///     condition: Some("path == '/health'".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCodeMapping {
    /// Status code to match
    #[serde(with = "status_code_serde")]
    pub from: StatusCode,
    
    /// Status code to replace with
    #[serde(with = "status_code_serde")]
    pub to: StatusCode,
    
    /// Optional condition expression (future: support simple conditions)
    pub condition: Option<String>,
}

/// Custom serialization for StatusCode
mod status_code_serde {
    use actix_web::http::StatusCode;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(status.as_u16())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = u16::deserialize(deserializer)?;
        StatusCode::from_u16(code).map_err(serde::de::Error::custom)
    }
}

/// Request transformer service.
///
/// Applies transformation rules to incoming requests before forwarding to backends.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::{RequestTransformer, RequestTransformation, HeaderTransformation, TransformAction};
/// use actix_web::test::TestRequest;
/// 
/// let config = RequestTransformation {
///     headers: vec![
///         HeaderTransformation {
///             action: TransformAction::Add,
///             name: "X-Custom".to_string(),
///             value: Some("test".to_string()),
///             pattern: None,
///             replacement: None,
///         }
///     ],
///     path: None,
///     query_params: vec![],
/// };
/// 
/// let transformer = RequestTransformer::new(config);
/// ```
pub struct RequestTransformer {
    config: RequestTransformation,
    path_regex: Option<Regex>,
}

impl RequestTransformer {
    /// Creates a new request transformer with the given configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Request transformation configuration
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::middleware::transform::{RequestTransformer, RequestTransformation};
    /// 
    /// let config = RequestTransformation::default();
    /// let transformer = RequestTransformer::new(config);
    /// ```
    pub fn new(config: RequestTransformation) -> Self {
        let path_regex = config.path.as_ref().and_then(|p| {
            Regex::new(&p.pattern).ok()
        });

        Self {
            config,
            path_regex,
        }
    }

    /// Transforms request headers according to configuration.
    /// 
    /// # Arguments
    /// 
    /// * `headers` - Mutable reference to request headers
    pub fn transform_headers(&self, headers: &mut HeaderMap) {
        for transform in &self.config.headers {
            match transform.action {
                TransformAction::Add | TransformAction::Set => {
                    if let Some(value) = &transform.value {
                        if let (Ok(name), Ok(val)) = (
                            HeaderName::from_str(&transform.name),
                            HeaderValue::from_str(value),
                        ) {
                            if transform.action == TransformAction::Add {
                                if !headers.contains_key(&name) {
                                    headers.insert(name, val);
                                }
                            } else {
                                headers.insert(name, val);
                            }
                        }
                    }
                }
                TransformAction::Remove => {
                    if let Ok(name) = HeaderName::from_str(&transform.name) {
                        headers.remove(&name);
                    }
                }
                TransformAction::Replace => {
                    if let (Some(pattern), Some(replacement)) =
                        (&transform.pattern, &transform.replacement)
                    {
                        if let Ok(regex) = Regex::new(pattern) {
                            if let Ok(name) = HeaderName::from_str(&transform.name) {
                                if let Some(value) = headers.get(&name) {
                                    if let Ok(value_str) = value.to_str() {
                                        let new_value = regex.replace_all(value_str, replacement.as_str());
                                        if let Ok(new_val) = HeaderValue::from_str(&new_value) {
                                            headers.insert(name, new_val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Transforms request path according to configuration.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Original request path
    /// 
    /// # Returns
    /// 
    /// Transformed path string
    pub fn transform_path(&self, path: &str) -> String {
        if let (Some(regex), Some(path_config)) = (&self.path_regex, &self.config.path) {
            regex.replace(path, path_config.replacement.as_str()).to_string()
        } else {
            path.to_string()
        }
    }

    /// Transforms query parameters according to configuration.
    /// 
    /// # Arguments
    /// 
    /// * `params` - Mutable reference to query parameters map
    pub fn transform_query_params(&self, params: &mut HashMap<String, String>) {
        for transform in &self.config.query_params {
            match transform.action {
                TransformAction::Add => {
                    if let Some(value) = &transform.value {
                        params.entry(transform.name.clone()).or_insert(value.clone());
                    }
                }
                TransformAction::Set => {
                    if let Some(value) = &transform.value {
                        params.insert(transform.name.clone(), value.clone());
                    }
                }
                TransformAction::Remove => {
                    params.remove(&transform.name);
                }
                TransformAction::Replace => {
                    // Query param replace not implemented yet
                }
            }
        }
    }
}

/// Response transformer service.
///
/// Applies transformation rules to backend responses before sending to clients.
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::middleware::transform::{ResponseTransformer, ResponseTransformation};
/// 
/// let config = ResponseTransformation::default();
/// let transformer = ResponseTransformer::new(config);
/// ```
pub struct ResponseTransformer {
    config: ResponseTransformation,
}

impl ResponseTransformer {
    /// Creates a new response transformer with the given configuration.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Response transformation configuration
    pub fn new(config: ResponseTransformation) -> Self {
        Self { config }
    }

    /// Transforms response headers according to configuration.
    /// 
    /// # Arguments
    /// 
    /// * `headers` - Mutable reference to response headers
    pub fn transform_headers(&self, headers: &mut HeaderMap) {
        for transform in &self.config.headers {
            match transform.action {
                TransformAction::Add | TransformAction::Set => {
                    if let Some(value) = &transform.value {
                        if let (Ok(name), Ok(val)) = (
                            HeaderName::from_str(&transform.name),
                            HeaderValue::from_str(value),
                        ) {
                            if transform.action == TransformAction::Add {
                                if !headers.contains_key(&name) {
                                    headers.insert(name, val);
                                }
                            } else {
                                headers.insert(name, val);
                            }
                        }
                    }
                }
                TransformAction::Remove => {
                    if let Ok(name) = HeaderName::from_str(&transform.name) {
                        headers.remove(&name);
                    }
                }
                TransformAction::Replace => {
                    if let (Some(pattern), Some(replacement)) =
                        (&transform.pattern, &transform.replacement)
                    {
                        if let Ok(regex) = Regex::new(pattern) {
                            if let Ok(name) = HeaderName::from_str(&transform.name) {
                                if let Some(value) = headers.get(&name) {
                                    if let Ok(value_str) = value.to_str() {
                                        let new_value = regex.replace_all(value_str, replacement.as_str());
                                        if let Ok(new_val) = HeaderValue::from_str(&new_value) {
                                            headers.insert(name, new_val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Transforms status code according to configuration mappings.
    /// 
    /// # Arguments
    /// 
    /// * `status` - Original status code
    /// * `path` - Request path (for condition evaluation)
    /// 
    /// # Returns
    /// 
    /// Transformed status code
    pub fn transform_status_code(&self, status: StatusCode, _path: &str) -> StatusCode {
        for mapping in &self.config.status_code_mappings {
            if status == mapping.from {
                // TODO: Evaluate condition if present
                // For now, always apply if status matches
                return mapping.to;
            }
        }
        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::HeaderName;

    #[test]
    fn test_header_transformation_add() {
        let config = RequestTransformation {
            headers: vec![HeaderTransformation {
                action: TransformAction::Add,
                name: "X-Custom".to_string(),
                value: Some("test-value".to_string()),
                pattern: None,
                replacement: None,
            }],
            path: None,
            query_params: vec![],
        };

        let transformer = RequestTransformer::new(config);
        let mut headers = HeaderMap::new();
        
        transformer.transform_headers(&mut headers);
        
        assert_eq!(
            headers.get("X-Custom").unwrap().to_str().unwrap(),
            "test-value"
        );
    }

    #[test]
    fn test_header_transformation_remove() {
        let config = RequestTransformation {
            headers: vec![HeaderTransformation {
                action: TransformAction::Remove,
                name: "Authorization".to_string(),
                value: None,
                pattern: None,
                replacement: None,
            }],
            path: None,
            query_params: vec![],
        };

        let transformer = RequestTransformer::new(config);
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer token")
        );
        
        transformer.transform_headers(&mut headers);
        
        assert!(headers.get("Authorization").is_none());
    }

    #[test]
    fn test_path_transformation() {
        let config = RequestTransformation {
            headers: vec![],
            path: Some(PathTransformation {
                pattern: r"^/api/v1/(.+)$".to_string(),
                replacement: "/$1".to_string(),
            }),
            query_params: vec![],
        };

        let transformer = RequestTransformer::new(config);
        let transformed = transformer.transform_path("/api/v1/users/123");
        
        assert_eq!(transformed, "/users/123");
    }

    #[test]
    fn test_query_param_transformation() {
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
                    action: TransformAction::Remove,
                    name: "debug".to_string(),
                    value: None,
                },
            ],
        };

        let transformer = RequestTransformer::new(config);
        let mut params = HashMap::new();
        params.insert("debug".to_string(), "true".to_string());
        
        transformer.transform_query_params(&mut params);
        
        assert!(params.contains_key("api_key"));
        assert!(!params.contains_key("debug"));
    }

    #[test]
    fn test_status_code_mapping() {
        let config = ResponseTransformation {
            headers: vec![],
            status_code_mappings: vec![StatusCodeMapping {
                from: StatusCode::NOT_FOUND,
                to: StatusCode::OK,
                condition: None,
            }],
        };

        let transformer = ResponseTransformer::new(config);
        let mapped = transformer.transform_status_code(StatusCode::NOT_FOUND, "/test");
        
        assert_eq!(mapped, StatusCode::OK);
    }
}
