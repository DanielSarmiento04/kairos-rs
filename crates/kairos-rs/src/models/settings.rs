use crate::middleware::rate_limit::RateLimitConfig;
use crate::models::router::Router;
use serde::{Deserialize, Serialize};

/// Configuration for AI capabilities.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiSettings {
    /// The AI provider to use (e.g., "openai", "serialization").
    pub provider: String,
    /// The model identifier (e.g., "gpt-4").
    pub model: String,
    /// API key for the provider. If not set, may be read from environment.
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

/// JWT authentication configuration for the gateway.
///
/// This structure defines the JWT validation parameters used by the
/// authentication middleware when protecting routes.
///
/// # Examples
///
/// ```json
/// {
///   "secret": "your-secret-key",
///   "issuer": "kairos-gateway",
///   "audience": "api-clients",
///   "required_claims": ["sub", "exp"]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwtSettings {
    /// Secret key used for JWT signature validation.
    /// Should be a strong, randomly generated secret.
    pub secret: String,

    /// Optional expected issuer for iss claim validation.
    /// If specified, JWT tokens must have a matching iss claim.
    pub issuer: Option<String>,

    /// Optional expected audience for aud claim validation.
    /// If specified, JWT tokens must have a matching aud claim.
    pub audience: Option<String>,

    /// List of claim names that must be present in valid tokens.
    /// Standard claims include: sub, exp, iat, iss, aud
    #[serde(default)]
    pub required_claims: Vec<String>,
}

impl Default for JwtSettings {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "please-change-this-secret".to_string()),
            issuer: None,
            audience: None,
            required_claims: vec!["sub".to_string(), "exp".to_string()],
        }
    }
}

/// Application configuration settings for the kairos-rs gateway.
///
/// This structure contains the complete configuration needed to run the gateway,
/// including version information, route definitions, and authentication settings.
/// The configuration is typically loaded from a JSON file and validated before use.
///
/// # Configuration File Format
///
/// ```json
/// {
///   "version": 1,
///   "jwt": {
///     "secret": "your-secret-key",
///     "issuer": "kairos-gateway",
///     "audience": "api-clients",
///     "required_claims": ["sub", "exp"]
///   },
///   "routers": [
///     {
///       "host": "http://backend-service",
///       "port": 8080,
///       "external_path": "/api/users/{id}",
///       "internal_path": "/v1/user/{id}",
///       "methods": ["GET", "POST", "PUT"],
///       "auth_required": true
///     }
///   ]
/// }
/// ```
///
/// # Examples
///
/// Loading and validating settings:
/// ```rust
/// # use std::fs;
/// # // Create a temporary config file for testing
/// # let config_content = r#"{"version": 1, "routers": []}"#;
/// # fs::write("./config.json", config_content).unwrap();
/// use kairos_rs::models::settings::Settings;
/// use kairos_rs::config::settings::load_settings;
///
/// let settings = load_settings().expect("Failed to load configuration");
/// settings.validate().expect("Invalid configuration");
/// println!("Loaded {} routes", settings.routers.len());
/// # // Clean up
/// # fs::remove_file("./config.json").ok();
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    /// Configuration schema version for compatibility checking.
    ///
    /// This field allows for future configuration format changes while
    /// maintaining backward compatibility. Currently expected to be `1`.
    pub version: u8,

    /// JWT authentication configuration.
    ///
    /// Optional JWT settings for routes that require authentication.
    /// If not specified, JWT authentication will use default settings
    /// or be disabled if no routes require authentication.
    #[serde(default)]
    pub jwt: Option<JwtSettings>,

    /// Advanced rate limiting configuration.
    ///
    /// Optional rate limiting settings with support for multiple strategies
    /// including per-IP, per-user, per-route, and composite limiting.
    /// If not specified, basic rate limiting will be used.
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,

    /// AI capabilities configuration.
    #[serde(default)]
    pub ai: Option<AiSettings>,

    /// Collection of route configurations defining how requests are forwarded.
    ///
    /// Each router defines a mapping from external client requests to internal
    /// service endpoints, including path transformation and method validation.
    /// The gateway will process routes in the order they appear, with static
    /// routes taking precedence over dynamic (parameterized) routes.
    pub routers: Vec<Router>,
}

impl Settings {
    /// Validates all router configurations and JWT settings.
    ///
    /// This method performs comprehensive validation of the entire configuration
    /// by validating each individual router and the JWT configuration. It ensures
    /// that all route definitions are properly formatted and contain valid values
    /// before the gateway starts.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all configurations are valid
    /// - `Err(String)` with the first validation error encountered
    ///
    /// # Validation Process
    ///
    /// 1. Validates JWT configuration if any routes require authentication
    /// 2. Iterates through all routers in configuration order
    /// 3. Calls `Router::validate()` on each router
    /// 4. Returns immediately on first validation failure
    /// 5. Only returns `Ok(())` if all configurations pass validation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kairos_rs::models::settings::Settings;
    /// use kairos_rs::models::router::{Router, Backend, Protocol};
    ///
    /// let settings = Settings {
    ///     version: 1,
    ///     jwt: None,
    ///     rate_limit: None,
    ///     ai: None,
    ///     routers: vec![
    ///         Router {
    ///             host: Some("http://localhost".to_string()),
    ///             port: Some(8080),
    ///             external_path: "/api/test".to_string(),
    ///             internal_path: "/test".to_string(),
    ///             methods: vec!["GET".to_string()],
    ///             auth_required: false,
    ///             backends: Some(vec![Backend {
    ///                 host: "http://localhost".to_string(),
    ///                 port: 8080,
    ///                 weight: 1,
    ///                 health_check_path: None,
    ///             }]),
    ///             load_balancing_strategy: Default::default(),
    ///             retry: None,
    ///             protocol: Protocol::Http,
    ///             request_transformation: None,
    ///             response_transformation: None,
    ///             ai_policy: None,
    ///         }
    ///     ],
    /// };
    ///
    /// assert!(settings.validate().is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the first validation error from any router or JWT configuration.
    /// Common errors include:
    /// - Missing JWT configuration when routes require authentication
    /// - Invalid host URLs (missing protocol)
    /// - Invalid port numbers (0 or out of range)
    /// - Malformed paths (not starting with `/`)
    /// - Invalid HTTP methods
    /// - Empty methods list
    pub fn validate(&self) -> Result<(), String> {
        // Check if any routes require authentication
        let has_auth_routes = self.routers.iter().any(|r| r.auth_required);

        if has_auth_routes && self.jwt.is_none() {
            return Err(
                "JWT configuration is required when routes have auth_required=true".to_string(),
            );
        }

        // Validate JWT settings if present
        if let Some(ref jwt) = self.jwt {
            if jwt.secret.is_empty() {
                return Err("JWT secret cannot be empty".to_string());
            }
            if jwt.secret == "please-change-this-secret" {
                return Err("JWT secret must be changed from default value".to_string());
            }
            if jwt.secret.len() < 32 {
                return Err("JWT secret should be at least 32 characters for security".to_string());
            }
        }

        // Validate all routers
        for route in &self.routers {
            route.validate()?;
        }

        Ok(())
    }
}
