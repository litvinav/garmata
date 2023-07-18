use std::collections::HashMap;
use url::Url;
use super::cookies::Cookie;

pub struct HttpRequest {
    http_version: String,
    headers: HashMap<String, String>,
    method: String,
    pub url: Url,
    body: String,
}

impl HttpRequest {
    pub fn headers(mut self, headers: &HashMap<String, String>) -> Self {
        for (k, v) in headers {
            let key = k.to_lowercase();
            if !["host", "cookie"].contains(&key.as_str()) {
                self.headers.insert(key, v.to_string());
            }
        };
        self
    }

    pub fn render(&self, cookies: Vec<&Cookie>) -> String {
        let headers = self.headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}\r\n"))
            .collect::<Vec<String>>()
            .join("");
        let path_with_query = match self.url.query() {
            Some(query) => self.url.path().to_string() + "?" + query,
            None => self.url.path().to_string(),
        };
        let cookies = if !cookies.is_empty() {
            let cookies = cookies
                .iter()
                .map(|c| format!("{}={}", c.name, c.value))
                .collect::<Vec<String>>()
                .join("; ");
            format!("cookie: {}\r\n", cookies)
        } else {
            "".into()
        };
        let optional_port = self.url
            .port()
            .map(|port| format!(":{port}"))
            .unwrap_or_default();
        let payload = format!(
            "{method} {path_with_query} HTTP/{version}\r\n\
            host: {hostname}{optional_port}\r\n\
            {headers}{cookies}\r\n{body}",
            version = self.http_version,
            method = self.method.to_uppercase(),
            hostname = self.url.host_str().unwrap(),
            body = self.body,
        );
        payload
    }

    pub fn set_body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn new(http_version: &str, method: &str, url: Url) -> Self {
        let mut request = Self {
            http_version: http_version.to_string(),
            headers: HashMap::new(),
            method: method.to_string(),
            url,
            body: String::default(),
        };
        // Ensure minimal headers are present but allow overrides.
        request.headers.insert("accept".into(), "*/*".into());
        request.headers.insert("accept-encoding".into(), "gzip, deflate, br".into());
        request
    }
}
