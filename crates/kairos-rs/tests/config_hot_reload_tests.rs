//! Integration tests for configuration hot reloading functionality.
//!
//! This module contains comprehensive tests for the ConfigWatcher and related
//! hot reload functionality, ensuring configuration changes are properly detected
//! and applied without requiring application restarts.

use kairos_rs::config::hot_reload::ConfigWatcher;
use kairos_rs::models::router::Protocol;
use kairos_rs::models::router::{Backend, Router};
use kairos_rs::models::settings::Settings;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_settings() -> Settings {
    Settings {
        version: 1,
        jwt: None,
        rate_limit: None,
        ai: None,
        routers: vec![Router {
            host: Some("http://localhost".to_string()),
            port: Some(3000),
            external_path: "/test".to_string(),
            internal_path: "/test".to_string(),
            methods: vec!["GET".to_string()],
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
        }],
    }
}

#[tokio::test]
async fn test_config_watcher_creation() {
    let settings = create_test_settings();
    let watcher = ConfigWatcher::new(settings.clone(), "test.json".to_string());

    let current = watcher.get_current_config().await;
    assert_eq!(current.settings.version, settings.version);
    assert_eq!(current.version, 1);
}

#[tokio::test]
async fn test_manual_reload() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let settings = create_test_settings();

    // Write initial config
    let config_json = serde_json::to_string_pretty(&settings).unwrap();
    temp_file.write_all(config_json.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Create a new temp file for the updated config to avoid write conflicts
    let mut temp_file2 = NamedTempFile::new().unwrap();
    let mut new_settings = create_test_settings();
    new_settings.version = 2;
    let new_config_json = serde_json::to_string_pretty(&new_settings).unwrap();
    temp_file2.write_all(new_config_json.as_bytes()).unwrap();
    temp_file2.flush().unwrap();

    // Create watcher with the updated config file
    let watcher = ConfigWatcher::new(
        new_settings.clone(),
        temp_file2.path().to_string_lossy().to_string(),
    );

    // Manual reload
    let result = watcher.manual_reload().await;
    if let Err(ref e) = result {
        println!("Manual reload failed with error: {}", e);
    }
    assert!(result.is_ok());

    let updated_config = result.unwrap();
    assert_eq!(updated_config.settings.version, 2);
    assert_eq!(updated_config.version, 2);
}

#[tokio::test]
async fn test_config_watcher_with_invalid_file() {
    let settings = create_test_settings();
    let watcher = ConfigWatcher::new(settings, "nonexistent.json".to_string());

    let result = watcher.manual_reload().await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err();

    // Check for the actual error patterns that can occur
    assert!(
        error_msg.contains("Failed to load config")
            || error_msg.contains("No such file or directory")
            || error_msg.contains("cannot find the file")
            || error_msg.to_lowercase().contains("not found")
    );
}

#[tokio::test]
async fn test_config_watcher_with_malformed_json() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let settings = create_test_settings();

    // Write malformed JSON directly
    temp_file.write_all(b"{ invalid json }").unwrap();
    temp_file.flush().unwrap();

    let watcher = ConfigWatcher::new(settings, temp_file.path().to_string_lossy().to_string());

    let result = watcher.manual_reload().await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(error_msg.contains("Invalid JSON") || error_msg.contains("Failed to load config"));
}

#[tokio::test]
async fn test_config_version_tracking() {
    // Update config multiple times using separate files
    for new_version in 2u8..=5u8 {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut updated_settings = create_test_settings();
        updated_settings.version = new_version;
        let new_config_json = serde_json::to_string_pretty(&updated_settings).unwrap();
        temp_file.write_all(new_config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let watcher = ConfigWatcher::new(
            updated_settings.clone(),
            temp_file.path().to_string_lossy().to_string(),
        );

        let result = watcher.manual_reload().await;
        assert!(
            result.is_ok(),
            "Failed to reload config for version {}: {:?}",
            new_version,
            result.err()
        );

        let updated_config = result.unwrap();

        assert_eq!(updated_config.settings.version, new_version);
        // The watcher version starts at 1 and increments, so after reload it should be 2
        assert_eq!(updated_config.version, 2u64);
    }
}
