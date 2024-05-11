use std::fs::read_to_string;
use std::io;

use clap::Parser;
use cli::Cli;

const CONFIG_PATH: &str = "config/cli/config.json";

fn main() -> io::Result<()> {
    let config = serde_json::from_str(&read_to_string(CONFIG_PATH)?).unwrap();
    println!("{}", Cli::parse().handle(config));
    Ok(())
}
