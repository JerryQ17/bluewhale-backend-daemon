use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;

pub async fn handler(State(state): State<AppState>) -> (StatusCode, &'static str) {
    match super::stop::handler(State(state.clone())).await {
        (StatusCode::OK, _) => super::start::handler(State(state)).await,
        (status, message) => (status, message),
    }
}
