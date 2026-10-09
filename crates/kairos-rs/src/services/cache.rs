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
use crate::models::router::Router;
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
    /// Opaque ETag value (e.g. `"abcdef0123456789"`). Preserved from the
    /// upstream `ETag` header when present, otherwise derived from the
    /// response body via [`compute_etag`]. Empty string means
    /// "no ETag available" — callers should skip 304 logic in that case.
    pub etag: String,
}

/// Read-only snapshot of cache statistics.
#[derive(Debug, Clone, Copy)]
pub struct CacheStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    /// Total entries evicted (LRU + TTL + explicit invalidate). Counter.
    pub evictions: u64,
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

/// Compute a strong ETag for a response body.
///
/// Uses `ahash` (non-cryptographic, fast, high-quality). The returned
/// value is a strong ETag per RFC 7232 §2.3 — wrapped in double quotes,
/// suitable for direct insertion into an `ETag` or `If-None-Match`
/// header.
///
/// Note: this is NOT a cryptographic hash. A determined attacker could
/// craft a body with the same hash, but for cache validation on a
/// trusted gateway this is fine. If cryptographic guarantees are needed
/// later, swap in `sha2` or `blake3`.
pub fn compute_etag(body: &[u8]) -> String {
    let mut hasher = AHasher::default();
    hasher.write(body);
    format!("\"{:016x}\"", hasher.finish())
}

/// Check whether an `If-None-Match` header value matches a stored ETag.
///
/// Per RFC 7232 §3.2, the comparison is byte-equivalent (the `W/` weak
/// prefix is stripped before comparison). The wildcard `*` matches any
/// current representation. Multiple ETags can be supplied as a
/// comma-separated string — a match against any one of them counts.
pub fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    let inm = if_none_match.trim();
    if inm == "*" {
        return true;
    }
    for tag in inm.split(',') {
        let t = tag.trim();
        // RFC 7232 §2.2: weak prefix is case-insensitive (W/ and w/ both
        // valid). Strip it before comparison so weak-equality holds.
        let stripped = if t.len() >= 2 && t[..2].eq_ignore_ascii_case("W/") {
            &t[2..]
        } else {
            t
        };
        if stripped == etag {
            return true;
        }
    }
    false
}

/// In-memory LRU + TTL cache backed by [`moka`].
pub struct InMemoryCache {
    cache: MokaCache<CacheKey, CachedResponse>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    evictions: Arc<AtomicU64>,
    default_ttl: Duration,
}

impl InMemoryCache {
    /// Create a new in-memory cache.
    ///
    /// * `max_capacity` - max number of entries before LRU eviction
    /// * `default_ttl` - default TTL applied to entries that don't override it
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        let evictions = Arc::new(AtomicU64::new(0));
        let evictions_for_listener = Arc::clone(&evictions);
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .eviction_listener(move |_key, _value, _cause| {
                // moka fires this on every eviction (LRU, TTL expiry,
                // manual invalidate). We don't care about the cause,
                // just count them — operators want to see eviction rate.
                evictions_for_listener.fetch_add(1, Ordering::Relaxed);
            })
            .build();
        Self {
            cache,
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions,
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
        let evictions = self.evictions.load(Ordering::Relaxed);
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
            evictions,
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

/// Decide whether a matched route should use the in-memory response cache.
///
/// Returns `true` only when the route has `cache.enabled = true` AND is
/// **not** auth-required. The auth check is a Phase 1 safety guard: cache
/// keys only hash `(method, path, query)`, so a cached response for one
/// authenticated user could otherwise be served to another.
///
/// See [`CacheConfig`] docs in `models/router.rs` for the full rationale.
pub fn should_cache_route(route: &Router) -> bool {
    if route.auth_required {
        return false;
    }
    match &route.cache {
        Some(cfg) if cfg.enabled => true,
        _ => false,
    }
}

impl CachedResponse {
    /// Build an `HttpResponse` from a cached entry, preserving status,
    /// headers, and body. Header values that fail to parse are silently
    /// dropped to keep cache hydration robust.
    pub fn to_response(&self) -> HttpResponse {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK);
        let mut builder = HttpResponse::build(status);
        let mut etag_already_set = false;
        for (name, value) in &self.headers {
            if let (Ok(n), Ok(v)) = (HeaderName::try_from(name.as_str()), HeaderValue::from_str(value)) {
                if n.as_str().eq_ignore_ascii_case("etag") {
                    etag_already_set = true;
                }
                builder.insert_header((n, v));
            }
        }
        // Inject our computed ETag only if the upstream didn't include one.
        // This makes `If-None-Match` round-trips work for clients that
        // ask us to cache even when the origin doesn't emit its own ETag.
        if !etag_already_set && !self.etag.is_empty() {
            if let Ok(v) = HeaderValue::from_str(&self.etag) {
                builder.insert_header((actix_web::http::header::ETAG, v));
            }
        }
        builder.body(self.body.clone())
    }

    /// Build a `CachedResponse` from an `HttpResponse` (status + headers)
    /// and a pre-materialized body. The caller must already have read the
    /// response body into bytes (actix bodies are one-shot streams).
    pub fn from_response_parts(status: u16, headers: &[(String, String)], body: Bytes) -> Self {
        // Preserve the upstream ETag if present, otherwise derive one
        // from the body. This lets clients use `If-None-Match` for
        // conditional GETs even when the origin didn't set its own
        // ETag header.
        let etag = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("etag"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| compute_etag(&body));
        Self {
            status,
            headers: headers.to_vec(),
            body,
            etag,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_cache_route_respects_auth_required() {
        use crate::models::router::{CacheConfig, Router};
        // auth_required: true + cache enabled -> must NOT cache
        let r = Router {
            host: Some("http://x".into()),
            port: Some(80),
            backends: None,
            protocol: crate::models::router::Protocol::Http,
            load_balancing_strategy: Default::default(),
            external_path: "/".into(),
            internal_path: "/".into(),
            methods: vec!["GET".into()],
            auth_required: true,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
            cache: Some(CacheConfig {
                enabled: true,
                ttl_secs: 60,
                max_size: 100,
                coalesce: false,
            }),
        };
        assert!(!should_cache_route(&r), "must skip cache for auth_required");
    }

    #[test]
    fn should_cache_route_allows_public_cache_enabled() {
        use crate::models::router::{CacheConfig, Router};
        let r = Router {
            host: Some("http://x".into()),
            port: Some(80),
            backends: None,
            protocol: crate::models::router::Protocol::Http,
            load_balancing_strategy: Default::default(),
            external_path: "/".into(),
            internal_path: "/".into(),
            methods: vec!["GET".into()],
            auth_required: false,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
            cache: Some(CacheConfig {
                enabled: true,
                ttl_secs: 60,
                max_size: 100,
                coalesce: false,
            }),
        };
        assert!(should_cache_route(&r), "should cache public + enabled");
    }

    #[test]
    fn should_cache_route_disables_when_explicitly_off() {
        use crate::models::router::{CacheConfig, Router};
        let r = Router {
            host: Some("http://x".into()),
            port: Some(80),
            backends: None,
            protocol: crate::models::router::Protocol::Http,
            load_balancing_strategy: Default::default(),
            external_path: "/".into(),
            internal_path: "/".into(),
            methods: vec!["GET".into()],
            auth_required: false,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
            cache: Some(CacheConfig {
                enabled: false,
                ttl_secs: 60,
                max_size: 100,
                coalesce: false,
            }),
        };
        assert!(!should_cache_route(&r), "must skip when cache.enabled=false");
    }

    #[test]
    fn should_cache_route_disables_when_cache_absent() {
        use crate::models::router::Router;
        let r = Router {
            host: Some("http://x".into()),
            port: Some(80),
            backends: None,
            protocol: crate::models::router::Protocol::Http,
            load_balancing_strategy: Default::default(),
            external_path: "/".into(),
            internal_path: "/".into(),
            methods: vec!["GET".into()],
            auth_required: false,
            retry: None,
            request_transformation: None,
            response_transformation: None,
            ai_policy: None,
            cache: None,
        };
        assert!(!should_cache_route(&r), "must skip when cache field absent");
    }

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
            etag: compute_etag(b"{\"ok\":true}"),
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
                    etag: compute_etag(b"x"),
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
                    etag: compute_etag(b"x"),
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
