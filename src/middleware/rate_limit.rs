//! Advanced rate limiting with per-route configuration support.
//! 
//! This module provides flexible rate limiting capabilities that go beyond
//! the basic global rate limiting. It supports different rate limits for
//! different route patterns and can be configured via environment variables.

use actix_web::{HttpResponse, Result, dev::ServiceRequest};
use actix_governor::{KeyExtractor, SimpleKeyExtractionError};
use log::{warn, info, debug};
use regex::Regex;

/// Configuration for route-specific rate limiting.
/// 
/// This structure defines rate limiting rules that can be applied to specific
/// route patterns, allowing fine-grained control over request rates for
/// different parts of the API.
#[derive(Debug, Clone)]
pub struct RouteRateLimit {
    /// Route pattern (regex) to match against request paths
    pub pattern: String,
    /// Requests per second allowed for this route
    pub requests_per_second: u64,
    /// Burst capacity for this route
    pub burst_size: u64,
    /// Compiled regex for efficient matching
    pub regex: Regex,
}

impl RouteRateLimit {
    /// Creates a new route rate limit configuration.
    /// 
    /// # Parameters
    /// 
    /// * `pattern` - Regex pattern to match request paths
    /// * `requests_per_second` - Maximum requests per second
    /// * `burst_size` - Maximum burst capacity
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::middleware::rate_limit::RouteRateLimit;
    /// 
    /// // Strict limits for admin endpoints
    /// let admin_limit = RouteRateLimit::new(r"^/admin/.*", 10, 20).unwrap();
    /// 
    /// // More permissive limits for health checks
    /// let health_limit = RouteRateLimit::new(r"^/health$", 100, 200).unwrap();
    /// ```
    pub fn new(pattern: &str, requests_per_second: u64, burst_size: u64) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;
        Ok(Self {
            pattern: pattern.to_string(),
            requests_per_second,
            burst_size,
            regex,
        })
    }
    
    /// Checks if this rate limit applies to the given path.
    pub fn matches(&self, path: &str) -> bool {
        self.regex.is_match(path)
    }
}

/// Custom key extractor for per-route rate limiting.
/// 
/// This extractor determines the rate limiting key based on both the client IP
/// and the route pattern, allowing different rate limits for different endpoints.
#[derive(Clone)]
pub struct RouteBasedKeyExtractor {
    /// Route-specific rate limits
    route_limits: Vec<RouteRateLimit>,
    /// Default rate limit for routes not matching any pattern
    default_requests_per_second: u64,
    default_burst_size: u64,
}

impl RouteBasedKeyExtractor {
    /// Creates a new route-based key extractor with configured limits.
    pub fn new(route_limits: Vec<RouteRateLimit>, default_rps: u64, default_burst: u64) -> Self {
        Self {
            route_limits,
            default_requests_per_second: default_rps,
            default_burst_size: default_burst,
        }
    }
    
    /// Gets the rate limit configuration for a given path.
    pub fn get_limit_for_path(&self, path: &str) -> (u64, u64) {
        for limit in &self.route_limits {
            if limit.matches(path) {
                debug!("Route {} matched pattern {} with limit {}/s", 
                       path, limit.pattern, limit.requests_per_second);
                return (limit.requests_per_second, limit.burst_size);
            }
        }
        
        debug!("Route {} using default limit {}/s", path, self.default_requests_per_second);
        (self.default_requests_per_second, self.default_burst_size)
    }
}

impl KeyExtractor for RouteBasedKeyExtractor {
    type Key = String;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // Get client IP for rate limiting key
        let client_ip = req
            .connection_info()
            .peer_addr()
            .unwrap_or("unknown")
            .to_string();
        
        // Get request path
        let path = req.path();
        
        // Find matching rate limit pattern
        let (rps, _burst) = self.get_limit_for_path(path);
        
        // Create a composite key that includes both IP and rate limit tier
        // This allows us to have different rate limits for the same IP on different routes
        let key = format!("{}:{}rps", client_ip, rps);
        
        Ok(key)
    }
}

/// Loads route-specific rate limits from environment variables.
/// 
/// This function parses environment variables to create route-specific rate limiting
/// configuration. The format is: `KAIROS_RATE_LIMIT_<NAME>=pattern:rps:burst`
/// 
/// # Environment Variable Format
/// 
/// ```bash
/// export KAIROS_RATE_LIMIT_ADMIN="^/admin/.*:5:10"
/// export KAIROS_RATE_LIMIT_AUTH="^/auth/.*:20:40"
/// export KAIROS_RATE_LIMIT_HEALTH="^/health$:200:400"
/// ```
/// 
/// # Returns
/// 
/// A vector of configured route rate limits parsed from environment variables.
pub fn load_route_rate_limits() -> Vec<RouteRateLimit> {
    let mut limits = Vec::new();
    
    for (key, value) in std::env::vars() {
        if key.starts_with("KAIROS_RATE_LIMIT_") && key != "KAIROS_RATE_LIMIT_DEFAULT" {
            match parse_rate_limit_config(&value) {
                Ok(limit) => {
                    info!("Loaded rate limit: {} -> {} req/s (burst: {})", 
                          limit.pattern, limit.requests_per_second, limit.burst_size);
                    limits.push(limit);
                },
                Err(e) => {
                    warn!("Failed to parse rate limit config '{}': {}", value, e);
                }
            }
        }
    }
    
    // Add some sensible defaults if no custom limits are configured
    if limits.is_empty() {
        info!("No custom rate limits configured, using built-in defaults");
        
        // Stricter limits for admin endpoints
        if let Ok(admin_limit) = RouteRateLimit::new(r"^/admin/.*", 10, 20) {
            limits.push(admin_limit);
        }
        
        // More permissive for health/metrics
        if let Ok(health_limit) = RouteRateLimit::new(r"^/(health|metrics)$", 200, 400) {
            limits.push(health_limit);
        }
    }
    
    limits
}

/// Parses a rate limit configuration string.
/// 
/// Format: "pattern:requests_per_second:burst_size"
/// Example: "^/api/.*:50:100"
fn parse_rate_limit_config(config: &str) -> Result<RouteRateLimit, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = config.split(':').collect();
    if parts.len() != 3 {
        return Err("Rate limit config must be in format 'pattern:rps:burst'".into());
    }
    
    let pattern = parts[0];
    let rps: u64 = parts[1].parse()?;
    let burst: u64 = parts[2].parse()?;
    
    RouteRateLimit::new(pattern, rps, burst)
        .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_rate_limit_matching() {
        let limit = RouteRateLimit::new(r"^/admin/.*", 10, 20).unwrap();
        
        assert!(limit.matches("/admin/status"));
        assert!(limit.matches("/admin/config/reload"));
        assert!(!limit.matches("/health"));
        assert!(!limit.matches("/api/admin"));
    }

    #[test]
    fn test_parse_rate_limit_config() {
        let config = "^/api/.*:50:100";
        let limit = parse_rate_limit_config(config).unwrap();
        
        assert_eq!(limit.pattern, "^/api/.*");
        assert_eq!(limit.requests_per_second, 50);
        assert_eq!(limit.burst_size, 100);
    }

    #[test]
    fn test_key_extractor_limit_selection() {
        let limits = vec![
            RouteRateLimit::new(r"^/admin/.*", 5, 10).unwrap(),
            RouteRateLimit::new(r"^/health$", 100, 200).unwrap(),
        ];
        
        let extractor = RouteBasedKeyExtractor::new(limits, 50, 100);
        
        assert_eq!(extractor.get_limit_for_path("/admin/status"), (5, 10));
        assert_eq!(extractor.get_limit_for_path("/health"), (100, 200));
        assert_eq!(extractor.get_limit_for_path("/api/test"), (50, 100)); // default
    }
}