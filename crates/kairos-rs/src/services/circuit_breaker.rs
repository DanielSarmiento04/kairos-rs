//! Circuit breaker implementation for upstream service protection.
//! 
//! This module provides a circuit breaker pattern implementation to protect
//! upstream services from cascading failures and provide fast failure responses
//! when services are unavailable.

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use log::{warn, info, debug};

/// State of a circuit breaker.
///
/// # States
///
/// * `Closed` - Normal operation, all requests pass through
/// * `Open` - Circuit tripped, requests fail fast without executing
/// * `HalfOpen` - Testing recovery, limited requests allowed through
///
/// # Examples
///
/// ```
/// use kairos_rs::services::circuit_breaker::CircuitState;
///
/// let state = CircuitState::Closed;
/// match state {
///     CircuitState::Closed => println!("Healthy"),
///     CircuitState::Open => println!("Degraded"),
///     CircuitState::HalfOpen => println!("Recovering"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests pass through
    Closed = 0,
    /// Circuit is open - failing fast
    Open = 1,
    /// Testing if service is back
    HalfOpen = 2,
}

impl From<u8> for CircuitState {
    fn from(value: u8) -> Self {
        match value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// Configuration parameters for circuit breaker behavior.
/// 
/// This structure defines the thresholds and timeouts that control when a circuit
/// breaker transitions between states (Closed, Open, HalfOpen). It provides
/// sensible defaults while allowing customization for different service requirements.
/// 
/// # Fields
/// 
/// * `failure_threshold` - Number of consecutive failures to open the circuit (default: 5)
/// * `success_threshold` - Number of consecutive successes to close the circuit (default: 3)
/// * `timeout` - Request timeout before considering operation failed (default: 60s)
/// * `reset_timeout` - Time to wait before transitioning from Open to HalfOpen (default: 30s)
/// 
/// # Usage
/// 
/// ```rust
/// use std::time::Duration;
/// use kairos_rs::services::circuit_breaker::CircuitBreakerConfig;
/// 
/// // Use defaults
/// let config = CircuitBreakerConfig::default();
/// 
/// // Custom configuration for sensitive service
/// let config = CircuitBreakerConfig {
///     failure_threshold: 3,  // More sensitive to failures
///     success_threshold: 5,  // More conservative recovery
///     timeout: Duration::from_secs(30),
///     reset_timeout: Duration::from_secs(60),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u64,
    pub success_threshold: u64,
    #[allow(dead_code)] // Intended for request timeout integration
    pub timeout: Duration,
    pub reset_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            reset_timeout: Duration::from_secs(30),
        }
    }
}

/// Circuit breaker implementation for protecting upstream services.
/// 
/// This struct implements the circuit breaker pattern to prevent cascading failures
/// by monitoring request success/failure rates and automatically failing fast when
/// an upstream service is degraded or unavailable.
/// 
/// # States
/// 
/// - **Closed**: Normal operation, requests pass through
/// - **Open**: Circuit is open, requests fail immediately  
/// - **HalfOpen**: Testing recovery, limited requests allowed
/// 
/// # Thread Safety
/// 
/// All operations are thread-safe using atomic operations and async RwLock.
/// Multiple concurrent requests can safely interact with the same circuit breaker.
/// 
/// # Architecture
/// 
/// Uses atomic counters for performance-critical paths and async locks only for
/// state transitions that require coordination.
/// 
/// # Example
/// 
/// ```rust
/// use std::sync::Arc;
/// use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CircuitBreakerConfig::default();
/// let breaker = CircuitBreaker::new("user-service".to_string(), config);
/// 
/// // Example operation that might fail
/// let result = breaker.call(async {
///     // Simulated HTTP request
///     if true { // For doctest, always succeed
///         Ok("Success response")
///     } else {
///         Err("Network error")
///     }
/// }).await;
/// 
/// match result {
///     Ok(response) => println!("Request succeeded: {}", response),
///     Err(e) => println!("Request failed: {:?}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    name: String,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker instance.
    ///
    /// # Parameters
    ///
    /// * `name` - Identifier for this circuit breaker (used in logging)
    /// * `config` - Configuration parameters for breaker behavior
    ///
    /// # Returns
    ///
    /// Arc-wrapped circuit breaker ready for shared use across threads
    ///
    /// # Examples
    ///
    /// ```
    /// use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    ///
    /// let config = CircuitBreakerConfig::default();
    /// let breaker = CircuitBreaker::new("my-service".to_string(), config);
    /// ```
    pub fn new(name: String, config: CircuitBreakerConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            state: AtomicU8::new(CircuitState::Closed as u8),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            name,
        })
    }

    /// Executes an operation with circuit breaker protection.
    ///
    /// Wraps the provided async operation with circuit breaker logic. If the circuit
    /// is open, fails fast without executing the operation. Otherwise, executes the
    /// operation and updates circuit state based on success/failure.
    ///
    /// # Parameters
    ///
    /// * `operation` - Async operation to execute
    ///
    /// # Returns
    ///
    /// Result from the operation or circuit breaker error
    ///
    /// # Errors
    ///
    /// * `CircuitBreakerError::CircuitOpen` - Circuit is open, request rejected
    /// * `CircuitBreakerError::OperationFailed` - Operation executed but failed
    ///
    /// # Examples
    ///
    /// ```
    /// # use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    /// # async fn example() {
    /// # let breaker = CircuitBreaker::new("test".to_string(), CircuitBreakerConfig::default());
    /// let result = breaker.call(async {
    ///     // Your async operation here
    ///     Ok::<_, String>("success")
    /// }).await;
    /// # }
    /// ```
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Check if circuit is open
        if self.is_open().await {
            debug!("Circuit breaker {} is open, failing fast", self.name);
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // Execute the operation
        match operation.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    async fn is_open(&self) -> bool {
        let current_state = CircuitState::from(self.state.load(Ordering::Relaxed));
        
        match current_state {
            CircuitState::Closed => false,
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        self.transition_to_half_open().await;
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => false,
        }
    }

    async fn on_success(&self) {
        let current_state = CircuitState::from(self.state.load(Ordering::Relaxed));
        
        match current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold {
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Open => {
                // This shouldn't happen, but handle gracefully
                debug!("Unexpected success in open state for circuit {}", self.name);
            }
        }
    }

    async fn on_failure(&self) {
        let current_state = CircuitState::from(self.state.load(Ordering::Relaxed));
        
        match current_state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure_count >= self.config.failure_threshold {
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state should open the circuit
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // Update last failure time
                *self.last_failure_time.write().await = Some(Instant::now());
            }
        }
    }

    async fn transition_to_open(&self) {
        self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
        *self.last_failure_time.write().await = Some(Instant::now());
        self.success_count.store(0, Ordering::Relaxed);
        
        warn!("Circuit breaker {} opened due to failures", self.name);
    }

    async fn transition_to_half_open(&self) {
        self.state.store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        
        info!("Circuit breaker {} transitioned to half-open", self.name);
    }

    async fn transition_to_closed(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        
        info!("Circuit breaker {} closed - service recovered", self.name);
    }

    /// Gets the current state of the circuit breaker.
    ///
    /// # Returns
    ///
    /// Current `CircuitState` (Closed, Open, or HalfOpen)
    ///
    /// # Examples
    ///
    /// ```
    /// # use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    /// # let breaker = CircuitBreaker::new("test".to_string(), CircuitBreakerConfig::default());
    /// match breaker.get_state() {
    ///     CircuitState::Closed => println!("Operating normally"),
    ///     CircuitState::Open => println!("Failing fast"),
    ///     CircuitState::HalfOpen => println!("Testing recovery"),
    /// }
    /// ```
    pub fn get_state(&self) -> CircuitState {
        CircuitState::from(self.state.load(Ordering::Relaxed))
    }

    /// Gets the current failure count.
    ///
    /// # Returns
    ///
    /// Number of consecutive failures in current state
    ///
    /// # Examples
    ///
    /// ```
    /// # use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    /// # let breaker = CircuitBreaker::new("test".to_string(), CircuitBreakerConfig::default());
    /// println!("Failures: {}", breaker.get_failure_count());
    /// ```
    pub fn get_failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Gets the current success count in HalfOpen state.
    ///
    /// # Returns
    ///
    /// Number of consecutive successes when testing recovery
    ///
    /// # Examples
    ///
    /// ```
    /// # use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    /// # let breaker = CircuitBreaker::new("test".to_string(), CircuitBreakerConfig::default());
    /// println!("Successes: {}", breaker.get_success_count());
    /// ```
    pub fn get_success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }
}

/// Errors that can occur when using a circuit breaker.
///
/// # Variants
///
/// * `CircuitOpen` - Circuit breaker is open, request rejected for fast failure
/// * `OperationFailed` - The wrapped operation executed but returned an error
///
/// # Examples
///
/// ```
/// use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
///
/// # async fn example() {
/// # let breaker = CircuitBreaker::new("test".to_string(), CircuitBreakerConfig::default());
/// match breaker.call(async { Err::<(), _>("network error") }).await {
///     Err(CircuitBreakerError::CircuitOpen) => {
///         println!("Service unavailable - circuit open");
///     }
///     Err(CircuitBreakerError::OperationFailed(e)) => {
///         println!("Operation failed: {}", e);
///     }
///     Ok(_) => println!("Success"),
/// }
/// # }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
}
