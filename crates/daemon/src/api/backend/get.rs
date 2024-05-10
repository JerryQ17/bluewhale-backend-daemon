use std::borrow::Cow;

use axum::extract::State;
use serde::Serialize;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tracing::{info, warn};

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct BackendStatus<'a> {
    commit: Cow<'a, str>,
    stdout: String,
    stderr: String,
}

pub async fn handler(State(state): State<AppState>) -> String {
    info!("Getting latest git commit info");
    let read_state = state.read().await;
    let get_latest_commit = Command::new("git")
        .current_dir(
            &read_state
                .config
                .backend
                .working_directory
                .join(&read_state.backend_path),
        )
        .args(["log", "-n", "1"])
        .output()
        .await;
    drop(read_state);

    let latest_commit = match get_latest_commit {
        Ok(ref output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout)
            } else {
                warn!("git exited with status: {}", output.status);
                Cow::Borrowed("Failed to find commit info")
            }
        }
        Err(e) => {
            warn!("Failed to run git: {}", e);
            Cow::Borrowed("Failed to find commit info")
        }
    };

    info!("Getting backend stdout and stderr");
    let mut backend_stdout = String::new();
    let mut backend_stderr = String::new();
    match state.write().await.backend {
        Some(ref mut backend) => {
            match &mut backend.stdout {
                Some(stdout) => match stdout.read_to_string(&mut backend_stdout).await {
                    Ok(n) => info!("Read {} bytes from backend stdout", n),
                    Err(e) => warn!("Failed to read from backend stdout: {}", e),
                },
                None => backend_stdout.push_str("Backend is running but has no stdout"),
            }
            match &mut backend.stderr {
                Some(stderr) => match stderr.read_to_string(&mut backend_stderr).await {
                    Ok(n) => info!("Read {} bytes from backend stderr", n),
                    Err(e) => warn!("Failed to read from backend stderr: {}", e),
                },
                None => backend_stderr.push_str("Backend is running but has no stderr"),
            }
        }
        None => {
            backend_stdout.push_str("Backend is not running");
            backend_stderr.push_str("Backend is not running");
        }
    };
    let backend_status = BackendStatus {
        commit: latest_commit,
        stdout: backend_stdout,
        stderr: backend_stderr,
    };
    serde_json::to_string(&backend_status).unwrap_or_else(|e| {
        warn!("failed to serialize result: {}", e);
        info!("Original result: {:?}", backend_status);
        String::new()
    })
}
