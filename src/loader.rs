use reqwest::Client;
use std::time::{Duration, Instant};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct RequestResult {
    pub request_number: u32,
    pub timestamp: String,
    pub status: Option<u16>,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
}

pub fn build_client(timeout: u64, concurrency: u32) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(timeout))
        .pool_max_idle_per_host(concurrency as usize)
        .build()
        .expect("Failed to build HTTP client")
}

pub async fn send_request(client: &Client, url: &str, request_number: u32) -> RequestResult {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let start = Instant::now();
    match client.get(url).send().await {
        Ok(response) => RequestResult {
            request_number,
            timestamp,
            status: Some(response.status().as_u16()),
            duration: start.elapsed(),
            success: response.status().is_success(),
            error: None,
        },
        Err(e) => RequestResult {
            request_number,
            timestamp,
            status: None,
            duration: start.elapsed(),
            success: false,
            error: Some(e.to_string()),
        },
    }
}
