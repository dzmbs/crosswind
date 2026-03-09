use serde_json::{Value, json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrosswindError {
    // Validation (exit code 2)
    #[error("airport code must be exactly 3 letters, got '{0}'")]
    InvalidAirportCode(String),

    #[error("{0}")]
    InvalidDate(String),

    #[error("{0}")]
    InvalidPassengers(String),

    // Network (exit code 3)
    #[error("request timed out — try increasing --timeout or check your connection")]
    Timeout,

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolution(String),

    #[error("TLS handshake failed: {0}")]
    TlsError(String),

    #[error("proxy error: {0}")]
    ProxyError(String),

    // Rate limit / blocked (exit code 4)
    #[error("rate limited by Google — wait a few minutes or use --proxy")]
    RateLimited,

    #[error("blocked by Google (HTTP {0}) — this usually means bot detection")]
    Blocked(u16),

    // Parse (exit code 5)
    #[error("could not find flight data in page — Google may have changed the page structure")]
    ScriptTagNotFound,

    #[error("failed to parse flight data: {0}")]
    ParseError(String),

    #[error("no flights found for this search")]
    NoResults,

    // General (exit code 1)
    #[error("unexpected HTTP status {0}")]
    HttpStatus(u16),

    #[error("{0}")]
    Other(String),
}

impl CrosswindError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::InvalidAirportCode(_) | Self::InvalidDate(_) | Self::InvalidPassengers(_) => 2,
            Self::Timeout
            | Self::ConnectionFailed(_)
            | Self::DnsResolution(_)
            | Self::TlsError(_)
            | Self::ProxyError(_) => 3,
            Self::RateLimited | Self::Blocked(_) => 4,
            Self::ScriptTagNotFound | Self::ParseError(_) | Self::NoResults => 5,
            Self::HttpStatus(_) | Self::Other(_) => 1,
        }
    }

    pub fn reason_code(&self) -> &str {
        match self {
            Self::InvalidAirportCode(_) => "invalid_airport_code",
            Self::InvalidDate(_) => "invalid_date",
            Self::InvalidPassengers(_) => "invalid_passengers",
            Self::Timeout => "timeout",
            Self::ConnectionFailed(_) => "connection_failed",
            Self::DnsResolution(_) => "dns_resolution",
            Self::TlsError(_) => "tls_error",
            Self::ProxyError(_) => "proxy_error",
            Self::RateLimited => "rate_limited",
            Self::Blocked(_) => "blocked",
            Self::ScriptTagNotFound => "script_tag_not_found",
            Self::ParseError(_) => "parse_error",
            Self::NoResults => "no_results",
            Self::HttpStatus(_) => "http_status",
            Self::Other(_) => "other",
        }
    }

    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::Timeout => Some("try increasing --timeout or check your connection"),
            Self::RateLimited => Some("wait a few minutes before retrying, or use --proxy"),
            Self::Blocked(_) => Some("try again later or use --proxy"),
            Self::ScriptTagNotFound => {
                Some("Google may have changed their page structure — check for updates")
            }
            Self::NoResults => Some("try a different date or route"),
            Self::InvalidAirportCode(_) => Some("use a 3-letter IATA code like JFK, LAX, BEG"),
            _ => None,
        }
    }

    pub fn to_json(&self) -> Value {
        let mut obj = json!({
            "v": 1,
            "status": "error",
            "code": self.reason_code(),
            "message": self.to_string(),
        });
        if let Some(hint) = self.hint() {
            obj["hint"] = json!(hint);
        }
        obj
    }
}
