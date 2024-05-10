use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{info, warn};

use crate::AppState;

pub async fn handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> (StatusCode, &'static str) {
    if let Ok(field) = multipart.next_field().await {
        let backend_config = state.read().await.config.backend.clone();
        let working_directory = backend_config.working_directory;
        let name = backend_config.name;
        match field {
            None => {
                warn!("No file provided");
                (StatusCode::BAD_REQUEST, "No file provided")
            }
            Some(field) => {
                let filename = match field.file_name() {
                    Some(name) => name.to_string(),
                    None => {
                        warn!("Failed to read filename, using config");
                        name
                    }
                };
                let filepath = working_directory.join(&filename);
                state.write().await.backend_path = filename
                    .split_once('.')
                    .map_or(&*filename, |(name, _)| name)
                    .to_string();
                info!("Creating temp file at {}", filepath.display());
                let mut file = match File::create(&filepath).await {
                    Ok(f) => f,
                    Err(_) => {
                        warn!("Failed to create temp file");
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to create temp file",
                        );
                    }
                };
                info!("Reading uploaded file");
                let bytes = match field.bytes().await {
                    Ok(b) => b,
                    Err(_) => {
                        warn!("Failed to read uploaded file");
                        return (StatusCode::BAD_REQUEST, "Failed to read uploaded file");
                    }
                };
                info!("Writing bytes to {}", filepath.display());
                if file.write_all(&bytes).await.is_err() {
                    warn!("Failed to write bytes to temp file");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to write to temp file",
                    );
                }
                info!("Extracting file to {}", working_directory.display());
                if Command::new("tar")
                    .arg("-xf")
                    .arg(&filepath)
                    .arg("-C")
                    .arg(working_directory)
                    .status()
                    .await
                    .is_err()
                {
                    warn!("Failed to extract file");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to extract file");
                }
                info!("Deleting temp file");
                if tokio::fs::remove_file(filepath).await.is_err() {
                    warn!("Failed to delete temp file");
                }
                (StatusCode::OK, "File uploaded successfully")
            }
        }
    } else {
        (StatusCode::BAD_REQUEST, "Failed to read file")
    }
}
