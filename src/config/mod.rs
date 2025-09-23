//! Configuration management for the kairos-rs gateway.
//! 
//! This module handles all aspects of gateway configuration, including loading
//! settings from files, environment variable processing, and configuration
//! validation. It provides secure and flexible configuration management with
//! comprehensive error handling.
//! 
//! # Module Organization
//! 
//! - [`settings`] - Configuration file loading and validation logic
//! 
//! # Configuration Sources
//! 
//! The gateway supports multiple configuration sources in order of precedence:
//! 1. **Environment Variables**: Runtime configuration overrides
//! 2. **Configuration Files**: Primary configuration source (JSON format)
//! 3. **Default Values**: Built-in fallback values
//! 
//! # Security Features
//! 
//! Configuration loading includes several security measures:
//! - **Path Traversal Protection**: Prevents access to files outside working directory
//! - **File Size Limits**: Prevents memory exhaustion attacks
//! - **Input Validation**: Comprehensive validation of all configuration values
//! - **Safe Defaults**: Secure default values for all optional settings
//! 
//! # Configuration Format
//! 
//! The primary configuration is stored in JSON format:
//! 
//! ```json
//! {
//!   "version": 1,
//!   "routers": [
//!     {
//!       "host": "http://backend-service",
//!       "port": 8080,
//!       "external_path": "/api/users/{id}",
//!       "internal_path": "/v1/user/{id}",
//!       "methods": ["GET", "POST", "PUT", "DELETE"]
//!     }
//!   ]
//! }
//! ```
//! 
//! # Environment Variables
//! 
//! - `KAIROS_CONFIG_PATH`: Path to configuration file (default: `./config.json`)
//! - `KAIROS_HOST`: Server bind address (default: `0.0.0.0`)
//! - `KAIROS_PORT`: Server port number (default: `5900`)
//! - `NO_COLOR`: Disable colored log output
//! 
//! # Examples
//! 
//! ```rust
//! # use std::fs;
//! # // Create a temporary config file for testing
//! # let config_content = r#"{"version": 1, "routers": []}"#;
//! # fs::write("./config.json", config_content).unwrap();
//! use kairos_rs::config::settings::load_settings;
//! 
//! // Load configuration with default settings
//! let config = load_settings().expect("Failed to load configuration");
//! 
//! // Validate configuration before use
//! config.validate().expect("Invalid configuration");
//! 
//! println!("Loaded {} routes", config.routers.len());
//! # // Clean up
//! # fs::remove_file("./config.json").ok();
//! ```
//! 
//! # Error Handling
//! 
//! Configuration errors are handled gracefully with detailed error messages:
//! - File system errors (permissions, not found)
//! - JSON parsing errors with line/column information
//! - Validation errors with specific field information
//! - Security violations with protective measures

pub mod hot_reload;
pub mod settings;
pub mod validation;
