//! Configuration hot-reload functionality for zero-downtime updates.
//! 
//! This module provides the ability to reload gateway configuration without
//! restarting the service, enabling dynamic route updates and configuration
//! changes in production environments.

use crate::config::validation::ConfigValidator;
use crate::models::settings::Settings;
use log::{info, warn, error};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

#[derive(Debug, Clone)]
pub struct ConfigUpdate {
    pub settings: Settings,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

pub struct ConfigWatcher {
    current_config: Arc<RwLock<ConfigUpdate>>,
    config_path: String,
    update_sender: broadcast::Sender<ConfigUpdate>,
    version_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl ConfigWatcher {
    pub fn new(initial_config: Settings, config_path: String) -> Self {
        let (update_sender, _) = broadcast::channel(100);
        
        let initial_update = ConfigUpdate {
            settings: initial_config,
            timestamp: chrono::Utc::now(),
            version: 1,
        };
        
        Self {
            current_config: Arc::new(RwLock::new(initial_update)),
            config_path,
            update_sender,
            version_counter: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }
    
    pub async fn get_current_config(&self) -> ConfigUpdate {
        self.current_config.read().await.clone()
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigUpdate> {
        self.update_sender.subscribe()
    }
    
    pub async fn start_watching(&self) {
        let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds
        let config_path = self.config_path.clone();
        let current_config = self.current_config.clone();
        let update_sender = self.update_sender.clone();
        let version_counter = self.version_counter.clone();
        
        tokio::spawn(async move {
            let mut last_modified = get_file_modified_time(&config_path).await;
            
            loop {
                interval.tick().await;
                
                match get_file_modified_time(&config_path).await {
                    Some(modified_time) => {
                        if Some(modified_time) != last_modified {
                            info!("Configuration file changed, reloading...");
                            
                            match Self::reload_config(&config_path).await {
                                Ok(new_settings) => {
                                    let version = version_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                                    let update = ConfigUpdate {
                                        settings: new_settings,
                                        timestamp: chrono::Utc::now(),
                                        version,
                                    };
                                    
                                    *current_config.write().await = update.clone();
                                    
                                    if let Err(e) = update_sender.send(update) {
                                        warn!("Failed to broadcast config update: {}", e);
                                    } else {
                                        info!("Configuration reloaded successfully (version {})", version);
                                    }
                                    
                                    last_modified = Some(modified_time);
                                }
                                Err(e) => {
                                    error!("Failed to reload configuration: {}", e);
                                    // Don't update last_modified so we'll try again
                                }
                            }
                        }
                    }
                    None => {
                        warn!("Could not get modification time for config file: {}", config_path);
                    }
                }
            }
        });
    }
    
    async fn reload_config(config_path: &str) -> Result<Settings, String> {
        // Load new configuration
        let new_settings = load_settings_from_path(config_path)
            .map_err(|e| format!("Failed to load config: {}", e))?;
        
        // Validate new configuration
        let validation_result = ConfigValidator::validate_comprehensive(&new_settings);
        if !validation_result.is_valid {
            return Err(format!(
                "Configuration validation failed: {}",
                validation_result.errors.join(", ")
            ));
        }
        
        // Log warnings but don't fail
        for warning in &validation_result.warnings {
            warn!("Config validation warning: {}", warning);
        }
        
        Ok(new_settings)
    }
    
    pub async fn manual_reload(&self) -> Result<ConfigUpdate, String> {
        let new_settings = Self::reload_config(&self.config_path).await?;
        
        let version = self.version_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        let update = ConfigUpdate {
            settings: new_settings,
            timestamp: chrono::Utc::now(),
            version,
        };
        
        *self.current_config.write().await = update.clone();
        
        if let Err(e) = self.update_sender.send(update.clone()) {
            warn!("Failed to broadcast manual config update: {}", e);
        }
        
        info!("Configuration manually reloaded (version {})", version);
        Ok(update)
    }
}

async fn get_file_modified_time(path: &str) -> Option<std::time::SystemTime> {
    tokio::fs::metadata(path)
        .await
        .ok()?
        .modified()
        .ok()
}

fn load_settings_from_path(path: &str) -> Result<Settings, Box<dyn std::error::Error>> {
    let config_content = std::fs::read_to_string(path)?;
    let settings: Settings = serde_json::from_str(&config_content)?;
    Ok(settings)
}

/// Configuration management service that handles hot-reload and provides
/// current configuration to other services
pub struct ConfigManager {
    watcher: ConfigWatcher,
}

impl ConfigManager {
    pub fn new(initial_config: Settings, config_path: String) -> Self {
        Self {
            watcher: ConfigWatcher::new(initial_config, config_path),
        }
    }
    
    pub async fn start(&self) {
        info!("Starting configuration hot-reload watcher");
        self.watcher.start_watching().await;
    }
    
    pub async fn get_current_config(&self) -> ConfigUpdate {
        self.watcher.get_current_config().await
    }
    
    pub fn subscribe_to_updates(&self) -> broadcast::Receiver<ConfigUpdate> {
        self.watcher.subscribe()
    }
    
    pub async fn reload_now(&self) -> Result<ConfigUpdate, String> {
        self.watcher.manual_reload().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::router::Router;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_settings() -> Settings {
        Settings {
            version: 1,
            routers: vec![
                Router {
                    host: "http://localhost".to_string(),
                    port: 3000,
                    external_path: "/test".to_string(),
                    internal_path: "/test".to_string(),
                    methods: vec!["GET".to_string()],
                }
            ],
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
        
        let watcher = ConfigWatcher::new(settings, temp_file.path().to_string_lossy().to_string());
        
        // Modify config file
        let mut new_settings = create_test_settings();
        new_settings.version = 2;
        let new_config_json = serde_json::to_string_pretty(&new_settings).unwrap();
        temp_file.write_all(new_config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
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
}