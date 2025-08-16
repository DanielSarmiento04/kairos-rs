use std::fs;

use crate::models::router::Settings;

pub fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let config_data = fs::read_to_string("config.json")?;
    let settings: Settings = serde_json::from_str(&config_data)?;
    Ok(settings)
}