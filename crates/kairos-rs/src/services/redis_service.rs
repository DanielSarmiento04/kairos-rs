//! Distributed Redis service for rate limiting and response caching.
//!
//! Provides high-throughput distributed state management across horizontally-scaled
//! gateway instances with automatic failover to in-memory algorithms.

use crate::models::settings::RedisConfig;
use crate::services::cache::CachedResponse;
use bytes::Bytes;
use log::{error, info, warn};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Serialized representation of [`CachedResponse`] for storage in Redis.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct RedisCachedResponse {
    status: u16,
    headers: Vec<(String, String)>,
    body_base64: String,
    etag: String,
}

/// Service managing distributed Redis operations for rate limiting and response caching.
#[derive(Clone)]
pub struct RedisService {
    config: RedisConfig,
    manager: Arc<RwLock<ConnectionManager>>,
}

impl RedisService {
    /// Creates and initializes a new `RedisService` connection pool.
    pub async fn new(config: RedisConfig) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(config.url.as_str())?;
        let manager = ConnectionManager::new(client).await?;
        info!("Connected to distributed Redis cluster at {}", config.url);
        Ok(Self {
            config,
            manager: Arc::new(RwLock::new(manager)),
        })
    }

    /// Returns a reference to the active Redis configuration.
    pub fn config(&self) -> &RedisConfig {
        &self.config
    }

    /// Checks if the Redis connection is active and responding.
    pub async fn ping(&self) -> bool {
        let mut conn = self.manager.write().await;
        match redis::cmd("PING").query_async::<String>(&mut *conn).await {
            Ok(reply) => reply == "PONG",
            Err(e) => {
                warn!("Redis ping failed: {}", e);
                false
            }
        }
    }

    /// Evaluates rate limiting using an atomic sliding window in Redis.
    ///
    /// # Lua Script Logic
    /// 1. Evicts entries older than `now - window_ms` using `ZREMRANGEBYSCORE`.
    /// 2. Counts requests remaining in the window with `ZCARD`.
    /// 3. If below `limit`, appends `now` with a unique ID using `ZADD`, refreshes TTL with `PEXPIRE`, and returns 1.
    /// 4. Otherwise returns 0 (rate limit exceeded).
    pub async fn check_sliding_window(
        &self,
        key: &str,
        window_duration: Duration,
        limit: u64,
    ) -> Result<bool, String> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let window_ms = window_duration.as_millis() as u64;
        let member = format!("{}-{}", now_ms, uuid::Uuid::new_v4());

        let lua_script = r#"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local window = tonumber(ARGV[2])
            local limit = tonumber(ARGV[3])
            local member = ARGV[4]
            local clear_before = now - window

            redis.call('ZREMRANGEBYSCORE', key, '-inf', clear_before)
            local current_count = redis.call('ZCARD', key)

            if current_count < limit then
                redis.call('ZADD', key, now, member)
                redis.call('PEXPIRE', key, window)
                return 1
            else
                return 0
            end
        "#;

        let mut conn = self.manager.write().await;

        let timeout_duration = Duration::from_millis(self.config.timeout_ms);
        let result = tokio::time::timeout(
            timeout_duration,
            redis::cmd("EVAL")
                .arg(lua_script)
                .arg(1)
                .arg(key)
                .arg(now_ms)
                .arg(window_ms)
                .arg(limit)
                .arg(member)
                .query_async::<i32>(&mut *conn),
        )
        .await;

        match result {
            Ok(Ok(allowed)) => Ok(allowed == 1),
            Ok(Err(redis_err)) => {
                error!("Redis sliding window script error: {}", redis_err);
                Err(redis_err.to_string())
            }
            Err(_) => {
                warn!("Redis sliding window operation timed out after {:?}", timeout_duration);
                Err("Redis timeout".to_string())
            }
        }
    }

    /// Fetches a cached HTTP response from Redis.
    pub async fn get_cached_response(&self, key: &str) -> Result<Option<CachedResponse>, String> {
        let redis_key = format!("kairos:cache:{}", key);
        let mut conn = self.manager.write().await;

        let timeout_duration = Duration::from_millis(self.config.timeout_ms);
        let result: Result<Option<String>, _> = tokio::time::timeout(
            timeout_duration,
            conn.get(&redis_key),
        )
        .await
        .map_err(|_| "Redis timeout".to_string())?;

        match result {
            Ok(Some(raw_json)) => {
                match serde_json::from_str::<RedisCachedResponse>(&raw_json) {
                    Ok(entry) => {
                        use base64::Engine;
                        let body_bytes = base64::engine::general_purpose::STANDARD
                            .decode(&entry.body_base64)
                            .unwrap_or_default();
                        Ok(Some(CachedResponse {
                            status: entry.status,
                            headers: entry.headers,
                            body: Bytes::from(body_bytes),
                            etag: entry.etag,
                        }))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached response from Redis: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Stores a cached HTTP response in Redis with TTL.
    pub async fn set_cached_response(
        &self,
        key: &str,
        response: &CachedResponse,
        ttl_secs: u64,
    ) -> Result<(), String> {
        use base64::Engine;
        let redis_key = format!("kairos:cache:{}", key);
        let body_b64 = base64::engine::general_purpose::STANDARD.encode(&response.body);

        let entry = RedisCachedResponse {
            status: response.status,
            headers: response.headers.clone(),
            body_base64: body_b64,
            etag: response.etag.clone(),
        };

        let raw_json = serde_json::to_string(&entry).map_err(|e| e.to_string())?;
        let mut conn = self.manager.write().await;

        let timeout_duration = Duration::from_millis(self.config.timeout_ms);
        let result: Result<(), _> = tokio::time::timeout(
            timeout_duration,
            conn.set_ex(&redis_key, raw_json, ttl_secs),
        )
        .await
        .map_err(|_| "Redis timeout".to_string())?;

        result.map_err(|e| e.to_string())
    }

    /// Invalidates all cached entries with key prefix in Redis.
    pub async fn invalidate_prefix(&self, prefix: &str) -> Result<usize, String> {
        let pattern = format!("kairos:cache:{}*", prefix);
        let mut conn = self.manager.write().await;

        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .map_err(|e| format!("Failed to find keys: {}", e))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let count = keys.len();
        let _: () = conn
            .del(&keys)
            .await
            .map_err(|e| format!("Failed to delete keys: {}", e))?;

        info!("Invalidated {} keys in Redis for pattern '{}'", count, pattern);
        Ok(count)
    }

    /// Clears all Kairos cache keys from Redis.
    pub async fn clear_cache(&self) -> Result<usize, String> {
        self.invalidate_prefix("").await
    }
}
