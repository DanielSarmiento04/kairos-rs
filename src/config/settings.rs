use crate::models::settings::Settings;
use log::{debug, warn};
use std::fs;
use std::path::Path;

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let config_path = std::env::var("KAIROS_CONFIG_PATH")
        .unwrap_or_else(|_| "./config.json".to_string());
    
    debug!("Loading configuration from: {}", config_path);
    
    // Validate path is safe to prevent path traversal attacks
    let path = Path::new(&config_path);
    
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
        .map_err(|e| format!("Invalid JSON in config file: {}", e))?;
    
    debug!("Successfully loaded configuration with {} routes", settings.routers.len());
    
    Ok(settings)
}
