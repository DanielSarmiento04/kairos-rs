//! Integration tests for configuration settings loading functionality.
//! 
//! This module contains comprehensive tests for the load_settings function and
//! related configuration loading functionality, ensuring proper file handling,
//! security validation, and error reporting.

use kairos_rs::config::settings::load_settings;
use kairos_rs::models::settings::Settings;
use kairos_rs::models::router::{Router, Backend};
use kairos_rs::models::router::Protocol;
use std::env;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn create_test_settings() -> Settings {
    Settings {
        version: 1,
        jwt: None,
        rate_limit: None,
        routers: vec![
            Router {
                host: Some("http://localhost".to_string()),
                port: Some(3000),
                external_path: "/api/test".to_string(),
                internal_path: "/test".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "http://localhost".to_string(),
                    port: 3000,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
                protocol: Protocol::Http,
                request_transformation: None,
                response_transformation: None,
                ai_policy: None,   
            }
        ],
    }
}

fn create_config_file(settings: &Settings) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    let config_json = serde_json::to_string_pretty(settings).unwrap();
    temp_file.write_all(config_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    temp_file
}

#[test]
fn test_load_settings_with_environment_variable() {
    let _lock = ENV_MUTEX.lock().unwrap();
    // Save original environment variable if it exists
    let original_path = env::var("KAIROS_CONFIG_PATH").ok();
    
    let settings = create_test_settings();
    let temp_file = create_config_file(&settings);

    // Test by setting environment variable to our test file
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    // Restore original environment variable if it existed
    if let Some(path) = original_path {
        env::set_var("KAIROS_CONFIG_PATH", path);
    } else {
        env::remove_var("KAIROS_CONFIG_PATH");
    }
    
    assert!(result.is_ok());
    let loaded_settings = result.unwrap();
    assert_eq!(loaded_settings.version, settings.version);
    assert_eq!(loaded_settings.routers.len(), settings.routers.len());
}

#[test]
fn test_load_settings_custom_path() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let settings = create_test_settings();
    let temp_file = create_config_file(&settings);
    
    // Set custom config path
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    // Clean up
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_ok());
    let loaded_settings = result.unwrap();
    assert_eq!(loaded_settings.version, settings.version);
    assert_eq!(loaded_settings.routers.len(), settings.routers.len());
}

#[test]
fn test_load_settings_file_not_found() {
    let _lock = ENV_MUTEX.lock().unwrap();
    env::set_var("KAIROS_CONFIG_PATH", "./nonexistent.json");
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot resolve config path"));
}

#[test]
fn test_load_settings_invalid_json() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"{ invalid json }").unwrap();
    temp_file.flush().unwrap();
    
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
}

#[test]
fn test_load_settings_malformed_structure() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let mut temp_file = NamedTempFile::new().unwrap();
    // Valid JSON but wrong structure
    temp_file.write_all(b"{\"wrong\": \"structure\"}").unwrap();
    temp_file.flush().unwrap();
    
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
}

#[test]
fn test_load_settings_path_traversal_protection() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let malicious_path = format!("{}/../../../etc/passwd", temp_dir.path().display());
    
    env::set_var("KAIROS_CONFIG_PATH", &malicious_path);
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_err());
    let error_message = result.unwrap_err().to_string();
    assert!(error_message.contains("Cannot resolve config path") || 
           error_message.contains("Config path outside working directory"));
}

#[test]
fn test_load_settings_file_size_limit() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let mut temp_file = NamedTempFile::new().unwrap();
    
    // Create a large config file (larger than 10MB limit)
    let large_content = "x".repeat(11 * 1024 * 1024);
    temp_file.write_all(large_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Config file too large"));
}

#[test]
fn test_load_settings_complex_configuration() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let complex_settings = Settings {
        version: 2,
        jwt: None,
        rate_limit: None,
        routers: vec![
            Router {
                host: Some("https://api.example.com".to_string()),
                port: Some(443),
                external_path: "/api/v1/users/{id}".to_string(),
                internal_path: "/users/{id}".to_string(),
                methods: vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "https://api.example.com".to_string(),
                    port: 443,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
                protocol: Protocol::Http,
                request_transformation: None,
                response_transformation: None,
                ai_policy: None,
            },
            Router {
                host: Some("http://internal-service".to_string()),
                port: Some(8080),
                external_path: "/internal/{service}/{action}".to_string(),
                internal_path: "/{service}/{action}".to_string(),
                methods: vec!["POST".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "http://internal-service".to_string(),
                    port: 8080,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
                protocol: Protocol::Http,
                request_transformation: None,
                response_transformation: None,
                ai_policy: None,
            },
            Router {
                host: Some("https://auth.example.com".to_string()),
                port: Some(443),
                external_path: "/auth/login".to_string(),
                internal_path: "/v2/auth/login".to_string(),
                methods: vec!["POST".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "https://auth.example.com".to_string(),
                    port: 443,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
                protocol: Protocol::Http,
                request_transformation: None,
                response_transformation: None,
                ai_policy: None,
            },
        ],
    };
    
    let temp_file = create_config_file(&complex_settings);
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    if let Err(e) = &result {
        println!("Error loading settings: {}", e);
    }
    assert!(result.is_ok());
    let loaded_settings = result.unwrap();
    assert_eq!(loaded_settings.version, 2);
    assert_eq!(loaded_settings.routers.len(), 3);
    
    // Verify first router details
    let first_router = &loaded_settings.routers[0];
    assert_eq!(first_router.host, Some("https://api.example.com".to_string()));
    assert_eq!(first_router.port, Some(443));
    assert_eq!(first_router.external_path, "/api/v1/users/{id}");
    assert_eq!(first_router.methods.len(), 3);
}

#[test]
fn test_load_settings_empty_routers() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let empty_settings = Settings {
        version: 1,
        jwt: None,
        rate_limit: None,
        routers: vec![],
    };
    
    let temp_file = create_config_file(&empty_settings);
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    if let Err(e) = &result {
        eprintln!("Error loading settings: {}", e);
    }
    assert!(result.is_ok());
    let loaded_settings = result.unwrap();
    assert_eq!(loaded_settings.version, 1);
    assert!(loaded_settings.routers.is_empty());
}

#[test]
fn test_load_settings_unicode_content() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let unicode_settings = Settings {
        version: 1,
        jwt: None,
        rate_limit: None,
        routers: vec![
            Router {
                host: Some("https://测试.example.com".to_string()),
                port: Some(443),
                external_path: "/api/用户/{id}".to_string(),
                internal_path: "/users/{id}".to_string(),
                methods: vec!["GET".to_string()],
                auth_required: false,
                backends: Some(vec![Backend {
                    host: "https://测试.example.com".to_string(),
                    port: 443,
                    weight: 1,
                    health_check_path: None,
                }]),
                load_balancing_strategy: Default::default(),
                retry: None,
                protocol: Protocol::Http,
                request_transformation: None,
                response_transformation: None,
                ai_policy: None,
            }
        ],
    };
    
    let temp_file = create_config_file(&unicode_settings);
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    assert!(result.is_ok());
    let loaded_settings = result.unwrap();
    assert_eq!(loaded_settings.routers[0].host, Some("https://测试.example.com".to_string()));
    assert_eq!(loaded_settings.routers[0].external_path, "/api/用户/{id}");
}

#[test]
fn test_load_settings_preserves_current_dir() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    let settings = create_test_settings();
    let temp_file = create_config_file(&settings);
    env::set_var("KAIROS_CONFIG_PATH", temp_file.path());
    
    let _result = load_settings();
    
    env::remove_var("KAIROS_CONFIG_PATH");
    
    // Ensure current directory wasn't changed
    let current_dir = env::current_dir().unwrap();
    assert_eq!(original_dir, current_dir);
}