//! Integration tests for the response-compression middleware (PR5 —
//! Compression / gzip / brotli).
//!
//! Validates that:
//! - `CompressionConfig::default()` produces a valid configuration and
//!   `build()` returns the middleware pair when enabled.
//! - `build()` returns `None` when `enabled = false`, so the caller can
//!   skip `.wrap()` entirely.
//! - `is_compressible_type` accepts both exact `application/json`
//!   matches and wildcard `text/*` matches, while rejecting unrelated
//!   types like `image/png` and `application/octet-stream`.
//! - `validate()` rejects bad levels (`0`, `>9`), empty `content_types`,
//!   and empty `algorithms`.
//! - `CompressionAlgorithm::token()` returns the canonical
//!   `Accept-Encoding` token (`gzip`, `br`, `deflate`).

use kairos_rs::middleware::compression::{
    build, CompressionAlgorithm, CompressionConfig,
};

/// `CompressionConfig::default()` must round-trip through `validate()`
/// and `build()` without panicking.
#[test]
fn test_default_config_is_valid_and_buildable() {
    let cfg = CompressionConfig::default();
    assert!(cfg.enabled, "default must enable compression");
    assert_eq!(cfg.level, 6);
    assert_eq!(cfg.min_size, 1024);
    assert!(cfg.validate().is_ok());
    let built = build(&cfg);
    assert!(built.is_some(), "build() must return Some when enabled");
}

/// `build()` returns `None` when the configuration disables
/// compression, so the caller can skip the `.wrap()` entirely.
#[test]
fn test_build_returns_none_when_disabled() {
    let mut cfg = CompressionConfig::default();
    cfg.enabled = false;
    assert!(build(&cfg).is_none());
}

/// `is_compressible_type` accepts exact matches and the `text/*`
/// wildcard, and rejects everything else.
#[test]
fn test_is_compressible_type() {
    let mut cfg = CompressionConfig::default();
    cfg.content_types = vec![
        "text/*".to_string(),
        "application/json".to_string(),
        "application/xml".to_string(),
    ];
    assert!(cfg.is_compressible_type("text/plain"));
    assert!(cfg.is_compressible_type("text/html; charset=utf-8"));
    assert!(cfg.is_compressible_type("application/json"));
    assert!(cfg.is_compressible_type("application/xml"));
    assert!(!cfg.is_compressible_type("image/png"));
    assert!(!cfg.is_compressible_type("application/octet-stream"));
    assert!(!cfg.is_compressible_type(""));
    assert!(!cfg.is_compressible_type("; charset=utf-8"));
}

/// `validate()` rejects compression levels outside `1..=9`.
#[test]
fn test_validate_rejects_bad_level() {
    let mut cfg = CompressionConfig::default();
    cfg.level = 0;
    assert!(cfg.validate().is_err());
    cfg.level = 10;
    assert!(cfg.validate().is_err());
    cfg.level = 1;
    assert!(cfg.validate().is_ok());
    cfg.level = 9;
    assert!(cfg.validate().is_ok());
}

/// `validate()` rejects an empty `content_types` whitelist.
#[test]
fn test_validate_rejects_empty_content_types() {
    let mut cfg = CompressionConfig::default();
    cfg.content_types.clear();
    assert!(cfg.validate().is_err());
    cfg.content_types.push(String::new());
    assert!(cfg.validate().is_err());
}

/// `validate()` rejects an empty `algorithms` list.
#[test]
fn test_validate_rejects_empty_algorithms() {
    let mut cfg = CompressionConfig::default();
    cfg.algorithms.clear();
    assert!(cfg.validate().is_err());
    cfg.algorithms = vec![CompressionAlgorithm::Gzip];
    assert!(cfg.validate().is_ok());
}

/// `CompressionAlgorithm::token()` returns the canonical
/// `Accept-Encoding` token. We rely on these strings in the
/// documentation and in any future per-algorithm routing logic, so a
/// test guards against accidental renames.
#[test]
fn test_algorithm_tokens() {
    assert_eq!(CompressionAlgorithm::Gzip.token(), "gzip");
    assert_eq!(CompressionAlgorithm::Brotli.token(), "br");
    assert_eq!(CompressionAlgorithm::Deflate.token(), "deflate");
}
