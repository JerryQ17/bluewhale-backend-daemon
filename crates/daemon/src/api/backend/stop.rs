use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use tracing::info;

pub async fn handler(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.read().await.backend.is_none() {
        (StatusCode::OK, "Backend already stopped")
    } else {
        let mut child = state.write().await.backend.take().unwrap();
        while child.try_wait().unwrap().is_some() {
            child.kill().await.unwrap();
            info!("killing backend");
        }
        (StatusCode::OK, "Backend stopped")
    }
}
