//! Integration tests for load balancing functionality.

use kairos_rs::models::router::{Backend, LoadBalancingStrategy, Protocol, Router};
use kairos_rs::services::load_balancer::{
    LoadBalancerFactory, RoundRobinBalancer, WeightedBalancer, LoadBalancer,
};

#[test]
fn test_backend_validation() {
    let valid_backend = Backend {
        host: "http://localhost".to_string(),
        port: 8080,
        weight: 1,
        health_check_path: Some("/health".to_string()),
    };
    assert!(valid_backend.validate().is_ok());

    let invalid_host_backend = Backend {
        host: "localhost".to_string(), // Missing protocol
        port: 8080,
        weight: 1,
        health_check_path: None,
    };
    assert!(invalid_host_backend.validate().is_err());

    let zero_port_backend = Backend {
        host: "http://localhost".to_string(),
        port: 0,
        weight: 1,
        health_check_path: None,
    };
    assert!(zero_port_backend.validate().is_err());

    let zero_weight_backend = Backend {
        host: "http://localhost".to_string(),
        port: 8080,
        weight: 0,
        health_check_path: None,
    };
    assert!(zero_weight_backend.validate().is_err());
}

#[test]
fn test_router_with_backends() {
    let router = Router {
        host: None,
        port: None,
        backends: Some(vec![
            Backend {
                host: "http://backend-1".to_string(),
                port: 8080,
                weight: 1,
                health_check_path: Some("/health".to_string()),
            },
            Backend {
                host: "http://backend-2".to_string(),
                port: 8080,
                weight: 2,
                health_check_path: Some("/health".to_string()),
            },
        ]),
        load_balancing_strategy: LoadBalancingStrategy::Weighted,
        external_path: "/api/test".to_string(),
        internal_path: "/test".to_string(),
        methods: vec!["GET".to_string()],
        auth_required: false,
        retry: None,
        protocol: Protocol::Http,
        request_transformation: None,
        response_transformation: None,
    };

    assert!(router.validate().is_ok());

    let backends = router.get_backends();
    assert_eq!(backends.len(), 2);
    assert_eq!(backends[0].host, "http://backend-1");
    assert_eq!(backends[1].host, "http://backend-2");
}

#[test]
fn test_router_legacy_mode() {
    let router = Router {
        host: Some("http://legacy-backend".to_string()),
        port: Some(8080),
        backends: None,
        load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
        external_path: "/api/legacy".to_string(),
        internal_path: "/legacy".to_string(),
        methods: vec!["GET".to_string()],
        auth_required: false,
        retry: None,
        protocol: Protocol::Http,
        request_transformation: None,
        response_transformation: None,
    };

    assert!(router.validate().is_ok());

    let backends = router.get_backends();
    assert_eq!(backends.len(), 1);
    assert_eq!(backends[0].host, "http://legacy-backend");
    assert_eq!(backends[0].port, 8080);
}

#[test]
fn test_load_balancer_factory() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::Weighted,
        LoadBalancingStrategy::IpHash,
    ];

    for strategy in strategies {
        let balancer = LoadBalancerFactory::create(&strategy);
        assert!(balancer.select_backend(&[], None).is_none());
    }
}

#[test]
fn test_round_robin_balancer() {
    let balancer = RoundRobinBalancer::new();
    let backends = vec![
        Backend {
            host: "http://backend-1".to_string(),
            port: 8080,
            weight: 1,
            health_check_path: None,
        },
        Backend {
            host: "http://backend-2".to_string(),
            port: 8080,
            weight: 1,
            health_check_path: None,
        },
        Backend {
            host: "http://backend-3".to_string(),
            port: 8080,
            weight: 1,
            health_check_path: None,
        },
    ];

    // Should cycle through backends
    for i in 0..9 {
        let selected = balancer.select_backend(&backends, None).unwrap();
        let expected = format!("http://backend-{}", (i % 3) + 1);
        assert_eq!(selected.host, expected);
    }
}

#[test]
fn test_weighted_balancer() {
    let balancer = WeightedBalancer::new();
    let backends = vec![
        Backend {
            host: "http://backend-1".to_string(),
            port: 8080,
            weight: 2,
            health_check_path: None,
        },
        Backend {
            host: "http://backend-2".to_string(),
            port: 8080,
            weight: 1,
            health_check_path: None,
        },
    ];

    let mut counts = std::collections::HashMap::new();
    for _ in 0..30 {
        let selected = balancer.select_backend(&backends, None).unwrap();
        *counts.entry(selected.host).or_insert(0) += 1;
    }

    // Backend-1 should get 2x traffic
    let backend_1_count = counts.get("http://backend-1").unwrap_or(&0);
    let backend_2_count = counts.get("http://backend-2").unwrap_or(&0);
    assert_eq!(*backend_1_count, 20);
    assert_eq!(*backend_2_count, 10);
}

#[test]
fn test_retry_config_validation() {
    use kairos_rs::models::router::RetryConfig;

    let valid_config = RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        backoff_multiplier: 2.0,
        retry_on_status_codes: vec![502, 503, 504],
        retry_on_connection_error: true,
    };
    assert!(valid_config.validate().is_ok());

    let too_many_retries = RetryConfig {
        max_retries: 20,
        ..valid_config.clone()
    };
    assert!(too_many_retries.validate().is_err());

    let invalid_backoff = RetryConfig {
        initial_backoff_ms: 10000,
        max_backoff_ms: 1000,
        ..valid_config.clone()
    };
    assert!(invalid_backoff.validate().is_err());

    let invalid_multiplier = RetryConfig {
        backoff_multiplier: 0.5,
        ..valid_config
    };
    assert!(invalid_multiplier.validate().is_err());
}

#[test]
fn test_retry_config_backoff_calculation() {
    use kairos_rs::models::router::RetryConfig;

    let config = RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        backoff_multiplier: 2.0,
        retry_on_status_codes: vec![502, 503, 504],
        retry_on_connection_error: true,
    };

    assert_eq!(config.calculate_backoff(0), 100);
    assert_eq!(config.calculate_backoff(1), 200);
    assert_eq!(config.calculate_backoff(2), 400);
    assert_eq!(config.calculate_backoff(3), 800);
    assert_eq!(config.calculate_backoff(4), 1600);
    assert_eq!(config.calculate_backoff(5), 3200);
    assert_eq!(config.calculate_backoff(10), 5000); // Should cap at max_backoff_ms
}
