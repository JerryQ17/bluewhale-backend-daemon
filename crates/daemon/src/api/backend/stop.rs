use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use tokio::process::Command;
use tracing::info;

pub async fn handler(State(state): State<AppState>) -> (StatusCode, &'static str) {
    state.write().await.backend = None;
    let port = state.read().await.config.backend.port;
    match Command::new("fuser")
        .arg(format!("{}/tcp", port))
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                let pid = String::from_utf8(output.stdout).unwrap();
                info!("Killing process with PID {}", pid);
                Command::new("kill")
                    .arg(pid)
                    .output()
                    .await
                    .expect("Failed to kill process");
                (StatusCode::OK, "Backend stopped")
            } else {
                (StatusCode::OK, "Backend already stopped")
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to stop backend"),
    }
}
