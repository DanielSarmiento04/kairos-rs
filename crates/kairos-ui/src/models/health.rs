//! Health check response structures.

use serde::{Deserialize, Serialize};

/// Health check response from /health endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
}

impl HealthResponse {
    /// Check if the service is healthy.
    pub fn is_healthy(&self) -> bool {
        self.status.to_lowercase() == "healthy" || self.status.to_lowercase() == "ok"
    }
    
    /// Format uptime in human-readable form.
    pub fn format_uptime(&self) -> String {
        let seconds = self.uptime_seconds;
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        
        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, secs)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, secs)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

/// Readiness probe response from /health/ready endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessResponse {
    pub status: String,
    pub timestamp: String,
}

impl ReadinessResponse {
    /// Check if the service is ready.
    pub fn is_ready(&self) -> bool {
        self.status.to_lowercase() == "ready"
    }
}

/// Liveness probe response from /health/live endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResponse {
    pub status: String,
    pub timestamp: String,
}

impl LivenessResponse {
    /// Check if the service is alive.
    pub fn is_alive(&self) -> bool {
        self.status.to_lowercase() == "alive"
    }
}
