pub struct Cookie {
    domain: String,
    including_subdomains: bool,
    path: String,
    pub name: String,
    pub value: String,
}
#[derive(Default)]
pub struct Cookiejar {
    cookies: Vec<Cookie>,
}
impl Cookiejar {
    pub fn get(&self, host: Option<&str>, path: &str) -> Vec<&Cookie> {
        if host.is_none() {
            return vec![];
        }
        let host = host.unwrap();
        let mut scoped_cookies = vec![];
        for cookie in &self.cookies {
            // Is a more strict implementation required?
            let valid_domain = if cookie.including_subdomains {
                host == cookie.domain
            } else {
                host.ends_with(&cookie.domain)
            };
            if valid_domain && path.starts_with(&cookie.path) {
                scoped_cookies.push(cookie);
            }
        }
        scoped_cookies
    }
    /// Cookies are set relative to the current domain. IPs don't work because IP dots would be subdomains.
    pub fn set_all(&mut self, cookies: &Vec<String>, current_host: String) {
        for cookie in cookies {
            let cookie: Vec<&str> = cookie.split("; ").collect();
            let key_val: Vec<&str> = cookie[0].split('=').collect();

            // https://datatracker.ietf.org/doc/html/rfc6265#section-4.1.2.3
            let (domain, including_subdomains) = match cookie
                .iter()
                .skip(1)
                .find(|&c| c.to_lowercase().starts_with("domain="))
            {
                Some(cookie_value) => {
                    // https://datatracker.ietf.org/doc/html/rfc6265#section-5.2.3
                    let mut domain = cookie_value
                        .split('=')
                        .nth(1)
                        .unwrap_or_else(||panic!("invalid Domain cookie value for {cookie_value}"))
                        .to_string();
                    if domain.starts_with('.') {
                        domain = domain.chars().skip(1).collect()
                    };
                    if domain.is_empty() {
                        continue;
                    }
                    (domain.to_lowercase(), true)
                }
                None => (current_host.to_lowercase(), false),
            };

            // https://datatracker.ietf.org/doc/html/rfc6265#section-5.1.4
            let path = match cookie
                .iter()
                .skip(1)
                .find(|&c| c.to_lowercase().starts_with("Path="))
            {
                Some(cookie_value) => {
                    let mut path = cookie_value
                        .split('=')
                        .nth(1)
                        .unwrap_or_else(|| panic!("invalid Path cookie value for {cookie_value}"))
                        .to_string();
                    if path.ends_with('/') {
                        let mut chars = path.chars();
                        chars.next_back();
                        path = chars.as_str().to_string();
                    }
                    path
                }
                None => "/".to_string(),
            };

            // Is a more strict implementation required?
            if domain.ends_with(&current_host) {
                let name = key_val[0].to_string();
                let value = key_val[1].to_string();
                let position = self
                    .cookies
                    .iter()
                    .position(|c| c.domain == domain && c.name == name);
                if let Some(index) = position {
                    self.cookies[index] = Cookie {
                        including_subdomains,
                        domain,
                        path,
                        name,
                        value,
                    };
                } else {
                    self.cookies.push(Cookie {
                        including_subdomains,
                        domain,
                        path,
                        name,
                        value,
                    });
                }
            }
        }
    }
}
