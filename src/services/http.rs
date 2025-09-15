use crate::models::error::GatewayError;
use crate::models::router::Router;
use crate::utils::path::format_route;
use crate::utils::route_matcher::RouteMatcher;

use actix_web::{
    http::{Method as ActixMethod, StatusCode},
    web, Error as ActixError, HttpRequest, HttpResponse,
};
use log::log;
use reqwest::{
    header::HeaderMap as ReqwestHeaderMap, header::HeaderName, header::HeaderValue, Client,
    Method as ReqwestMethod,
};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

// Route handler structure
#[derive(Clone)]
pub struct RouteHandler {
    client: Client,
    route_matcher: Arc<RouteMatcher>,
    timeout_seconds: u64,
}

impl RouteHandler {
    pub fn new(routes: Vec<Router>, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to create HTTP client");

        let route_matcher = Arc::new(
            RouteMatcher::new(routes).expect("Failed to create route matcher")
        );

        Self {
            client,
            route_matcher,
            timeout_seconds,
        }
    }

    pub async fn handle_request(
        &self,
        req: HttpRequest,
        body: web::Bytes,
    ) -> Result<HttpResponse, ActixError> {
        let path = req.path().to_string();
        let method = req.method().clone();

        // Convert Actix method to Reqwest method
        let reqwest_method = self.parse_method(&method);

        // Convert headers
        let reqwest_headers = self.build_headers_optimized(req.headers());

        // Find matching route using the new pattern matching function
        let (route, transformed_internal_path) = self.route_matcher.find_match(&path)
            .map_err(|e| match e {
                crate::utils::route_matcher::RouteMatchError::NoMatch { path } => {
                    GatewayError::RouteNotFound { path }
                }
                _ => GatewayError::Config { 
                    message: e.to_string(), 
                    route: path.clone() 
                }
            })?;

        // Validate method is allowed
        if !route.methods.iter().any(|m| m == method.as_str()) {
            return Err(GatewayError::MethodNotAllowed { 
                method: method.to_string(), 
                path: path.clone() 
            }.into());
        }

        let target_url = format_route(&route.host, &route.port, &transformed_internal_path);

        log!(log::Level::Info, "Forwarding request to: {}", target_url);
        log!(
            log::Level::Debug,
            "Request details: method={}, path={}, headers={:?}",
            method,
            path,
            reqwest_headers
        );
        // print route details
        log!(
            log::Level::Debug,
            "Route details: host={}, port={}, external_path={}, internal_path={}, methods={:?}",
            route.host,
            route.port,
            route.external_path,
            route.internal_path,
            route.methods
        );

        // Forward the request with converted method
        let forwarded_req = self
            .client
            .request(reqwest_method, &target_url)
            .body(body.to_vec())
            .headers(reqwest_headers);

        // Execute request with timeout
        let response = match timeout(
            Duration::from_secs(self.timeout_seconds),
            forwarded_req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(GatewayError::Upstream { 
                message: e.to_string(),
                url: target_url.clone(),
                status: None,
            }.into()),
            Err(_) => return Err(GatewayError::Timeout { 
                timeout: self.timeout_seconds 
            }.into()),
        };

        // Convert upstream response to HttpResponse
        let mut builder =
            HttpResponse::build(StatusCode::from_u16(response.status().as_u16()).unwrap());

        // Forward headers with proper conversion
        for (key, value) in response.headers() {
            if !key.as_str().starts_with("connection") {
                if let Ok(header_value) =
                    actix_web::http::header::HeaderValue::from_bytes(value.as_bytes())
                {
                    builder.insert_header((key.as_str(), header_value));
                }
            }
        }

        // Handle the response body
        match response.bytes().await {
            Ok(bytes) => Ok(builder.body(bytes)),
            Err(e) => Err(GatewayError::Upstream { 
                message: e.to_string(),
                url: target_url,
                status: None,
            }.into()),
        }
    }

    fn build_headers_optimized(
        &self,
        original_headers: &actix_web::http::header::HeaderMap,
    ) -> ReqwestHeaderMap {
        let mut reqwest_headers = ReqwestHeaderMap::with_capacity(original_headers.len());
        
        // Skip problematic headers more efficiently
        const SKIP_HEADERS: &[&str] = &["host", "connection", "upgrade", "proxy-connection"];
        
        for (key, value) in original_headers {
            let key_str = key.as_str().to_lowercase();
            if SKIP_HEADERS.iter().any(|&skip| key_str.starts_with(skip)) {
                continue;
            }

            // More efficient header conversion
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(key.as_ref()),
                HeaderValue::from_bytes(value.as_bytes())
            ) {
                reqwest_headers.insert(header_name, header_value);
            }
        }
        
        // Set default User-Agent if not present
        reqwest_headers.entry("user-agent")
            .or_insert_with(|| HeaderValue::from_static("kairos-rs/0.2.0"));
        
        reqwest_headers
    }

    fn parse_method(&self, method: &ActixMethod) -> ReqwestMethod {
        match method {
            &ActixMethod::GET => ReqwestMethod::GET,
            &ActixMethod::POST => ReqwestMethod::POST,
            &ActixMethod::PUT => ReqwestMethod::PUT,
            &ActixMethod::DELETE => ReqwestMethod::DELETE,
            &ActixMethod::HEAD => ReqwestMethod::HEAD,
            &ActixMethod::OPTIONS => ReqwestMethod::OPTIONS,
            &ActixMethod::CONNECT => ReqwestMethod::CONNECT,
            &ActixMethod::PATCH => ReqwestMethod::PATCH,
            &ActixMethod::TRACE => ReqwestMethod::TRACE,
            _ => ReqwestMethod::GET, // or another default, or panic! if you want to handle this differently
        }
    }
}
