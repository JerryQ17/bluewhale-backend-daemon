use daemon::State;

const CONFIG_PATH: &str = "./crates/daemon/config/config.json";

#[tokio::main]
async fn main() {
    let state = State::new(CONFIG_PATH);
}
