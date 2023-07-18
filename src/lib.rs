// MIT License - free as in freedom; Full license in the LICENSE file
pub mod configuration;
pub mod http;

use async_std::task::spawn;
use configuration::*;
use http::cookies::{Cookie, Cookiejar};
use http::request::HttpRequest;
use http::response::HttpResponse;
use http::*;
use native_tls::TlsConnector;
use std::{
    io::{Read, Write},
    net::{IpAddr, TcpStream},
    str::FromStr,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use trust_dns_resolver::Resolver as DnsResolver;
use url::Url;

trait ReadAndWrite: Write + Read {}
impl<T: Write + Read> ReadAndWrite for T {}

#[derive(Debug)]
pub struct GarmataError {
    pub reason: String,
}

pub async fn run(config: String, is_debug: bool) -> Result<Vec<HttpResult>, GarmataError> {
    let config: Configuration = match std::fs::File::open(&config) {
        Ok(file) => match serde_yaml::from_reader(file) {
            Ok(config) => config,
            Err(e) => {
                return Err(GarmataError {
                    reason: format!("cannot parse {config}: {e}"),
                })
            }
        },
        Err(e) => {
            return Err(GarmataError {
                reason: e.to_string(),
            })
        }
    };

    let mut all_groups = vec![];
    let results = Arc::new(RwLock::new(Vec::new()));
    for group in config.groups {
        let scheme = config.scheme.clone();
        let target = config.target.clone();
        let http_version = config.http_version.clone();
        let results = results.clone();
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(group.duration))
            .expect(&format!(
                "invalid duration provided for group {}",
                &group.name
            ));
        let handle = spawn(async move {
            loop {
                let mut all_user_flows = vec![];
                for _ in 0..group.users {
                    let http_version = http_version.clone();
                    let scheme = scheme.clone();
                    let target = target.clone();
                    let group_name = group.name.clone();
                    let results = results.clone();
                    let flows = group.flows.clone();
                    let handle = spawn(async move {
                        for flow in &flows {
                            match execute(
                                &http_version,
                                &scheme,
                                &target,
                                flow,
                                &group_name,
                                is_debug,
                            ) {
                                Ok(result) => results.write().unwrap().push(result),
                                Err(e) => {
                                    eprintln!("{}", e.reason);
                                    break;
                                }
                            }
                        }
                    });
                    all_user_flows.push(handle);
                }
                for user_flow in all_user_flows {
                    user_flow.await
                }
                if Instant::now() >= deadline {
                    break;
                }
            }
        });
        all_groups.push(handle);
    }

    for group in all_groups {
        group.await
    }
    let results = results.read().unwrap().as_slice().to_vec();
    Ok(results)
}

fn execute(
    http_version: &str,
    scheme: &String,
    target: &String,
    flow: &Flow,
    group_name: &str,
    is_debug: bool,
) -> Result<HttpResult, GarmataError> {
    let mut method = flow.method.clone();
    let start_timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let url = match Url::parse(&format!("{scheme}://{target}{}", flow.path)) {
        Ok(url) => url,
        Err(e) => {
            return Err(GarmataError {
                reason: e.to_string(),
            })
        }
    };

    let mut redirects = 0;
    let mut redirect_duration = Duration::from_secs(0);
    let mut http_request = HttpRequest::new(http_version, &method, url)
        .headers(&flow.headers)
        .set_body(&flow.body);
    let mut cookiejar = Cookiejar::default();
    cookiejar.set_all(
        &flow.cookies,
        http_request.url.host_str().unwrap().to_string(),
    );

    loop {
        let (addr, dns_duration) = dns_resolve(&http_request.url)?;
        let port = http_request.url.port_or_known_default().unwrap();
        let (stream, connect_duration) = tcp_connect(addr, port)?;
        let (mut stream, tls_duration) = tls_handshake(stream, &http_request.url, flow.insecure)?;

        let (sending_duration, waiting_duration, download_duration, response) = request(
            &mut stream,
            &http_request,
            cookiejar.get(http_request.url.host_str(), http_request.url.path()),
            is_debug,
        )?;

        if redirects == flow.max_redirects
            || !["301", "302", "303", "307", "308"].contains(&response.status.as_str())
        {
            return Ok(HttpResult {
                group: group_name.to_owned(),
                flow: flow.name.clone(),
                start_timestamp,
                dns_duration,
                connect_duration,
                tls_duration,
                sending_duration,
                waiting_duration,
                download_duration,
                redirect_duration,
                response_status: response.status,
                total_duration: dns_duration
                    + connect_duration
                    + tls_duration
                    + redirect_duration
                    + sending_duration
                    + waiting_duration
                    + download_duration,
            });
        }

        if ["301", "303"].contains(&response.status.as_str()) {
            method = "GET".into();
        }

        let location = response
            .headers
            .iter()
            .find(|(k, _)| k == "location")
            .unwrap_or_else(|| panic!("response of {} did not provide the expected location header but returned the status code {}", &http_request.url, &response.status));

        if let Some(domain) = http_request.url.host_str() {
            let cookies = response
                .headers
                .iter()
                .filter(|(key, _)| key == "set-cookie")
                .map(|(_, v)| v.to_string())
                .collect();
            cookiejar.set_all(&cookies, domain.to_string());
        }

        if location.1.starts_with("http") {
            let url = Url::parse(&location.1).unwrap();
            http_request = HttpRequest::new(http_version, &method, url);
        } else {
            http_request.url.set_path(&location.1)
        };

        redirect_duration += dns_duration
            + connect_duration
            + tls_duration
            + sending_duration
            + waiting_duration
            + download_duration;
        redirects += 1;
    }
}

fn request(
    stream: &mut Box<dyn ReadAndWrite>,
    http_request: &HttpRequest,
    cookies: Vec<&Cookie>,
    is_debug: bool,
) -> Result<(Duration, Duration, Duration, HttpResponse), GarmataError> {
    let payload = http_request.render(cookies);
    if is_debug {
        println!("{payload}");
    }

    let start = Instant::now();
    if stream.write_all(payload.as_bytes()).is_err() {
        return Err(GarmataError {
            reason: "cannot send request to the server".into(),
        });
    };
    if stream.flush().is_err() {
        return Err(GarmataError {
            reason: format!(
                "unexpected I/O errors while sending request to {}",
                http_request.url
            ),
        });
    };
    let sending_duration = start.elapsed();

    let mut start = Instant::now();
    let mut waiting_duration = None;
    let mut payload: Vec<u8> = vec![];
    loop {
        let mut chunk = [0u8; 512];
        match stream.read(&mut chunk) {
            Ok(size) => {
                stream.flush().unwrap();
                if waiting_duration.is_none() {
                    waiting_duration = Some(start.elapsed());
                    start = Instant::now();
                }
                payload.extend_from_slice(&chunk);
                if size == 0 || chunk.len() != size {
                    let download_duration = start.elapsed();
                    let payload = String::from_utf8_lossy(&payload);
                    if is_debug {
                        println!("{payload}");
                    }
                    match HttpResponse::try_from(payload) {
                        Ok(response) => {
                            return Ok((
                                sending_duration,
                                waiting_duration.unwrap(),
                                download_duration,
                                response,
                            ))
                        }
                        Err(mut e) => {
                            e.reason += &format!(" from url {}", http_request.url);
                            return Err(e);
                        }
                    }
                }
            }
            Err(_) => {
                return Err(GarmataError {
                    reason: format!(
                        "could not read server's response for url {}",
                        http_request.url
                    ),
                });
            }
        }
    }
}

fn tls_handshake(
    stream: TcpStream,
    url: &Url,
    allow_insecure_certificates: bool,
) -> Result<(Box<dyn ReadAndWrite>, Duration), GarmataError> {
    if url.scheme() == "https" {
        let tls_connector = TlsConnector::builder()
            .danger_accept_invalid_hostnames(allow_insecure_certificates)
            .danger_accept_invalid_certs(allow_insecure_certificates)
            .build()
            .unwrap();
        let domain = url.host_str().unwrap();
        let start = Instant::now();
        match tls_connector.connect(domain, stream) {
            Ok(mut stream) => match stream.flush() {
                Ok(_) => Ok((Box::new(stream), start.elapsed())),
                Err(_) => Err(GarmataError {
                    reason: format!("unexpected I/O errors while tls handshake to {domain}"),
                }),
            },
            Err(_) => Err(GarmataError {
                reason: format!("cannot establish a tls handshake to {domain}"),
            }),
        }
    } else {
        Ok((Box::new(stream), Duration::default()))
    }
}

fn tcp_connect(addr: IpAddr, port: u16) -> Result<(TcpStream, Duration), GarmataError> {
    let start = Instant::now();
    match TcpStream::connect((addr, port)) {
        Ok(mut stream) => match stream.flush() {
            Ok(_) => Ok((stream, start.elapsed())),
            Err(_) => Err(GarmataError {
                reason: format!("unexpected I/O errors while connection to {addr}:{port}"),
            }),
        },
        Err(_) => Err(GarmataError {
            reason: format!("cannot connect to {addr}:{port}"),
        }),
    }
}

fn dns_resolve(url: &Url) -> Result<(IpAddr, Duration), GarmataError> {
    if url.domain().is_none() {
        let mut ip_str = url.host_str().unwrap();
        if ip_str.starts_with('[') {
            ip_str = &ip_str[1..ip_str.len() - 1];
        }
        match IpAddr::from_str(ip_str) {
            Ok(addr) => Ok((addr, Duration::default())),
            Err(e) => Err(GarmataError {
                reason: e.to_string(),
            }),
        }
    } else {
        let resolver = DnsResolver::from_system_conf().unwrap();
        let start = Instant::now();
        let result = resolver.lookup_ip(url.host_str().unwrap());
        let duration = start.elapsed();
        match result {
            Ok(response) => {
                if let Some(addr) = response.iter().find(|i| i.is_ipv4() || i.is_ipv6()) {
                    Ok((addr, duration))
                } else {
                    Err(GarmataError {
                        reason: "unresolved hostname".into(),
                    })
                }
            }
            Err(e) => Err(GarmataError {
                reason: e.to_string(),
            }),
        }
    }
}
