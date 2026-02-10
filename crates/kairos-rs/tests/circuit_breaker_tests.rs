use kairos_rs::services::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState};
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
