use std::{time::Duration, collections::HashMap, borrow::Cow};

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: String,
    pub headers: HashMap<String,String>
}
impl HttpResponse {
    /// Try parse the http request payload into a HttpResponse.
    pub fn from_payload(payload: Cow<str>) -> Option<HttpResponse> {
        let header_body_split: Vec<(usize, &str)> = payload.match_indices("\r\n\r\n").collect();
        let body_start_index = match header_body_split.first() {
            Some((body_start_index, _)) => body_start_index,
            None => return None,
        };
        let head: Vec<&str> = payload[0..*body_start_index].split("\r\n").collect();
        let mut response = HttpResponse {
            status: head.first().unwrap().split(' ').nth(1).unwrap().to_string(),
            headers: HashMap::new()
        };
        for i in 1..head.len() {
            let mut key_val = head[i].split(": ");
            if let (Some(k),Some(v)) = (key_val.next(), key_val.next()) {
                response.headers.insert(k.to_lowercase(), v.to_string());
            }
        }
        Some(response)
    }
}

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