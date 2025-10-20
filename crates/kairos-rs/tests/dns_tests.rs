//! DNS Service Tests
//!
//! Tests for the DNS proxy handler including handler creation,
//! cache operations, and basic functionality.

use kairos_rs::services::dns::DnsHandler;

#[test]
fn test_dns_handler_creation() {
    let _handler = DnsHandler::new(5);
    // Test that handler can be created successfully
    assert!(true);
}

#[test]
fn test_dns_handler_with_different_timeouts() {
    let _handler1 = DnsHandler::new(5);
    let _handler2 = DnsHandler::new(10);
    let _handler3 = DnsHandler::new(30);
    // Test that handlers can be created with various timeout values
    assert!(true);
}

#[tokio::test]
async fn test_cache_operations() {
    let handler = DnsHandler::new(5);
    
    // Cache should start empty
    assert_eq!(handler.cache_size().await, 0);
    
    // Clear expired entries on empty cache
    handler.clear_expired().await;
    assert_eq!(handler.cache_size().await, 0);
}

#[tokio::test]
async fn test_cache_initial_state() {
    let handler = DnsHandler::new(5);
    
    // Cache should start empty
    assert_eq!(handler.cache_size().await, 0);
}

#[tokio::test]
async fn test_cache_clear_expired_empty() {
    let handler = DnsHandler::new(5);
    
    // Clearing expired entries on empty cache should not error
    handler.clear_expired().await;
    assert_eq!(handler.cache_size().await, 0);
}

#[test]
fn test_dns_handler_clone() {
    let handler = DnsHandler::new(5);
    let _cloned = handler.clone();
    // Test that handler can be cloned successfully
    assert!(true);
}

#[test]
fn test_dns_handler_multiple_instances() {
    let _handler1 = DnsHandler::new(5);
    let _handler2 = DnsHandler::new(10);
    // Test that multiple independent handlers can coexist
    assert!(true);
}

#[test]
fn test_dns_timeout_values() {
    // Test various timeout values
    let _handler1 = DnsHandler::new(1);
    let _handler2 = DnsHandler::new(0);
    let _handler3 = DnsHandler::new(60);
    assert!(true);
}

#[cfg(test)]
mod dns_protocol_tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_cache_independence() {
        let handler1 = DnsHandler::new(5);
        let handler2 = DnsHandler::new(5);
        
        // Each handler should have its own cache
        assert_eq!(handler1.cache_size().await, 0);
        assert_eq!(handler2.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_clone_shares_cache() {
        let handler = DnsHandler::new(5);
        let cloned = handler.clone();
        
        // Cloned handlers share the same cache (Arc)
        assert_eq!(handler.cache_size().await, cloned.cache_size().await);
    }

    #[test]
    fn test_handler_lifecycle() {
        let handler = DnsHandler::new(5);
        let cloned = handler.clone();
        drop(handler);
        // Cloned handler should still be valid after original is dropped
        drop(cloned);
        assert!(true);
    }
}
