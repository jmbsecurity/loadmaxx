use chrono::Utc;
use reqwest::Client;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post { body: String, content_type: String },
}

#[derive(Debug, Clone)]
pub struct RequestResult {
    pub request_number: u32,
    pub timestamp: String,
    pub status: Option<u16>,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<String>,
    pub response_body: Option<String>,
}

const MAX_BODY_CAPTURE: usize = 512;

pub fn build_client(timeout: u64, concurrency: u32, force_http2: bool) -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .pool_max_idle_per_host(concurrency as usize);

    if force_http2 {
        builder = builder.http2_prior_knowledge();
    }

    builder.build().expect("Failed to build HTTP client")
}

pub async fn send_request(
    client: &Client,
    url: &str,
    request_number: u32,
    method: &HttpMethod,
) -> RequestResult {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let start = Instant::now();

    let request = match method {
        HttpMethod::Get => client.get(url),
        HttpMethod::Post { body, content_type } => client
            .post(url)
            .header("Content-Type", content_type.as_str())
            .body(body.clone()),
    };

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let success = status.is_success();

            let response_body = if !success {
                response
                    .text()
                    .await
                    .ok()
                    .map(|b| b.chars().take(MAX_BODY_CAPTURE).collect::<String>())
            } else {
                None
            };

            RequestResult {
                request_number,
                timestamp,
                status: Some(status.as_u16()),
                duration: start.elapsed(),
                success,
                error: None,
                response_body,
            }
        }
        Err(e) => RequestResult {
            request_number,
            timestamp,
            status: None,
            duration: start.elapsed(),
            success: false,
            error: Some(e.to_string()),
            response_body: None,
        },
    }
}
