//! High-performance in-memory response cache for the Kairos API Gateway.
//!
//! Provides both:
//! - [`InMemoryCache`] & [`CacheService`] (backed by `moka` with ETag / `If-None-Match` 304 support and per-route `CacheConfig` opt-in).
//! - [`ResponseCache`] (lock-free/low-contention `AHashMap` TTL cache with `X-Cache` telemetry and prefix invalidation).

use crate::models::router::Router;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use ahash::{AHashMap, AHasher};
use bytes::Bytes;
use log::{debug, trace};
use moka::future::Cache as MokaCache;
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache key derived from request components (method + path + sorted query).
pub type CacheKey = String;

/// A cached HTTP response: status + headers + body + ETag.
#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Bytes,
    /// Opaque ETag value (e.g. `"abcdef0123456789"`). Preserved from the
    /// upstream `ETag` header when present, otherwise derived from the
    /// response body via [`compute_etag`].
    pub etag: String,
}

/// Read-only snapshot of `InMemoryCache` statistics.
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
#[allow(async_fn_in_trait)]
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

/// Compute a strong ETag for a response body using `ahash`.
pub fn compute_etag(body: &[u8]) -> String {
    let mut hasher = AHasher::default();
    hasher.write(body);
    format!("\"{:016x}\"", hasher.finish())
}

/// Check whether an `If-None-Match` header value matches a stored ETag.
pub fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    let inm = if_none_match.trim();
    if inm == "*" {
        return true;
    }
    for tag in inm.split(',') {
        let t = tag.trim();
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
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        let evictions = Arc::new(AtomicU64::new(0));
        let evictions_for_listener = Arc::clone(&evictions);
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .eviction_listener(move |_key, _value, _cause| {
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

    /// Synchronously run all pending maintenance tasks.
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
pub fn should_cache_route(route: &Router) -> bool {
    if route.auth_required {
        return false;
    }
    matches!(&route.cache, Some(cfg) if cfg.enabled)
}

impl CachedResponse {
    /// Build an `HttpResponse` from a cached entry, preserving status,
    /// headers, body, and ETag.
    pub fn to_response(&self) -> HttpResponse {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK);
        let mut builder = HttpResponse::build(status);
        let mut etag_already_set = false;
        for (name, value) in &self.headers {
            if let (Ok(n), Ok(v)) = (
                HeaderName::try_from(name.as_str()),
                HeaderValue::from_str(value),
            ) {
                if n.as_str().eq_ignore_ascii_case("etag") {
                    etag_already_set = true;
                }
                builder.insert_header((n, v));
            }
        }
        if !etag_already_set && !self.etag.is_empty() {
            if let Ok(v) = HeaderValue::from_str(&self.etag) {
                builder.insert_header((actix_web::http::header::ETAG, v));
            }
        }
        builder.body(self.body.clone())
    }

    /// Build a `CachedResponse` from status, headers, and a pre-materialized body.
    pub fn from_response_parts(status: u16, headers: &[(String, String)], body: Bytes) -> Self {
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

/// Cached HTTP response entry with expiration and headers for [`ResponseCache`].
#[derive(Clone, Debug)]
pub struct CachedHttpEntry {
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

impl CachedHttpEntry {
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

        builder.insert_header(("X-Cache", "HIT"));
        builder.insert_header((
            "X-Cache-TTL",
            HeaderValue::from_str(&self.remaining_ttl_secs().to_string()).unwrap(),
        ));

        builder.body(self.body.clone())
    }
}

/// Statistics summary for [`ResponseCache`] observability.
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

/// Thread-safe in-memory response cache with TTL and prefix invalidation.
pub struct ResponseCache {
    storage: RwLock<AHashMap<String, CachedHttpEntry>>,
    max_entries: usize,
    default_ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl ResponseCache {
    /// Creates a new `ResponseCache` with capacity and default TTL.
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

    #[inline]
    fn make_key(method: &str, uri: &str) -> String {
        format!("{}:{}", method, uri)
    }

    #[inline]
    pub fn is_cacheable(method: &str, status: u16) -> bool {
        let is_idempotent = method == "GET" || method == "HEAD";
        let is_cacheable_status = status == 200 || status == 203 || status == 204 || status == 206;
        is_idempotent && is_cacheable_status
    }

    pub fn get(&self, method: &str, uri: &str) -> Option<CachedHttpEntry> {
        let key = Self::make_key(method, uri);

        {
            let map = self.storage.read().unwrap();
            if let Some(entry) = map.get(&key) {
                if !entry.is_expired() {
                    self.hits.fetch_add(1, Ordering::Relaxed);
                    trace!("Cache HIT for key {}", key);
                    return Some(entry.clone());
                }
            } else {
                self.misses.fetch_add(1, Ordering::Relaxed);
                trace!("Cache MISS for key {}", key);
                return None;
            }
        }

        let mut map = self.storage.write().unwrap();
        if let Some(entry) = map.get(&key) {
            if entry.is_expired() {
                map.remove(&key);
                self.evictions.fetch_add(1, Ordering::Relaxed);
                self.misses.fetch_add(1, Ordering::Relaxed);
                trace!("Cache EXPIRED for key {}", key);
                return None;
            }
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Some(entry.clone());
        }

        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

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

        for (k, v) in headers {
            if k.eq_ignore_ascii_case("cache-control") {
                let v_lower = v.to_lowercase();
                if v_lower.contains("no-store") || v_lower.contains("private") {
                    debug!("Skipping cache for {} due to Cache-Control: {}", uri, v);
                    return;
                }
            }
            if k.eq_ignore_ascii_case("set-cookie") {
                debug!("Skipping cache for {} due to Set-Cookie header", uri);
                return;
            }
        }

        let ttl = custom_ttl.unwrap_or(self.default_ttl);
        let now = Instant::now();
        let entry = CachedHttpEntry {
            status,
            headers: headers.to_vec(),
            body,
            created_at: now,
            expires_at: now + ttl,
        };

        let key = Self::make_key(method, uri);
        let mut map = self.storage.write().unwrap();

        if map.len() >= self.max_entries && !map.contains_key(&key) {
            let expired_keys: Vec<String> = map
                .iter()
                .filter(|(_, v)| v.is_expired())
                .map(|(k, _)| k.clone())
                .collect();

            let evicted_count = expired_keys.len();
            for k in expired_keys {
                map.remove(&k);
            }

            if map.len() >= self.max_entries {
                if let Some(oldest_key) = map
                    .iter()
                    .min_by_key(|(_, v)| v.created_at)
                    .map(|(k, _)| k.clone())
                {
                    map.remove(&oldest_key);
                    self.evictions
                        .fetch_add((evicted_count + 1) as u64, Ordering::Relaxed);
                }
            } else if evicted_count > 0 {
                self.evictions
                    .fetch_add(evicted_count as u64, Ordering::Relaxed);
            }
        }

        map.insert(key, entry);
    }

    pub fn invalidate(&self, uri: &str) -> bool {
        let mut map = self.storage.write().unwrap();
        let get_key = Self::make_key("GET", uri);
        let head_key = Self::make_key("HEAD", uri);
        let r1 = map.remove(&get_key).is_some();
        let r2 = map.remove(&head_key).is_some();
        r1 || r2
    }

    pub fn invalidate_prefix(&self, path_prefix: &str) -> usize {
        let mut map = self.storage.write().unwrap();
        let keys_to_remove: Vec<String> = map
            .keys()
            .filter(|k| {
                if let Some((_, uri)) = k.split_once(':') {
                    uri.starts_with(path_prefix)
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for k in keys_to_remove {
            map.remove(&k);
        }
        count
    }

    pub fn clear(&self) {
        let mut map = self.storage.write().unwrap();
        map.clear();
    }

    pub fn stats(&self) -> CacheStatsSummary {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);
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
            evictions,
            current_entries,
            max_entries: self.max_entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_and_get() {
        let cache = ResponseCache::new(100, Duration::from_secs(30));
        let headers = vec![("content-type".to_string(), "application/json".to_string())];
        let body = actix_web::web::Bytes::from_static(b"{\"ok\":true}");

        cache.put("GET", "/api/users", 200, &headers, body.clone(), None);

        let cached = cache.get("GET", "/api/users");
        assert!(cached.is_some());
        let entry = cached.unwrap();
        assert_eq!(entry.status, 200);
        assert_eq!(entry.body, body);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.current_entries, 1);
    }

    #[test]
    fn test_cache_miss_and_expiration() {
        let cache = ResponseCache::new(100, Duration::from_millis(20));
        let headers = vec![];
        let body = actix_web::web::Bytes::from_static(b"data");

        assert!(cache.get("GET", "/api/missing").is_none());

        cache.put("GET", "/api/short", 200, &headers, body, None);
        assert!(cache.get("GET", "/api/short").is_some());

        std::thread::sleep(Duration::from_millis(30));
        assert!(cache.get("GET", "/api/short").is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_non_cacheable_methods() {
        let cache = ResponseCache::new(100, Duration::from_secs(30));
        let body = actix_web::web::Bytes::from_static(b"created");

        cache.put("POST", "/api/users", 200, &[], body, None);
        assert!(cache.get("POST", "/api/users").is_none());
    }

    #[test]
    fn should_cache_route_respects_auth_required() {
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
        assert!(!should_cache_route(&r));
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
        assert!(cache.get(&key).await.is_none());
        cache.put(key.clone(), resp, Duration::from_secs(60)).await;
        cache.flush().await;
        let hit = cache.get(&key).await.expect("should hit after put");
        assert_eq!(hit.status, 200);
        assert_eq!(hit.body.as_ref(), b"{\"ok\":true}");
    }
}
