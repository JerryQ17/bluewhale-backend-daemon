use std::fs::File;
use std::io::{self, read_to_string};

use axum::Router;
use time::format_description::parse_owned;
use time::UtcOffset;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::fmt::{self, time::OffsetTime};
use tracing_subscriber::layer::SubscriberExt;

use daemon::config::Config;
use daemon::AppState;

const CONFIG_PATH: &str = "config/daemon/config.json";

#[tokio::main]
async fn main() -> io::Result<()> {
    let config_str = read_to_string(File::open(CONFIG_PATH)?)?;
    let config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => panic!("Invalid config: {}", e),
    };

    config_tracing(&config)?;

    let listener = TcpListener::bind((config.daemon.addr, config.daemon.port)).await?;
    let app = Router::new().with_state(AppState::new(config));
    axum::serve(listener, app).await
}

fn config_tracing(config: &Config) -> io::Result<()> {
    let time_fmt = parse_owned::<2>(&config.daemon.time_format).expect("Invalid time format");
    let time_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let timer = OffsetTime::new(time_offset, time_fmt);
    let console_subscriber = fmt::layer()
        .with_writer(io::stdout)
        .with_ansi(true)
        .with_timer(timer.clone());
    let file_subscriber = fmt::layer()
        .with_writer(File::create(&config.daemon.log_filepath)?)
        .with_ansi(false)
        .with_timer(timer);
    let subscriber = tracing_subscriber::registry()
        .with(console_subscriber)
        .with(file_subscriber);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set default subscriber");
    info!("Tracing configuration complete");
    info!("Logging to file: {}", config.daemon.log_filepath.display());
    Ok(())
}
