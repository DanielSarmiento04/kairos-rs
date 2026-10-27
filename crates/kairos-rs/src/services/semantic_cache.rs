//! Semantic caching engine for AI / LLM requests in the Kairos API Gateway.
//!
//! Semantic caching matches incoming LLM prompts based on **semantic meaning**
//! rather than exact string equality using dense vector embeddings and cosine similarity:
//!
//! $$\text{sim}(u, v) = \frac{u \cdot v}{\|u\|_2 \|v\|_2}$$
//!
//! If the similarity score meets or exceeds the configured `similarity_threshold`
//! (e.g., `0.85`), the cached completion is served directly with sub-millisecond
//! latency, saving upstream AI token costs and providing instant responses.

use ahash::AHasher;
use bytes::Bytes;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

fn default_true() -> bool {
    true
}

fn default_similarity_threshold() -> f32 {
    0.85
}

fn default_semantic_ttl() -> u64 {
    86400 // 24 hours
}

fn default_embedding_provider() -> String {
    "openai".to_string()
}

fn default_embedding_model() -> String {
    "text-embedding-3-small".to_string()
}

fn default_max_entries() -> usize {
    5000
}

/// Configuration for semantic AI response caching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticCacheConfig {
    /// Whether semantic caching is enabled. Default: true
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Minimum cosine similarity threshold (0.0 to 1.0) required to consider a cache hit.
    /// Default: 0.85 (e.g. 0.90 for high precision, 0.80 for broader match).
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,

    /// Time-to-live for cached responses in seconds. Default: 86400 (24 hours).
    #[serde(default = "default_semantic_ttl")]
    pub ttl_seconds: u64,

    /// Provider for vector embeddings ("openai", "cohere", "local", "mock").
    #[serde(default = "default_embedding_provider")]
    pub embedding_provider: String,

    /// Model name used for vector embeddings.
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,

    /// Maximum number of vector entries retained in memory. Default: 5000.
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            similarity_threshold: default_similarity_threshold(),
            ttl_seconds: default_semantic_ttl(),
            embedding_provider: default_embedding_provider(),
            embedding_model: default_embedding_model(),
            max_entries: default_max_entries(),
        }
    }
}

/// A single cached semantic response entry.
#[derive(Clone, Debug)]
pub struct SemanticCacheEntry {
    /// Unique identifier for this entry
    pub id: String,
    /// Original prompt text
    pub prompt: String,
    /// Normalized embedding vector
    pub vector: Vec<f32>,
    /// HTTP response status
    pub status: u16,
    /// HTTP response headers
    pub headers: Vec<(String, String)>,
    /// Cached response body bytes
    pub body: Bytes,
    /// Creation instant
    pub created_at: Instant,
    /// Expiration instant
    pub expires_at: Instant,
}

impl SemanticCacheEntry {
    /// Returns true if this entry has passed its expiration time.
    #[inline]
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Remaining TTL in seconds.
    #[inline]
    pub fn remaining_ttl_secs(&self) -> u64 {
        let now = Instant::now();
        if self.expires_at > now {
            self.expires_at.duration_since(now).as_secs()
        } else {
            0
        }
    }
}

/// Observability statistics for [`SemanticCache`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Cache hit ratio percentage (0.0 - 100.0)
    pub hit_ratio: f64,
    /// Current number of stored vector entries
    pub current_entries: usize,
    /// Maximum capacity configured
    pub max_entries: usize,
    /// Total entries evicted due to capacity or TTL
    pub evictions: u64,
}

/// Computes the cosine similarity between two vector slices.
///
/// $$\text{sim}(a, b) = \frac{\sum a_i b_i}{\sqrt{\sum a_i^2} \sqrt{\sum b_i^2}}$$
///
/// Returns a value between -1.0 and 1.0 (clamped to [0.0, 1.0] for non-negative embeddings).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Generates a deterministic normalized feature vector from text using
/// character n-grams and word hashing.
///
/// This provides immediate local semantic similarity without requiring external
/// network calls or API keys, ideal for offline/edge usage and reproducible tests.
pub fn vectorize_local(text: &str, dims: usize) -> Vec<f32> {
    let mut vec = vec![0.0f32; dims.max(16)];
    let lower = text.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    // 1. Character tri-grams
    if chars.len() >= 3 {
        for window in chars.windows(3) {
            let mut hasher = AHasher::default();
            for c in window {
                hasher.write_u32(*c as u32);
            }
            let h = hasher.finish();
            let idx = (h as usize) % vec.len();
            vec[idx] += 1.0;
        }
    }

    // 2. Character 4-grams
    if chars.len() >= 4 {
        for window in chars.windows(4) {
            let mut hasher = AHasher::default();
            for c in window {
                hasher.write_u32(*c as u32);
            }
            let h = hasher.finish();
            let idx = (h as usize) % vec.len();
            vec[idx] += 1.5;
        }
    }

    // 3. Word tokens
    for word in lower.split_whitespace() {
        let mut hasher = AHasher::default();
        hasher.write(word.as_bytes());
        let h = hasher.finish();
        let idx = (h as usize) % vec.len();
        vec[idx] += 2.0;
    }

    // Normalize to unit length (L2 norm)
    let norm = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > f32::EPSILON {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}

/// Extracts prompt or query text from common LLM JSON request payloads.
///
/// Handles:
/// - OpenAI Chat format: `{"messages": [{"role": "user", "content": "..."}]}`
/// - OpenAI Completions format: `{"prompt": "..."}`
/// - Generic formats: `{"query": "..."}`, `{"input": "..."}`
pub fn extract_prompt(body: &[u8]) -> Option<String> {
    if body.is_empty() {
        return None;
    }

    // Attempt JSON parsing
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
        // 1. Check messages array (OpenAI / Anthropic style)
        if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
            // Find last user message
            for msg in messages.iter().rev() {
                let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
                if role == "user" || role.is_empty() {
                    if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }

        // 2. Check "prompt" string or array
        if let Some(prompt) = json.get("prompt") {
            if let Some(s) = prompt.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            } else if let Some(arr) = prompt.as_array() {
                if let Some(s) = arr.last().and_then(|v| v.as_str()) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        // 3. Check "query" or "input"
        for key in &["query", "input", "text"] {
            if let Some(s) = json.get(key).and_then(|v| v.as_str()) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    // Fallback: check if plain text
    if let Ok(text) = std::str::from_utf8(body) {
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.len() <= 4096 {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// In-memory semantic vector cache for AI request caching.
pub struct SemanticCache {
    config: SemanticCacheConfig,
    entries: RwLock<Vec<SemanticCacheEntry>>,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl SemanticCache {
    /// Creates a new `SemanticCache` instance with configuration.
    pub fn new(config: SemanticCacheConfig) -> Self {
        Self {
            config,
            entries: RwLock::new(Vec::new()),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Returns a reference to the active configuration.
    pub fn config(&self) -> &SemanticCacheConfig {
        &self.config
    }

    /// Looks up a cached response matching `query_vector` with cosine similarity $\ge$ `threshold`.
    ///
    /// Returns the matching entry and calculated similarity score, or `None` on miss.
    pub fn lookup(&self, query_vector: &[f32], threshold: f32) -> Option<(SemanticCacheEntry, f32)> {
        let entries = match self.entries.read() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        let mut best_match: Option<(SemanticCacheEntry, f32)> = None;

        for entry in entries.iter() {
            if entry.is_expired() {
                continue;
            }

            let sim = cosine_similarity(query_vector, &entry.vector);
            if sim >= threshold {
                match &best_match {
                    Some((_, best_sim)) if sim > *best_sim => {
                        best_match = Some((entry.clone(), sim));
                    }
                    None => {
                        best_match = Some((entry.clone(), sim));
                    }
                    _ => {}
                }
            }
        }

        if let Some((entry, sim)) = best_match {
            self.hits.fetch_add(1, Ordering::Relaxed);
            debug!(
                "Semantic cache HIT (similarity: {:.4} >= threshold: {:.4}) for prompt: '{}'",
                sim, threshold, entry.prompt
            );
            Some((entry, sim))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            trace!("Semantic cache MISS for query vector");
            None
        }
    }

    /// Stores a new prompt response and its embedding vector in the cache.
    pub fn insert(
        &self,
        prompt: String,
        vector: Vec<f32>,
        status: u16,
        headers: Vec<(String, String)>,
        body: Bytes,
        custom_ttl: Option<Duration>,
    ) {
        let ttl = custom_ttl.unwrap_or_else(|| Duration::from_secs(self.config.ttl_seconds));
        let now = Instant::now();
        let expires_at = now + ttl;

        let entry = SemanticCacheEntry {
            id: uuid::Uuid::new_v4().to_string(),
            prompt,
            vector,
            status,
            headers,
            body,
            created_at: now,
            expires_at,
        };

        if let Ok(mut entries) = self.entries.write() {
            // Prune expired entries first
            let initial_len = entries.len();
            entries.retain(|e| !e.is_expired());
            let pruned = initial_len.saturating_sub(entries.len());
            if pruned > 0 {
                self.evictions.fetch_add(pruned as u64, Ordering::Relaxed);
            }

            // Enforce capacity limit
            if entries.len() >= self.config.max_entries {
                entries.remove(0); // Evict oldest
                self.evictions.fetch_add(1, Ordering::Relaxed);
            }

            entries.push(entry);
            info!("Stored new semantic cache entry (current size: {})", entries.len());
        }
    }

    /// Returns a snapshot of observability statistics.
    pub fn stats(&self) -> SemanticCacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);
        let current_entries = self.entries.read().map(|e| e.len()).unwrap_or(0);

        let total = hits + misses;
        let hit_ratio = if total == 0 {
            0.0
        } else {
            (hits as f64 / total as f64) * 100.0
        };

        SemanticCacheStats {
            hits,
            misses,
            hit_ratio,
            current_entries,
            max_entries: self.config.max_entries,
            evictions,
        }
    }

    /// Clears all entries from the semantic cache.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            let count = entries.len();
            entries.clear();
            self.evictions.fetch_add(count as u64, Ordering::Relaxed);
            info!("Cleared semantic cache ({} entries evicted)", count);
        }
    }
}
