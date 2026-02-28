use crate::models::{HealthResponse, LivenessResponse, ReadinessResponse};
use leptos::prelude::*;

use super::GATEWAY_BASE_URL;

/// Fetches health status from the gateway /health endpoint.
///
/// Returns comprehensive health information including service status,
/// version, timestamp, and uptime.
#[server(GetHealth, "/api")]
pub async fn get_health() -> Result<HealthResponse, ServerFnError> {
    let url = format!("{}/health", GATEWAY_BASE_URL);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| {
            let msg = format!(
                "Failed to connect to Kairos Gateway at {}: {}. Make sure the gateway is running with: cargo run --bin kairos-gateway",
                GATEWAY_BASE_URL, e
            );
            ServerFnError::new(msg)
        })?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Gateway returned error: {}",
            response.status()
        )));
    }

    let health = response
        .json::<HealthResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse health response: {}", e)))?;

    Ok(health)
}

/// Fetches readiness status from the gateway /ready endpoint.
#[server(GetReadiness, "/api")]
pub async fn get_readiness() -> Result<ReadinessResponse, ServerFnError> {
    let url = format!("{}/ready", GATEWAY_BASE_URL);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    let readiness = response
        .json::<ReadinessResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse readiness response: {}", e)))?;

    Ok(readiness)
}

/// Fetches liveness status from the gateway /live endpoint.
#[server(GetLiveness, "/api")]
pub async fn get_liveness() -> Result<LivenessResponse, ServerFnError> {
    let url = format!("{}/live", GATEWAY_BASE_URL);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    let liveness = response
        .json::<LivenessResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse liveness response: {}", e)))?;

    Ok(liveness)
}
