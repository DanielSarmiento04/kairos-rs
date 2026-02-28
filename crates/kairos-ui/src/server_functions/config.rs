use crate::models::{
    AiSettings, CorsConfig, JwtSettings, MetricsConfig, RateLimitConfig, ServerConfig, Settings,
};
use leptos::prelude::*;

use super::GATEWAY_BASE_URL;

/// Fetches the current gateway configuration.
///
/// Note: This requires implementing a configuration endpoint on the gateway backend.
/// For now, this returns a mock error indicating the feature is not yet implemented.
#[server(GetConfiguration, "/api")]
pub async fn get_configuration() -> Result<Settings, ServerFnError> {
    // TODO: Implement configuration endpoint in backend
    // For now, return an error indicating this feature is not yet available
    Err(ServerFnError::new(
        "Configuration endpoint not yet implemented in gateway backend".to_string(),
    ))
}

/// Updates the gateway configuration.
///
/// Note: This requires implementing a configuration update endpoint on the gateway backend.
/// For now, this returns a mock error indicating the feature is not yet implemented.
#[server(UpdateConfiguration, "/api")]
pub async fn update_configuration(settings: Settings) -> Result<(), ServerFnError> {
    // Validate configuration before sending
    settings
        .validate()
        .map_err(|e| ServerFnError::new(format!("Invalid configuration: {}", e)))?;

    // TODO: Implement configuration update endpoint in backend
    Err(ServerFnError::new(
        "Configuration update endpoint not yet implemented in gateway backend".to_string(),
    ))
}

/// Triggers a configuration hot-reload on the gateway.
///
/// Note: This requires implementing a reload endpoint on the gateway backend.
#[server(TriggerReload, "/api")]
pub async fn trigger_reload() -> Result<(), ServerFnError> {
    // TODO: Implement reload endpoint in backend
    Err(ServerFnError::ServerError(
        "Configuration reload endpoint not yet implemented in gateway backend".to_string(),
    ))
}

// ============================================================================
// Configuration Management Server Functions
// ============================================================================

/// Fetches the complete configuration from the gateway.
#[server(GetConfig, "/api")]
pub async fn get_config() -> Result<Settings, ServerFnError> {
    let url = format!("{}/api/config", GATEWAY_BASE_URL);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Gateway returned error: {}",
            response.status()
        )));
    }

    let config = response
        .json::<Settings>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse config response: {}", e)))?;

    Ok(config)
}

/// Updates JWT configuration settings.
#[server(UpdateJwtConfig, "/api")]
pub async fn update_jwt_config(jwt_config: JwtSettings) -> Result<(), ServerFnError> {
    let url = format!("{}/api/config/jwt", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&jwt_config)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to update JWT config ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Updates rate limiting configuration settings.
#[server(UpdateRateLimitConfig, "/api")]
pub async fn update_rate_limit_config(
    rate_limit_config: RateLimitConfig,
) -> Result<(), ServerFnError> {
    let url = format!("{}/api/config/rate-limit", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&rate_limit_config)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to update rate limit config ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Updates CORS configuration settings.
#[server(UpdateCorsConfig, "/api")]
pub async fn update_cors_config(cors_config: CorsConfig) -> Result<(), ServerFnError> {
    let url = format!("{}/api/config/cors", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&cors_config)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to update CORS config ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Updates metrics configuration settings.
#[server(UpdateMetricsConfig, "/api")]
pub async fn update_metrics_config(metrics_config: MetricsConfig) -> Result<(), ServerFnError> {
    let url = format!("{}/api/config/metrics", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&metrics_config)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to update metrics config ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Updates server configuration settings.
#[server(UpdateServerConfig, "/api")]
pub async fn update_server_config(server_config: ServerConfig) -> Result<(), ServerFnError> {
    let url = format!("{}/api/config/server", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&server_config)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to update server config ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Updates AI configuration settings.
#[server(UpdateAiConfig, "/api")]
pub async fn update_ai_config(ai_config: AiSettings) -> Result<(), ServerFnError> {
    let url = format!("{}/api/config/ai", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&ai_config)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Failed to update AI config ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}
