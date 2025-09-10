use serde::{Deserialize, Serialize};
// use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Https,
}

impl Protocol {
    /// Returns a list of all supported protocols
    pub fn all() -> &'static [Protocol] {
        &[Protocol::Http, Protocol::Https]
    }

    /// Try to parse a string into a protocol
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" => Some(Protocol::Http),
            "https" => Some(Protocol::Https),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
        }
    }
}
