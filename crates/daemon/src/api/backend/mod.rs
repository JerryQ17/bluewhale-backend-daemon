pub mod get;
pub mod put;

use axum::routing;
use axum::Router;

use crate::AppState;

pub const PATH: &str = "/backend";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(get::handler))
        .route("/", routing::put(put::handler))
}
