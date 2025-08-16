mod config;
mod models;

use crate::config::settings::load_settings;
use crate::models::router::{Router, Settings};


use serde_json::Result;

// use env_logger;
use log::{error, info, warn};

fn main() -> Result<()> {

    info!("Logger initialized");
    warn!("This is a warning message");
    error!("This is an error message");

    // Load the settings from the configuration file
    let config: Settings = load_settings().expect("Failed to load settings");

    // Print the parsed configuration
    println!("Configuration: {:?}", config);

    Ok(())
}
