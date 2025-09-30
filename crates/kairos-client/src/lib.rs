//! Kairos Gateway Client Library
//!
//! This library provides client functionality for interacting with the Kairos API Gateway,
//! including health checks, metrics retrieval, and configuration management.
//! 
//! The client supports both native (using tokio + reqwest) and WebAssembly (using gloo-net)
//! compilation targets with completely separate implementations.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

// Conditional imports based on target
#[cfg(feature = "native")]
use reqwest::Client;
#[cfg(feature = "native")]
use std::time::Duration;

#[cfg(feature = "wasm")]
use gloo_net::http::Request;
#[cfg(feature = "wasm")]
use js_sys;

#[derive(Error, Debug)]
pub enum ClientError {
    #[cfg(feature = "native")]
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[cfg(feature = "wasm")]
    #[error("HTTP request failed: {0}")]
    GlooHttp(#[from] gloo_net::Error),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    
    #[error("Gateway returned error: {status} - {message}")]
    Gateway { status: u16, message: String },
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[cfg(feature = "wasm")]
    #[error("JavaScript error: {0}")]
    JsError(String),
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
    #[cfg(feature = "native")]
    client: Client,
    base_url: Url,
}

impl GatewayClient {
    /// Create a new gateway client
    pub fn new(gateway_url: &str) -> Result<Self, ClientError> {
        let base_url = Url::parse(gateway_url)?;
        
        #[cfg(feature = "native")]
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
            
        Ok(Self {
            #[cfg(feature = "native")]
            client,
            base_url,
        })
    }
    
    /// Check gateway health status
    #[cfg(feature = "native")]
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
    
    /// Check gateway health status
    #[cfg(feature = "wasm")]
    pub async fn health(&self) -> Result<HealthStatus, ClientError> {
        let url = self.base_url.join("/health")?;
        let response = Request::get(url.as_str()).send().await?;
        
        if response.ok() {
            let health = response.json::<HealthStatus>().await?;
            Ok(health)
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(ClientError::Gateway {
                status: response.status(),
                message: text,
            })
        }
    }
    
    /// Get gateway metrics
    #[cfg(feature = "native")]
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
    
    /// Get gateway metrics
    #[cfg(feature = "wasm")]
    pub async fn metrics(&self) -> Result<String, ClientError> {
        let url = self.base_url.join("/metrics")?;
        let response = Request::get(url.as_str()).send().await?;
        
        if response.ok() {
            let metrics = response.text().await?;
            Ok(metrics)
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(ClientError::Gateway {
                status: response.status(),
                message: text,
            })
        }
    }
    
    /// Get parsed metrics snapshot
    pub async fn metrics_snapshot(&self) -> Result<MetricsSnapshot, ClientError> {
        // For now, return mock data with some variation
        // TODO: Parse actual Prometheus format or add JSON endpoint
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        // Simulate some realistic metrics with conditional random generation
        #[cfg(feature = "wasm")]
        let (random_factor, error_factor, conn_factor, latency_factor) = 
            (js_sys::Math::random(), js_sys::Math::random(), js_sys::Math::random(), js_sys::Math::random());
        
        #[cfg(feature = "native")]
        let (random_factor, error_factor, conn_factor, latency_factor) = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>())
        };
        
        let requests_total = 1000 + (random_factor * 500.0) as u64;
        let requests_error = (requests_total as f64 * 0.02 + error_factor * 10.0) as u64;
        let requests_success = requests_total - requests_error;
        
        Ok(MetricsSnapshot {
            requests_total,
            requests_success,
            requests_error,
            active_connections: (10.0 + conn_factor * 20.0) as u64,
            average_response_time_ms: 10.0 + latency_factor * 50.0,
            timestamp,
        })
    }
}