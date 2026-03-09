use std::sync::Arc;
use std::time::Duration;

use wreq::Client;
use wreq::cookie::Jar;
use wreq_util::Emulation;

use crate::error::CrosswindError;

const BASE_URL: &str = "https://www.google.com/travel/flights";

fn cache_buster() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

pub async fn fetch_html(url: &str, timeout_secs: u64) -> Result<String, CrosswindError> {
    let jar = Arc::new(Jar::default());
    let google_uri: wreq::Uri = "https://www.google.com".parse().unwrap();
    jar.add(
        "SOCS=CAESEwgDEgk2MjA5NDM1NjAaAmVuIAEaBgiA_Le-Bg",
        &google_uri,
    );
    jar.add("CONSENT=PENDING+987", &google_uri);

    let client = Client::builder()
        .emulation(Emulation::Chrome137)
        .cookie_provider(jar)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| CrosswindError::ConnectionFailed(e.to_string()))?;

    let response = client.get(url).send().await.map_err(map_request_error)?;

    let status = response.status().as_u16();
    match status {
        200 => {}
        429 => return Err(CrosswindError::RateLimited),
        403 | 503 => return Err(CrosswindError::Blocked(status)),
        s if s >= 400 => return Err(CrosswindError::HttpStatus(s)),
        _ => {}
    }

    response
        .text()
        .await
        .map_err(|e| CrosswindError::ConnectionFailed(e.to_string()))
}

pub async fn fetch_flights(
    tfs: &str,
    currency: &str,
    lang: &str,
    timeout_secs: u64,
) -> Result<String, CrosswindError> {
    let url = format!(
        "{}?tfs={}&hl={}&curr={}&tfu=EgQIABABIgA&cx={}",
        BASE_URL,
        tfs,
        lang,
        currency,
        cache_buster(),
    );
    fetch_html(&url, timeout_secs).await
}

fn map_request_error(e: wreq::Error) -> CrosswindError {
    let msg = e.to_string();
    if e.is_timeout() {
        CrosswindError::Timeout
    } else if e.is_connect() {
        if msg.contains("dns") || msg.contains("resolve") {
            CrosswindError::DnsResolution(msg)
        } else {
            CrosswindError::ConnectionFailed(msg)
        }
    } else if msg.contains("tls") || msg.contains("ssl") || msg.contains("certificate") {
        CrosswindError::TlsError(msg)
    } else {
        CrosswindError::ConnectionFailed(msg)
    }
}
