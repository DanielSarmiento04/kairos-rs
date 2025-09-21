use crate::models::settings::Settings;
use log::{debug, warn};
use std::fs;
use std::path::Path;

/// Loads and validates application configuration from file system.
/// 
/// This function safely loads the gateway configuration from a JSON file,
/// with comprehensive security checks and validation. It supports configurable
/// file paths via environment variables and implements multiple security measures
/// to prevent common configuration-related vulnerabilities.
/// 
/// # Configuration File Location
/// 
/// The configuration file path is determined by:
/// 1. `KAIROS_CONFIG_PATH` environment variable (if set)
/// 2. Default: `./config.json` (relative to current working directory)
/// 
/// # Security Features
/// 
/// - **Path Traversal Protection**: Ensures config file is within working directory
/// - **File Size Limits**: Prevents memory exhaustion attacks (max 10MB)
/// - **Path Canonicalization**: Resolves symlinks and relative paths safely
/// - **Access Validation**: Verifies file readability before processing
/// 
/// # File Format
/// 
/// Expected JSON structure:
/// ```json
/// {
///   "version": 1,
///   "routers": [
///     {
///       "host": "http://backend-service",
///       "port": 8080,
///       "external_path": "/api/users/{id}",
///       "internal_path": "/v1/user/{id}",
///       "methods": ["GET", "POST", "PUT"]
///     }
///   ]
/// }
/// ```
/// 
/// # Returns
/// 
/// - `Ok(Settings)` - Successfully loaded and parsed configuration
/// - `Err(Box<dyn std::error::Error>)` - Configuration loading or parsing error
/// 
/// # Examples
/// 
/// ```rust
/// use kairos_rs::config::settings::load_settings;
/// 
/// // Load configuration with default path
/// let config = load_settings().expect("Failed to load configuration");
/// println!("Loaded {} routes", config.routers.len());
/// 
/// // Note: Custom path examples require the file to exist
/// // std::env::set_var("KAIROS_CONFIG_PATH", "/etc/kairos/config.json");
/// // let config = load_settings().expect("Failed to load custom configuration");
/// ```
/// 
/// # Error Conditions
/// 
/// This function returns errors for:
/// - **File Not Found**: Configuration file doesn't exist at specified path
/// - **Permission Denied**: Insufficient permissions to read configuration file
/// - **Path Traversal**: Config path attempts to escape working directory
/// - **File Too Large**: Configuration file exceeds 10MB size limit
/// - **Invalid JSON**: Malformed JSON syntax in configuration file
/// - **Schema Validation**: JSON doesn't match expected Settings structure
/// 
/// # Environment Variables
/// 
/// - `KAIROS_CONFIG_PATH`: Custom path to configuration file (optional)
/// 
/// # Logging
/// 
/// The function logs:
/// - Debug: Configuration file path and successful load information
/// - Warning: Security violations (path outside working directory)
/// 
/// # Thread Safety
/// 
/// This function is safe to call from multiple threads, though it's typically
/// called once during application startup.
pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let config_path = std::env::var("KAIROS_CONFIG_PATH")
        .unwrap_or_else(|_| "./config.json".to_string());
    
    debug!("Loading configuration from: {}", config_path);
    
    // Validate path is safe to prevent path traversal attacks
    let path = Path::new(&config_path);
    
    // Check if file exists first before canonicalizing
    if !path.exists() {
        return Err(format!("Cannot resolve config path '{}'", config_path).into());
    }
    
    // Resolve to absolute path and verify it's within allowed directories
    let canonical_path = path.canonicalize()
        .map_err(|e| format!("Cannot resolve config path '{}': {}", config_path, e))?;
    
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Cannot get current directory: {}", e))?;
    
    // Check if config file is in current directory or subdirectory
    if !canonical_path.starts_with(&current_dir) {
        warn!("Config path '{}' is outside working directory", config_path);
        return Err("Config path outside working directory".into());
    }
    
    // Check file size to prevent memory exhaustion
    let metadata = fs::metadata(&canonical_path)
        .map_err(|e| format!("Cannot read config file metadata: {}", e))?;
    
    const MAX_CONFIG_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    if metadata.len() > MAX_CONFIG_SIZE {
        return Err(format!("Config file too large: {} bytes (max: {} bytes)", 
                         metadata.len(), MAX_CONFIG_SIZE).into());
    }
    
    let config_data = fs::read_to_string(&canonical_path)
        .map_err(|e| format!("Cannot read config file: {}", e))?;
    
    let settings: Settings = serde_json::from_str(&config_data)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    debug!("Successfully loaded configuration with {} routes", settings.routers.len());
    
    Ok(settings)
}
