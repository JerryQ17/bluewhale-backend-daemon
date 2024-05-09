use serde::{Deserialize, Serialize};
use std::fs::File;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub backend: BackendConfig,
    pub daemon: DaemonConfig,
}

impl Config {
    pub fn read<P: AsRef<Path>>(path: P) {
        serde_json::from_reader(File::open(path).unwrap()).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub working_directory: PathBuf,
    pub addr: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub addr: IpAddr,
    pub port: u16,
    pub time_format: String,
    pub log_filepath: PathBuf,
}

impl From<DaemonConfig> for SocketAddr {
    fn from(config: DaemonConfig) -> Self {
        SocketAddr::new(config.addr, config.port)
    }
}
