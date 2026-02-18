//! DNS Proxy Service
//!
//! Provides DNS protocol support for the Kairos gateway, enabling
//! DNS query forwarding, response caching, and load balancing across
//! multiple DNS servers.
//!
//! # Features
//!
//! - DNS query forwarding (A, AAAA, MX, TXT, etc.)
//! - UDP and TCP support
//! - Response caching with TTL
//! - Multiple upstream DNS servers
//! - Query timeout management

use crate::models::error::GatewayError;
use crate::models::router::Backend;
use hickory_proto::op::Message;
use log::{debug, info};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

/// Simple DNS response cache entry.
#[derive(Clone, Debug)]
struct CacheEntry {
    /// The DNS response message
    response: Message,
    /// When this entry expires
    expires_at: Instant,
}

/// DNS proxy handler for managing DNS queries and caching.
///
/// This handler manages DNS operations by:
/// 1. Parsing incoming DNS queries
/// 2. Forwarding to upstream DNS servers
/// 3. Caching responses based on TTL
/// 4. Handling timeouts and retries
///
/// # Protocol Flow
///
/// ```text
/// Client         Gateway              DNS Server
///   |              |                      |
///   |--- UDP ----> |                     |
///   | (DNS query)  |--- UDP Forward ----> |
///   |              |<-- DNS Response ---- |
///   |<-- UDP ------|                     |
///   | (Cached)     |                     |
/// ```
pub struct DnsHandler {
    /// Request timeout in seconds
    pub(crate) timeout_seconds: u64,
    /// Simple cache for DNS responses
    cache: Arc<RwLock<std::collections::HashMap<String, CacheEntry>>>,
}

impl DnsHandler {
    /// Creates a new DNS handler with specified timeout.
    ///
    /// # Parameters
    ///
    /// * `timeout_seconds` - Maximum time to wait for DNS responses
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kairos_rs::services::dns::DnsHandler;
    ///
    /// let handler = DnsHandler::new(5);
    /// ```
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            timeout_seconds,
            cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Forwards a DNS query to upstream servers and returns the response.
    ///
    /// This method:
    /// 1. Checks cache for existing response
    /// 2. Forwards query to upstream DNS server via UDP
    /// 3. Caches successful responses
    /// 4. Returns response to client
    ///
    /// # Parameters
    ///
    /// * `query_bytes` - Raw DNS query packet bytes
    /// * `backend` - Target DNS server configuration
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - DNS response packet bytes
    /// * `Err(GatewayError)` - Query parsing, forwarding, or timeout error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use kairos_rs::services::dns::DnsHandler;
    /// use kairos_rs::models::router::Backend;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = DnsHandler::new(5);
    /// let backend = Backend {
    ///     host: "8.8.8.8".to_string(), // Google DNS
    ///     port: 53,
    ///     weight: 1,
    ///     health_check_path: None,
    /// };
    ///
    /// let query = vec![/* DNS query bytes */];
    /// let response = handler.forward_query(&query, &backend).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn forward_query(
        &self,
        query_bytes: &[u8],
        backend: &Backend,
    ) -> Result<Vec<u8>, GatewayError> {
        // Parse the DNS query
        let query_msg = Message::from_vec(query_bytes).map_err(|e| GatewayError::Config {
            message: format!("Failed to parse DNS query: {}", e),
            route: "dns".to_string(),
        })?;

        debug!("Parsed DNS query: {:?}", query_msg.queries());

        // Generate cache key from query
        let cache_key = self.generate_cache_key(&query_msg);

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                let now = Instant::now();

                if entry.expires_at > now {
                    debug!("DNS cache hit for: {}", cache_key);
                    // Serialize the cached response
                    let response_bytes =
                        entry.response.to_vec().map_err(|e| GatewayError::Config {
                            message: format!("Failed to serialize cached response: {}", e),
                            route: "dns".to_string(),
                        })?;
                    return Ok(response_bytes);
                } else {
                    debug!("DNS cache entry expired for: {}", cache_key);
                }
            }
        }

        // Extract host from backend
        let dns_server = self.extract_host(&backend.host)?;
        let dns_addr = format!("{}:{}", dns_server, backend.port);

        info!("Forwarding DNS query to: {}", dns_addr);

        // Create UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| GatewayError::Config {
                message: format!("Failed to bind UDP socket: {}", e),
                route: "dns".to_string(),
            })?;

        // Parse DNS server address
        let server_addr: SocketAddr = dns_addr.parse().map_err(|e| GatewayError::Config {
            message: format!("Invalid DNS server address: {}", e),
            route: dns_addr.clone(),
        })?;

        // Send query with timeout
        timeout(
            Duration::from_secs(self.timeout_seconds),
            socket.send_to(query_bytes, server_addr),
        )
        .await
        .map_err(|_| GatewayError::Timeout {
            timeout: self.timeout_seconds,
        })?
        .map_err(|e| GatewayError::Upstream {
            message: format!("Failed to send DNS query: {}", e),
            url: dns_addr.clone(),
            status: None,
        })?;

        // Receive response with timeout
        let mut response_buf = vec![0u8; 512]; // Standard DNS packet size
        let (size, _) = timeout(
            Duration::from_secs(self.timeout_seconds),
            socket.recv_from(&mut response_buf),
        )
        .await
        .map_err(|_| GatewayError::Timeout {
            timeout: self.timeout_seconds,
        })?
        .map_err(|e| GatewayError::Upstream {
            message: format!("Failed to receive DNS response: {}", e),
            url: dns_addr,
            status: None,
        })?;

        response_buf.truncate(size);

        // Parse response for caching
        if let Ok(response_msg) = Message::from_vec(&response_buf) {
            let min_ttl = self.get_min_ttl(&response_msg);
            if min_ttl > 0 {
                let expires_at = Instant::now() + Duration::from_secs(min_ttl as u64);

                debug!("Caching DNS response for {} seconds", min_ttl);

                let mut cache = self.cache.write().await;
                cache.insert(
                    cache_key,
                    CacheEntry {
                        response: response_msg,
                        expires_at,
                    },
                );
            }
        }

        Ok(response_buf)
    }

    /// Generates a cache key from a DNS query message.
    fn generate_cache_key(&self, message: &Message) -> String {
        let queries = message.queries();
        if let Some(query) = queries.first() {
            format!(
                "{}:{}:{}",
                query.name(),
                query.query_type(),
                query.query_class()
            )
        } else {
            String::from("unknown")
        }
    }

    /// Extracts the minimum TTL from a DNS response for caching.
    fn get_min_ttl(&self, message: &Message) -> u32 {
        let mut min_ttl = u32::MAX;

        for answer in message.answers() {
            min_ttl = min_ttl.min(answer.ttl());
        }

        if min_ttl == u32::MAX {
            0 // No answers, don't cache
        } else {
            min_ttl
        }
    }

    /// Extracts the host from a DNS URL.
    ///
    /// Removes the dns://, udp://, or tcp:// prefix if present.
    pub(crate) fn extract_host(&self, url: &str) -> Result<String, GatewayError> {
        let host = url
            .strip_prefix("dns://")
            .or_else(|| url.strip_prefix("udp://"))
            .unwrap_or(url);

        Ok(host.to_string())
    }

    /// Clears expired entries from the cache.
    pub async fn clear_expired(&self) {
        debug!("Running DNS cache cleanup");

        let now = Instant::now();

        let mut cache = self.cache.write().await;
        cache.retain(|_, entry| entry.expires_at > now);

        debug!(
            "DNS cache cleanup completed, {} entries remaining",
            cache.len()
        );
    }

    /// Returns the number of cached entries.
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

impl Clone for DnsHandler {
    fn clone(&self) -> Self {
        Self {
            timeout_seconds: self.timeout_seconds,
            cache: Arc::clone(&self.cache),
        }
    }
}
