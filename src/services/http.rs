use crate::models::error::GatewayError;
use crate::models::router::Router;
use crate::utils::path::{find_matching_route, format_route};

use actix_web::{
    http::{Method as ActixMethod, StatusCode},
    web, Error as ActixError, HttpRequest, HttpResponse,
};
use log::log;
use reqwest::{
    header::HeaderMap as ReqwestHeaderMap, header::HeaderName, header::HeaderValue, Client,
    Method as ReqwestMethod,
};
use std::{collections::HashMap, sync::Arc};
use tokio::time::{timeout, Duration};

// Route handler structure
#[derive(Clone)]
pub struct RouteHandler {
    client: Client,
    route_map: Arc<HashMap<String, Router>>,
    timeout_seconds: u64,
}

impl RouteHandler {
    pub fn new(routes: Vec<Router>, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(32)
            .build()
            .expect("Failed to create HTTP client");

        let route_map = Arc::new(
            routes
                .into_iter()
                .map(|route| (route.external_path.clone(), route))
                .collect(),
        );

        Self {
            client,
            route_map,
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
        let reqwest_method = match method {
            ActixMethod::GET => ReqwestMethod::GET,
            ActixMethod::POST => ReqwestMethod::POST,
            ActixMethod::PUT => ReqwestMethod::PUT,
            ActixMethod::DELETE => ReqwestMethod::DELETE,
            ActixMethod::HEAD => ReqwestMethod::HEAD,
            ActixMethod::OPTIONS => ReqwestMethod::OPTIONS,
            ActixMethod::CONNECT => ReqwestMethod::CONNECT,
            ActixMethod::PATCH => ReqwestMethod::PATCH,
            ActixMethod::TRACE => ReqwestMethod::TRACE,
            _ => return Err(GatewayError::Internal("Unsupported HTTP method".to_string()).into()),
        };

        // Convert headers
        let mut reqwest_headers = ReqwestHeaderMap::new();
        for (key, value) in req.headers() {
        
            if key.as_str().to_lowercase() == "host" || key.as_str().to_lowercase().starts_with("connection") {
                continue;
            }

            if let Ok(header_name) = HeaderName::from_bytes(key.as_ref()) {
                if let Ok(header_value) = HeaderValue::from_bytes(value.as_bytes()) {
                    reqwest_headers.insert(header_name, header_value);
                }
            }
        }
        // Ensure User-Agent header is set
        if !reqwest_headers.contains_key("user-agent") {
            reqwest_headers.insert(
                HeaderName::from_static("user-agent"),
                HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"),
            );
        }

        // Find matching route
        let route = self
            .route_map
            .get(&path)
            .ok_or_else(|| GatewayError::Config(format!("No route found for path: {}", path)))?;

        // let route = find_matching_route(&self.route_map, &path)
        //     .ok_or_else(|| GatewayError::Config(format!("No route found for path: {}", path)))?;

        // Validate method is allowed
        if !route.methods.iter().any(|m| m == method.as_str()) {
            return Ok(HttpResponse::MethodNotAllowed().finish());
        }

        let target_url = format_route(&route.host, &route.port, &route.internal_path);

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
            .headers(reqwest_headers);

        // add body if method is different than GET or HEAD
        let forwarded_req = if method != ActixMethod::GET && method != ActixMethod::HEAD {
            forwarded_req.body(body.to_vec())
        } else {
            forwarded_req.body(vec![])
        };

        // Execute request with timeout
        let response = match timeout(
            Duration::from_secs(self.timeout_seconds),
            forwarded_req.send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(GatewayError::Upstream(e.to_string()).into()),
            Err(_) => return Err(GatewayError::Timeout.into()),
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
            Err(e) => Err(GatewayError::Upstream(e.to_string()).into()),
        }
    }
}
