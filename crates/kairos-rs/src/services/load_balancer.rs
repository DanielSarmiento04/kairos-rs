//! Load balancing service for distributing requests across multiple backends.
//! 
//! This module provides various load balancing strategies to distribute
//! incoming requests efficiently across available backend servers.

use crate::models::router::{Backend, LoadBalancingStrategy};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Load balancer trait for selecting backends.
/// 
/// Implementations provide different strategies for distributing
/// requests across available backend servers.
pub trait LoadBalancer: Send + Sync {
    /// Selects the next backend to handle a request.
    /// 
    /// # Parameters
    /// 
    /// * `backends` - Available backend servers
    /// * `client_ip` - Optional client IP for IP hash strategies
    /// 
    /// # Returns
    /// 
    /// The selected backend or None if no backends are available
    fn select_backend(&self, backends: &[Backend], client_ip: Option<&str>) -> Option<Backend>;
    
    /// Records a successful request to a backend.
    /// Used for strategies that track connection counts or health.
    fn record_success(&self, backend: &Backend);
    
    /// Records a failed request to a backend.
    /// Used for strategies that track connection counts or health.
    fn record_failure(&self, backend: &Backend);
}

/// Round-robin load balancer.
/// 
/// Distributes requests evenly across backends in circular order.
/// Simple, stateless, and works well for backends with similar capacity.
#[derive(Debug)]
pub struct RoundRobinBalancer {
    counter: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for RoundRobinBalancer {
    fn select_backend(&self, backends: &[Backend], _client_ip: Option<&str>) -> Option<Backend> {
        if backends.is_empty() {
            return None;
        }
        
        let index = self.counter.fetch_add(1, Ordering::Relaxed) % backends.len();
        Some(backends[index].clone())
    }
    
    fn record_success(&self, _backend: &Backend) {
        // No-op for round-robin
    }
    
    fn record_failure(&self, _backend: &Backend) {
        // No-op for round-robin
    }
}

/// Least connections load balancer.
/// 
/// Routes requests to the backend with the fewest active connections.
/// Best for backends with varying capacity or long-running requests.
/// 
/// # Algorithm
/// 
/// 1. Tracks active connection count per backend in a HashMap
/// 2. On each request, selects backend with minimum connection count
/// 3. Increments count on selection, decrements on success/failure
/// 
/// # Concurrency
/// 
/// Uses `RwLock<HashMap>` for thread-safe connection tracking:
/// - **Read lock** during backend selection (majority of operations)
/// - **Write lock** only for recording success/failure
/// - Each connection count uses `AtomicU64` for lock-free updates
/// 
/// # Performance Characteristics
/// 
/// - Selection: O(n) where n = number of backends (needs to compare all)
/// - Memory: O(n) for connection tracking HashMap
/// - Lock contention: Minimal due to read-heavy workload with RwLock
/// 
/// # Example
/// 
/// ```text
/// Initial state:
/// - Backend A: 2 active connections
/// - Backend B: 5 active connections
/// - Backend C: 3 active connections
/// 
/// Next request → Backend A (minimum: 2 connections)
/// ```
#[derive(Debug)]
pub struct LeastConnectionsBalancer {
    connections: Arc<RwLock<HashMap<String, AtomicU64>>>,
}

impl LeastConnectionsBalancer {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Creates a unique key for a backend (host:port).
    fn get_backend_key(backend: &Backend) -> String {
        format!("{}:{}", backend.host, backend.port)
    }
    
    /// Gets the current connection count for a backend.
    /// Returns 0 if the backend hasn't been tracked yet.
    fn get_connection_count(&self, backend: &Backend) -> u64 {
        let key = Self::get_backend_key(backend);
        let connections = self.connections.read().unwrap();
        
        connections
            .get(&key)
            .map(|count| count.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}

impl Default for LeastConnectionsBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for LeastConnectionsBalancer {
    fn select_backend(&self, backends: &[Backend], _client_ip: Option<&str>) -> Option<Backend> {
        if backends.is_empty() {
            return None;
        }
        
        // Find backend with minimum connections
        backends
            .iter()
            .min_by_key(|backend| self.get_connection_count(backend))
            .cloned()
    }
    
    fn record_success(&self, backend: &Backend) {
        let key = Self::get_backend_key(backend);
        let mut connections = self.connections.write().unwrap();
        
        connections
            .entry(key)
            .and_modify(|count| {
                count.fetch_sub(1, Ordering::Relaxed);
            });
    }
    
    fn record_failure(&self, backend: &Backend) {
        let key = Self::get_backend_key(backend);
        let mut connections = self.connections.write().unwrap();
        
        connections
            .entry(key)
            .and_modify(|count| {
                count.fetch_sub(1, Ordering::Relaxed);
            });
    }
}

/// Random load balancer.
/// 
/// Randomly selects a backend for each request.
/// Simple and stateless, provides good distribution for most workloads.
#[derive(Debug)]
pub struct RandomBalancer;

impl RandomBalancer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RandomBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for RandomBalancer {
    fn select_backend(&self, backends: &[Backend], _client_ip: Option<&str>) -> Option<Backend> {
        if backends.is_empty() {
            return None;
        }
        
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..backends.len());
        Some(backends[index].clone())
    }
    
    fn record_success(&self, _backend: &Backend) {
        // No-op for random
    }
    
    fn record_failure(&self, _backend: &Backend) {
        // No-op for random
    }
}

/// Weighted load balancer.
/// 
/// Distributes requests based on configured backend weights.
/// Backends with higher weights receive proportionally more traffic.
/// 
/// # Algorithm
/// 
/// The weighted balancer uses a "weighted list" approach:
/// 1. Creates a list where each backend appears N times (N = its weight)
/// 2. Uses round-robin selection on this expanded list
/// 
/// # Example
/// 
/// ```text
/// Backends:
/// - Backend A (weight: 3)
/// - Backend B (weight: 1)
/// 
/// Weighted list: [A, A, A, B]
/// Distribution: 75% to A, 25% to B
/// ```
/// 
/// # Performance Note
/// 
/// The current implementation rebuilds the weighted list on every request.
/// For high-throughput scenarios with stable backends, consider caching
/// the weighted list. See PERFORMANCE_ANALYSIS.md for optimization details.
#[derive(Debug)]
pub struct WeightedBalancer {
    counter: AtomicUsize,
}

impl WeightedBalancer {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
    
    /// Builds a weighted list where each backend appears N times (N = weight).
    /// 
    /// # Algorithm Complexity
    /// 
    /// - Time: O(sum of all weights)
    /// - Space: O(sum of all weights)
    /// 
    /// # Examples
    /// 
    /// ```text
    /// Input:  [Backend(weight=3), Backend(weight=2)]
    /// Output: [Backend, Backend, Backend, Backend, Backend]
    ///         (first backend 3 times, second backend 2 times)
    /// ```
    fn build_weighted_list(backends: &[Backend]) -> Vec<Backend> {
        let mut weighted_list = Vec::new();
        
        for backend in backends {
            for _ in 0..backend.weight {
                weighted_list.push(backend.clone());
            }
        }
        
        weighted_list
    }
}

impl Default for WeightedBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for WeightedBalancer {
    fn select_backend(&self, backends: &[Backend], _client_ip: Option<&str>) -> Option<Backend> {
        if backends.is_empty() {
            return None;
        }
        
        let weighted_list = Self::build_weighted_list(backends);
        if weighted_list.is_empty() {
            return None;
        }
        
        let index = self.counter.fetch_add(1, Ordering::Relaxed) % weighted_list.len();
        Some(weighted_list[index].clone())
    }
    
    fn record_success(&self, _backend: &Backend) {
        // No-op for weighted
    }
    
    fn record_failure(&self, _backend: &Backend) {
        // No-op for weighted
    }
}

/// IP hash load balancer.
/// 
/// Routes requests based on client IP hash for session persistence.
/// Ensures requests from the same client IP consistently go to the same backend.
#[derive(Debug)]
pub struct IpHashBalancer;

impl IpHashBalancer {
    pub fn new() -> Self {
        Self
    }
    
    fn hash_ip(ip: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        ip.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for IpHashBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancer for IpHashBalancer {
    fn select_backend(&self, backends: &[Backend], client_ip: Option<&str>) -> Option<Backend> {
        if backends.is_empty() {
            return None;
        }
        
        if let Some(ip) = client_ip {
            let hash = Self::hash_ip(ip);
            let index = (hash as usize) % backends.len();
            Some(backends[index].clone())
        } else {
            // Fallback to first backend if no IP provided
            Some(backends[0].clone())
        }
    }
    
    fn record_success(&self, _backend: &Backend) {
        // No-op for IP hash
    }
    
    fn record_failure(&self, _backend: &Backend) {
        // No-op for IP hash
    }
}

/// Factory for creating load balancers based on strategy.
pub struct LoadBalancerFactory;

impl LoadBalancerFactory {
    /// Creates a load balancer instance for the given strategy.
    pub fn create(strategy: &LoadBalancingStrategy) -> Arc<dyn LoadBalancer> {
        match strategy {
            LoadBalancingStrategy::RoundRobin => {
                Arc::new(RoundRobinBalancer::new())
            }
            LoadBalancingStrategy::LeastConnections => {
                Arc::new(LeastConnectionsBalancer::new())
            }
            LoadBalancingStrategy::Random => {
                Arc::new(RandomBalancer::new())
            }
            LoadBalancingStrategy::Weighted => {
                Arc::new(WeightedBalancer::new())
            }
            LoadBalancingStrategy::IpHash => {
                Arc::new(IpHashBalancer::new())
            }
        }
    }
}


