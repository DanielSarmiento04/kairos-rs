mod config;
mod models;
mod logs;

use crate::config::settings::load_settings;
use crate::models::router::Settings;
use crate::logs::logger::configure_logger;

use serde_json::Result;

// use env_logger;
use log::{error, info, warn, trace, debug};

fn main() -> Result<()> {

    configure_logger();
    
    // Load the settings from the configuration file
    let config: Settings = load_settings().expect("Failed to load settings");
    
    info!("Logger initialized");
    warn!("This is a warning message");
    error!("This is an error message");
    debug!("This is a debug message");
    trace!("This is a trace message");

    // Print the parsed configuration
    println!("Configuration: {:?}", config);

    Ok(())
}
