use crate::models::router::Router;
use serde::{Deserialize, Serialize};

/// Application configuration settings for the kairos-rs gateway.
/// 
/// This structure contains the complete configuration needed to run the gateway,
/// including version information and all route definitions. The configuration
/// is typically loaded from a JSON file and validated before use.
/// 
/// # Configuration File Format
/// 
/// ```json
/// {
///   "version": 1,
///   "routers": [
///     {
///       "host": "http://backend-service",
///       "port": 8080,
///       "external_path": "/api/users/{id}",
///       "internal_path": "/v1/user/{id}",
///       "methods": ["GET", "POST", "PUT"]
///     }
///   ]
/// }
/// ```
/// 
/// # Examples
/// 
/// Loading and validating settings:
/// ```rust
/// use kairos_rs::models::settings::Settings;
/// use kairos_rs::config::settings::load_settings;
/// 
/// let settings = load_settings().expect("Failed to load configuration");
/// settings.validate().expect("Invalid configuration");
/// println!("Loaded {} routes", settings.routers.len());
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    /// Configuration schema version for compatibility checking.
    /// 
    /// This field allows for future configuration format changes while
    /// maintaining backward compatibility. Currently expected to be `1`.
    pub version: u8,
    
    /// Collection of route configurations defining how requests are forwarded.
    /// 
    /// Each router defines a mapping from external client requests to internal
    /// service endpoints, including path transformation and method validation.
    /// The gateway will process routes in the order they appear, with static
    /// routes taking precedence over dynamic (parameterized) routes.
    pub routers: Vec<Router>,
}

impl Settings {
    /// Validates all router configurations in the settings.
    /// 
    /// This method performs comprehensive validation of the entire configuration
    /// by validating each individual router. It ensures that all route definitions
    /// are properly formatted and contain valid values before the gateway starts.
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if all router configurations are valid
    /// - `Err(String)` with the first validation error encountered
    /// 
    /// # Validation Process
    /// 
    /// 1. Iterates through all routers in configuration order
    /// 2. Calls `Router::validate()` on each router
    /// 3. Returns immediately on first validation failure
    /// 4. Only returns `Ok(())` if all routers pass validation
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::models::settings::Settings;
    /// use kairos_rs::models::router::Router;
    /// 
    /// let settings = Settings {
    ///     version: 1,
    ///     routers: vec![
    ///         Router {
    ///             host: "http://localhost".to_string(),
    ///             port: 8080,
    ///             external_path: "/api/test".to_string(),
    ///             internal_path: "/test".to_string(),
    ///             methods: vec!["GET".to_string()],
    ///         }
    ///     ],
    /// };
    /// 
    /// assert!(settings.validate().is_ok());
    /// ```
    /// 
    /// # Errors
    /// 
    /// Returns the first validation error from any router in the configuration.
    /// Common errors include:
    /// - Invalid host URLs (missing protocol)
    /// - Invalid port numbers (0 or out of range)
    /// - Malformed paths (not starting with `/`)
    /// - Invalid HTTP methods
    /// - Empty methods list
    pub fn validate(&self) -> Result<(), String> {
        for route in &self.routers {
            route.validate()?;
        }
        Ok(())
    }
}
