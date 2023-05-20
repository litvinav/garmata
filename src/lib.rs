use std::{
    io::{Write, Read},
    net::{IpAddr, TcpStream},
    str::FromStr,
    time::{Duration, Instant}, sync::{Arc, RwLock},
};
use native_tls::TlsConnector;
use serde::Deserialize;
use trust_dns_resolver::Resolver as DnsResolver;
use url::Url;

pub struct Phase {
    pub arrival_rate: u64,
    pub duration: u64,
}

fn default_scheme() -> String { "https".into() }
fn default_http_version() -> String { "2".into() }
fn default_users() -> usize { 1 }

#[derive(Deserialize)]
pub struct Configuration { 
    #[serde(default = "default_scheme")]
    pub scheme: String,
    #[serde(default = "default_http_version")]
    pub http_version: String,
    pub target: String,
    pub playlist: Vec<Playlist>,
}

#[derive(Deserialize, Clone)]
pub struct Flow {
    #[serde(default)]
    pub name: String,
    pub path: String,
    pub method: String,
    #[serde(default)]
    pub insecure: bool,
}

#[derive(Deserialize, Clone)]
pub struct Playlist {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_users")]
    pub users: usize,
    pub duration: u64,
    pub flow: Vec<Flow>
}

#[derive(Debug, Clone)]
pub struct SendResult {
    pub group: String,
    pub flow: String,
    pub start_timestamp: String,
    pub dns_duration: Option<Duration>,
    pub connect_duration: Duration,
    pub tls_duration: Option<Duration>,
    pub sending_duration: Duration,
    pub waiting_duration: Duration,
    pub download_duration: Duration,
    pub total_duration: Duration,
}

trait ReadAndWrite: Write + Read {}
impl<T: Write + Read> ReadAndWrite for T {}

fn send(http_version: &String, scheme: &String, target: &String, flow: &Flow, group_name: &String) -> Result<SendResult, GarmataError> {
    let start_timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let url = Url::parse(&format!("{scheme}://{target}{path}", path = flow.path)).unwrap();

    let (addr, dns_duration) = dns_resolve(&url)?;

    let port = url.port_or_known_default().unwrap();
    let (stream, connect_duration) = tcp_connect(addr, port)?;

    let (mut stream, tls_duration) = tls_handshake(stream, &url, flow.insecure)?;

    let (sending_duration, waiting_duration, download_duration) = request(&mut stream, &url, &flow.method, http_version)?;

    Ok(SendResult {
        group: group_name.clone(),
        flow: flow.name.clone(),
        start_timestamp,
        dns_duration,
        connect_duration,
        tls_duration,
        sending_duration,
        // Todo: redirect_duration
        waiting_duration,
        download_duration,
        total_duration:
            if dns_duration.is_none() { Duration::from_secs(0) } else { dns_duration.unwrap() }
            + connect_duration
            + if tls_duration.is_none() { Duration::from_secs(0) } else { tls_duration.unwrap() }
            + sending_duration
            + waiting_duration
            + download_duration
    })
}

#[derive(Debug)]
pub struct GarmataError {
    pub reason: String,
}

fn dns_resolve(url: &Url) -> Result<(IpAddr, Option<Duration>), GarmataError> {
    if url.domain().is_none() {
        let mut ip_str = url.host_str().unwrap();
        if ip_str.starts_with('[') {
            ip_str = &ip_str[1..ip_str.len() - 1];
        }
        match IpAddr::from_str(ip_str) {
            Ok(addr) => Ok((addr, None)),
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
                    Ok((addr, Some(duration)))
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

fn tls_handshake(stream: TcpStream, url: &Url, allow_insecure_certificates: bool) -> Result<(Box<dyn ReadAndWrite> , Option<Duration>), GarmataError> {
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
                Ok(_) => Ok((Box::new(stream), Some(start.elapsed()))),
                Err(_) => Err(GarmataError { reason: format!("unexpected I/O errors while tls handshake to {domain}") }),
            },
            Err(_) => Err(GarmataError { reason: format!("cannot establish a tls handshake to {domain}") }),
        }
    } else {
        Ok((Box::new(stream), None))
    }
}

fn request(stream: &mut Box<dyn ReadAndWrite>, url: &Url, method: &String, version: &String) -> Result<(Duration, Duration, Duration), GarmataError> {
    let payload = format!(
        "{} {} HTTP/{}\r\n\
        Host: {}\r\n\
        Accept: */*\r\n\
        Accept-Encoding: gzip, deflate, br\r\n\
        \r\n",
        method.to_uppercase(),
        version,
        url.path(),
        url.host_str().unwrap()
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
    loop {
        let mut one_byte_buf = [0_u8];
        match stream.read(&mut one_byte_buf) {
            Ok(size) => {
                if waiting_duration.is_none() {
                    waiting_duration = Some(start.elapsed());
                    start = Instant::now();
                }
                if size == 0  {
                    let download_duration = start.elapsed();
                    return Ok((sending_duration, waiting_duration.unwrap(), download_duration))
                }
            },
            Err(_) => return Err(GarmataError { reason: "could not read the server's response".into() }),
        }
    };
}

pub fn run(config: String) -> Vec<SendResult> {
    let file = std::fs::File::open(&config).expect(format!("no {} file found!", &config).as_str());
    let config: Configuration = serde_yaml::from_reader(file).expect("invalid configuration");

    let mut todos = vec![];
    let results = Arc::new(RwLock::new(Vec::new()));
    for entry in config.playlist {
        let scheme = config.scheme.clone();
        let target = config.target.clone();
        let http_version = config.http_version.clone();
        let results = results.clone();

        let handle = std::thread::spawn(move || {
            let deadline = Instant::now().checked_add(Duration::from_secs(entry.duration)).expect("invalid duration provided!");
            loop {
                if Instant::now() >= deadline {
                    break;
                }
                for flow in &entry.flow {
                    match send(&http_version, &scheme, &target, flow, &entry.name) {
                        Ok(result) => results.write().unwrap().push(result),
                        Err(e) => panic!("{}", e.reason),
                    }
                }
            }
        });
        todos.push(handle);
    }

    // Await all
    for todo in todos {
        todo.join().expect("Could not join on the associated thread");
    }
    // Return only the results
    let results = results.read().unwrap().as_slice().to_vec();
    results
}