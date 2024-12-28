

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