
use actix_web::HttpResponse;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Request timeout")]
    Timeout,
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Invalid route configuration: {0}")]
    Config(String),
    #[error("Upstream service error: {0}")]
    Upstream(String),
}

impl actix_web::error::ResponseError for GatewayError {
    fn error_response(&self) -> HttpResponse {
        let error_message = self.to_string();
        match self {
            GatewayError::Timeout => {
                HttpResponse::GatewayTimeout().json(json!({
                    "error": error_message,
                    "type": "timeout"
                }))
            }
            GatewayError::Internal(msg) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": error_message,
                    "type": "internal"
                }))
            }
            GatewayError::Config(msg) => {
                HttpResponse::BadGateway().json(json!({
                    "error": error_message,
                    "type": "config"
                }))
            }
            GatewayError::Upstream(msg) => {
                HttpResponse::BadGateway().json(json!({
                    "error": error_message,
                    "type": "upstream"
                }))
            }
        }
    }
}