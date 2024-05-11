pub mod get;
pub mod put;
pub mod start;
pub mod stop;
pub mod restart;

use axum::routing;
use axum::Router;

use crate::AppState;

pub const PATH: &str = "/backend";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(get::handler))
        .route("/", routing::put(put::handler))
        .route("/start", routing::patch(start::handler))
        .route("/stop", routing::patch(stop::handler))
        .route("/restart", routing::patch(restart::handler))
}
