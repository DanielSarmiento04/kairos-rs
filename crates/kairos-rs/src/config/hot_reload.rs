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

/// Represents a configuration update event.
///
/// Contains the new settings, timestamp of the update, and a monotonically
/// increasing version number for tracking configuration changes.
///
/// # Examples
///
/// ```
/// use kairos_rs::config::hot_reload::ConfigUpdate;
/// use kairos_rs::models::settings::Settings;
///
/// let settings = Settings::default();
/// let update = ConfigUpdate {
///     settings,
///     timestamp: chrono::Utc::now(),
///     version: 1,
/// };
///
/// println!("Config version: {}", update.version);
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in tests and future features
pub struct ConfigUpdate {
    /// The updated gateway settings
    pub settings: Settings,
    /// When this configuration was loaded
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Monotonically increasing version number
    pub version: u64,
}

/// Watches a configuration file for changes and broadcasts updates.
///
/// Monitors the configuration file for modifications and automatically reloads
/// and validates the new configuration. Broadcasts updates to all subscribers.
///
/// # Examples
///
/// ```no_run
/// use kairos_rs::config::hot_reload::ConfigWatcher;
/// use kairos_rs::models::settings::Settings;
///
/// # async fn example() {
/// let settings = Settings::default();
/// let watcher = ConfigWatcher::new(settings, "./config.json".to_string());
///
/// // Start watching for changes
/// watcher.start_watching().await;
///
/// // Subscribe to updates
/// let mut receiver = watcher.subscribe();
/// # }
/// ```
#[allow(dead_code)] // Used in tests and future features
pub struct ConfigWatcher {
    current_config: Arc<RwLock<ConfigUpdate>>,
    config_path: String,
    update_sender: broadcast::Sender<ConfigUpdate>,
    version_counter: Arc<std::sync::atomic::AtomicU64>,
}

#[allow(dead_code)] // Used in tests and future features
impl ConfigWatcher {
    /// Creates a new configuration watcher.
    ///
    /// # Parameters
    ///
    /// * `initial_config` - Initial gateway settings
    /// * `config_path` - Path to configuration file to watch
    ///
    /// # Returns
    ///
    /// New `ConfigWatcher` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use kairos_rs::config::hot_reload::ConfigWatcher;
    /// use kairos_rs::models::settings::Settings;
    ///
    /// let settings = Settings::default();
    /// let watcher = ConfigWatcher::new(settings, "./config.json".to_string());
    /// ```
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
    
    /// Gets the current configuration.
    ///
    /// # Returns
    ///
    /// Current `ConfigUpdate` with settings and version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use kairos_rs::config::hot_reload::ConfigWatcher;
    /// # use kairos_rs::models::settings::Settings;
    /// # async fn example() {
    /// # let watcher = ConfigWatcher::new(Settings::default(), "./config.json".to_string());
    /// let config = watcher.get_current_config().await;
    /// println!("Current version: {}", config.version);
    /// # }
    /// ```
    pub async fn get_current_config(&self) -> ConfigUpdate {
        self.current_config.read().await.clone()
    }
    
    /// Subscribes to configuration update notifications.
    ///
    /// # Returns
    ///
    /// Broadcast receiver for `ConfigUpdate` events
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use kairos_rs::config::hot_reload::ConfigWatcher;
    /// # use kairos_rs::models::settings::Settings;
    /// # async fn example() {
    /// # let watcher = ConfigWatcher::new(Settings::default(), "./config.json".to_string());
    /// let mut receiver = watcher.subscribe();
    /// 
    /// // Wait for updates
    /// while let Ok(update) = receiver.recv().await {
    ///     println!("New config version: {}", update.version);
    /// }
    /// # }
    /// ```
    pub fn subscribe(&self) -> broadcast::Receiver<ConfigUpdate> {
        self.update_sender.subscribe()
    }
    
    /// Starts watching the configuration file for changes.
    ///
    /// Spawns a background task that checks the file every 5 seconds for
    /// modifications and automatically reloads when changes are detected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use kairos_rs::config::hot_reload::ConfigWatcher;
    /// # use kairos_rs::models::settings::Settings;
    /// # async fn example() {
    /// let watcher = ConfigWatcher::new(Settings::default(), "./config.json".to_string());
    /// watcher.start_watching().await;
    /// 
    /// // Watcher is now monitoring the file in the background
    /// # }
    /// ```
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
    
    /// Manually triggers a configuration reload.
    ///
    /// Forces an immediate reload of the configuration file, bypassing the
    /// automatic file modification detection.
    ///
    /// # Returns
    ///
    /// Result containing the new `ConfigUpdate` or error message
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or configuration is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use kairos_rs::config::hot_reload::ConfigWatcher;
    /// # use kairos_rs::models::settings::Settings;
    /// # async fn example() {
    /// # let watcher = ConfigWatcher::new(Settings::default(), "./config.json".to_string());
    /// match watcher.manual_reload().await {
    ///     Ok(update) => println!("Reloaded version {}", update.version),
    ///     Err(e) => eprintln!("Reload failed: {}", e),
    /// }
    /// # }
    /// ```
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

#[allow(dead_code)] // Used for file watching functionality
async fn get_file_modified_time(path: &str) -> Option<std::time::SystemTime> {
    tokio::fs::metadata(path)
        .await
        .ok()?
        .modified()
        .ok()
}

#[allow(dead_code)] // Used for configuration loading
fn load_settings_from_path(path: &str) -> Result<Settings, Box<dyn std::error::Error>> {
    let config_content = std::fs::read_to_string(path)?;
    let settings: Settings = serde_json::from_str(&config_content)?;
    Ok(settings)
}

/// Configuration management service that handles hot-reload and provides
/// current configuration to other services
#[allow(dead_code)] // Used in future features
pub struct ConfigManager {
    watcher: ConfigWatcher,
}

#[allow(dead_code)] // Used in future features
impl ConfigManager {
    /// Creates a new configuration manager.
    ///
    /// # Parameters
    ///
    /// * `initial_config` - Initial gateway settings
    /// * `config_path` - Path to configuration file to watch
    ///
    /// # Returns
    ///
    /// New `ConfigManager` instance
    pub fn new(initial_config: Settings, config_path: String) -> Self {
        Self {
            watcher: ConfigWatcher::new(initial_config, config_path),
        }
    }
    
    /// Starts the configuration file watcher.
    ///
    /// Begins monitoring the configuration file for changes in the background.
    pub async fn start(&self) {
        info!("Starting configuration hot-reload watcher");
        self.watcher.start_watching().await;
    }
    
    /// Gets the current configuration.
    ///
    /// # Returns
    ///
    /// Current `ConfigUpdate` with settings and version
    pub async fn get_current_config(&self) -> ConfigUpdate {
        self.watcher.get_current_config().await
    }
    
    /// Subscribes to configuration update notifications.
    ///
    /// # Returns
    ///
    /// Broadcast receiver for `ConfigUpdate` events
    pub fn subscribe_to_updates(&self) -> broadcast::Receiver<ConfigUpdate> {
        self.watcher.subscribe()
    }
    
    /// Manually triggers a configuration reload.
    ///
    /// # Returns
    ///
    /// Result containing the new `ConfigUpdate` or error message
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or configuration is invalid
    pub async fn reload_now(&self) -> Result<ConfigUpdate, String> {
        self.watcher.manual_reload().await
    }
}