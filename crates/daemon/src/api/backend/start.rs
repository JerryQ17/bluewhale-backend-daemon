use axum::extract::State;
use axum::http::StatusCode;
use tokio::process::Command;
use tracing::{info, warn};

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.read().await.backend.is_some() {
        (StatusCode::OK, "Backend already running")
    } else {
        let read_state = state.read().await;
        let wd = &read_state.config.backend.working_directory.join(&read_state.backend_path);
        drop(read_state);
        info!("Working directory: {}", wd.display());
        info!("Installing dependencies");
        if Command::new("mvn")
            .current_dir(wd)
            .arg("install")
            .output()
            .await
            .is_err()
        {
            warn!("Failed to install dependencies");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to install dependencies",
            );
        }
        info!("Starting backend");
        match Command::new("mvn")
            .current_dir(wd)
            .arg("spring-boot:run")
            .spawn()
        {
            Ok(child) => {
                state.write().await.backend = Some(child);
                (StatusCode::OK, "Backend started")
            }
            Err(_) => {
                warn!("Failed to start backend");
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to start backend")
            }
        }
    }
}
