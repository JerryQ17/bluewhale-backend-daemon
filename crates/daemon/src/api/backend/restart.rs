use std::borrow::Cow;

use axum::extract::State;

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> Cow<'static, str> {
    match state.restart().await {
        Ok(_) => Cow::Borrowed("Backend restarted\n"),
        Err(e) => Cow::Owned(format!("Failed to restart backend: {}\n", e)),
    }
}
