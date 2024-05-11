use std::borrow::Cow;
use std::io::Write;

use axum::extract::{Multipart, State};
use tokio::process::Command;
use tracing::{info, warn};

use crate::AppState;

const FIELD_NAME: &str = "spring-boot-tar-gz-archive";

pub async fn handler(State(state): State<AppState>, mut multipart: Multipart) -> Cow<'static, str> {
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some(FIELD_NAME) => {
                info!("Creating temp file");
                let mut temp = match tempfile::NamedTempFile::new() {
                    Ok(t) => t,
                    Err(e) => {
                        let msg = format!("Failed to create temp file: {}", e);
                        warn!("{}", &msg);
                        return Cow::Owned(msg);
                    }
                };
                info!("Reading uploaded bytes");
                let bytes = match field.bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        let msg = format!("Failed to read bytes from field: {}", e);
                        warn!("{}", &msg);
                        return Cow::Owned(msg);
                    }
                };
                info!("Writing {} bytes to temp file", bytes.len());
                if let Err(e) = temp.write_all(bytes.as_ref()) {
                    let msg = format!("Failed to write bytes to temp file: {}", e);
                    warn!("{}", &msg);
                    return Cow::Owned(msg);
                }
                let temp_path = temp.path();
                let backend_path = state.read().await.path();
                info!("Extracting file to {}", &backend_path.display());
                if let Err(e) = Command::new("tar")
                    .arg("-xf")
                    .arg(temp_path)
                    .args(["--strip-components=1", "-C"])
                    .arg(backend_path)
                    .output()
                    .await
                {
                    let msg = format!("Failed to extract file: {}", e);
                    warn!("{}", &msg);
                    return Cow::Owned(msg);
                }
                return Cow::Borrowed("File uploaded successfully");
            }
            invalid => warn!("Invalid field name: {:?}", invalid),
        }
    }
    Cow::Borrowed("No valid part provided")
}
