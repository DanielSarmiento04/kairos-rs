use serde::Deserialize;
use serde_yaml;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    app_name: String,
    port: u16,
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    // Read YAML file content
    let yaml_content = fs::read_to_string("config.yml")?;
    
    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content)?;

    println!("App Name: {}", config.app_name);
    println!("Port: {}", config.port);
    println!("Debug: {}", config.debug);

    Ok(())
}
