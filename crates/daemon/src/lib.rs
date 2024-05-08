use std::fs::File;
use std::path::Path;

use tokio::process::Child;

use crate::config::Config;

pub mod config;

pub struct AppState {
    pub backend: Option<Child>,
    pub config: Config,
}

impl AppState {
    pub fn new<S: AsRef<Path>>(config_path: S) -> Self {
        Self {
            backend: None,
            config: serde_json::from_reader(File::open(config_path).unwrap()).unwrap(),
        }
    }
}
