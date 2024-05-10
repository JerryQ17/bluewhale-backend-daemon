use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;

pub async fn handler(State(state): State<AppState>) -> (StatusCode, &'static str) {
    if state.read().await.backend.is_none() {
        (StatusCode::OK, "Backend already stopped")
    } else if state
        .write()
        .await
        .backend
        .take()
        .unwrap()
        .kill()
        .await
        .is_err()
    {
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to stop backend")
    } else {
        (StatusCode::OK, "Backend stopped")
    }
}
