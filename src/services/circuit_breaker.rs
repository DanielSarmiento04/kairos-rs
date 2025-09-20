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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed = 0,   // Normal operation
    Open = 1,     // Circuit is open, failing fast
    HalfOpen = 2, // Testing if service is back
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
/// let config = CircuitBreakerConfig::default();
/// let breaker = CircuitBreaker::new("user-service".to_string(), config);
/// 
/// // Check if request should proceed
/// if breaker.can_execute().await {
///     match make_request().await {
///         Ok(response) => {
///             breaker.record_success().await;
///             response
///         }
///         Err(error) => {
///             breaker.record_failure().await;
///             error
///         }
///     }
/// } else {
///     // Circuit is open, fail fast
///     return Err("Service unavailable");
/// }
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

    pub fn get_state(&self) -> CircuitState {
        CircuitState::from(self.state.load(Ordering::Relaxed))
    }

    pub fn get_failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    pub fn get_success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,
    #[error("Operation failed: {0}")]
    OperationFailed(E),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            reset_timeout: Duration::from_secs(1),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Should start in closed state
        assert_eq!(cb.get_state(), CircuitState::Closed);
        
        // Successful operations should keep it closed
        let result = cb.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            reset_timeout: Duration::from_secs(1),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // First failure
        let result = cb.call(async { Err::<i32, &str>("error") }).await;
        assert!(result.is_err());
        assert_eq!(cb.get_state(), CircuitState::Closed);
        
        // Second failure should open the circuit
        let result = cb.call(async { Err::<i32, &str>("error") }).await;
        assert!(result.is_err());
        assert_eq!(cb.get_state(), CircuitState::Open);
        
        // Next call should fail fast
        let result = cb.call(async { Ok::<i32, &str>(42) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            reset_timeout: Duration::from_millis(100),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Cause failure to open circuit
        let _ = cb.call(async { Err::<i32, &str>("error") }).await;
        assert_eq!(cb.get_state(), CircuitState::Open);
        
        // Wait for reset timeout
        sleep(Duration::from_millis(150)).await;
        
        // Next call should transition to half-open
        let result = cb.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        
        // Another success should close the circuit
        let result = cb.call(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }
}