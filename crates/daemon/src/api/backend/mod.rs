pub mod get;

use axum::routing;
use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", routing::get(get::handler))
}
