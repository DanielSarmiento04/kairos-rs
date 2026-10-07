//! Integration tests for the PR6 response caching layer.
//!
//! Covers the public surface of `kairos_rs::services::cache`:
//! - `compute_etag` — deterministic, quoted, sensitive to body
//! - `etag_matches` — RFC 7232 §3.2 semantics (wildcard, weak prefix,
//!   comma-separated list)
//! - `is_cacheable` — method + status + cache-control + set-cookie rules
//! - `should_cache_route` — security guard (skip when auth_required)
//! - `InMemoryCache` — LRU eviction, TTL expiry, eviction counter
//!
//! HTTP-level end-to-end tests (actix server) live in the gateway crate.
//! Here we focus on the cache primitives so a regression in either the
//! helper functions or the in-memory backend is caught fast.

use std::time::Duration;

use bytes::Bytes;
use kairos_rs::models::router::{CacheConfig, Protocol, Router};
use kairos_rs::services::cache::{
    compute_cache_key, compute_etag, etag_matches, is_cacheable, should_cache_route,
    CacheService, CachedResponse, InMemoryCache,
};
use tokio::time::sleep;

// ---------- helpers ----------

fn make_router(auth_required: bool, cache_enabled: bool) -> Router {
    Router {
        host: Some("http://backend".into()),
        port: Some(80),
        backends: None,
        protocol: Protocol::Http,
        load_balancing_strategy: Default::default(),
        external_path: "/api/test".into(),
        internal_path: "/test".into(),
        methods: vec!["GET".into()],
        auth_required,
        retry: None,
        request_transformation: None,
        response_transformation: None,
        ai_policy: None,
        cache: if cache_enabled {
            Some(CacheConfig {
                enabled: true,
                ttl_secs: 60,
                max_size: 100,
            })
        } else {
            None
        },
    }
}

fn sample_response(body: &[u8], etag: &str) -> CachedResponse {
    CachedResponse {
        status: 200,
        headers: vec![("content-type".to_string(), "application/json".to_string())],
        body: Bytes::copy_from_slice(body),
        etag: etag.to_string(),
    }
}

// ---------- compute_etag ----------

#[test]
fn compute_etag_is_deterministic() {
    let a = compute_etag(b"hello");
    let b = compute_etag(b"hello");
    assert_eq!(a, b, "same body must yield same etag");
}

#[test]
fn compute_etag_is_quoted_rfc7232() {
    let tag = compute_etag(b"anything");
    assert!(tag.starts_with('"'), "etag must start with quote, got: {tag}");
    assert!(tag.ends_with('"'), "etag must end with quote, got: {tag}");
    // 16 hex chars inside the quotes (64-bit ahash output)
    let inner = &tag[1..tag.len() - 1];
    assert_eq!(inner.len(), 16, "expected 16 hex chars inside quotes");
    assert!(inner.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn compute_etag_differs_per_body() {
    assert_ne!(compute_etag(b"hello"), compute_etag(b"world"));
    // single-byte difference should change the hash
    assert_ne!(compute_etag(b"abc"), compute_etag(b"abd"));
    // empty vs non-empty
    assert_ne!(compute_etag(b""), compute_etag(b"x"));
}

// ---------- etag_matches ----------

#[test]
fn etag_matches_exact() {
    let etag = "\"abc123\"";
    assert!(etag_matches(etag, etag));
    assert!(!etag_matches("\"def456\"", etag));
}

#[test]
fn etag_matches_wildcard() {
    // RFC 7232: `*` matches any current representation.
    assert!(etag_matches("*", "\"anything\""));
    assert!(etag_matches(" * ", "\"anything\"")); // trim whitespace
}

#[test]
fn etag_matches_strips_weak_prefix() {
    let etag = "\"abc123\"";
    assert!(etag_matches("W/\"abc123\"", etag));
    assert!(etag_matches("w/\"abc123\"", etag)); // case-insensitive prefix
}

#[test]
fn etag_matches_handles_comma_list() {
    let etag = "\"abc123\"";
    assert!(etag_matches("\"def456\", \"abc123\", \"ghi789\"", etag));
    assert!(etag_matches("\"abc123\",\"def456\"", etag));
    assert!(!etag_matches("\"def456\", \"ghi789\"", etag));
}

#[test]
fn etag_matches_handles_whitespace() {
    let etag = "\"abc123\"";
    assert!(etag_matches("  \"abc123\"  ", etag));
    assert!(etag_matches("\t\"abc123\"\t", etag));
}

// ---------- is_cacheable ----------

#[test]
fn is_cacheable_only_get() {
    let hdrs = vec![];
    assert!(is_cacheable("GET", 200, &hdrs));
    assert!(!is_cacheable("POST", 200, &hdrs));
    assert!(!is_cacheable("PUT", 200, &hdrs));
    assert!(!is_cacheable("DELETE", 200, &hdrs));
    assert!(!is_cacheable("PATCH", 200, &hdrs));
    assert!(!is_cacheable("HEAD", 200, &hdrs));
}

#[test]
fn is_cacheable_only_2xx() {
    let hdrs = vec![];
    assert!(is_cacheable("GET", 200, &hdrs));
    assert!(is_cacheable("GET", 201, &hdrs));
    assert!(is_cacheable("GET", 204, &hdrs));
    assert!(is_cacheable("GET", 299, &hdrs));
    assert!(!is_cacheable("GET", 199, &hdrs));
    assert!(!is_cacheable("GET", 301, &hdrs));
    assert!(!is_cacheable("GET", 404, &hdrs));
    assert!(!is_cacheable("GET", 500, &hdrs));
}

#[test]
fn is_cacheable_skips_set_cookie() {
    let hdrs = vec![("Set-Cookie".to_string(), "session=secret".to_string())];
    assert!(
        !is_cacheable("GET", 200, &hdrs),
        "must not cache responses with Set-Cookie (would leak per-user state)"
    );
}

#[test]
fn is_cacheable_skips_cache_control_no_store() {
    let hdrs = vec![(
        "Cache-Control".to_string(),
        "no-store, max-age=0".to_string(),
    )];
    assert!(!is_cacheable("GET", 200, &hdrs));
    // case-insensitive
    let hdrs = vec![("cache-control".to_string(), "NO-STORE".to_string())];
    assert!(!is_cacheable("GET", 200, &hdrs));
}

#[test]
fn is_cacheable_allows_cacheable_header() {
    let hdrs = vec![("Cache-Control".to_string(), "public, max-age=60".to_string())];
    assert!(is_cacheable("GET", 200, &hdrs));
}

// ---------- should_cache_route ----------

#[test]
fn should_cache_route_skips_when_auth_required() {
    // SECURITY: cache keys only hash (method, path, query) — not JWT
    // claims. Caching an auth-required route would leak per-user data.
    let r = make_router(true, true);
    assert!(!should_cache_route(&r));
}

#[test]
fn should_cache_route_allows_when_public_and_enabled() {
    let r = make_router(false, true);
    assert!(should_cache_route(&r));
}

#[test]
fn should_cache_route_skips_when_cache_disabled() {
    let r = make_router(false, false);
    assert!(!should_cache_route(&r));
}

// ---------- compute_cache_key ----------

#[test]
fn compute_cache_key_is_deterministic() {
    let a = compute_cache_key("GET", "/foo", "a=1&b=2");
    let b = compute_cache_key("GET", "/foo", "a=1&b=2");
    assert_eq!(a, b);
}

#[test]
fn compute_cache_key_treats_query_order_as_distinct() {
    // PR6 documents query as opaque to the cache layer. We do NOT
    // normalise query order — `?a=1&b=2` and `?b=2&a=1` are treated as
    // distinct cache keys. If/when upstream sort is added, update both
    // this test and `compute_cache_key`.
    let a = compute_cache_key("GET", "/foo", "a=1&b=2");
    let b = compute_cache_key("GET", "/foo", "b=2&a=1");
    assert_ne!(
        a, b,
        "current implementation treats query order as part of the key"
    );
}

#[test]
fn compute_cache_key_differs_per_method_path() {
    let base = compute_cache_key("GET", "/foo", "");
    assert_ne!(base, compute_cache_key("POST", "/foo", ""));
    assert_ne!(base, compute_cache_key("GET", "/bar", ""));
}

// ---------- InMemoryCache ----------

#[tokio::test]
async fn inmemory_hit_miss_round_trip() {
    let cache = InMemoryCache::new(16, Duration::from_secs(60));
    let key = compute_cache_key("GET", "/foo", "");
    // First read: miss
    assert!(cache.get(&key).await.is_none());
    // Write
    let resp = sample_response(b"{\"ok\":true}", &compute_etag(b"{\"ok\":true}"));
    cache.put(key.clone(), resp, Duration::from_secs(60)).await;
    cache.flush().await;
    // Second read: hit
    let hit = cache.get(&key).await.expect("should hit after put");
    assert_eq!(hit.status, 200);
    assert_eq!(hit.body.as_ref(), b"{\"ok\":true}");
    assert_eq!(hit.etag, compute_etag(b"{\"ok\":true}"));
    // Stats
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
        .put(key.clone(), sample_response(b"x", "fake"), Duration::from_secs(60))
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
        .put(key.clone(), sample_response(b"x", "fake"), Duration::from_millis(100))
        .await;
    assert!(cache.get(&key).await.is_some());
    // Wait past TTL — moka is lazy, so we may need to nudge it.
    sleep(Duration::from_millis(250)).await;
    cache.flush().await;
    assert!(
        cache.get(&key).await.is_none(),
        "entry should have expired after TTL"
    );
}

#[tokio::test]
async fn inmemory_lru_eviction_keeps_size_under_capacity() {
    // moka's eviction policy (TinyLFU + SLRU) is approximate — testing
    // "exactly which entry was evicted" is brittle. Instead, verify the
    // invariants we actually care about: with capacity 2 and 5 inserts,
    // (a) entries count stays ≤ capacity, and (b) the eviction counter
    // increased by ≥ 3 (5 inserts − 2 capacity).
    let cache = InMemoryCache::new(2, Duration::from_secs(60));
    for path in ["/a", "/b", "/c", "/d", "/e"] {
        let k = compute_cache_key("GET", path, "");
        cache
            .put(k, sample_response(b"x", "ex"), Duration::from_secs(60))
            .await;
    }
    cache.flush().await;
    let s = cache.stats();
    assert!(s.entries <= 2, "entries must respect capacity, got {}", s.entries);
    assert!(
        s.evictions >= 3,
        "expected ≥ 3 evictions (5 inserts, capacity 2), got {}",
        s.evictions
    );
}

#[tokio::test]
async fn inmemory_eviction_counter_increments() {
    // Capacity 1 → every insert after the first evicts.
    let cache = InMemoryCache::new(1, Duration::from_secs(60));
    for path in ["/a", "/b", "/c", "/d"] {
        let k = compute_cache_key("GET", path, "");
        cache
            .put(k, sample_response(b"x", "ex"), Duration::from_secs(60))
            .await;
    }
    cache.flush().await;
    let s = cache.stats();
    assert!(
        s.evictions >= 3,
        "expected at least 3 evictions (capacity 1, 4 inserts), got {}",
        s.evictions
    );
}

#[tokio::test]
async fn inmemory_etag_round_trip_via_compute_etag() {
    // Verify the helper produces an etag that etag_matches will recognise
    // — i.e. the storage path is consistent with the lookup path.
    let cache = InMemoryCache::new(16, Duration::from_secs(60));
    let body = b"{\"id\":42}";
    let key = compute_cache_key("GET", "/items/42", "");
    cache
        .put(
            key.clone(),
            sample_response(body, &compute_etag(body)),
            Duration::from_secs(60),
        )
        .await;
    let hit = cache.get(&key).await.unwrap();
    let stored_etag = &hit.etag;
    // Client sends If-None-Match with the same etag — should match.
    assert!(etag_matches(stored_etag, stored_etag));
    // Client sends wildcard — should match.
    assert!(etag_matches("*", stored_etag));
    // Client sends a stale etag — should NOT match.
    assert!(!etag_matches("\"stale-etag\"", stored_etag));
}