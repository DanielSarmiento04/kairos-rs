//! Kairos Gateway Client Library
//!
//! This library provides client functionality for interacting with the Kairos API Gateway,
//! including health checks, metrics retrieval, and configuration management.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    
    #[error("Gateway returned error: {status} - {message}")]
    Gateway { status: u16, message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct MetricsSnapshot {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub active_connections: u64,
    pub average_response_time_ms: f64,
    pub timestamp: String,
}

/// Client for interacting with Kairos API Gateway
pub struct GatewayClient {
    client: Client,
    base_url: Url,
}

impl GatewayClient {
    /// Create a new gateway client
    pub fn new(gateway_url: &str) -> Result<Self, ClientError> {
        let base_url = Url::parse(gateway_url)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
            
        Ok(Self {
            client,
            base_url,
        })
    }
    
    /// Check gateway health status
    pub async fn health(&self) -> Result<HealthStatus, ClientError> {
        let url = self.base_url.join("/health")?;
        let response = self.client.get(url).send().await?;
        
        if response.status().is_success() {
            let health = response.json::<HealthStatus>().await?;
            Ok(health)
        } else {
            Err(ClientError::Gateway {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }
    
    /// Get gateway metrics
    pub async fn metrics(&self) -> Result<String, ClientError> {
        let url = self.base_url.join("/metrics")?;
        let response = self.client.get(url).send().await?;
        
        if response.status().is_success() {
            let metrics = response.text().await?;
            Ok(metrics)
        } else {
            Err(ClientError::Gateway {
                status: response.status().as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }
    
    /// Get parsed metrics snapshot
    pub async fn metrics_snapshot(&self) -> Result<MetricsSnapshot, ClientError> {
        // TODO: Parse Prometheus format or add JSON endpoint
        // For now, return mock data
        Ok(MetricsSnapshot {
            requests_total: 1234,
            requests_success: 1200,
            requests_error: 34,
            active_connections: 12,
            average_response_time_ms: 15.5,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}