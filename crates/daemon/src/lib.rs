use std::sync::Arc;

use tokio::process::Child;
use tokio::sync::RwLock;

use crate::config::Config;

pub mod api;
pub mod config;

#[derive(Debug, Clone)]
pub struct AppState(Arc<RwLock<RawAppState>>);

impl AppState {
    pub fn new(config: Config) -> Self {
        Self(Arc::new(RwLock::new(RawAppState::new(config))))
    }

    pub fn into_inner(self) -> Arc<RwLock<RawAppState>> {
        self.0
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, RawAppState> {
        self.0.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, RawAppState> {
        self.0.write().await
    }
}

#[derive(Debug)]
pub struct RawAppState {
    pub backend: Option<Child>,
    pub backend_path: String,
    pub config: Config,
}

impl RawAppState {
    pub fn new(config: Config) -> Self {
        Self {
            backend: None,
            backend_path: String::new(),
            config,
        }
    }
}
