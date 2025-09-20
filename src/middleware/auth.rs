//! JWT authentication middleware for the Kairos-rs gateway.
//! 
//! This module provides JWT token validation middleware that can be applied
//! to routes requiring authentication. It supports configurable JWT validation
//! with proper error handling and security best practices.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
    body::BoxBody,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub algorithm: Algorithm,
    pub required_claims: HashSet<String>,
    pub issuer: Option<String>,
    pub audience: Option<String>,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            algorithm: Algorithm::HS256,
            required_claims: HashSet::new(),
            issuer: None,
            audience: None,
        }
    }
}

impl JwtConfig {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            ..Default::default()
        }
    }
    
    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.issuer = Some(issuer);
        self
    }
    
    pub fn with_audience(mut self, audience: String) -> Self {
        self.audience = Some(audience);
        self
    }
    
    pub fn with_required_claims(mut self, claims: Vec<String>) -> Self {
        self.required_claims = claims.into_iter().collect();
        self
    }
}

pub struct JwtAuth {
    config: Rc<JwtConfig>,
}

impl JwtAuth {
    pub fn new(config: JwtConfig) -> Self {
        Self {
            config: Rc::new(config),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
            config: self.config.clone(),
        })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    config: Rc<JwtConfig>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Extract JWT token from Authorization header
            let token = match extract_jwt_token(&req) {
                Ok(token) => token,
                Err(error_msg) => {
                    warn!("JWT extraction failed: {}", error_msg);
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "error": error_msg,
                                "type": "authentication_error",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }))
                            .map_into_boxed_body()
                    ));
                }
            };

            // Validate JWT token
            match validate_jwt_token(&token, &config) {
                Ok(claims) => {
                    debug!("JWT validation successful for user: {}", claims.sub);
                    
                    // Add claims to request extensions for downstream use
                    req.extensions_mut().insert(claims);
                    
                    // Continue to the next service
                    let res = service.call(req).await?;
                    Ok(res.map_into_boxed_body())
                }
                Err(error) => {
                    warn!("JWT validation failed: {}", error);
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "error": "Invalid or expired token",
                                "type": "authentication_error",
                                "timestamp": chrono::Utc::now().to_rfc3339()
                            }))
                            .map_into_boxed_body()
                    ))
                }
            }
        })
    }
}

fn extract_jwt_token(req: &ServiceRequest) -> Result<String, &'static str> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or("Missing Authorization header")?;

    let auth_str = auth_header.to_str()
        .map_err(|_| "Invalid Authorization header format")?;

    if !auth_str.starts_with("Bearer ") {
        return Err("Authorization header must start with 'Bearer '");
    }

    Ok(auth_str[7..].to_string()) // Remove "Bearer " prefix
}

fn validate_jwt_token(token: &str, config: &JwtConfig) -> Result<Claims, String> {
    let mut validation = Validation::new(config.algorithm);
    
    // Configure validation parameters
    if let Some(ref issuer) = config.issuer {
        validation.set_issuer(&[issuer]);
    }
    
    if let Some(ref audience) = config.audience {
        validation.set_audience(&[audience]);
    }

    let decoding_key = DecodingKey::from_secret(config.secret.as_ref());
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token validation failed: {}", e))?;

    let claims = token_data.claims;
    
    // Validate required claims
    for required_claim in &config.required_claims {
        match required_claim.as_str() {
            "roles" => {
                if claims.roles.is_none() {
                    return Err("Missing required 'roles' claim".to_string());
                }
            }
            "iss" => {
                if claims.iss.is_none() {
                    return Err("Missing required 'iss' claim".to_string());
                }
            }
            "aud" => {
                if claims.aud.is_none() {
                    return Err("Missing required 'aud' claim".to_string());
                }
            }
            _ => {
                // For other claims, we'd need to extend the Claims struct
                debug!("Unknown required claim: {}", required_claim);
            }
        }
    }

    Ok(claims)
}

/// Helper function to create JWT tokens for testing
pub fn create_test_token(claims: Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    
    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    
    encode(&header, &claims, &encoding_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse, Result};
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn protected_handler() -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Access granted to protected resource"
        })))
    }

    #[actix_web::test]
    async fn test_missing_auth_header() {
        let config = JwtConfig::new("test-secret".to_string());
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth::new(config))
                .route("/protected", web::get().to(protected_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_invalid_token() {
        let config = JwtConfig::new("test-secret".to_string());
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth::new(config))
                .route("/protected", web::get().to(protected_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", "Bearer invalid-token"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_valid_token() {
        let secret = "test-secret";
        let config = JwtConfig::new(secret.to_string());
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let claims = Claims {
            sub: "test-user".to_string(),
            exp: now + 3600, // 1 hour from now
            iat: now,
            iss: None,
            aud: None,
            roles: None,
        };
        
        let token = create_test_token(claims, secret).unwrap();
        
        let app = test::init_service(
            App::new()
                .wrap(JwtAuth::new(config))
                .route("/protected", web::get().to(protected_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}