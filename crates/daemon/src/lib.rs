use std::io;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use tokio::fs::canonicalize;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{info, warn};

pub mod api;
pub mod config;

#[derive(Debug, Clone)]
pub struct AppState(Arc<RwLock<Backend>>);

impl AppState {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self(Arc::new(RwLock::new(Backend::new(path))))
    }

    pub fn into_inner(self) -> Arc<RwLock<Backend>> {
        self.0
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, Backend> {
        self.0.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, Backend> {
        self.0.write().await
    }
}

#[derive(Debug, Default)]
pub struct Backend {
    process: Option<Child>,
    path: PathBuf,
}

impl Backend {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            process: None,
            path: path.into(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub async fn commit_info(&self) -> io::Result<(String, String)> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(["log", "-n", "1"])
            .output()
            .await?;
        Ok((
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }

    pub async fn stdout(&mut self) -> io::Result<String> {
        match &mut self.process {
            Some(process) => match process.stdout.take() {
                Some(mut stdout) => {
                    let mut output = String::new();
                    let n = stdout.read_to_string(&mut output).await?;
                    info!("Read {} bytes from backend stdout", n);
                    process.stdout = Some(stdout);
                    Ok(output)
                }
                None => {
                    warn!("Backend is running but has no stdout");
                    Ok("Backend is running but has no stdout".to_string())
                }
            },
            None => {
                warn!("Backend is not running");
                Ok("Backend is not running".to_string())
            }
        }
    }

    pub async fn stderr(&mut self) -> io::Result<String> {
        match &mut self.process {
            Some(process) => match process.stderr.take() {
                Some(mut stderr) => {
                    let mut output = String::new();
                    let n = stderr.read_to_string(&mut output).await?;
                    info!("Read {} bytes from backend stderr", n);
                    process.stderr = Some(stderr);
                    Ok(output)
                }
                None => {
                    warn!("Backend is running but has no stderr");
                    Ok("Backend is running but has no stderr".to_string())
                }
            },
            None => {
                warn!("Backend is not running");
                Ok("Backend is not running".to_string())
            }
        }
    }

    pub async fn start(&mut self) -> io::Result<()> {
        if self.process.is_some() {
            warn!("Backend is already running");
            return Ok(());
        }
        info!("Installing dependencies");
        match Command::new("mvn")
            .current_dir(&self.path)
            .arg("install")
            .output()
            .await
        {
            Ok(output) => {
                if !output.status.success() {
                    let msg = format!(
                        "Failed to install dependencies\nmaven status: {}\n\nMaven stdout: \n{}\nMaven stderr: \n\n{}\n",
                        output.status,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    warn!("{}", &msg);
                    return Err(io::Error::new(io::ErrorKind::Other, msg));
                }
            }
            Err(e) => {
                warn!("Failed to install dependencies: {}", e);
                return Err(e);
            }
        }
        for entry in self.path.join("target").read_dir()?.filter_map(Result::ok) {
            if entry.file_name().to_string_lossy().ends_with(".jar") {
                let path = canonicalize(entry.path()).await?;
                info!("Found jar: {}", entry.path().display());
                self.process = Some(
                    Command::new("java")
                        .arg("-jar")
                        .arg(&path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()?,
                );
                return Ok(());
            }
        }
        warn!("No jar found in target directory");
        Err(io::Error::new(io::ErrorKind::NotFound, "No jar found"))
    }

    pub async fn stop(&mut self) -> io::Result<()> {
        match &mut self.process {
            Some(process) => {
                process.kill().await?;
                self.process = None;
                Ok(())
            }
            None => {
                warn!("Backend is not running");
                Ok(())
            }
        }
    }

    pub async fn restart(&mut self) -> io::Result<()> {
        self.stop().await?;
        self.start().await
    }
}
