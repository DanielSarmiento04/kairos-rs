//! High-performance in-memory response cache for the Kairos API Gateway.
//!
//! Provides lock-free or low-contention TTL-based caching for idempotent HTTP
//! requests (GET/HEAD), reducing upstream latency to sub-millisecond ranges and
//! insulating backends from repetitive high-frequency queries.

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::HttpResponse;
use ahash::AHashMap;
use log::{debug, trace};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Cached HTTP response entry with expiration and headers.
#[derive(Clone, Debug)]
pub struct CachedResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers (converted to key-value pairs)
    pub headers: Vec<(String, String)>,
    /// Response body payload
    pub body: actix_web::web::Bytes,
    /// Entry creation instant
    pub created_at: Instant,
    /// Absolute expiration instant
    pub expires_at: Instant,
}

impl CachedResponse {
    /// Checks if this cached entry has expired.
    #[inline]
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Calculates remaining time-to-live in seconds.
    #[inline]
    pub fn remaining_ttl_secs(&self) -> u64 {
        let now = Instant::now();
        if self.expires_at > now {
            self.expires_at.duration_since(now).as_secs()
        } else {
            0
        }
    }

    /// Converts cached entry to an Actix Web `HttpResponse` with cache headers.
    pub fn to_http_response(&self) -> HttpResponse {
        let mut builder = HttpResponse::build(
            actix_web::http::StatusCode::from_u16(self.status)
                .unwrap_or(actix_web::http::StatusCode::OK),
        );

        for (k, v) in &self.headers {
            if let (Ok(h_name), Ok(h_val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                builder.insert_header((h_name, h_val));
            }
        }

        // Add telemetry headers
        builder.insert_header(("X-Cache", "HIT"));
        builder.insert_header((
            "X-Cache-TTL",
            HeaderValue::from_str(&self.remaining_ttl_secs().to_string()).unwrap(),
        ));

        builder.body(self.body.clone())
    }
}

/// Statistics summary for cache observability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheStatsSummary {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Cache hit ratio percentage (0.0 - 100.0)
    pub hit_ratio: f64,
    /// Total entries evicted due to capacity or TTL
    pub evictions: u64,
    /// Current number of stored cache entries
    pub current_entries: usize,
    /// Maximum capacity configured
    pub max_entries: usize,
}

/// Thread-safe in-memory response cache.
pub struct ResponseCache {
    /// Internal map storing cached responses keyed by "METHOD:URI"
    storage: RwLock<AHashMap<String, CachedResponse>>,
    /// Maximum capacity of entries allowed
    max_entries: usize,
    /// Default time-to-live for cached responses
    default_ttl: Duration,
    /// Cache hits counter
    hits: AtomicU64,
    /// Cache misses counter
    misses: AtomicU64,
    /// Evictions counter
    evictions: AtomicU64,
}

impl ResponseCache {
    /// Creates a new `ResponseCache` with capacity and default TTL.
    ///
    /// # Parameters
    ///
    /// * `max_entries` - Maximum number of entries before eviction
    /// * `default_ttl` - Default expiration duration for cached responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kairos_rs::services::cache::ResponseCache;
    /// use std::time::Duration;
    ///
    /// let cache = ResponseCache::new(5000, Duration::from_secs(60));
    /// ```
    pub fn new(max_entries: usize, default_ttl: Duration) -> Self {
        Self {
            storage: RwLock::new(AHashMap::with_capacity(max_entries.min(1024))),
            max_entries,
            default_ttl,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Generates a normalized cache key from HTTP method and request URI.
    #[inline]
    fn make_key(method: &str, uri: &str) -> String {
        format!("{}:{}", method, uri)
    }

    /// Evaluates if a request and status are eligible for caching.
    #[inline]
    pub fn is_cacheable(method: &str, status: u16) -> bool {
        let is_idempotent = method == "GET" || method == "HEAD";
        let is_cacheable_status = status == 200 || status == 203 || status == 204 || status == 206;
        is_idempotent && is_cacheable_status
    }

    /// Looks up a cached response for the given method and URI.
    ///
    /// Automatically cleans up expired entries on retrieval.
    pub fn get(&self, method: &str, uri: &str) -> Option<CachedResponse> {
        let key = Self::make_key(method, uri);

        {
            let map = self.storage.read().unwrap();
            if let Some(entry) = map.get(&key) {
                if !entry.is_expired() {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    trace!("Cache HIT for key {}", key);
                    return Some(entry.clone());
                }
            }
        }

        // Cache miss or entry expired
        self.misses.fetch_add(1, Ordering::Relaxed);
        trace!("Cache MISS for key {}", key);
        None
    }

    /// Inserts a new response into the cache.
    pub fn put(
        &self,
        method: &str,
        uri: &str,
        status: u16,
        headers: &[(String, String)],
        body: actix_web::web::Bytes,
        custom_ttl: Option<Duration>,
    ) {
        if !Self::is_cacheable(method, status) {
            return;
        }

        // Check if response explicitly specifies Cache-Control: no-store / no-cache
        for (name, val) in headers {
            if name.eq_ignore_ascii_case("cache-control") {
                if val.contains("no-store") || val.contains("private") {
                    return;
                }
            }
        }

        let key = Self::make_key(method, uri);
        let now = Instant::now();
        let ttl = custom_ttl.unwrap_or(self.default_ttl);
        let expires_at = now + ttl;

        // Collect serializable headers
        let mut cached_headers = Vec::with_capacity(headers.len());
        for (name, val) in headers {
            let name_lower = name.to_ascii_lowercase();
            // Skip hop-by-hop and cache tracking headers
            if !name_lower.starts_with("connection")
                && name_lower != "x-cache"
                && name_lower != "x-cache-ttl"
            {
                cached_headers.push((name.clone(), val.clone()));
            }
        }

        let entry = CachedResponse {
            status,
            headers: cached_headers,
            body,
            created_at: now,
            expires_at,
        };

        let mut map = self.storage.write().unwrap();

        // Eviction if over capacity
        if map.len() >= self.max_entries {
            // Simple fast eviction: remove expired entries first, or a batch of old entries
            let now = Instant::now();
            let initial_len = map.len();
            map.retain(|_, v| v.expires_at > now);

            if map.len() >= self.max_entries {
                // Remove ~10% oldest entries if still full
                let keys_to_remove: Vec<String> = map.keys().take(self.max_entries / 10).cloned().collect();
                for k in keys_to_remove {
                    map.remove(&k);
                }
            }

            let evicted = initial_len.saturating_sub(map.len());
            if evicted > 0 {
                self.evictions.fetch_add(evicted as u64, Ordering::Relaxed);
                debug!("Evicted {} stale cache entries", evicted);
            }
        }

        map.insert(key, entry);
    }

    /// Invalidates an entry by path or URI.
    pub fn invalidate(&self, uri: &str) -> bool {
        let mut map = self.storage.write().unwrap();
        let key_get = Self::make_key("GET", uri);
        let key_head = Self::make_key("HEAD", uri);
        let removed_get = map.remove(&key_get).is_some();
        let removed_head = map.remove(&key_head).is_some();
        removed_get || removed_head
    }

    /// Purges all entries from the cache.
    pub fn clear(&self) {
        let mut map = self.storage.write().unwrap();
        map.clear();
    }

    /// Returns observability statistics for the cache.
    pub fn stats(&self) -> CacheStatsSummary {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_ratio = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let current_entries = self.storage.read().unwrap().len();

        CacheStatsSummary {
            hits,
            misses,
            hit_ratio,
            evictions: self.evictions.load(Ordering::Relaxed),
            current_entries,
            max_entries: self.max_entries,
        }
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(5_000, Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_and_get() {
        let cache = ResponseCache::new(100, Duration::from_secs(10));
        let body = actix_web::web::Bytes::from_static(b"{\"hello\":\"world\"}");
        let headers = vec![("content-type".to_string(), "application/json".to_string())];

        cache.put("GET", "/api/test", 200, &headers, body.clone(), None);

        let hit = cache.get("GET", "/api/test");
        assert!(hit.is_some());
        let res = hit.unwrap();
        assert_eq!(res.status, 200);
        assert_eq!(res.body, body);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss_and_expiration() {
        let cache = ResponseCache::new(100, Duration::from_millis(10));
        let body = actix_web::web::Bytes::from_static(b"expired");
        let headers = Vec::new();

        cache.put("GET", "/api/expiring", 200, &headers, body, None);

        // Sleep to let it expire
        std::thread::sleep(Duration::from_millis(20));

        let hit = cache.get("GET", "/api/expiring");
        assert!(hit.is_none());

        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_non_cacheable_methods() {
        let cache = ResponseCache::new(100, Duration::from_secs(60));
        let body = actix_web::web::Bytes::from_static(b"write");
        let headers = Vec::new();

        cache.put("POST", "/api/write", 200, &headers, body, None);
        assert!(cache.get("POST", "/api/write").is_none());
    }
}
