use serde::Deserialize;
use std::collections::HashMap;

fn default_scheme() -> String { "https".into() }
fn default_http_version() -> String { "1.1".into() }
fn default_users() -> usize { 1 }
fn default_max_redirects() -> u32 { 50 }

#[derive(Deserialize)]
pub struct Configuration { 
    #[serde(default = "default_scheme")]
    pub scheme: String,
    #[serde(default = "default_http_version")]
    pub http_version: String,
    pub target: String,
    pub groups: Vec<Group>,
}

#[derive(Deserialize, Clone)]
pub struct Flow {
    #[serde(default)]
    pub name: String,
    pub path: String,
    pub method: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub headers: HashMap<String,String>,
    #[serde(default)]
    pub insecure: bool,
}

#[derive(Deserialize, Clone)]
pub struct Group {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_users")]
    pub users: usize,
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,
    pub duration: u64,
    pub flow: Vec<Flow>
}