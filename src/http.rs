use std::time::Duration;
pub mod cookies;
pub mod response;
pub mod request;

#[derive(Debug, Clone)]
pub struct HttpResult {
    pub group: String,
    pub flow: String,
    pub start_timestamp: String,
    pub dns_duration: Duration,
    pub connect_duration: Duration,
    pub tls_duration: Duration,
    pub redirect_duration: Duration,
    pub sending_duration: Duration,
    pub waiting_duration: Duration,
    pub download_duration: Duration,
    pub total_duration: Duration,
    pub response_status: String,
}