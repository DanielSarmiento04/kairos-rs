use serde::Deserialize;
use serde_yaml;
use std::fs;

#[derive(Debug, Deserialize)]
struct Route {
    domain: String,  // The host
    port: u16,
}

#[derive(Debug, Deserialize)]
struct Config {
    version: String,
    routes: Vec<Route>,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    

    // Read YAML file content
    let yaml_content = fs::read_to_string("config.yml")?;
    
    // Parse YAML into the Config struct
    let config: Config = serde_yaml::from_str(&yaml_content)?;

    println!("Version: {}", config.version);

    let routes = config.routes;
    for route in routes {
        println!("Domain: {}, Port: {}", route.domain, route.port);
    }


    Ok(())
}
