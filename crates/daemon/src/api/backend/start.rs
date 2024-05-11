use std::borrow::Cow;

use axum::extract::State;
use tracing::{info, warn};

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> Cow<'static, str> {
    match state.start().await {
        Ok(_) => {
            info!("Backend started");
            Cow::Borrowed("Backend started\n")
        }
        Err(e) => {
            let msg = format!("Failed to start backend: {}\n", e);
            warn!("{}", &msg);
            Cow::Owned(msg)
        }
    }
}
