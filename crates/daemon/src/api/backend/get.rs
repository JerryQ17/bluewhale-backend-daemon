use std::borrow::Cow;

use axum::extract::State;
use tracing::{info, warn};

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> String {
    let mut backend = state.lock();
    let commit_info = match backend.commit_info() {
        Ok((stdout, _)) => {
            info!("Get commit info from backend successfully");
            Cow::Owned(stdout)
        }
        Err(e) => {
            warn!("Failed to get commit info: {}", e);
            Cow::Borrowed("Failed to get commit info")
        }
    };
    let stdout = match backend.stdout() {
        Ok(stdout) => {
            info!("Get stdout from backend successfully");
            stdout
        }
        Err(e) => {
            warn!("Failed to get stdout: {}", e);
            Cow::Borrowed("Failed to get stdout")
        }
    };
    let stderr = match backend.stderr() {
        Ok(stderr) => {
            info!("Get stderr from backend successfully");
            stderr
        }
        Err(e) => {
            warn!("Failed to get stderr: {}", e);
            Cow::Borrowed("Failed to get stderr")
        }
    };
    format!(
        "Commit Info:\n{}\n\nStandard Output:\n{}\n\nStandard Error:\n{}\n",
        commit_info, stdout, stderr
    )
}
