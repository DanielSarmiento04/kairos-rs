pub fn format_route(host: &str, port: &u16, internal_path: &str) -> String {
    format!(
        "{}:{}{}",
        host,
        port,
        internal_path
    )
}