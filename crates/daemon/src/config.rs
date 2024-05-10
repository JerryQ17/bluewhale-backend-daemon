use std::env::current_dir;
use std::fs::{canonicalize, create_dir_all, read_to_string};
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub backend: BackendConfig,
    pub daemon: DaemonConfig,
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<serde_json::Result<Self>> {
        match serde_json::from_str::<Self>(&read_to_string(path)?) {
            Ok(mut config) => {
                create_dir_all(&config.backend.working_directory)?;
                config.backend.working_directory = canonicalize(config.backend.working_directory)?;
                let log_dir = current_dir()?.join(&config.daemon.log_directory);
                create_dir_all(&log_dir)?;
                config.daemon.log_directory = canonicalize(log_dir)?;
                Ok(Ok(config))
            }
            Err(e) => Ok(Err(e)),
        }
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
    pub log_directory: PathBuf,
    pub log_filename: String,
}

impl From<DaemonConfig> for SocketAddr {
    fn from(config: DaemonConfig) -> Self {
        SocketAddr::new(config.addr, config.port)
    }
}
