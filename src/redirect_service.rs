use reqwest;
use reqwest::Method;

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

// let mapt

// make request and return the response
pub fn make_request(
    url: &str,
    method: &str
) -> Result<String, reqwest::Error> {
    // todo
    Ok("".to_string())
}