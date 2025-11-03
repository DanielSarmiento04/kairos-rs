//! Server functions for communicating with the Kairos Gateway API.
//! 
//! These functions run on the server-side and make HTTP requests to the
//! Kairos Gateway backend (default: http://localhost:5900).

use leptos::prelude::*;
use crate::models::*;

/// Base URL for the Kairos Gateway API
const GATEWAY_BASE_URL: &str = "http://localhost:5900";

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
        return Err(ServerFnError::new(format!("Gateway returned error: {}", response.status())));
    }
    
    let health = response.json::<HealthResponse>()
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
    
    let readiness = response.json::<ReadinessResponse>()
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
    
    let liveness = response.json::<LivenessResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse liveness response: {}", e)))?;
    
    Ok(liveness)
}

/// Fetches Prometheus metrics from the gateway /metrics endpoint.
/// 
/// Parses the Prometheus text format and returns structured metrics data
/// including request counts, response times, circuit breaker states, etc.
#[server(GetMetrics, "/api")]
pub async fn get_metrics() -> Result<MetricsData, ServerFnError> {
    let url = format!("{}/metrics", GATEWAY_BASE_URL);
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(ServerFnError::new(format!("Gateway returned error: {}", response.status())));
    }
    
    let text = response.text()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read metrics response: {}", e)))?;
    
    let metrics = MetricsData::parse_prometheus(&text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse metrics: {}", e)))?;
    
    Ok(metrics)
}

/// Fetches the current gateway configuration.
/// 
/// Note: This requires implementing a configuration endpoint on the gateway backend.
/// For now, this returns a mock error indicating the feature is not yet implemented.
#[server(GetConfiguration, "/api")]
pub async fn get_configuration() -> Result<Settings, ServerFnError> {
    // TODO: Implement configuration endpoint in backend
    // For now, return an error indicating this feature is not yet available
    Err(ServerFnError::new(
        "Configuration endpoint not yet implemented in gateway backend".to_string()
    ))
}

/// Updates the gateway configuration.
/// 
/// Note: This requires implementing a configuration update endpoint on the gateway backend.
/// For now, this returns a mock error indicating the feature is not yet implemented.
#[server(UpdateConfiguration, "/api")]
pub async fn update_configuration(settings: Settings) -> Result<(), ServerFnError> {
    // Validate configuration before sending
    settings.validate()
        .map_err(|e| ServerFnError::new(format!("Invalid configuration: {}", e)))?;
    
    // TODO: Implement configuration update endpoint in backend
    Err(ServerFnError::new(
        "Configuration update endpoint not yet implemented in gateway backend".to_string()
    ))
}

/// Lists all configured routes from the gateway.
#[server(ListRoutes, "/api")]
pub async fn list_routes() -> Result<Vec<Router>, ServerFnError> {
    let url = format!("{}/api/routes", GATEWAY_BASE_URL);
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!("Gateway returned error {}: {}", status, error_text)));
    }
    
    // Parse the RouteResponse wrapper
    #[derive(serde::Deserialize)]
    struct RouteResponse {
        routes: Option<Vec<Router>>,
    }
    
    let route_response = response.json::<RouteResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse routes response: {}", e)))?;
    
    Ok(route_response.routes.unwrap_or_default())
}

/// Gets a specific route by its external path.
#[server(GetRoute, "/api")]
pub async fn get_route(external_path: String) -> Result<Router, ServerFnError> {
    // URL encode the external_path
    let encoded_path = urlencoding::encode(&external_path.trim_start_matches('/'));
    let url = format!("{}/api/routes/{}", GATEWAY_BASE_URL, encoded_path);
    
    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!("Gateway returned error {}: {}", status, error_text)));
    }
    
    // Parse the RouteResponse wrapper
    #[derive(serde::Deserialize)]
    struct RouteResponse {
        route: Option<Router>,
    }
    
    let route_response = response.json::<RouteResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse route response: {}", e)))?;
    
    route_response.route.ok_or_else(|| ServerFnError::new("Route not found".to_string()))
}

/// Creates a new route in the gateway configuration.
#[server(CreateRoute, "/api")]
pub async fn create_route(route: Router) -> Result<(), ServerFnError> {
    // Validate route before sending
    route.validate()
        .map_err(|e| ServerFnError::new(format!("Invalid route: {}", e)))?;
    
    let url = format!("{}/api/routes", GATEWAY_BASE_URL);
    
    let client = reqwest::Client::new();
    let response = client.post(&url)
        .json(&route)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!("Failed to create route ({}): {}", status, error_text)));
    }
    
    Ok(())
}

/// Updates an existing route in the gateway configuration.
#[server(UpdateRoute, "/api")]
pub async fn update_route(route: Router) -> Result<(), ServerFnError> {
    // Validate route before sending
    route.validate()
        .map_err(|e| ServerFnError::new(format!("Invalid route: {}", e)))?;
    
    // URL encode the external_path
    let encoded_path = urlencoding::encode(&route.external_path.trim_start_matches('/'));
    let url = format!("{}/api/routes/{}", GATEWAY_BASE_URL, encoded_path);
    
    let client = reqwest::Client::new();
    let response = client.put(&url)
        .json(&route)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!("Failed to update route ({}): {}", status, error_text)));
    }
    
    Ok(())
}

/// Deletes a route from the gateway configuration.
#[server(DeleteRoute, "/api")]
pub async fn delete_route(external_path: String) -> Result<(), ServerFnError> {
    // URL encode the external_path
    let encoded_path = urlencoding::encode(&external_path.trim_start_matches('/'));
    let url = format!("{}/api/routes/{}", GATEWAY_BASE_URL, encoded_path);
    
    let client = reqwest::Client::new();
    let response = client.delete(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!("Failed to delete route ({}): {}", status, error_text)));
    }
    
    Ok(())
}

/// Tests a route by making a request through the gateway.
/// 
/// Returns the status code and response time for diagnostics.
#[server(TestRoute, "/api")]
pub async fn test_route(external_path: String, method: String) -> Result<(u16, f64), ServerFnError> {
    let url = format!("{}{}", GATEWAY_BASE_URL, external_path);
    
    let start = std::time::Instant::now();
    
    let client = reqwest::Client::new();
    let response = match method.as_str() {
        "GET" => client.get(&url).send().await,
        "POST" => client.post(&url).send().await,
        "PUT" => client.put(&url).send().await,
        "DELETE" => client.delete(&url).send().await,
        "HEAD" => client.head(&url).send().await,
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url).send().await,
        "PATCH" => client.patch(&url).send().await,
        _ => return Err(ServerFnError::ServerError(format!("Unsupported method: {}", method))),
    };
    
    let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
    
    match response {
        Ok(resp) => Ok((resp.status().as_u16(), elapsed)),
        Err(e) => Err(ServerFnError::ServerError(format!("Request failed: {}", e))),
    }
}

/// Triggers a configuration hot-reload on the gateway.
/// 
/// Note: This requires implementing a reload endpoint on the gateway backend.
#[server(TriggerReload, "/api")]
pub async fn trigger_reload() -> Result<(), ServerFnError> {
    // TODO: Implement reload trigger endpoint in backend
    Err(ServerFnError::ServerError(
        "Configuration reload endpoint not yet implemented in gateway backend".to_string()
    ))
}
