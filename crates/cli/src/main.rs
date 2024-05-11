use std::env::current_dir;
use std::fs::{canonicalize, File};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::blocking::multipart::Form;
use reqwest::blocking::Client;

#[derive(Debug, Parser)]
#[clap(version = "0.1.0", about = "A command line interface for the daemon.")]
struct Cli {
    #[command(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    #[clap(name = "status", about = "Get the status of the daemon.")]
    Status,
    #[clap(name = "start", about = "Start the daemon.")]
    Start,
    #[clap(name = "stop", about = "Stop the daemon.")]
    Stop,
    #[clap(name = "restart", about = "Restart the daemon.")]
    Restart,
    #[clap(name = "update", about = "Update the daemon.")]
    Update {
        #[arg(help = "The directory to update.")]
        dir: Option<PathBuf>,
    },
}

impl SubCommand {
    const PREFIX: &'static str = "http://127.0.0.1:8081/backend/";

    pub fn handle(self) -> String {
        let client = Client::new();
        match self {
            SubCommand::Status => client.get(Self::PREFIX).send().unwrap().text().unwrap(),
            SubCommand::Start => client
                .patch(format!("{}start", Self::PREFIX))
                .send()
                .unwrap()
                .text()
                .unwrap(),
            SubCommand::Stop => client
                .patch(format!("{}stop", Self::PREFIX))
                .send()
                .unwrap()
                .text()
                .unwrap(),
            SubCommand::Restart => client
                .patch(format!("{}restart", Self::PREFIX))
                .send()
                .unwrap()
                .text()
                .unwrap(),
            SubCommand::Update { dir } => {
                let dir = canonicalize(
                    current_dir()
                        .unwrap()
                        .join(dir.unwrap_or_else(|| PathBuf::from("."))),
                )
                .unwrap();
                println!("{:?}", &dir);
                let temp_name = format!("{}.tar.gz", dir.file_stem().unwrap().to_string_lossy());
                println!("{:?}", &temp_name);
                let temp = File::create(&temp_name).expect("Failed to create temp file.");

                let enc = GzEncoder::new(&temp, Compression::default());
                let mut tar = tar::Builder::new(enc);
                tar.append_dir_all(".", &dir).expect("Failed to append.");

                let form = Form::new().file("file", temp_name).unwrap();

                client
                    .put(Self::PREFIX)
                    .multipart(form)
                    .send()
                    .unwrap()
                    .text()
                    .unwrap()
            }
        }
    }
}

fn main() {
    println!("{}", Cli::parse().sub_cmd.handle());
}
