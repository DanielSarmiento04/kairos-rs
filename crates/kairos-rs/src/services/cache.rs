//! Response cache service for the Kairos gateway.
//!
//! Phase 1: in-memory cache backed by [`moka`]. Caches HTTP responses keyed
//! by request components. Only safe responses are stored (GET, 2xx,
//! no `Set-Cookie`, no `Cache-Control: no-store`).
//!
//! Future phases can plug in a Redis-backed implementation behind the same
//! [`CacheService`] trait.

use ahash::AHasher;
use bytes::Bytes;
use moka::future::Cache as MokaCache;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// Re-export actix types for the helper methods below. Importing actix
// directly in this module keeps `CachedResponse` self-contained.
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::HttpResponse;

/// Cache key derived from request components (method + path + sorted query).
pub type CacheKey = String;

/// A cached HTTP response: status + headers + body.
#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Bytes,
}

/// Read-only snapshot of cache statistics.
#[derive(Debug, Clone, Copy)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    /// `hits / (hits + misses)`, or 0.0 if no requests yet.
    pub hit_rate: f64,
}

/// Pluggable cache service trait.
pub trait CacheService: Send + Sync {
    /// Look up a cached response. Returns `None` on miss.
    async fn get(&self, key: &CacheKey) -> Option<CachedResponse>;
    /// Store a response. TTL is per-call to allow per-route overrides.
    async fn put(&self, key: CacheKey, value: CachedResponse, ttl: Duration);
    /// Read current stats.
    fn stats(&self) -> CacheStatsSnapshot;
    /// Invalidate all entries.
    async fn clear(&self);
}

/// In-memory LRU + TTL cache backed by [`moka`].
pub struct InMemoryCache {
    cache: MokaCache<CacheKey, CachedResponse>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    default_ttl: Duration,
}

impl InMemoryCache {
    /// Create a new in-memory cache.
    ///
    /// * `max_capacity` - max number of entries before LRU eviction
    /// * `default_ttl` - default TTL applied to entries that don't override it
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .build();
        Self {
            cache,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            default_ttl,
        }
    }

    /// Expose the default TTL so the route layer can use it as a fallback.
    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    /// Synchronously run all pending maintenance tasks (entry counting,
    /// expiration, eviction). Intended for use in tests and at shutdown
    /// to ensure consistent stats before reading `entry_count()`.
    pub async fn flush(&self) {
        self.cache.run_pending_tasks().await;
    }
}

impl CacheService for InMemoryCache {
    async fn get(&self, key: &CacheKey) -> Option<CachedResponse> {
        match self.cache.get(key).await {
            Some(v) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(v)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    async fn put(&self, key: CacheKey, value: CachedResponse, ttl: Duration) {
        // Phase 1: moka 0.12 only supports per-cache-instance TTL via the
        // builder. Per-entry TTL would require implementing the `Expiry`
        // trait, deferred to a later phase. We accept `ttl` in the trait
        // signature for forward-compat but currently use the cache's
        // default TTL set at construction time.
        let _ = ttl;
        self.cache.insert(key, value).await;
    }

    fn stats(&self) -> CacheStatsSnapshot {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };
        CacheStatsSnapshot {
            hits,
            misses,
            entries: self.cache.entry_count(),
            hit_rate,
        }
    }

    async fn clear(&self) {
        self.cache.invalidate_all();
    }
}

/// Compute a cache key from request components.
///
/// Uses `ahash` (non-cryptographic, fast, high-quality) over
/// `method | path | sorted_query`. Collisions are vanishingly rare for
/// realistic request volumes; cache lookups are also tolerant of false
/// positives (we'd just return a wrong cached body, which is bad — so we
/// rely on the 64-bit hash space being effectively collision-free).
pub fn compute_cache_key(method: &str, path: &str, query: &str) -> CacheKey {
    let mut hasher = AHasher::default();
    hasher.write(method.as_bytes());
    hasher.write_u8(b'|');
    hasher.write(path.as_bytes());
    hasher.write_u8(b'|');
    hasher.write(query.as_bytes());
    format!("{:x}", hasher.finish())
}

/// Decide whether a response is safe to cache.
///
/// Phase 1 rules: only `GET` method, only 2xx status, skip if response
/// has `Set-Cookie` or `Cache-Control: no-store`.
pub fn is_cacheable(method: &str, status: u16, headers: &[(String, String)]) -> bool {
    if !method.eq_ignore_ascii_case("GET") {
        return false;
    }
    if !(200..300).contains(&status) {
        return false;
    }
    for (name, value) in headers {
        let lname = name.to_ascii_lowercase();
        if lname == "set-cookie" {
            return false;
        }
        if lname == "cache-control" && value.to_ascii_lowercase().contains("no-store") {
            return false;
        }
    }
    true
}

impl CachedResponse {
    /// Build an `HttpResponse` from a cached entry, preserving status,
    /// headers, and body. Header values that fail to parse are silently
    /// dropped to keep cache hydration robust.
    pub fn to_response(&self) -> HttpResponse {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK);
        let mut builder = HttpResponse::build(status);
        for (name, value) in &self.headers {
            if let (Ok(n), Ok(v)) = (HeaderName::try_from(name.as_str()), HeaderValue::from_str(value)) {
                builder.insert_header((n, v));
            }
        }
        builder.body(self.body.clone())
    }

    /// Build a `CachedResponse` from an `HttpResponse` (status + headers)
    /// and a pre-materialized body. The caller must already have read the
    /// response body into bytes (actix bodies are one-shot streams).
    pub fn from_response_parts(status: u16, headers: &[(String, String)], body: Bytes) -> Self {
        Self {
            status,
            headers: headers.to_vec(),
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic() {
        let a = compute_cache_key("GET", "/foo", "a=1&b=2");
        let b = compute_cache_key("GET", "/foo", "a=1&b=2");
        assert_eq!(a, b);
    }

    #[test]
    fn cache_key_differs_by_method() {
        assert_ne!(
            compute_cache_key("GET", "/foo", ""),
            compute_cache_key("POST", "/foo", ""),
        );
    }

    #[test]
    fn cache_key_differs_by_path() {
        assert_ne!(
            compute_cache_key("GET", "/foo", ""),
            compute_cache_key("GET", "/bar", ""),
        );
    }

    #[test]
    fn is_cacheable_only_get() {
        let headers = vec![];
        assert!(is_cacheable("GET", 200, &headers));
        assert!(!is_cacheable("POST", 200, &headers));
        assert!(!is_cacheable("PUT", 200, &headers));
        assert!(!is_cacheable("DELETE", 200, &headers));
    }

    #[test]
    fn is_cacheable_only_2xx() {
        let headers = vec![];
        assert!(is_cacheable("GET", 200, &headers));
        assert!(is_cacheable("GET", 201, &headers));
        assert!(is_cacheable("GET", 204, &headers));
        assert!(!is_cacheable("GET", 301, &headers));
        assert!(!is_cacheable("GET", 404, &headers));
        assert!(!is_cacheable("GET", 500, &headers));
    }

    #[test]
    fn is_cacheable_skips_set_cookie() {
        let headers = vec![("Set-Cookie".to_string(), "session=abc".to_string())];
        assert!(!is_cacheable("GET", 200, &headers));
    }

    #[test]
    fn is_cacheable_skips_cache_control_no_store() {
        let headers = vec![(
            "Cache-Control".to_string(),
            "no-store, max-age=0".to_string(),
        )];
        assert!(!is_cacheable("GET", 200, &headers));
    }

    #[test]
    fn is_cacheable_allows_cacheable_header() {
        let headers = vec![("Cache-Control".to_string(), "public, max-age=60".to_string())];
        assert!(is_cacheable("GET", 200, &headers));
    }

    #[tokio::test]
    async fn inmemory_basic_hit_miss() {
        let cache = InMemoryCache::new(16, Duration::from_secs(60));
        let key = compute_cache_key("GET", "/foo", "");
        let resp = CachedResponse {
            status: 200,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: Bytes::from_static(b"{\"ok\":true}"),
        };
        // miss first
        assert!(cache.get(&key).await.is_none());
        // put + hit
        cache.put(key.clone(), resp, Duration::from_secs(60)).await;
        cache.flush().await; // ensure moka's entry_count is up to date
        let hit = cache.get(&key).await.expect("should hit after put");
        assert_eq!(hit.status, 200);
        assert_eq!(hit.body.as_ref(), b"{\"ok\":true}");
        // stats
        let s = cache.stats();
        assert_eq!(s.hits, 1);
        assert_eq!(s.misses, 1);
        assert_eq!(s.entries, 1);
        assert!((s.hit_rate - 0.5).abs() < 1e-9);
    }

    #[tokio::test]
    async fn inmemory_clear_resets() {
        let cache = InMemoryCache::new(16, Duration::from_secs(60));
        let key = compute_cache_key("GET", "/foo", "");
        cache
            .put(
                key.clone(),
                CachedResponse {
                    status: 200,
                    headers: vec![],
                    body: Bytes::from_static(b"x"),
                },
                Duration::from_secs(60),
            )
            .await;
        assert!(cache.get(&key).await.is_some());
        cache.clear().await;
        assert!(cache.get(&key).await.is_none());
    }

    #[tokio::test]
    async fn inmemory_ttl_expires() {
        let cache = InMemoryCache::new(16, Duration::from_millis(100));
        let key = compute_cache_key("GET", "/foo", "");
        cache
            .put(
                key.clone(),
                CachedResponse {
                    status: 200,
                    headers: vec![],
                    body: Bytes::from_static(b"x"),
                },
                Duration::from_millis(100),
            )
            .await;
        assert!(cache.get(&key).await.is_some());
        // wait > TTL
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(cache.get(&key).await.is_none(), "entry should have expired");
    }
}
