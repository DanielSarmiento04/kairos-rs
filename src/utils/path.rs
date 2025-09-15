/// Formats a complete URL for upstream service requests.
/// 
/// This utility function constructs the target URL for forwarding requests
/// to upstream services by combining the host, port, and internal path.
/// 
/// # Parameters
/// 
/// * `host` - The upstream service host URL including protocol (e.g., "http://backend-service")
/// * `port` - The port number for the upstream service
/// * `internal_path` - The internal path to append to the host URL
/// 
/// # Returns
/// 
/// A complete URL string ready for HTTP requests
/// 
/// # URL Format
/// 
/// The returned URL follows the format: `{host}:{port}{internal_path}`
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::utils::path::format_route;
/// 
/// let url = format_route("http://api-server", &8080, "/v1/users/123");
/// assert_eq!(url, "http://api-server:8080/v1/users/123");
/// 
/// let url = format_route("https://secure-api", &443, "/auth/token");
/// assert_eq!(url, "https://secure-api:443/auth/token");
/// ```
/// 
/// # Use Cases
/// 
/// This function is primarily used by the route handler when:
/// - Forwarding client requests to upstream services
/// - Converting internal path patterns to actual URLs
/// - Building target URLs for HTTP client requests
/// 
/// # Notes
/// 
/// - The function assumes the host includes the protocol (http:// or https://)
/// - The internal_path should start with a forward slash (`/`)
/// - Port numbers are always included in the output, even for standard ports
pub fn format_route(host: &str, port: &u16, internal_path: &str) -> String {
    format!(
        "{}:{}{}",
        host,
        port,
        internal_path
    )
}