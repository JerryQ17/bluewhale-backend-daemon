use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use daemon::AppState;

const CONFIG_PATH: &str = "./crates/daemon/config/config.json";

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new(CONFIG_PATH));
    let listener = TcpListener::bind(SocketAddr::from(state.config.daemon))
        .await
        .unwrap();
    let app = Router::new().with_state(state);
    axum::serve(listener, app).await.unwrap()
}
