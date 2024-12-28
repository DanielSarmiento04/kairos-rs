use reqwest;
use reqwest::Method::{
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
    CONNECT,
    TRACE
};

pub fn format_route(
    domain: &str,
    port: u16,
    protocol: &str,
    internal_path: &str
) -> String {
    format!(
       "{}://{}:{}{}", protocol, domain, port, internal_path
    )
}

// make request and return the response
pub fn make_request(
    url: &str,
    method: &str
) -> Result<String, reqwest::Error> {
    // todo
    Ok("".to_string())
}