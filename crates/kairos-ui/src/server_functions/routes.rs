use crate::models::Router;
use leptos::prelude::*;

use super::GATEWAY_BASE_URL;

/// Lists all configured routes from the gateway.
#[server(ListRoutes, "/api")]
pub async fn list_routes() -> Result<Vec<Router>, ServerFnError> {
    let url = format!("{}/api/routes", GATEWAY_BASE_URL);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Gateway returned error {}: {}",
            status, error_text
        )));
    }

    // Parse the RouteResponse wrapper
    #[derive(serde::Deserialize)]
    struct RouteResponse {
        routes: Option<Vec<Router>>,
    }

    let route_response = response
        .json::<RouteResponse>()
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
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::new(format!(
            "Gateway returned error {}: {}",
            status, error_text
        )));
    }

    // Parse the RouteResponse wrapper
    #[derive(serde::Deserialize)]
    struct RouteResponse {
        route: Option<Router>,
    }

    let route_response = response
        .json::<RouteResponse>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse route response: {}", e)))?;

    route_response
        .route
        .ok_or_else(|| ServerFnError::new("Route not found".to_string()))
}

/// Creates a new route in the gateway configuration.
#[server(CreateRoute, "/api")]
pub async fn create_route(route: Router) -> Result<(), ServerFnError> {
    // Validate route before sending
    route
        .validate()
        .map_err(|e| ServerFnError::new(format!("Invalid route: {}", e)))?;

    let url = format!("{}/api/routes", GATEWAY_BASE_URL);

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&route)
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
            "Failed to create route ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Updates an existing route in the gateway configuration.
#[server(UpdateRoute, "/api")]
pub async fn update_route(route: Router) -> Result<(), ServerFnError> {
    // Validate route before sending
    route
        .validate()
        .map_err(|e| ServerFnError::new(format!("Invalid route: {}", e)))?;

    // URL encode the external_path
    let encoded_path = urlencoding::encode(&route.external_path.trim_start_matches('/'));
    let url = format!("{}/api/routes/{}", GATEWAY_BASE_URL, encoded_path);

    let client = reqwest::Client::new();
    let response = client
        .put(&url)
        .json(&route)
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
            "Failed to update route ({}): {}",
            status, error_text
        )));
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
    let response = client
        .delete(&url)
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
            "Failed to delete route ({}): {}",
            status, error_text
        )));
    }

    Ok(())
}

/// Tests a route by making a request through the gateway.
///
/// Returns the status code and response time for diagnostics.
#[server(TestRoute, "/api")]
pub async fn test_route(
    external_path: String,
    method: String,
) -> Result<(u16, f64), ServerFnError> {
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
        _ => {
            return Err(ServerFnError::ServerError(format!(
                "Unsupported method: {}",
                method
            )))
        }
    };

    let elapsed = start.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds

    match response {
        Ok(resp) => Ok((resp.status().as_u16(), elapsed)),
        Err(e) => Err(ServerFnError::ServerError(format!("Request failed: {}", e))),
    }
}
