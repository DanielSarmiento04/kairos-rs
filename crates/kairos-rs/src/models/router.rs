use serde::{Deserialize, Serialize};

/// Configuration for HTTP route forwarding in the kairos-rs gateway.
/// 
/// A `Router` defines how external requests are mapped to internal services,
/// including host translation, path transformation, method validation, and
/// optional authentication requirements.
/// 
/// # Examples
/// 
/// Basic route configuration:
/// ```json
/// {
///   "host": "http://backend-service",
///   "port": 8080,
///   "external_path": "/api/users/{id}",
///   "internal_path": "/v1/user/{id}",
///   "methods": ["GET", "POST", "PUT"],
///   "auth_required": false
/// }
/// ```
/// 
/// Protected route with JWT authentication:
/// ```json
/// {
///   "host": "https://auth-service", 
///   "port": 443,
///   "external_path": "/auth/profile",
///   "internal_path": "/user/profile",
///   "methods": ["GET", "PUT"],
///   "auth_required": true
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Router {
    /// Target host URL including protocol (http:// or https://).
    /// This is where the gateway will forward matching requests.
    /// 
    /// # Examples
    /// - `"http://localhost"`
    /// - `"https://api.example.com"`
    /// - `"http://backend-service"` (for container environments)
    pub host: String,
    
    /// Target port number for the upstream service.
    /// Must be a valid port number between 1 and 65535.
    pub port: u16,
    
    /// External path pattern that clients will use to access the service.
    /// Supports dynamic parameters using `{parameter_name}` syntax.
    /// Must start with a forward slash (`/`).
    /// 
    /// # Examples
    /// - `"/api/users"` (static path)
    /// - `"/api/users/{id}"` (dynamic parameter)
    /// - `"/api/users/{id}/posts/{post_id}"` (multiple parameters)
    pub external_path: String,
    
    /// Internal path pattern for the upstream service.
    /// Parameters from `external_path` can be used here with the same `{parameter_name}` syntax.
    /// Must start with a forward slash (`/`).
    /// 
    /// # Examples
    /// - `"/v1/user"` (path translation)
    /// - `"/v1/user/{id}"` (parameter forwarding)
    /// - `"/legacy/api/user/{id}"` (path and parameter transformation)
    pub internal_path: String,
    
    /// List of allowed HTTP methods for this route.
    /// Only requests with these methods will be forwarded.
    /// 
    /// # Valid Methods
    /// - `GET`, `POST`, `PUT`, `DELETE`, `HEAD`, `OPTIONS`, `PATCH`, `TRACE`
    /// 
    /// # Examples
    /// - `["GET"]` (read-only endpoint)
    /// - `["POST", "PUT"]` (write operations)
    /// - `["GET", "POST", "PUT", "DELETE"]` (full CRUD)
    pub methods: Vec<String>,

    /// Whether JWT authentication is required for this route.
    /// 
    /// When `true`, the request must include a valid JWT token in the
    /// Authorization header. The token will be validated according to
    /// the global JWT configuration.
    /// 
    /// # Default
    /// 
    /// If not specified, defaults to `false` (no authentication required).
    /// 
    /// # Examples
    /// - `false` for public endpoints (health checks, static content)
    /// - `true` for protected endpoints (user data, admin operations)
    #[serde(default)]
    pub auth_required: bool,
}

impl Router {
    /// Validates the router configuration for correctness and security.
    /// 
    /// This method performs comprehensive validation of all router fields:
    /// - Host URL format validation (must include protocol)
    /// - Port range validation (1-65535)
    /// - Path format validation (must start with `/`)
    /// - HTTP method validation (against standard methods)
    /// - Ensures at least one method is specified
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the configuration is valid
    /// - `Err(String)` with a descriptive error message if validation fails
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::models::router::Router;
    /// 
    /// let router = Router {
    ///     host: "http://localhost".to_string(),
    ///     port: 8080,
    ///     external_path: "/api/users".to_string(),
    ///     internal_path: "/v1/users".to_string(),
    ///     methods: vec!["GET".to_string(), "POST".to_string()],
    ///     auth_required: false,
    /// };
    /// 
    /// assert!(router.validate().is_ok());
    /// ```
    /// 
    /// # Errors
    /// 
    /// This method will return an error if:
    /// - Host doesn't start with `http://` or `https://`
    /// - Port is 0 (ports 1-65535 are valid)
    /// - External or internal path doesn't start with `/`
    /// - No HTTP methods are specified
    /// - An invalid HTTP method is provided
    pub fn validate(&self) -> Result<(), String> {
        log::debug!(
            "Validating Router: host={}, port={}, external_path={}, internal_path={}, methods={:?}",
            self.host, self.port, self.external_path, self.internal_path, self.methods
        );

        // Validate host format
        if !self.host.starts_with("http://") && !self.host.starts_with("https://") {
            return Err("Host must start with http:// or https://".to_string());
        }

        // Validate port range (u16 max is 65535, so only check lower bound)
        if self.port == 0 {
            return Err("Port must be between 1 and 65535".to_string());
        }

        // Validate paths start with '/'
        if !self.external_path.starts_with('/') {
            return Err("External path must start with '/'".to_string());
        }

        if !self.internal_path.starts_with('/') {
            return Err("Internal path must start with '/'".to_string());
        }

        // Validate HTTP methods
        if self.methods.is_empty() {
            return Err("At least one HTTP method must be specified".to_string());
        }

        let valid_methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "TRACE"];
        for method in &self.methods {
            if !valid_methods.contains(&method.as_str()) {
                return Err(format!("Invalid HTTP method: {}", method));
            }
        }

        Ok(())
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Settings {
//     pub version: u8,
//     pub routers: Vec<Router>,
// }
