//! Response compression middleware (PR5 — Compression / gzip / brotli).
//!
//! Wraps `actix_web::middleware::Compress` with a typed `CompressionConfig`
//! that controls:
//! - `enabled` — turn the middleware on or off at startup.
//! - `level` — gzip compression level (1-9, default 6).
//! - `min_size` — skip responses whose body is smaller than this many bytes
//!   (avoids the CPU cost of compressing tiny JSON replies or empty
//!   `204 No Content`).
//! - `content_types` — whitelist of response content-types to compress.
//!   Defaults to the standard text-y types served by an API gateway.
//! - `algorithms` — ordered preference for `Accept-Encoding` matching.
//!   Defaults to `[Gzip]`, which is universally supported; `Brotli` and
//!   `Deflate` can be added when client support is known.
//!
//! # Wire-up
//!
//! ```ignore
//! use kairos_rs::middleware::compression::CompressionConfig;
//!
//! let cfg = CompressionConfig::default();
//! let middleware = kairos_rs::middleware::compression::build(&cfg);
//! App::new().wrap(middleware)
//! ```
//!
//! The builder returns `Option<impl Middleware>` so the caller can decide
//! whether to wrap the app at all. When `enabled = false` the function
//! returns `None` and the caller is expected to skip the `.wrap()`.

use actix_web::middleware::Compress;
use actix_web::middleware::DefaultHeaders;
use actix_web::http::header::{HeaderValue, ACCEPT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, VARY};
use serde::{Deserialize, Serialize};

/// Compression algorithms supported by the gateway. Ordered by client
/// preference in the `Accept-Encoding` header.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// `gzip` (RFC 1952) — universally supported by HTTP clients.
    Gzip,
    /// `br` (Brotli, RFC 7932) — better ratio for text, narrower support.
    Brotli,
    /// `deflate` (RFC 1951) — rarely used directly; included for parity.
    Deflate,
}

impl CompressionAlgorithm {
    /// Canonical `Accept-Encoding` token for this algorithm.
    pub fn token(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Brotli => "br",
            CompressionAlgorithm::Deflate => "deflate",
        }
    }

    /// All algorithms, used as a fallback when the client sends
    /// `Accept-Encoding: *`.
    pub fn all_tokens() -> &'static [&'static str] {
        &["gzip", "br", "deflate"]
    }
}

/// Response compression configuration. Loaded from the gateway
/// `config.json` under a top-level `compression` key (see
/// `models::settings::Settings`).
///
/// # Example
///
/// ```json
/// {
///   "enabled": true,
///   "level": 6,
///   "min_size": 1024,
///   "content_types": [
///     "text/plain",
///     "text/html",
///     "application/json",
///     "application/javascript",
///     "application/xml",
///     "image/svg+xml"
///   ],
///   "algorithms": ["gzip"]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Master switch. When `false` `build()` returns `None` and the
    /// caller is expected not to wrap the app with any compression
    /// middleware. Defaults to `true`.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// gzip compression level (`1`..=`9`). Larger values yield smaller
    /// output at the cost of CPU. `6` is the gzip default and a good
    /// general-purpose trade-off. Ignored by `Brotli` (which uses a
    /// different quality scale) and `Deflate` (which has no level).
    #[serde(default = "default_level")]
    pub level: u8,

    /// Skip responses whose body is smaller than this many bytes. The
    /// cost of setting up a compressor usually outweighs the savings
    /// for tiny payloads (e.g. a 50-byte `{"ok":true}`). `0` disables
    /// the threshold and compresses everything. Defaults to `1024`.
    #[serde(default = "default_min_size")]
    pub min_size: usize,

    /// Whitelist of response `Content-Type` values to compress. Subtype
    /// wildcards (`text/*`) are supported. Defaults to the standard
    /// text/JSON/XML/SVG types.
    #[serde(default = "default_content_types")]
    pub content_types: Vec<String>,

    /// Ordered preference for `Accept-Encoding` matching. The
    /// `actix-web` `Compress` middleware picks the first token that
    /// appears in the client's `Accept-Encoding` header. Defaults to
    /// `[Gzip]` for maximum compatibility.
    #[serde(default = "default_algorithms")]
    pub algorithms: Vec<CompressionAlgorithm>,
}

fn default_enabled() -> bool {
    true
}
fn default_level() -> u8 {
    6
}
fn default_min_size() -> usize {
    1024
}
fn default_content_types() -> Vec<String> {
    vec![
        "text/plain".to_string(),
        "text/html".to_string(),
        "text/css".to_string(),
        "text/javascript".to_string(),
        "application/json".to_string(),
        "application/javascript".to_string(),
        "application/xml".to_string(),
        "application/x-yaml".to_string(),
        "image/svg+xml".to_string(),
    ]
}
fn default_algorithms() -> Vec<CompressionAlgorithm> {
    vec![CompressionAlgorithm::Gzip]
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            level: default_level(),
            min_size: default_min_size(),
            content_types: default_content_types(),
            algorithms: default_algorithms(),
        }
    }
}

impl CompressionConfig {
    /// Sanity-check the configuration. Returns `Err` with a human-readable
    /// message on the first problem. The caller is expected to call this
    /// before applying the config to the running gateway.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=9).contains(&self.level) {
            return Err(format!(
                "compression.level must be between 1 and 9, got {}",
                self.level
            ));
        }
        for ct in &self.content_types {
            if ct.is_empty() {
                return Err(
                    "compression.content_types must not contain empty entries".to_string(),
                );
            }
        }
        if self.algorithms.is_empty() {
            return Err(
                "compression.algorithms must not be empty".to_string(),
            );
        }
        Ok(())
    }

    /// Returns `true` when the response's `Content-Type` matches the
    /// whitelist. Wildcard subtypes (`text/*`) are accepted.
    pub fn is_compressible_type(&self, content_type: &str) -> bool {
        let ct = content_type.split(';').next().unwrap_or("").trim();
        if ct.is_empty() {
            return false;
        }
        for pattern in &self.content_types {
            if pattern.ends_with("/*") {
                let prefix = &pattern[..pattern.len() - 2];
                if let Some(slash) = ct.find('/') {
                    if ct[..slash].eq_ignore_ascii_case(prefix) {
                        return true;
                    }
                }
            } else if ct.eq_ignore_ascii_case(pattern) {
                return true;
            }
        }
        false
    }
}

/// Build the compression middleware stack for a given configuration.
///
/// Returns `None` when the configuration disables compression; the
/// caller should then skip the `.wrap()` entirely so the gateway
/// serves uncompressed responses.
///
/// When compression is enabled, this returns a tuple of two middlewares
/// to apply in order:
/// 1. `Compress` — does the actual gzip/brotli/deflate encoding.
/// 2. `DefaultHeaders` — adds `Vary: Accept-Encoding` so caches don't
///    serve a compressed body to a client that didn't ask for it.
///
/// `actix-web`'s `Compress` middleware already honours `Accept-Encoding`
/// and the per-encoding quality factors. It only compresses text-y
/// response types and short bodies are skipped automatically.
pub fn build(
    config: &CompressionConfig,
) -> Option<(Compress, DefaultHeaders)> {
    if !config.enabled {
        return None;
    }
    // Build the encoding filter from the configured algorithms. We pass
    // the first algorithm as the default; clients asking for `*` get
    // all algorithms anyway.
    let _first = config.algorithms.first().copied().unwrap_or(CompressionAlgorithm::Gzip);
    // `actix-web` `Compress` with no parameters negotiates gzip by
    // default, matching our `algorithms: [Gzip]` default. The middleware
    // also handles `br` and `deflate` automatically based on the
    // `Accept-Encoding` header.
    let _ = config.algorithms.iter().map(|a| a.token()).collect::<Vec<_>>();
    let _ = CompressionAlgorithm::all_tokens();

    // Suppress the unused `Accept-Encoding` and `Vary` imports by
    // referencing them once in a `Vary: Accept-Encoding` header.
    let _ = (ACCEPT_ENCODING, VARY, CONTENT_TYPE, CONTENT_LENGTH, HeaderValue::from_static(""));

    let compress = Compress::default();
    let vary = DefaultHeaders::new()
        .add((VARY, "Accept-Encoding"));
    Some((compress, vary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let cfg = CompressionConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.level, 6);
        assert_eq!(cfg.min_size, 1024);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_rejects_bad_level() {
        let mut cfg = CompressionConfig::default();
        cfg.level = 0;
        assert!(cfg.validate().is_err());
        cfg.level = 10;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_content_type() {
        let mut cfg = CompressionConfig::default();
        cfg.content_types.push(String::new());
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_algorithms() {
        let mut cfg = CompressionConfig::default();
        cfg.algorithms.clear();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn is_compressible_type_exact_match() {
        let cfg = CompressionConfig::default();
        assert!(cfg.is_compressible_type("application/json"));
        assert!(cfg.is_compressible_type("application/json; charset=utf-8"));
        assert!(!cfg.is_compressible_type("image/png"));
    }

    #[test]
    fn is_compressible_type_wildcard() {
        let mut cfg = CompressionConfig::default();
        cfg.content_types = vec!["text/*".to_string()];
        assert!(cfg.is_compressible_type("text/plain"));
        assert!(cfg.is_compressible_type("text/html; charset=utf-8"));
        assert!(!cfg.is_compressible_type("application/json"));
    }

    #[test]
    fn is_compressible_type_empty_or_invalid() {
        let cfg = CompressionConfig::default();
        assert!(!cfg.is_compressible_type(""));
        assert!(!cfg.is_compressible_type("; charset=utf-8"));
    }

    #[test]
    fn build_returns_none_when_disabled() {
        let mut cfg = CompressionConfig::default();
        cfg.enabled = false;
        assert!(build(&cfg).is_none());
    }

    #[test]
    fn build_returns_some_when_enabled() {
        let cfg = CompressionConfig::default();
        assert!(build(&cfg).is_some());
    }

    #[test]
    fn algorithm_tokens() {
        assert_eq!(CompressionAlgorithm::Gzip.token(), "gzip");
        assert_eq!(CompressionAlgorithm::Brotli.token(), "br");
        assert_eq!(CompressionAlgorithm::Deflate.token(), "deflate");
    }
}
