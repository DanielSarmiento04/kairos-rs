use crate::models::{AggregationInterval, MetricsData};
use chrono::{DateTime, Utc};
use leptos::prelude::*;

use super::GATEWAY_BASE_URL;

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
        return Err(ServerFnError::new(format!(
            "Gateway returned error: {}",
            response.status()
        )));
    }

    let text = response
        .text()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to read metrics response: {}", e)))?;

    let metrics = MetricsData::parse_prometheus(&text)
        .map_err(|e| ServerFnError::new(format!("Failed to parse metrics: {}", e)))?;

    Ok(metrics)
}

/// Lists all available historical metrics.
#[server(ListMetrics, "/api")]
pub async fn list_metrics() -> Result<Vec<String>, ServerFnError> {
    let url = format!("{}/api/metrics/list", GATEWAY_BASE_URL);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Gateway returned error: {}",
            response.status()
        )));
    }

    let metrics = response
        .json::<Vec<String>>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse metrics list: {}", e)))?;

    Ok(metrics)
}

/// Fetches historical metrics data.
#[server(GetHistoricalMetrics, "/api")]
pub async fn get_historical_metrics(
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    interval: Option<AggregationInterval>,
) -> Result<serde_json::Value, ServerFnError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/metrics/history", GATEWAY_BASE_URL);

    let mut query = vec![
        ("name", name),
        ("start", start.to_rfc3339()),
        ("end", end.to_rfc3339()),
    ];

    if let Some(interval) = interval {
        let interval_str = serde_json::to_string(&interval)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();
        query.push(("interval", interval_str));
    }

    let response = client
        .get(&url)
        .query(&query)
        .send()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to connect to gateway: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::new(format!(
            "Gateway returned error: {}",
            response.status()
        )));
    }

    let data = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to parse historical metrics: {}", e)))?;

    Ok(data)
}
