use std::env::current_dir;
use std::fs::File;
use std::io::{self, read_to_string};
use std::net::SocketAddr;

use axum::Router;
use time::format_description::parse_owned;
use time::UtcOffset;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::fmt::{self, time::OffsetTime};
use tracing_subscriber::layer::SubscriberExt;

use daemon::config::Config;
use daemon::{api, AppState};

const CONFIG_PATH: &str = "config/daemon/config.json";

#[tokio::main]
async fn main() -> io::Result<()> {
    let config_str = read_to_string(File::open(CONFIG_PATH)?)?;
    let config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => panic!("Invalid config: {}", e),
    };

    config_tracing(&config)?;

    let socket_addr = SocketAddr::from((config.daemon.addr, config.daemon.port));
    let listener = TcpListener::bind(socket_addr).await?;
    info!("Listening on {}", socket_addr);
    let app = Router::new()
        .nest(api::backend::PATH, api::backend::routes())
        .with_state(AppState::new(config));
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

    let log_directory = current_dir()?.join(&config.daemon.log_directory);
    std::fs::create_dir_all(&log_directory)?;
    let log_filepath = log_directory.join(&config.daemon.log_filename);
    let log_file = File::create(&log_filepath)?;

    let file_subscriber = fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_timer(timer);
    let subscriber = tracing_subscriber::registry()
        .with(console_subscriber)
        .with(file_subscriber);
    tracing::subscriber::set_global_default(subscriber).expect("failed to set default subscriber");
    info!("Tracing configuration complete");
    info!("Logging to file: {}", log_filepath.display());
    Ok(())
}
