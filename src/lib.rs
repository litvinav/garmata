pub mod configuration;
pub mod http;

use async_std::task::spawn;
use configuration::*;
use std::{
    io::{Write, Read},
    net::{IpAddr, TcpStream},
    str::FromStr,
    time::{Duration, Instant}, sync::{Arc, RwLock},
};
use http::*;
use native_tls::TlsConnector;
use trust_dns_resolver::Resolver as DnsResolver;
use url::Url;

#[derive(Debug)]
pub struct GarmataError {
    pub reason: String,
}
trait ReadAndWrite: Write + Read {}
impl<T: Write + Read> ReadAndWrite for T {}

fn dns_resolve(url: &Url) -> Result<(IpAddr, Duration), GarmataError> {
    if url.domain().is_none() {
        let mut ip_str = url.host_str().unwrap();
        if ip_str.starts_with('[') {
            ip_str = &ip_str[1..ip_str.len() - 1];
        }
        match IpAddr::from_str(ip_str) {
            Ok(addr) => Ok((addr, Duration::default())),
            Err(e) => Err(GarmataError { reason: e.to_string() }),
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
                    Err(GarmataError { reason: "unresolved hostname".into() })
                }
            }
            Err(e) => Err(GarmataError { reason: e.to_string() }),
        }
    }
}

fn tcp_connect(addr: IpAddr, port: u16) -> Result<(TcpStream, Duration), GarmataError> {
    let start = Instant::now();
    match TcpStream::connect((addr, port)) {
        Ok(mut stream) => match stream.flush() {
            Ok(_) => Ok((stream, start.elapsed())),
            Err(_) => Err(GarmataError { reason: format!("unexpected I/O errors while connection to {addr}:{port}") }),
        },
        Err(_) => Err(GarmataError { reason: format!("cannot connect to {addr}:{port}") }),
    }
}

fn tls_handshake(stream: TcpStream, url: &Url, allow_insecure_certificates: bool) -> Result<(Box<dyn ReadAndWrite> , Duration), GarmataError> {
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
                Err(_) => Err(GarmataError { reason: format!("unexpected I/O errors while tls handshake to {domain}") }),
            },
            Err(_) => Err(GarmataError { reason: format!("cannot establish a tls handshake to {domain}") }),
        }
    } else {
        Ok((Box::new(stream), Duration::default()))
    }
}

fn request(stream: &mut Box<dyn ReadAndWrite>, url: &Url, flow: &Flow, version: &String) -> Result<(Duration, Duration, Duration, HttpResponse), GarmataError> {
    let payload = format!(
        "{} {} HTTP/{}\r\n\
        Host: {}\r\n\
        Accept: */*\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        {}\r\n{}",
        flow.method.to_uppercase(),
        url.path(),
        version,
        url.host_str().unwrap(),
        flow.headers.iter().map(|(k,v)| format!("{k}: {v}\r\n")).collect::<Vec<String>>().join(""),
        flow.body
    );
    let start = Instant::now();
    if stream.write_all(payload.as_bytes()).is_err() {
        return Err(GarmataError { reason: "cannot send request to the server".into() })
    };
    if stream.flush().is_err() {
        return Err(GarmataError { reason: format!("unexpected I/O errors while sending request to {url}") });
    };
    let sending_duration = start.elapsed();

    let mut start = Instant::now();
    let mut waiting_duration = None;
    let mut payload: Vec<u8> = vec![];
    loop {
        let mut chunk = [0u8; 32];
        match stream.read(&mut chunk) {
            Ok(size) => {
                if waiting_duration.is_none() {
                    waiting_duration = Some(start.elapsed());
                    start = Instant::now();
                }
                payload.append(&mut chunk.to_vec());
                if size == 0 || chunk.last() == Some(&0u8) {
                    let download_duration = start.elapsed();
                    match HttpResponse::from_payload(String::from_utf8_lossy(&payload)) {
                        Some(response) => return Ok((sending_duration, waiting_duration.unwrap(), download_duration, response)),
                        None => return Err(GarmataError { reason: format!("could not parse http response for {}", url.as_str()) })
                    }
                }
            },
            Err(_) => return Err(GarmataError { reason: "could not read the server's response".into() }),
        }
    };
}

fn execute(http_version: &String, scheme: &String, target: &String, flow: &Flow, group_name: &String, max_redirects: u32) -> Result<HttpResult, GarmataError> {
    let start_timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let mut url = match Url::parse(&format!("{scheme}://{target}{path}", path = flow.path)) {
        Ok(url) => url,
        Err(e) => return Err(GarmataError { reason: e.to_string() }),
    };

    let (addr, dns_duration) = dns_resolve(&url)?;

    let port = url.port_or_known_default().unwrap();
    let (stream, connect_duration) = tcp_connect(addr, port)?;

    let (mut stream, tls_duration) = tls_handshake(stream, &url, flow.insecure)?;

    let mut redirects = 0;
    let mut redirect_duration = Duration::from_secs(0);
    loop {
        let (sending_duration, waiting_duration, download_duration, response) = request(&mut stream, &url, &flow, http_version)?;

        if redirects == max_redirects || !["301", "302", "308"].contains(&response.status.as_str()) {
            return Ok(HttpResult {
                group: group_name.clone(),
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
                total_duration:
                    dns_duration
                    + connect_duration
                    + tls_duration
                    + redirect_duration
                    + sending_duration
                    + waiting_duration
                    + download_duration
            });
        }
        let location = response.headers.get("location").unwrap(); // expected due to status code
        if location.starts_with("http") {
            url = Url::parse(&location).unwrap();
        } else {
            url.set_path(location)
        }

        redirect_duration += sending_duration + waiting_duration + download_duration;
        redirects += 1;
    }
}

pub async fn run(config: String) -> Result<Vec<HttpResult>, GarmataError> {
    let file = match std::fs::File::open(&config) {
        Ok(file) => file,
        Err(_) => return Err(GarmataError { reason: format!("file {} not found", &config) }),
    };
    let config: Configuration = match serde_yaml::from_reader(file) {
        Ok(config) => config,
        Err(e) => return Err(GarmataError { reason: format!("Cannot parse {}: {}", &config, e.to_string()) }),
    };

    let mut all_groups = vec![];
    let results = Arc::new(RwLock::new(Vec::new()));
    for entry in config.groups {
        let scheme = config.scheme.clone();
        let target = config.target.clone();
        let http_version = config.http_version.clone();
        let results = results.clone();
        let deadline = match Instant::now().checked_add(Duration::from_secs(entry.duration)) {
            Some(deadline) => deadline,
            None => {
                return Err(GarmataError { reason: format!("invalid duration provided for group {}", &entry.name) });
            },
        };
        let handle = spawn(async move {
            loop {
                if Instant::now() >= deadline {
                    break;
                }
                let mut all_user_flows = vec![];
                for _ in 0..entry.users {
                    let http_version = http_version.clone();
                    let scheme = scheme.clone();
                    let target = target.clone();
                    let group_name = entry.name.clone();
                    let max_redirects = entry.max_redirects.clone();
                    let flows = entry.flow.clone();
                    let results = results.clone();
                    let handle = spawn(async move {
                        for flow in &flows {
                            match execute(&http_version, &scheme, &target, flow, &group_name, max_redirects) {
                                Ok(result) => results.write().unwrap().push(result),
                                Err(e) => {
                                    eprintln!("{}", e.reason);
                                    break;
                                },
                            }
                        }
                    });
                    all_user_flows.push(handle);
                }
                for user_flow in all_user_flows {
                    user_flow.await
                }
            }
        });
        all_groups.push(handle);
    }

    for group in all_groups {
        group.await
    }
    // Return only the results
    let results = results.read().unwrap().as_slice().to_vec();
    Ok(results)
}