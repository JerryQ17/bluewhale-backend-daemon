use std::borrow::Cow;

use axum::extract::State;
use tracing::{info, warn};

use crate::AppState;

pub async fn handler(State(state): State<AppState>) -> Cow<'static, str> {
    match state.stop() {
        Ok(s) => {
            info!("Backend stopped");
            s
        }
        Err(e) => {
            let msg = format!("Failed to stop backend: {}\n", e);
            warn!("{}", &msg);
            Cow::Owned(msg)
        }
    }
}
