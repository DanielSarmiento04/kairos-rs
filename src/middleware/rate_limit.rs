//! Advanced rate limiting middleware with multi-dimensional limiting strategies.
//! 
//! This module provides sophisticated rate limiting capabilities beyond basic
//! per-IP limiting, including per-user, per-route, and composite limiting
//! strategies with sliding window algorithms and burst allowances.

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error as ActixError, HttpMessage,
};
use futures::future::{LocalBoxFuture, Ready};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
    task::{Context, Poll},
};
use serde::{Deserialize, Serialize};
use log::{debug, warn, info};

/// Configuration for advanced rate limiting with multiple strategies.
/// 
/// Supports different rate limiting approaches including per-IP, per-user,
/// per-route, and composite strategies with configurable time windows,
/// burst allowances, and sliding window algorithms.
/// 
/// # Examples
/// 
/// ```rust
/// use std::time::Duration;
/// use kairos_rs::middleware::rate_limit::{RateLimitConfig, LimitStrategy, WindowType};
/// 
/// // Basic per-IP rate limiting
/// let config = RateLimitConfig {
///     strategy: LimitStrategy::PerIP,
///     requests_per_window: 100,
///     window_duration: Duration::from_secs(60),
///     burst_allowance: 20,
///     window_type: WindowType::SlidingWindow,
///     enable_redis: false,
///     redis_key_prefix: "kairos_rl".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// The limiting strategy to apply
    pub strategy: LimitStrategy,
    /// Number of requests allowed per time window
    pub requests_per_window: u64,
    /// Duration of the rate limiting window
    #[serde(with = "duration_serde")]
    pub window_duration: Duration,
    /// Additional requests allowed as burst capacity
    pub burst_allowance: u64,
    /// Type of time window algorithm to use
    pub window_type: WindowType,
    /// Whether to use Redis for distributed rate limiting
    pub enable_redis: bool,
    /// Redis key prefix for distributed limits
    pub redis_key_prefix: String,
}

/// Strategies for rate limiting with different dimensions.
/// 
/// Each strategy applies rate limits based on different request characteristics,
/// allowing for fine-grained control over traffic patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LimitStrategy {
    /// Rate limit based on client IP address
    PerIP,
    /// Rate limit based on authenticated user ID (requires JWT middleware)
    PerUser,
    /// Rate limit based on the requested route/endpoint
    PerRoute,
    /// Rate limit based on IP and route combination
    PerIPAndRoute,
    /// Rate limit based on user and route combination
    PerUserAndRoute,
    /// Composite strategy applying multiple limits
    Composite(Vec<RateLimitConfig>),
}

/// Time window algorithms for rate limiting calculations.
/// 
/// Different algorithms provide trade-offs between memory usage,
/// accuracy, and computational overhead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowType {
    /// Fixed time windows that reset at regular intervals
    FixedWindow,
    /// Sliding window that moves continuously with time
    SlidingWindow,
    /// Token bucket algorithm for smooth rate limiting
    TokenBucket,
}

/// Tracks request counts and timing for rate limiting decisions.
/// 
/// This structure maintains the state needed to determine whether
/// a request should be allowed or rejected based on the configured
/// rate limiting strategy.
#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    /// Number of requests in the current window
    pub request_count: u64,
    /// Timestamp of the window start
    pub window_start: Instant,
    /// Timestamps of individual requests for sliding window
    pub request_times: Vec<Instant>,
    /// Available tokens for token bucket algorithm
    pub available_tokens: f64,
    /// Last token refill time
    pub last_refill: Instant,
}

impl RateLimitEntry {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            request_count: 0,
            window_start: now,
            request_times: Vec::new(),
            available_tokens: 0.0,
            last_refill: now,
        }
    }
}

/// In-memory rate limiting store with time-based cleanup.
/// 
/// Maintains rate limiting state for different keys (IP, user, route)
/// with automatic cleanup of expired entries to prevent memory leaks.
#[derive(Debug)]
pub struct RateLimitStore {
    /// Storage for rate limiting entries
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    /// Last cleanup time to prevent memory growth
    last_cleanup: Arc<RwLock<Instant>>,
}

impl RateLimitStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Checks if a request should be allowed based on rate limiting rules.
    /// 
    /// This method implements the core rate limiting logic, checking
    /// against the configured strategy and updating internal counters.
    /// 
    /// # Parameters
    /// 
    /// * `key` - The rate limiting key (IP, user ID, route, etc.)
    /// * `config` - Rate limiting configuration to apply
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Request is allowed, counters updated
    /// * `Ok(false)` - Request should be rate limited
    /// * `Err()` - Internal error during rate limiting check
    pub fn check_rate_limit(&self, key: &str, config: &RateLimitConfig) -> Result<bool, String> {
        // Cleanup old entries periodically
        self.cleanup_expired_entries(config);

        let mut entries = self.entries.write().map_err(|e| format!("Lock error: {}", e))?;
        let entry = entries.entry(key.to_string()).or_insert_with(RateLimitEntry::new);

        match config.window_type {
            WindowType::FixedWindow => self.check_fixed_window(entry, config),
            WindowType::SlidingWindow => self.check_sliding_window(entry, config),
            WindowType::TokenBucket => self.check_token_bucket(entry, config),
        }
    }

    fn check_fixed_window(&self, entry: &mut RateLimitEntry, config: &RateLimitConfig) -> Result<bool, String> {
        let now = Instant::now();
        let window_duration = config.window_duration;

        // Check if we're in a new window
        if now.duration_since(entry.window_start) >= window_duration {
            entry.window_start = now;
            entry.request_count = 0;
        }

        // Check rate limit
        let allowed = entry.request_count < (config.requests_per_window + config.burst_allowance);
        
        if allowed {
            entry.request_count += 1;
        }

        Ok(allowed)
    }

    fn check_sliding_window(&self, entry: &mut RateLimitEntry, config: &RateLimitConfig) -> Result<bool, String> {
        let now = Instant::now();
        let window_duration = config.window_duration;

        // Remove requests outside the sliding window
        entry.request_times.retain(|&time| now.duration_since(time) < window_duration);

        // Check rate limit
        let current_count = entry.request_times.len() as u64;
        let allowed = current_count < (config.requests_per_window + config.burst_allowance);

        if allowed {
            entry.request_times.push(now);
        }

        Ok(allowed)
    }

    fn check_token_bucket(&self, entry: &mut RateLimitEntry, config: &RateLimitConfig) -> Result<bool, String> {
        let now = Instant::now();
        let time_passed = now.duration_since(entry.last_refill).as_secs_f64();
        
        // Calculate token refill rate (tokens per second)
        let refill_rate = config.requests_per_window as f64 / config.window_duration.as_secs_f64();
        
        // For new entries, initialize with the base request limit (not burst)
        // This allows immediate requests while respecting the configured rate
        if entry.available_tokens == 0.0 && time_passed < 0.001 {
            entry.available_tokens = config.requests_per_window as f64;
        }
        
        // Refill tokens based on time passed
        let max_tokens = (config.requests_per_window + config.burst_allowance) as f64;
        entry.available_tokens = (entry.available_tokens + time_passed * refill_rate).min(max_tokens);
        entry.last_refill = now;

        // Check if we have tokens available
        let allowed = entry.available_tokens >= 1.0;
        
        if allowed {
            entry.available_tokens -= 1.0;
        }

        Ok(allowed)
    }

    fn cleanup_expired_entries(&self, config: &RateLimitConfig) {
        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.write().unwrap();
        
        // Only cleanup every 5 minutes to avoid performance impact
        if now.duration_since(*last_cleanup) < Duration::from_secs(300) {
            return;
        }

        if let Ok(mut entries) = self.entries.write() {
            let cleanup_threshold = config.window_duration * 2; // Keep entries for 2x window duration
            entries.retain(|_, entry| {
                now.duration_since(entry.window_start) < cleanup_threshold
            });
            
            info!("Rate limiter cleanup: {} entries retained", entries.len());
        }

        *last_cleanup = now;
    }
}

/// Advanced rate limiting middleware factory.
/// 
/// Creates middleware instances that apply sophisticated rate limiting
/// strategies based on configuration. Supports multiple limiting dimensions
/// and algorithms for fine-grained traffic control.
#[derive(Clone)]
pub struct AdvancedRateLimit {
    config: RateLimitConfig,
    store: Arc<RateLimitStore>,
}

impl AdvancedRateLimit {
    /// Creates a new advanced rate limiting middleware with the specified configuration.
    /// 
    /// # Parameters
    /// 
    /// * `config` - Rate limiting configuration defining strategy and limits
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use kairos_rs::middleware::rate_limit::{AdvancedRateLimit, RateLimitConfig, LimitStrategy, WindowType};
    /// use std::time::Duration;
    /// 
    /// let config = RateLimitConfig {
    ///     strategy: LimitStrategy::PerIPAndRoute,
    ///     requests_per_window: 60,
    ///     window_duration: Duration::from_secs(60),
    ///     burst_allowance: 10,
    ///     window_type: WindowType::SlidingWindow,
    ///     enable_redis: false,
    ///     redis_key_prefix: "kairos".to_string(),
    /// };
    /// 
    /// let middleware = AdvancedRateLimit::new(config);
    /// ```
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            store: Arc::new(RateLimitStore::new()),
        }
    }

    /// Extracts the rate limiting key based on the configured strategy.
    /// 
    /// Different strategies require different key extraction logic to
    /// identify the dimension being rate limited.
    fn extract_key(&self, req: &ServiceRequest) -> Result<String, String> {
        match &self.config.strategy {
            LimitStrategy::PerIP => {
                let ip = req.connection_info()
                    .peer_addr()
                    .unwrap_or("unknown")
                    .to_string();
                Ok(format!("ip:{}", ip))
            },
            LimitStrategy::PerUser => {
                // Extract user ID from JWT claims
                if let Some(claims) = req.extensions().get::<crate::middleware::auth::Claims>() {
                    Ok(format!("user:{}", claims.sub))
                } else {
                    Err("No user claims found for per-user rate limiting".to_string())
                }
            },
            LimitStrategy::PerRoute => {
                let route = req.path();
                Ok(format!("route:{}", route))
            },
            LimitStrategy::PerIPAndRoute => {
                let conn_info = req.connection_info();
                let ip = conn_info.peer_addr().unwrap_or("unknown");
                let route = req.path();
                Ok(format!("ip_route:{}:{}", ip, route))
            },
            LimitStrategy::PerUserAndRoute => {
                if let Some(claims) = req.extensions().get::<crate::middleware::auth::Claims>() {
                    let route = req.path();
                    Ok(format!("user_route:{}:{}", claims.sub, route))
                } else {
                    Err("No user claims found for per-user-route rate limiting".to_string())
                }
            },
            LimitStrategy::Composite(configs) => {
                // For composite strategies, we'll check the first config's strategy
                // In a full implementation, this would check all strategies
                if let Some(first_config) = configs.first() {
                    let temp_middleware = AdvancedRateLimit::new(first_config.clone());
                    temp_middleware.extract_key(req)
                } else {
                    Err("No configurations in composite strategy".to_string())
                }
            }
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AdvancedRateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Transform = AdvancedRateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(AdvancedRateLimitMiddleware {
            service: Arc::new(service),
            config: self.config.clone(),
            store: self.store.clone(),
        }))
    }
}

/// Advanced rate limiting middleware implementation.
/// 
/// Handles the actual rate limiting logic during request processing,
/// checking limits and rejecting requests that exceed configured thresholds.
pub struct AdvancedRateLimitMiddleware<S> {
    service: Arc<S>,
    config: RateLimitConfig,
    store: Arc<RateLimitStore>,
}

impl<S, B> Service<ServiceRequest> for AdvancedRateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();
        let store = self.store.clone();

        Box::pin(async move {
            // Create a temporary instance to extract the key
            let temp_limiter = AdvancedRateLimit {
                config: config.clone(),
                store: store.clone(),
            };

            // Extract rate limiting key
            let key = match temp_limiter.extract_key(&req) {
                Ok(key) => key,
                Err(err) => {
                    debug!("Failed to extract rate limiting key: {}", err);
                    // For extraction errors, allow the request but log the issue
                    return service.call(req).await;
                }
            };

            // Check rate limit
            match store.check_rate_limit(&key, &config) {
                Ok(true) => {
                    debug!("Rate limit check passed for key: {}", key);
                    service.call(req).await
                },
                Ok(false) => {
                    warn!("Rate limit exceeded for key: {}", key);
                    
                    // Return an actix error which will be handled properly
                    let error_msg = serde_json::json!({
                        "error": "Rate limit exceeded",
                        "message": "Too many requests. Please try again later.",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "type": "rate_limit_error"
                    }).to_string();
                    
                    Err(ActixError::from(actix_web::error::ErrorTooManyRequests(error_msg)))
                },
                Err(err) => {
                    warn!("Rate limiting error: {}", err);
                    // On internal error, allow the request but log the issue
                    service.call(req).await
                }
            }
        })
    }
}

/// Serialization/deserialization support for Duration in configuration.
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limit_entry_creation() {
        let entry = RateLimitEntry::new();
        assert_eq!(entry.request_count, 0);
        assert!(entry.request_times.is_empty());
        assert_eq!(entry.available_tokens, 0.0);
    }

    #[test]
    fn test_fixed_window_rate_limiting() {
        let store = RateLimitStore::new();
        let config = RateLimitConfig {
            strategy: LimitStrategy::PerIP,
            requests_per_window: 2,
            window_duration: Duration::from_secs(60),
            burst_allowance: 1,
            window_type: WindowType::FixedWindow,
            enable_redis: false,
            redis_key_prefix: "test".to_string(),
        };

        // First 3 requests should be allowed (2 + 1 burst)
        assert!(store.check_rate_limit("test_key", &config).unwrap());
        assert!(store.check_rate_limit("test_key", &config).unwrap());
        assert!(store.check_rate_limit("test_key", &config).unwrap());
        
        // Fourth request should be rejected
        assert!(!store.check_rate_limit("test_key", &config).unwrap());
    }

    #[test]
    fn test_token_bucket_rate_limiting() {
        let store = RateLimitStore::new();
        let config = RateLimitConfig {
            strategy: LimitStrategy::PerIP,
            requests_per_window: 10,
            window_duration: Duration::from_secs(10),
            burst_allowance: 5,
            window_type: WindowType::TokenBucket,
            enable_redis: false,
            redis_key_prefix: "test".to_string(),
        };

        // Initial request should be allowed (gets initial tokens)
        assert!(store.check_rate_limit("bucket_test", &config).unwrap());
    }

    #[test]
    fn test_rate_limit_config_serialization() {
        let config = RateLimitConfig {
            strategy: LimitStrategy::PerIP,
            requests_per_window: 100,
            window_duration: Duration::from_secs(60),
            burst_allowance: 20,
            window_type: WindowType::SlidingWindow,
            enable_redis: false,
            redis_key_prefix: "kairos".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RateLimitConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.requests_per_window, deserialized.requests_per_window);
        assert_eq!(config.window_duration, deserialized.window_duration);
    }
}